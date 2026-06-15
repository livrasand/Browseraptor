#![allow(unexpected_cfgs)]

mod app;
mod browser;
mod config;
mod default_browser;
mod hotkey;
mod plugin;
mod single_instance;
mod ui;

use std::borrow::Cow;
use std::path::PathBuf;
use std::process;

use anyhow::Result;
use gpui::{AssetSource, SharedString};

#[cfg(target_os = "macos")]
use objc2::MainThreadMarker;
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};

use clap::{Parser, Subcommand};
use gpui::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::app::{AppCommand, AppState};
use crate::browser::Browser;

/// Asset source que resuelve archivos desde el directorio del proyecto (desarrollo)
/// o desde el bundle de macOS (produccion).
pub struct AppAssets {
    base: PathBuf,
}

impl AppAssets {
    pub fn new() -> Self {
        Self {
            base: Self::resolve_base(),
        }
    }

    #[cfg(target_os = "macos")]
    fn resolve_base() -> PathBuf {
        // Prefer the bundle resources path only when we're actually running inside
        // an `.app` bundle. When launching via `cargo run` (debug/dev) the
        // mainBundle may exist but not point at a .app, which makes resourcePath
        // point somewhere that doesn't contain our `assets/` directory. In that
        // case fall back to the project directory so assets load correctly.
        use objc::{class, msg_send, sel, sel_impl};
        use std::ffi::CStr;
        unsafe {
            let bundle: cocoa::base::id = msg_send![class!(NSBundle), mainBundle];
            if !bundle.is_null() {
                // Check the bundle path and only use resourcePath when it looks like
                // an actual .app bundle (contains ".app"). This avoids picking up
                // an unrelated resource path when running the binary directly.
                let bundle_path_id: cocoa::base::id = msg_send![bundle, bundlePath];
                if !bundle_path_id.is_null() {
                    let bundle_cstr: *const std::os::raw::c_char =
                        msg_send![bundle_path_id, UTF8String];
                    if !bundle_cstr.is_null() {
                        let bundle_path =
                            CStr::from_ptr(bundle_cstr).to_string_lossy().into_owned();
                        if bundle_path.contains(".app") {
                            let resources: cocoa::base::id = msg_send![bundle, resourcePath];
                            if !resources.is_null() {
                                let cstr: *const std::os::raw::c_char =
                                    msg_send![resources, UTF8String];
                                if !cstr.is_null() {
                                    let res_path =
                                        CStr::from_ptr(cstr).to_string_lossy().into_owned();
                                    return PathBuf::from(res_path);
                                }
                            }
                        }
                    }
                }
            }
        }
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    #[cfg(not(target_os = "macos"))]
    fn resolve_base() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }
}

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let full_path = self.base.join(path);
        match std::fs::read(&full_path) {
            Ok(data) => Ok(Some(Cow::Owned(data))),
            Err(e) => Err(e.into()),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let full_path = self.base.join(path);
        std::fs::read_dir(&full_path)
            .map(|entries| {
                entries
                    .filter_map(|entry| {
                        entry
                            .ok()
                            .and_then(|e| e.file_name().into_string().ok())
                            .map(SharedString::from)
                    })
                    .collect()
            })
            .map_err(|e| e.into())
    }
}

#[derive(Parser)]
#[command(name = "browseraptor", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    /// URL to open (passed by macOS when acting as default browser)
    #[arg(index = 1, required = false)]
    url_arg: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Open {
        url: String,
        #[arg(long)]
        no_ui: bool,
    },
    List,
    #[command(subcommand)]
    Config(ConfigCommands),
}

#[derive(Subcommand)]
enum ConfigCommands {
    Show,
    Edit,
}

fn main() {
    // In release builds, also log to a file so we can diagnose issues
    // without a terminal attached (e.g. when launched as .app bundle)
    #[cfg(not(debug_assertions))]
    {
        use std::fs::OpenOptions;
        if let Ok(log_file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/browseraptor.log")
        {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::new("info"))
                .with_writer(std::sync::Mutex::new(log_file))
                .init();
        } else {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::new("info"))
                .init();
        }
    }
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Diagnostic: log all args to understand how macOS passes the URL
    let raw_args: Vec<String> = std::env::args().collect();
    tracing::info!("Args: {:?}", raw_args);

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Open { url, no_ui }) => cmd_open(&url, no_ui),
        Some(Commands::List) => cmd_list(),
        Some(Commands::Config(ConfigCommands::Show)) => cmd_config_show(),
        Some(Commands::Config(ConfigCommands::Edit)) => cmd_config_edit(),
        None => {
            // URL can arrive as positional arg or env var (legacy)
            let open_url = cli
                .url_arg
                .or_else(|| std::env::var("BROWSERAPTOR_OPEN_URL").ok());

            let ipc_cmd = match &open_url {
                Some(url) => format!("show-selector {}", url),
                None => "show-selector".to_string(),
            };
            tracing::info!("IPC command: {}", ipc_cmd);
            match single_instance::try_send_to_existing(&ipc_cmd) {
                Ok(true) => {
                    tracing::info!("Existing instance found, exiting");
                    process::exit(0);
                }
                Ok(false) => {
                    run_daemon_with_url(open_url);
                }
                Err(e) => {
                    tracing::error!("Error checking for existing instance: {}", e);
                    run_daemon_with_url(open_url);
                }
            }
        }
    }
}

fn run_daemon_with_url(initial_url: Option<String>) {
    tracing::info!("Starting Browseraptor daemon…");

    let state = AppState::new();
    let tx = state.command_tx.clone();
    let rx = state.command_rx;

    // Share browsers via Arc<Mutex<...>> so background detection can update the
    // list and the UI can read a snapshot without blocking.
    let browsers = std::sync::Arc::new(std::sync::Mutex::new(state.installed_browsers.clone()));
    {
        let guard = browsers.lock().unwrap();
        if guard.is_empty() {
            tracing::warn!("No browsers detected");
        } else {
            tracing::info!(
                "Detected browsers: {}",
                guard
                    .iter()
                    .map(|b| b.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    let gpui_app = gpui::Application::new().with_assets(AppAssets::new());
    // Use GPUI's built-in open_urls hook (handles application:openURLs: delegate).
    // Must be registered before run() so macOS delivers URLs to the daemon.
    {
        let tx_urls = tx.clone();
        gpui_app.on_open_urls(move |urls| {
            if let Some(url) = urls.into_iter().next() {
                tracing::info!("on_open_urls: {}", url);
                let _ = tx_urls.send(AppCommand::ShowSelector(Some(url)));
            }
        });
    }
    gpui_app.run(move |cx: &mut gpui::App| {
        hotkey::start_hotkey_listener(tx.clone());
        tracing::info!("Hotkey listener started");

        // Start single instance listener
        single_instance::start_listener(tx.clone());
        tracing::info!("Single instance listener started");

        // Set activation policy (macOS only)
        // In dev mode (debug), show dock icon for easier debugging.
        // In production, do NOT call setActivationPolicy — let LSUIElement=true
        // in Info.plist handle hiding the dock icon, while keeping the app able
        // to receive Apple Events (GetURL) and show windows on demand.
        #[cfg(target_os = "macos")]
        {
            let is_dev = std::env::var("BROWSERAPTOR_DEV").is_ok() || cfg!(debug_assertions);
            if is_dev {
                let mtm = MainThreadMarker::new().expect("must be on main thread");
                let app = NSApplication::sharedApplication(mtm);
                app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
                tracing::info!("Dock icon visible (dev mode)");
            } else {
                tracing::info!("Dock icon hidden via LSUIElement (production mode)");
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            tracing::info!(
                "Dock icon hiding is macOS-only, window will be visible on this platform"
            );
        }

        // Check if Browseraptor is the default browser
        let is_default = default_browser::is_browseraptor_default();
        tracing::info!("Browseraptor is default browser: {}", is_default);

        // Show prompt if not default
        if !is_default {
            tracing::info!("Showing default browser prompt");
            cx.activate(true);
            ui::default_prompt::DefaultBrowserPrompt::show(cx);
        }

        // If launched with a URL (e.g. first click before daemon was running), show selector now
        if let Some(url) = initial_url {
            tracing::info!("Initial URL from launch: {}", url);
            let _ = tx.send(AppCommand::ShowSelector(Some(url)));
        }

        let browsers_for_selector = browsers.clone();

        // Use a background thread that blocks on recv() + an async oneshot channel
        // to notify the foreground executor immediately — zero polling latency.
        let cx_clone = cx.to_async();
        let (cmd_tx, mut cmd_rx) = futures::channel::mpsc::unbounded::<AppCommand>();

        std::thread::spawn(move || {
            while let Ok(cmd) = rx.recv() {
                if cmd_tx.unbounded_send(cmd).is_err() {
                    break;
                }
            }
        });

        cx.foreground_executor()
            .spawn(async move {
                use futures::StreamExt;
                while let Some(cmd) = cmd_rx.next().await {
                    let browsers_for_selector = browsers_for_selector.clone();
                    let tx_clone2 = tx.clone();
                    let update_result = cx_clone.update(|cx| match cmd {
                        AppCommand::ShowSelector(url) => {
                            let url = url.unwrap_or_else(|| "https://example.com".into());
                            tracing::info!("ShowSelector: opening window for {}", url);
                            // Snapshot current browsers list for the selector.
                            let b_snapshot = {
                                let guard = browsers_for_selector.lock().unwrap();
                                guard.clone()
                            };
                            let domain = url::Url::parse(&url)
                                .ok()
                                .and_then(|u| u.host_str().map(|s| s.to_string()))
                                .unwrap_or_default();
                            #[cfg(target_os = "macos")]
                            {
                                use objc::{class, msg_send, sel, sel_impl};
                                unsafe {
                                    let workspace: cocoa::base::id =
                                        msg_send![class!(NSWorkspace), sharedWorkspace];
                                    let running_app: cocoa::base::id =
                                        msg_send![class!(NSRunningApplication), currentApplication];
                                    let _: bool =
                                        msg_send![running_app, activateWithOptions: 2usize];
                                    let ns_app: cocoa::base::id =
                                        msg_send![class!(NSApplication), sharedApplication];
                                    let _: () = msg_send![ns_app, activateIgnoringOtherApps: true];
                                    let _ = workspace;
                                }
                            }
                            cx.activate(true);
                            let bounds = gpui::Bounds::centered(
                                None,
                                gpui::size(gpui::px(420.0), gpui::px(460.0)),
                                cx,
                            );
                            let win = cx.open_window(
                                gpui::WindowOptions {
                                    window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                                    titlebar: None,
                                    focus: true,
                                    show: true,
                                    ..Default::default()
                                },
                                |_, cx| {
                                    cx.new(|cx| {
                                        ui::selector::DaemonSelector::new(
                                            url,
                                            domain,
                                            b_snapshot,
                                            tx.clone(),
                                            cx,
                                        )
                                    })
                                },
                            );
                            tracing::info!("open_window result: {:?}", win.is_ok());
                        }
                        AppCommand::OpenWith(name) => {
                            let guard = browsers_for_selector.lock().unwrap();
                            if let Some(b) = find_browser(&name, &guard) {
                                let _ = crate::browser::launcher::launch(&b, "https://example.com");
                            }
                        }
                        AppCommand::ShowPluginSearch => {
                            let _ = tx.send(AppCommand::ShowSelector(None));
                        }
                        AppCommand::RefreshBrowsers => {
                            tracing::info!("Refreshing browser list…");
                            // Spawn a background rescan and notify via BrowsersDetected.
                            let tx3 = tx_clone2.clone();
                            std::thread::spawn(move || {
                                let browsers = crate::browser::detector::detect_installed();
                                let _ = tx3.send(AppCommand::BrowsersDetected(browsers));
                            });
                        }
                        AppCommand::BrowsersDetected(bvec) => {
                            // Update the shared browsers list and log.
                            let mut guard = browsers_for_selector.lock().unwrap();
                            *guard = bvec.clone();
                            tracing::info!(
                                "Detected browsers: {}",
                                guard
                                    .iter()
                                    .map(|b| b.name())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            );
                        }
                        AppCommand::ChannelFetched => {
                            tracing::info!("Plugin channel fetched");
                        }
                        AppCommand::Quit => {
                            tracing::info!("Shutting down…");
                            cx.quit();
                        }
                    });
                    if let Err(e) = update_result {
                        tracing::error!("cx_clone.update failed: {:?}", e);
                    }
                }
            })
            .detach();

        let bounds = gpui::Bounds::centered(None, gpui::size(gpui::px(1.0), gpui::px(1.0)), cx);
        let _ = cx.open_window(
            gpui::WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                show: false,
                ..Default::default()
            },
            |_, cx| cx.new(|_| HiddenView),
        );
    });
}

struct HiddenView;
impl gpui::Render for HiddenView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        gpui::div()
    }
}

#[allow(dead_code)]
struct SelectorPlaceholder(String, Vec<Browser>);
impl gpui::Render for SelectorPlaceholder {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        gpui::div().child(gpui::div().child(format!("Open with… {}", self.0)))
    }
}

fn find_browser(name: &str, browsers: &[Browser]) -> Option<Browser> {
    browsers
        .iter()
        .find(|b| b.name().eq_ignore_ascii_case(name))
        .cloned()
}

// ── CLI commands ──

fn cmd_open(url: &str, no_ui: bool) {
    let parsed = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => {
            let with_scheme = format!("https://{}", url);
            match url::Url::parse(&with_scheme) {
                Ok(u) => u,
                Err(e) => {
                    tracing::error!("Invalid URL: {} ({})", url, e);
                    process::exit(1);
                }
            }
        }
    };

    let url_str = parsed.as_str().to_string();
    let domain = parsed.host_str().map(|h| h.to_string()).unwrap_or_default();
    tracing::info!("Opening: {}", url_str);

    let installed = browser::detector::detect_installed();
    let config = config::Config::load().unwrap_or_default();

    if !no_ui {
        if let Some(remembered) = config
            .remembered
            .iter()
            .find(|r| r.domain == domain || domain.ends_with(&format!(".{}", r.domain)))
        {
            tracing::info!(
                "Remembered: {} → {}",
                remembered.domain,
                remembered.browser.name()
            );
            if let Err(e) = browser::launcher::launch(&remembered.browser, &url_str) {
                tracing::error!("Launch failed: {}", e);
                process::exit(1);
            }
            return;
        }

        if let Some(rule) = config::rules::evaluate_rules(&config.rules, &parsed) {
            tracing::info!(
                "Rule matched: {} → {}",
                rule.domain.as_deref().unwrap_or("*"),
                rule.browser.name()
            );
            if let Err(e) = browser::launcher::launch(&rule.browser, &url_str) {
                tracing::error!("Launch failed: {}", e);
                process::exit(1);
            }
            return;
        }

        if let Some(default) = &config.default_browser {
            if !config.always_show_selector {
                tracing::info!("Default: {}", default.name());
                if let Err(e) = browser::launcher::launch(default, &url_str) {
                    tracing::error!("Launch failed: {}", e);
                    process::exit(1);
                }
                return;
            }
        }
    }

    match ui::selector::run_selector_standalone(&url_str, &installed) {
        Some(ui::selector::SelectorResult::Selected { browser, .. }) => {
            tracing::info!("Selected: {}", browser.name());
            if let Err(e) = browser::launcher::launch(&browser, &url_str) {
                tracing::error!("Launch failed: {}", e);
                process::exit(1);
            }
        }
        Some(ui::selector::SelectorResult::Cancelled) => {
            tracing::info!("Cancelled");
        }
        None => {
            tracing::info!("No selection");
        }
    }
}

fn cmd_list() {
    println!("Installed browsers:");
    for b in browser::detector::detect_installed() {
        println!("  ✓ {}", b.name());
    }
}

fn cmd_config_show() {
    match config::Config::load() {
        Ok(c) => println!("{}", serde_yaml::to_string(&c).unwrap()),
        Err(e) => {
            tracing::error!("Failed to load config: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_config_edit() {
    let config_path = directories::ProjectDirs::from("com", "browseraptor", "browseraptor")
        .map(|d| d.config_dir().join("config.yaml"))
        .expect("cannot determine config directory");

    if !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(
            &config_path,
            serde_yaml::to_string(&config::Config::default()).unwrap(),
        );
    }

    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vim".to_string());

    let status = std::process::Command::new(&editor)
        .arg(&config_path)
        .status()
        .expect("failed to launch editor");

    if !status.success() {
        process::exit(1);
    }
}
