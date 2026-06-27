use cocoa::appkit::{NSStatusBar, NSVariableStatusItemLength};
use cocoa::base::{YES, id, nil};
use cocoa::foundation::NSString;
use objc::{msg_send, sel, sel_impl};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;

static TRAY_ITEM: AtomicPtr<objc::runtime::Object> = AtomicPtr::new(ptr::null_mut());

pub fn setup_tray() {
    unsafe {
        let bar = NSStatusBar::systemStatusBar(nil);
        let item = bar.statusItemWithLength_(NSVariableStatusItemLength);
        let _: () = msg_send![item, setLength: 32.0f64];

        let button: id = msg_send![item, button];
        if button != nil {
            let title = NSString::alloc(nil).init_str("BAP");
            let _: () = msg_send![button, setTitle: title];
            tracing::info!("Button configured with title 'BAP'");
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

        let is_visible: bool = msg_send![item, isVisible];
        tracing::info!("Tray icon ready, isVisible={}", is_visible);

        TRAY_ITEM.store(item, Ordering::SeqCst);
    }
}
