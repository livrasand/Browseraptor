#![allow(dead_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use wasmtime::{Engine, Instance, Module, Store, TypedFunc};

pub const CHANNEL_URL: &str =
    "https://raw.githubusercontent.com/livrasand/browseraptor_channel/main/channel_v1.json";

/// Entry for a plugin in the remote channel index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPluginEntry {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    pub version: String,
    pub path: String,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Top-level channel index (channel_v1.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelIndex {
    pub version: u32,
    pub plugins: HashMap<String, ChannelPluginEntry>,
}

/// A plugin that the user has installed locally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPlugin {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Fetch the channel index via HTTP. Returns None if the request fails (offline, etc.).
pub fn fetch_channel() -> Option<ChannelIndex> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("browseraptor/0.2")
        .build()
        .ok()?;
    let resp = client.get(CHANNEL_URL).send().ok()?;
    let bytes = resp.bytes().ok()?;
    serde_json::from_slice(&bytes).ok()
}

use std::sync::{Mutex, OnceLock};

// Cache for the last-fetched channel index (plugins list). Stored as
// Vec<(id, entry)> for iteration in the UI.
static CHANNEL_CACHE: OnceLock<Mutex<Option<Vec<(String, ChannelPluginEntry)>>>> = OnceLock::new();

/// Return a cloned copy of the cached channel index, if present.
pub fn get_channel_cache() -> Option<Vec<(String, ChannelPluginEntry)>> {
    let m = CHANNEL_CACHE.get_or_init(|| Mutex::new(None));
    m.lock().unwrap().clone()
}

/// Fetch the channel index in a background thread and populate the cache. When
/// finished, send a simple AppCommand::ChannelFetched to notify the UI.
pub fn fetch_channel_async(tx: std::sync::mpsc::Sender<crate::app::AppCommand>) {
    std::thread::spawn(move || {
        let fetched = fetch_channel().map(|idx| idx.plugins.into_iter().collect::<Vec<_>>());
        let m = CHANNEL_CACHE.get_or_init(|| Mutex::new(None));
        *m.lock().unwrap() = fetched.clone();
        // Notify the UI (no payload) so the component can read the cached value.
        let _ = tx.send(crate::app::AppCommand::ChannelFetched);
    });
}

/// Result returned by a plugin after evaluating a URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResult {
    pub browser: Option<String>,
    pub profile: Option<String>,
    pub cancel: bool,
}

/// Metadata for a plugin, read from its manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: Option<String>,
}

/// WASM plugin host state (empty for now).
struct PluginHost;

/// A loaded and ready-to-run plugin.
pub struct Plugin {
    pub manifest: PluginManifest,
    engine: Engine,
    module: Module,
}

impl Plugin {
    pub fn evaluate(&self, url: &str) -> anyhow::Result<PluginResult> {
        let mut store = Store::new(&self.engine, PluginHost);
        let instance = Instance::new(&mut store, &self.module, &[])?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow::anyhow!("no `memory` export in wasm module"))?;

        let alloc: TypedFunc<i32, i32> = instance.get_typed_func(&mut store, "alloc")?;
        let dealloc: TypedFunc<(i32, i32), ()> = instance.get_typed_func(&mut store, "dealloc")?;
        let evaluate: TypedFunc<(i32, i32), i32> =
            instance.get_typed_func(&mut store, "evaluate")?;

        let url_bytes = url.as_bytes();
        let len = url_bytes.len() as i32;
        let ptr = alloc.call(&mut store, len)?;

        memory.write(&mut store, ptr as usize, url_bytes)?;

        let result_ptr = evaluate.call(&mut store, (ptr, len))?;

        let result_len = {
            let mut len_bytes = [0u8; 4];
            memory.read(&store, result_ptr as usize, &mut len_bytes)?;
            i32::from_le_bytes(len_bytes) as usize
        };

        let mut result_data = vec![0u8; result_len];
        memory.read(&store, (result_ptr + 4) as usize, &mut result_data)?;

        dealloc.call(&mut store, (ptr, len))?;
        dealloc.call(&mut store, (result_ptr, result_len as i32))?;

        let result: PluginResult = serde_json::from_slice(&result_data)
            .map_err(|e| anyhow::anyhow!("invalid plugin result: {e}"))?;

        Ok(result)
    }
}

/// Load a plugin from a `.wasm` file.
pub fn load_plugin(manifest: PluginManifest, wasm_bytes: &[u8]) -> anyhow::Result<Plugin> {
    let engine = Engine::default();
    let module = Module::new(&engine, wasm_bytes)?;

    Ok(Plugin {
        manifest,
        engine,
        module,
    })
}

/// Evaluate a URL against all loaded plugins.
pub fn evaluate_plugins(plugins: &[Plugin], url: &str) -> anyhow::Result<Option<PluginResult>> {
    for plugin in plugins {
        let result = plugin.evaluate(url)?;
        if result.cancel {
            return Ok(Some(result));
        }
        if result.browser.is_some() {
            return Ok(Some(result));
        }
    }
    Ok(None)
}
