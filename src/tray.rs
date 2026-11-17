use cocoa::appkit::{NSStatusBar, NSVariableStatusItemLength};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSSize, NSString};
use objc::{class, msg_send, sel, sel_impl};
use std::path::PathBuf;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

static TRAY_ITEM: AtomicPtr<objc::runtime::Object> = AtomicPtr::new(ptr::null_mut());

pub fn setup_tray() {
    unsafe {
        let bar = NSStatusBar::systemStatusBar(nil);
        let item = bar.statusItemWithLength_(NSVariableStatusItemLength);
        if item == nil {
            tracing::error!("Failed to create status item");
            return;
        }

        let button: id = msg_send![item, button];
        if button != nil {
            let icon_path = resolve_icon_path();
            let path_str = NSString::alloc(nil).init_str(icon_path.to_string_lossy().as_ref());
            let image: id = msg_send![class!(NSImage), alloc];
            let image: id = msg_send![image, initWithContentsOfFile: path_str];
            if image != nil {
                let _: () = msg_send![image, setSize: NSSize::new(18.0, 18.0)];
                let _: () = msg_send![image, setTemplate: NO];
                let _: () = msg_send![button, setImage: image];
                let _: () = msg_send![button, setTitle: NSString::alloc(nil).init_str("")];
            } else {
                let title = NSString::alloc(nil).init_str("BAP");
                let _: () = msg_send![button, setTitle: title];
                tracing::warn!("Failed to load tray icon: {}", icon_path.display());
            }
        } else {
            tracing::error!("Failed to get button from NSStatusItem");
        }

        let _: () = msg_send![item, setVisible: YES];

        let menu: id = msg_send![cocoa::appkit::NSMenu::alloc(nil), init];
        let quit_title = NSString::alloc(nil).init_str("Quit Browseraptor");
        let quit_key = NSString::alloc(nil).init_str("q");
        let quit_item: id = msg_send![
            cocoa::appkit::NSMenuItem::alloc(nil),
            initWithTitle: quit_title
            action: sel!(terminate:)
            keyEquivalent: quit_key
        ];
        let _: () = msg_send![menu, addItem: quit_item];
        let _: () = msg_send![item, setMenu: menu];

        let _: id = msg_send![item, retain];
        let is_visible: bool = msg_send![item, isVisible];
        tracing::info!("Tray icon ready, isVisible={}", is_visible);

        TRAY_ITEM.store(item, Ordering::SeqCst);
    }
}

fn resolve_icon_path() -> PathBuf {
    unsafe {
        let bundle: id = msg_send![class!(NSBundle), mainBundle];
        if bundle != nil {
            let bundle_path: id = msg_send![bundle, bundlePath];
            if bundle_path != nil {
                let bundle_cstr: *const std::os::raw::c_char = msg_send![bundle_path, UTF8String];
                if !bundle_cstr.is_null() {
                    let bundle_path = std::ffi::CStr::from_ptr(bundle_cstr).to_string_lossy().into_owned();
                    if bundle_path.contains(".app") {
                        let resources: id = msg_send![bundle, resourcePath];
                        if resources != nil {
                            let cstr: *const std::os::raw::c_char = msg_send![resources, UTF8String];
                            if !cstr.is_null() {
                                let res_path = std::ffi::CStr::from_ptr(cstr).to_string_lossy().into_owned();
                                return PathBuf::from(res_path).join("assets").join("browseraptor-logo.png");
                            }
                        }
                    }
                }
            }
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("browseraptor-logo.png")
}
