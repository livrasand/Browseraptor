use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::thread;

use crate::app::AppCommand;

const LOCK_NAME: &str = "browseraptor.lock";

fn lock_path() -> PathBuf {
    let dirs = directories::ProjectDirs::from("com", "browseraptor", "browseraptor")
        .expect("cannot determine project directories");
    let runtime_dir = dirs.runtime_dir().unwrap_or_else(|| dirs.cache_dir());
    runtime_dir.join(LOCK_NAME)
}

/// Try to connect to an existing instance and send a command.
/// Returns Ok(true) if command was sent to existing instance.
/// Returns Ok(false) if no existing instance found.
/// Returns Err if there was an error.
pub fn try_send_to_existing(command: &str) -> Result<bool, Box<dyn std::error::Error>> {
    #[cfg(unix)]
    return try_send_to_existing_unix(command);

    #[cfg(windows)]
    return try_send_to_existing_windows(command);

    #[cfg(not(any(unix, windows)))]
    {
        tracing::warn!("Single instance not supported on this platform");
        Ok(false)
    }
}

/// Start listening for commands from new instances.
/// Commands are sent to the daemon via the provided channel.
pub fn start_listener(tx: Sender<AppCommand>) {
    #[cfg(unix)]
    start_listener_unix(tx);

    #[cfg(windows)]
    start_listener_windows(tx);

    #[cfg(not(any(unix, windows)))]
    {
        tracing::warn!("Single instance listener not supported on this platform");
    }
}

#[cfg(unix)]
fn try_send_to_existing_unix(command: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use std::io::Read;
    use std::os::unix::net::UnixStream;

    let path = lock_path();

    match UnixStream::connect(&path) {
        Ok(mut stream) => {
            // Set a short timeout: if the process is dead the write may succeed
            // but the read of the ACK will fail or timeout
            let timeout = std::time::Duration::from_millis(500);
            stream.set_read_timeout(Some(timeout))?;
            stream.set_write_timeout(Some(timeout))?;

            if stream.write_all(command.as_bytes()).is_err()
                || stream.write_all(b"\n").is_err()
                || stream.flush().is_err()
            {
                tracing::info!("Socket write failed, treating as no existing instance");
                return Ok(false);
            }

            // Wait for ACK byte from the live instance
            let mut ack = [0u8; 1];
            match stream.read_exact(&mut ack) {
                Ok(_) => {
                    tracing::info!("Command sent to existing instance");
                    Ok(true)
                }
                Err(_) => {
                    tracing::info!("No live instance found (socket orphan), starting fresh");
                    Ok(false)
                }
            }
        }
        Err(_) => {
            tracing::info!("No existing instance found");
            Ok(false)
        }
    }
}

#[cfg(unix)]
fn start_listener_unix(tx: Sender<AppCommand>) {
    use std::os::unix::net::UnixListener;

    let path = lock_path();

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    // Remove existing socket if present
    let _ = fs::remove_file(&path);

    thread::spawn(move || {
        let listener = match UnixListener::bind(&path) {
            Ok(l) => {
                tracing::info!("Single instance socket listening at {:?}", path);
                l
            }
            Err(e) => {
                tracing::error!("Failed to bind socket: {}", e);
                return;
            }
        };

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let tx_clone = tx.clone();
                    thread::spawn(move || {
                        handle_client_unix(stream, tx_clone);
                    });
                }
                Err(e) => {
                    tracing::error!("Connection failed: {}", e);
                }
            }
        }
    });
}

#[cfg(unix)]
fn handle_client_unix(mut stream: std::os::unix::net::UnixStream, tx: Sender<AppCommand>) {
    use std::io::Write;
    let reader_stream = stream.try_clone().ok();
    let reader = BufReader::new(match reader_stream {
        Some(s) => s,
        None => return,
    });

    for line in reader.lines() {
        match line {
            Ok(cmd_str) => {
                tracing::info!("Received command from client: {}", cmd_str);

                let command = match parse_command(&cmd_str) {
                    Some(cmd) => cmd,
                    None => {
                        tracing::warn!("Unknown command: {}", cmd_str);
                        // Still send ACK so client doesn't hang
                        let _ = stream.write_all(b"\x06");
                        continue;
                    }
                };

                // Send ACK to confirm we are alive
                let _ = stream.write_all(b"\x06");
                let _ = stream.flush();

                if let Err(e) = tx.send(command) {
                    tracing::error!("Failed to send command to daemon: {}", e);
                }
            }
            Err(e) => {
                tracing::error!("Failed to read from client: {}", e);
                break;
            }
        }
    }
}

#[cfg(windows)]
fn try_send_to_existing_windows(command: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let path = lock_path();

    // Try to open the pipe/file for writing
    match OpenOptions::new().write(true).open(&path) {
        Ok(mut file) => {
            file.write_all(command.as_bytes())?;
            file.write_all(b"\n")?;
            file.flush()?;
            tracing::info!("Command sent to existing instance");
            Ok(true)
        }
        Err(_) => {
            tracing::info!("No existing instance found");
            Ok(false)
        }
    }
}

#[cfg(windows)]
fn start_listener_windows(tx: Sender<AppCommand>) {
    use std::fs::OpenOptions;
    use std::io::Read;

    let path = lock_path();

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    thread::spawn(move || {
        // Create a named pipe-like mechanism using a file
        loop {
            match OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&path)
            {
                Ok(mut file) => {
                    tracing::info!("Single instance listener ready at {:?}", path);

                    let mut buffer = String::new();
                    let mut reader = BufReader::new(&file);

                    while let Ok(line) = reader.read_line(&mut buffer) {
                        if line == 0 {
                            break;
                        }

                        let cmd_str = buffer.trim().to_string();
                        buffer.clear();

                        tracing::info!("Received command from client: {}", cmd_str);

                        let command = match parse_command(&cmd_str) {
                            Some(cmd) => cmd,
                            None => {
                                tracing::warn!("Unknown command: {}", cmd_str);
                                continue;
                            }
                        };

                        if let Err(e) = tx.send(command) {
                            tracing::error!("Failed to send command to daemon: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to open listener: {}", e);
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }
        }
    });
}

fn parse_command(cmd_str: &str) -> Option<AppCommand> {
    let parts: Vec<&str> = cmd_str.split_whitespace().collect();

    match parts.get(0).map(|s| *s) {
        Some("show-selector") => {
            let url = parts.get(1).map(|s| s.to_string());
            Some(AppCommand::ShowSelector(url))
        }
        // Legacy: "show-settings" now opens the main window
        Some("show-settings") => Some(AppCommand::ShowSelector(None)),
        Some("open-with") => {
            if let Some(name) = parts.get(1) {
                Some(AppCommand::OpenWith(name.to_string()))
            } else {
                None
            }
        }
        Some("show-plugin-search") => Some(AppCommand::ShowPluginSearch),
        Some("refresh-browsers") => Some(AppCommand::RefreshBrowsers),
        Some("quit") => Some(AppCommand::Quit),
        _ => None,
    }
}
