//! Application state and command channel.
//!
//! Defines the command enum used for IPC between the tray/hotkey
//! modules and the GPUI main loop.

use std::sync::mpsc;

#[derive(Debug, Clone)]
pub enum AppCommand {
    ShowSelector(Option<String>),

    OpenWith(String),
    RefreshBrowsers,
    ShowPluginSearch,
    Quit,

    BrowsersDetected(Vec<crate::browser::Browser>),

    ChannelFetched,
}

pub struct AppState {
    pub installed_browsers: Vec<crate::browser::Browser>,
    pub command_tx: mpsc::Sender<AppCommand>,
    pub command_rx: mpsc::Receiver<AppCommand>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
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
