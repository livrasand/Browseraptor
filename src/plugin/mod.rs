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
    /// Ruta local al archivo `.wasm` (solo para plugins de desarrollo).
    /// Si es `None`, el plugin proviene de un canal remoto.
    #[serde(default)]
    pub local_wasm_path: Option<String>,
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
    /// Check if the WASM module exports a function with the given name.
    pub fn has_function(&self, name: &str) -> bool {
        self.module.get_export(name).is_some()
    }

    /// Generic WASM function caller.
    /// Calls a function with signature `fn(ptr: i32, len: i32) -> i32`.
    /// If `input` is None, passes (0, 0).
    /// Reads result from the returned pointer as [4-byte LE length][data].
    pub fn call_function(&self, name: &str, input: Option<&[u8]>) -> anyhow::Result<Vec<u8>> {
        tracing::warn!("call_function: plugin={}, func={}", self.manifest.id, name);
        let mut store = Store::new(&self.engine, PluginHost);
        let instance = Instance::new(&mut store, &self.module, &[])?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow::anyhow!("no `memory` export in wasm module"))?;

        let alloc: TypedFunc<i32, i32> = instance.get_typed_func(&mut store, "alloc")?;
        let dealloc: TypedFunc<(i32, i32), ()> = instance.get_typed_func(&mut store, "dealloc")?;
        let func: TypedFunc<(i32, i32), i32> = instance.get_typed_func(&mut store, name)?;

        let (input_ptr, input_len) = if let Some(data) = input {
            let len = data.len() as i32;
            let ptr = alloc.call(&mut store, len)?;
            memory.write(&mut store, ptr as usize, data)?;
            (ptr, len)
        } else {
            (0i32, 0i32)
        };

        let result_ptr = func.call(&mut store, (input_ptr, input_len))?;

        let result_len = {
            let mut len_bytes = [0u8; 4];
            memory.read(&store, result_ptr as usize, &mut len_bytes)?;
            i32::from_le_bytes(len_bytes) as usize
        };

        let mut result_data = vec![0u8; result_len];
        memory.read(&store, (result_ptr + 4) as usize, &mut result_data)?;

        // Cleanup: dealloc input buffer and result buffer
        if let Some(_data) = input {
            dealloc.call(&mut store, (input_ptr, input_len))?;
        }
        dealloc.call(&mut store, (result_ptr, result_len as i32))?;

        Ok(result_data)
    }

    /// Obtiene la configuracion actual del plugin como JSON string.
    pub fn get_config(&self) -> anyhow::Result<String> {
        let data = self.call_function("get_config", None)?;
        String::from_utf8(data).map_err(|e| anyhow::anyhow!("invalid utf-8 from get_config: {e}"))
    }

    /// Envia una nueva configuracion JSON al plugin.
    /// Devuelve el JSON de respuesta del plugin ({"ok":true} o {"error":"..."}).
    pub fn set_config(&self, json: &str) -> anyhow::Result<String> {
        let data = self.call_function("set_config", Some(json.as_bytes()))?;
        String::from_utf8(data).map_err(|e| anyhow::anyhow!("invalid utf-8 from set_config: {e}"))
    }

    pub fn evaluate(&self, url: &str) -> anyhow::Result<PluginResult> {
        let data = self.call_function("evaluate", Some(url.as_bytes()))?;
        let result: PluginResult = serde_json::from_slice(&data)
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

/// Carga un plugin de desarrollo desde un archivo `.wasm`.
/// Busca el `manifest.json` en el mismo directorio.
pub fn load_dev_plugin_from_path(wasm_path: &str) -> anyhow::Result<InstalledPlugin> {
    use std::fs;
    use std::path::Path;

    let wasm_path = Path::new(wasm_path);
    let parent = wasm_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("no parent directory for wasm path"))?;

    // Leer manifest.json del mismo directorio
    let manifest_path = parent.join("manifest.json");
    let manifest_content = fs::read_to_string(&manifest_path)
        .map_err(|e| anyhow::anyhow!("cannot read manifest.json: {e}"))?;
    let manifest: PluginManifest = serde_json::from_str(&manifest_content)
        .map_err(|e| anyhow::anyhow!("invalid manifest.json: {e}"))?;

    // Validar que el archivo .wasm existe y es un modulo WASM valido
    let wasm_bytes =
        fs::read(wasm_path).map_err(|e| anyhow::anyhow!("cannot read wasm file: {e}"))?;

    // Verificar que es un WASM valido compilandolo
    let engine = Engine::default();
    Module::new(&engine, &wasm_bytes).map_err(|e| anyhow::anyhow!("invalid wasm module: {e}"))?;

    // Verificar que exporta las funciones minimas requeridas
    // (la validacion se hace arriba con Module::new)

    Ok(InstalledPlugin {
        id: manifest.id,
        name: manifest.name,
        version: manifest.version,
        description: manifest.description,
        local_wasm_path: Some(wasm_path.to_string_lossy().to_string()),
    })
}

/// Carga un `InstalledPlugin` desde la configuracion y devuelve un `Plugin`
/// listo para evaluar URLs. Si el plugin no tiene `local_wasm_path`,
/// devuelve error (los plugins de canal remoto no se cargan localmente).
pub fn load_installed_plugin(installed: &InstalledPlugin) -> anyhow::Result<Plugin> {
    let wasm_path = installed
        .local_wasm_path
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("plugin {} no tiene ruta local", installed.id))?;

    let wasm_bytes = std::fs::read(wasm_path)
        .map_err(|e| anyhow::anyhow!("cannot read wasm at {wasm_path}: {e}"))?;

    let manifest = PluginManifest {
        id: installed.id.clone(),
        name: installed.name.clone(),
        version: installed.version.clone(),
        author: String::new(),
        description: installed.description.clone(),
    };

    load_plugin(manifest, &wasm_bytes)
}
