use std::process::Command;

use super::Browser;

/// Detect installed browsers by querying all apps registered as http/https handlers.
/// Falls back to checking known .app paths if the system query fails.
pub fn detect_installed() -> Vec<Browser> {
    // Try to get all handlers via LSCopyAllHandlersForURLScheme (macOS)
    #[cfg(target_os = "macos")]
    {
        let mut found = detect_via_launch_services();
        if !found.is_empty() {
            // Always exclude ourselves
            found.retain(|b| {
                !matches!(b, Browser::Other { name: n, .. } if n.to_lowercase().contains("browseraptor"))
            });
            return found;
        }
    }

    // Fallback: check known browsers by path / mdfind
    let candidates = vec![
        Browser::Chrome,
        Browser::Firefox,
        Browser::Brave,
        Browser::Edge,
        Browser::Safari,
        Browser::Arc,
        Browser::Orion,
    ];
    candidates.into_iter().filter(|b| is_installed(b)).collect()
}

#[cfg(target_os = "macos")]
fn detect_via_launch_services() -> Vec<Browser> {
    use objc2::rc::Retained;
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::{NSURL, NSString};

    let url_str = NSString::from_str("https://example.com");
    let ns_url = NSURL::URLWithString(&url_str);
    let ns_url = match ns_url {
        Some(u) => u,
        None => return vec![],
    };

    let app_urls: Retained<objc2_foundation::NSArray<NSURL>> =
        NSWorkspace::sharedWorkspace().URLsForApplicationsToOpenURL(&ns_url);

    let mut browsers: Vec<Browser> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Collect paths first, then process in parallel using threads
    let mut paths: Vec<String> = Vec::new();
    let count = app_urls.count();
    for i in 0..count {
        let app_url = app_urls.objectAtIndex(i);
        if let Some(p) = app_url.path().map(|p| p.to_string()) {
            paths.push(p);
        }
    }

    // Process each app path concurrently — each spawns its own thread for plutil
    let handles: Vec<_> = paths
        .into_iter()
        .map(|app_path| {
            std::thread::spawn(move || -> Option<(String, String, Option<String>)> {
                let plist_path = format!("{}/Contents/Info.plist", app_path);
                if !std::path::Path::new(&plist_path).exists() {
                    return None;
                }
                // Single plutil call — parse plist once, extract all values
                let (bundle_id, display_name) = plist_read_ids(&plist_path)?;
                if bundle_id.to_lowercase().contains("browseraptor") {
                    return None;
                }
                Some((app_path, bundle_id, display_name))
            })
        })
        .collect();
    let results: Vec<(String, String, Option<String>)> = handles
        .into_iter()
        .filter_map(|h| h.join().ok().flatten())
        .collect();

    for (app_path, bundle_id, display_name) in results {
        // Deduplicate
        if !seen.insert(bundle_id.clone()) {
            continue;
        }

        let browser = match bundle_id.as_str() {
            "com.google.Chrome" | "com.google.Chrome.beta" | "com.google.Chrome.canary" => Browser::Chrome,
            "org.mozilla.firefox" | "org.mozilla.firefoxdeveloperedition" => Browser::Firefox,
            "com.brave.Browser" | "com.brave.Browser.beta" => Browser::Brave,
            "com.microsoft.Edge" | "com.microsoft.Edge.beta" => Browser::Edge,
            "com.apple.Safari" | "com.apple.SafariTechnologyPreview" => Browser::Safari,
            "company.thebrowser.Browser" => Browser::Arc,
            "com.kagi.Orion" => Browser::Orion,
            _ => {
                let name = display_name.unwrap_or_else(|| {
                    std::path::Path::new(&app_path)
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| bundle_id.clone())
                });
                Browser::Other { name, app_path: Some(app_path) }
            }
        };

        browsers.push(browser);
    }

    // Sort: known browsers first, then Others alphabetically
    browsers.sort_by(|a, b| {
        let a_ord = known_order(a);
        let b_ord = known_order(b);
        a_ord.cmp(&b_ord).then_with(|| a.name().cmp(b.name()))
    });

    browsers
}

fn known_order(b: &Browser) -> u8 {
    match b {
        Browser::Chrome  => 0,
        Browser::Firefox => 1,
        Browser::Brave   => 2,
        Browser::Edge    => 3,
        Browser::Safari  => 4,
        Browser::Arc     => 5,
        Browser::Orion   => 6,
        Browser::Other { .. } => 7,
    }
}

/// Read CFBundleIdentifier and display name from an Info.plist in a single plutil call.
fn plist_read_ids(plist: &str) -> Option<(String, Option<String>)> {
    let xml = Command::new("plutil")
        .args(["-convert", "xml1", "-o", "-", plist])
        .output()
        .ok()?;
    if !xml.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&xml.stdout);
    let bundle_id = extract_plist_string(&text, "CFBundleIdentifier")?;
    let display_name = extract_plist_string(&text, "CFBundleDisplayName")
        .or_else(|| extract_plist_string(&text, "CFBundleName"));
    Some((bundle_id, display_name))
}

fn extract_plist_string(text: &str, key: &str) -> Option<String> {
    let key_tag = format!("<key>{}</key>", key);
    let pos = text.find(&key_tag)?;
    let after = &text[pos + key_tag.len()..];
    let start = after.find("<string>")? + "<string>".len();
    let end = after[start..].find("</string>")?;
    let value = after[start..start + end].trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}


fn is_installed(browser: &Browser) -> bool {
    let path = format!("/Applications/{}.app", browser.exec_name());
    if std::path::Path::new(&path).exists() {
        return true;
    }
    let user_path = format!(
        "{}/Applications/{}.app",
        std::env::var("HOME").unwrap_or_default(),
        browser.exec_name()
    );
    if std::path::Path::new(&user_path).exists() {
        return true;
    }
    mdfind_check(browser)
}

fn mdfind_check(browser: &Browser) -> bool {
    let bundle_id = browser.bundle_id();
    if bundle_id.is_empty() {
        return false;
    }
    let output = Command::new("mdfind")
        .args(["kMDItemCFBundleIdentifier", "==", bundle_id])
        .output()
        .ok();
    match output {
        Some(out) => !out.stdout.is_empty() && out.stdout.len() > 1,
        None => false,
    }
}
