/// Get the bundle ID of the default HTTP handler
pub fn get_default_http_handler() -> Option<String> {
    #[cfg(target_os = "macos")]
    return get_default_http_handler_macos();
    
    #[cfg(target_os = "windows")]
    return get_default_http_handler_windows();
    
    #[cfg(target_os = "linux")]
    return get_default_http_handler_linux();
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        tracing::warn!("Default browser detection not supported on this platform");
        None
    }
}

/// Check if Browseraptor is the default HTTP handler
pub fn is_browseraptor_default() -> bool {
    match get_default_http_handler() {
        Some(bundle_id) => {
            // Browseraptor's bundle ID would typically be something like "com.browseraptor.Browseraptor"
            // For now, we'll check if it contains "browseraptor"
            bundle_id.to_lowercase().contains("browseraptor")
        }
        None => false,
    }
}

/// Automatically attempt to set Browseraptor as the default browser
pub fn set_as_default_browser() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    return set_as_default_browser_macos();
    
    #[cfg(not(target_os = "macos"))]
    return Err("Automatic default browser setting is only supported on macOS.".to_string());
}

#[cfg(target_os = "macos")]
fn set_as_default_browser_macos() -> Result<(), String> {
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString;
    use objc::{msg_send, sel, sel_impl, class};
    
    #[link(name = "CoreServices", kind = "framework")]
    extern "C" {
        fn LSSetDefaultHandlerForURLScheme(
            inURLScheme: id,
            inHandlerBundleID: id,
        ) -> i32;
    }
    
    unsafe {
        #[allow(unexpected_cfgs)]
        let bundle: id = msg_send![class!(NSBundle), mainBundle];
        
        if bundle == nil {
            return Err("Not running as an app bundle.".to_string());
        }
        
        #[allow(unexpected_cfgs)]
        let bundle_id: id = msg_send![bundle, bundleIdentifier];
        
        if bundle_id == nil {
            return Err("App has no bundle identifier.".to_string());
        }
        
        let http_str = NSString::alloc(nil).init_str("http");
        let https_str = NSString::alloc(nil).init_str("https");
        
        let status_http = LSSetDefaultHandlerForURLScheme(http_str, bundle_id);
        let status_https = LSSetDefaultHandlerForURLScheme(https_str, bundle_id);
        
        if status_http == 0 && status_https == 0 {
            Ok(())
        } else {
            Err(format!("LaunchServices error: http={}, https={}", status_http, status_https))
        }
    }
}
/// Open System Settings to the default browser configuration
pub fn open_default_browser_settings() {
    #[cfg(target_os = "macos")]
    open_default_browser_settings_macos();
    
    #[cfg(target_os = "windows")]
    open_default_browser_settings_windows();
    
    #[cfg(target_os = "linux")]
    open_default_browser_settings_linux();
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        tracing::warn!("Opening default browser settings not supported on this platform");
    }
}

#[cfg(target_os = "macos")]
fn get_default_http_handler_macos() -> Option<String> {
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString;
    use objc::{msg_send, sel, sel_impl, class};
    
    unsafe {
        #[allow(unexpected_cfgs)]
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        
        // Create a test URL
        let url_string = NSString::alloc(nil).init_str("http://example.com");
        #[allow(unexpected_cfgs)]
        let url: id = msg_send![class!(NSURL), URLWithString: url_string];
        
        if url == nil {
            tracing::error!("Failed to create NSURL");
            return None;
        }
        
        // Get the default app for this URL
        #[allow(unexpected_cfgs)]
        let app_url: id = msg_send![workspace, URLForApplicationToOpenURL: url];
        
        if app_url == nil {
            tracing::error!("Failed to get default application");
            return None;
        }
        
        // Get the bundle ID from the app URL
        #[allow(unexpected_cfgs)]
        let bundle: id = msg_send![class!(NSBundle), bundleWithURL: app_url];
        if bundle == nil {
            tracing::error!("Failed to get bundle");
            return None;
        }
        
        #[allow(unexpected_cfgs)]
        let bundle_id: id = msg_send![bundle, bundleIdentifier];
        if bundle_id == nil {
            tracing::error!("Failed to get bundle identifier");
            return None;
        }
        
        #[allow(unexpected_cfgs)]
        let c_string: *const i8 = msg_send![bundle_id, UTF8String];
        if c_string.is_null() {
            tracing::error!("Failed to get UTF8 string");
            return None;
        }
        
        let bundle_id_str = std::ffi::CStr::from_ptr(c_string).to_string_lossy().to_string();
        tracing::info!("Default HTTP handler: {}", bundle_id_str);
        
        Some(bundle_id_str)
    }
}

#[cfg(target_os = "macos")]
fn open_default_browser_settings_macos() {
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString;
    use objc::{msg_send, sel, sel_impl, class};
    
    unsafe {
        #[allow(unexpected_cfgs)]
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        
        // Open System Settings > General > Default web browser
        let url_string = NSString::alloc(nil).init_str("x-apple.systempreferences:com.apple.preference.general");
        #[allow(unexpected_cfgs)]
        let url: id = msg_send![class!(NSURL), URLWithString: url_string];
        
        if url != nil {
            #[allow(unexpected_cfgs)]
            let _: () = msg_send![workspace, openURL: url];
            tracing::info!("Opened System Settings");
        } else {
            tracing::error!("Failed to create System Settings URL");
        }
    }
}

#[cfg(target_os = "windows")]
fn get_default_http_handler_windows() -> Option<String> {
    use winreg::RegKey;
    use winreg::enums::*;
    
    let hklm = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\Shell\Associations\UrlAssociations\http\UserChoice";
    
    match hklm.open_subkey(path) {
        Ok(key) => {
            match key.get_value::<String, _>("ProgId") {
                Ok(prog_id) => {
                    tracing::info!("Default HTTP handler: {}", prog_id);
                    Some(prog_id)
                }
                Err(e) => {
                    tracing::error!("Failed to read ProgId: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to open registry key: {}", e);
            None
        }
    }
}

#[cfg(target_os = "windows")]
fn open_default_browser_settings_windows() {
    use std::process::Command;
    
    // Open Windows Settings > Apps > Default apps
    let _ = Command::new("cmd")
        .args(&["/c", "start", "ms-settings:defaultapps"])
        .spawn();
    
    tracing::info!("Opened Windows Settings");
}

#[cfg(target_os = "linux")]
fn get_default_http_handler_linux() -> Option<String> {
    use std::process::Command;
    
    // Try xdg-settings to get default browser
    match Command::new("xdg-settings")
        .args(&["get", "default-web-browser"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let browser = String::from_utf8_lossy(&output.stdout).trim().to_string();
                tracing::info!("Default HTTP handler: {}", browser);
                Some(browser)
            } else {
                tracing::error!("xdg-settings failed: {:?}", output);
                None
            }
        }
        Err(e) => {
            tracing::error!("Failed to run xdg-settings: {}", e);
            None
        }
    }
}

#[cfg(target_os = "linux")]
fn open_default_browser_settings_linux() {
    use std::process::Command;
    
    // Try to open GNOME settings or KDE settings
    let _ = Command::new("xdg-open")
        .arg("settings://default-apps")
        .spawn();
    
    tracing::info!("Attempted to open desktop settings");
}
