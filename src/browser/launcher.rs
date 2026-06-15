use std::process::Command;

use anyhow::{Context, Result};

use super::Browser;

/// Launch the given browser with the provided URL.
pub fn launch(browser: &Browser, url: &str) -> Result<()> {
    let bundle_id = browser.bundle_id();
    if bundle_id.is_empty() {
        return open_fallback(browser, url);
    }

    let status = Command::new("open")
        .args(["-b", bundle_id, url])
        .status()
        .context("failed to execute `open` command")?;

    if !status.success() {
        anyhow::bail!("`open` exited with non-zero status");
    }
    Ok(())
}

fn open_fallback(browser: &Browser, url: &str) -> Result<()> {
    // For Other browsers with a known path, open directly
    #[cfg(target_os = "macos")]
    if let Browser::Other { app_path: Some(path), .. } = browser {
        let status = Command::new("open")
            .args(["-a", path.as_str(), url])
            .status()
            .context("failed to open browser via app path")?;
        if !status.success() {
            anyhow::bail!("failed to launch browser at path: {}", path);
        }
        return Ok(());
    }

    let exec = browser.exec_name();
    // Try `open -a "BrowserName"` as fallback
    let status = Command::new("open")
        .args(["-a", exec, url])
        .status()
        .context("failed to open browser via `open -a`")?;

    if !status.success() {
        anyhow::bail!("failed to launch browser: {}", exec);
    }
    Ok(())
}
