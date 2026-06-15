//! Application state and command channel.
//!
//! Defines the command enum used for IPC between the tray/hotkey
//! modules and the GPUI main loop.

use std::sync::mpsc;

/// Commands sent from the tray / hotkeys to the GPUI app.
#[derive(Debug, Clone)]
pub enum AppCommand {
    /// Show the browser selector (with optional URL).
    ShowSelector(Option<String>),

    /// Open a URL directly with the named browser.
    OpenWith(String),
    /// Re-scan installed browsers.
    RefreshBrowsers,
    /// Show the plugin search/browser panel.
    ShowPluginSearch,
    /// Quit the application.
    Quit,

    /// Internal: browsers were detected (background task -> UI)
    BrowsersDetected(Vec<crate::browser::Browser>),

    /// Internal: channel fetch finished (plugin search should read plugin::get_channel_cache())
    ChannelFetched,
}

/// Shared application state.
pub struct AppState {
    pub installed_browsers: Vec<crate::browser::Browser>,
    pub command_tx: mpsc::Sender<AppCommand>,
    pub command_rx: mpsc::Receiver<AppCommand>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        // Spawn a background thread to detect browsers and notify the UI via the
        // command channel. This avoids blocking startup while still allowing the
        // main loop to receive the results and update the UI when ready.
        let tx_clone = tx.clone();
        std::thread::spawn(move || {
            let browsers = crate::browser::detector::detect_installed();
            let _ = tx_clone.send(AppCommand::BrowsersDetected(browsers));
        });

        Self {
            installed_browsers: Vec::new(),
            command_tx: tx,
            command_rx: rx,
        }
    }
}
