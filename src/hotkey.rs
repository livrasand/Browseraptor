use std::ptr::NonNull;
use std::sync::mpsc;

use crate::app::AppCommand;

#[cfg(target_os = "macos")]
use objc2_app_kit::{NSEvent, NSEventMask, NSEventModifierFlags};

pub fn start_hotkey_listener(tx: mpsc::Sender<AppCommand>) {
    #[cfg(target_os = "macos")]
    start_hotkey_listener_macos(tx);

    #[cfg(target_os = "windows")]
    start_hotkey_listener_windows(tx);

    #[cfg(target_os = "linux")]
    start_hotkey_listener_linux(tx);

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        tracing::warn!("Hotkey listener not supported on this platform");
    }
}

#[cfg(target_os = "macos")]
fn start_hotkey_listener_macos(tx: mpsc::Sender<AppCommand>) {
    use block2::RcBlock;

    let bindings: Vec<HotkeyBind> = vec![
        HotkeyBind {
            key: "s",
            mods: ModFlags {
                cmd: true,
                shift: true,
                ..Default::default()
            },
            action: HotkeyAction::OpenBrowser("Safari"),
        },
        HotkeyBind {
            key: "c",
            mods: ModFlags {
                cmd: true,
                shift: true,
                ..Default::default()
            },
            action: HotkeyAction::OpenBrowser("Chrome"),
        },
        HotkeyBind {
            key: "f",
            mods: ModFlags {
                cmd: true,
                shift: true,
                ..Default::default()
            },
            action: HotkeyAction::OpenBrowser("Firefox"),
        },
        HotkeyBind {
            key: " ",
            mods: ModFlags {
                cmd: true,
                shift: true,
                ..Default::default()
            },
            action: HotkeyAction::ShowSelector,
        },
        HotkeyBind {
            key: ",",
            mods: ModFlags {
                cmd: true,
                ..Default::default()
            },
            action: HotkeyAction::ShowSelector,
        },
        HotkeyBind {
            key: "r",
            mods: ModFlags {
                cmd: true,
                shift: true,
                ..Default::default()
            },
            action: HotkeyAction::RefreshBrowsers,
        },
        HotkeyBind {
            key: "q",
            mods: ModFlags {
                cmd: true,
                ..Default::default()
            },
            action: HotkeyAction::Quit,
        },
        HotkeyBind {
            key: "p",
            mods: ModFlags {
                shift: true,
                ctrl: true,
                ..Default::default()
            },
            action: HotkeyAction::ShowPluginSearch,
        },
    ];

    let block: RcBlock<dyn Fn(NonNull<NSEvent>)> = RcBlock::new(move |event: NonNull<NSEvent>| {
        handle_key_event_macos(event, &bindings, &tx);
    });

    let mask = NSEventMask::KeyDown;
    let _monitor = NSEvent::addGlobalMonitorForEventsMatchingMask_handler(mask, &block);
}

#[cfg(target_os = "macos")]
fn handle_key_event_macos(
    event: NonNull<NSEvent>,
    bindings: &[HotkeyBind],
    tx: &mpsc::Sender<AppCommand>,
) {
    let event = unsafe { event.as_ref() };
    let mods = event.modifierFlags();
    let chars = event.charactersIgnoringModifiers();

    let chars_str = chars
        .as_ref()
        .and_then(|s| {
            let cstr: *const std::os::raw::c_char = s.UTF8String();
            if cstr.is_null() {
                None
            } else {
                unsafe { std::ffi::CStr::from_ptr(cstr) }
                    .to_str()
                    .ok()
                    .map(|s| s.to_string())
            }
        })
        .unwrap_or_default();

    let cmd_pressed = mods.contains(NSEventModifierFlags::Command);
    let shift_pressed = mods.contains(NSEventModifierFlags::Shift);
    let alt_pressed = mods.contains(NSEventModifierFlags::Option);
    let ctrl_pressed = mods.contains(NSEventModifierFlags::Control);

    for bind in bindings {
        if bind.key == chars_str
            && bind.mods.cmd == cmd_pressed
            && bind.mods.shift == shift_pressed
            && bind.mods.alt == alt_pressed
            && bind.mods.ctrl == ctrl_pressed
        {
            let cmd = match bind.action {
                HotkeyAction::OpenBrowser(name) => AppCommand::OpenWith(name.to_string()),
                HotkeyAction::ShowSelector => AppCommand::ShowSelector(None),

                HotkeyAction::RefreshBrowsers => AppCommand::RefreshBrowsers,
                HotkeyAction::ShowPluginSearch => AppCommand::ShowPluginSearch,
                HotkeyAction::Quit => AppCommand::Quit,
            };
            let _ = tx.send(cmd);
            return;
        }
    }
}

#[cfg(target_os = "windows")]
fn start_hotkey_listener_windows(_tx: mpsc::Sender<AppCommand>) {
    tracing::warn!("Global hotkeys not yet implemented for Windows. Use CLI commands instead.");
}

#[cfg(target_os = "linux")]
fn start_hotkey_listener_linux(_tx: mpsc::Sender<AppCommand>) {
    tracing::warn!("Global hotkeys not yet implemented for Linux. Use CLI commands instead.");
}

#[cfg(target_os = "macos")]
struct HotkeyBind {
    key: &'static str,
    mods: ModFlags,
    action: HotkeyAction,
}

#[cfg(target_os = "macos")]
#[derive(Default)]
struct ModFlags {
    cmd: bool,
    shift: bool,
    alt: bool,
    ctrl: bool,
}

#[cfg(target_os = "macos")]
enum HotkeyAction {
    OpenBrowser(&'static str),
    ShowSelector,
    RefreshBrowsers,
    ShowPluginSearch,
    Quit,
}
