use objc2::rc::Retained;
use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString, NSRunningApplication, NSWorkspace};
use objc2_foundation::NSString;

pub fn get_clipboard_text() -> Option<String> {
    unsafe {
        let pasteboard = NSPasteboard::generalPasteboard();
        let text = pasteboard.stringForType(NSPasteboardTypeString)?;
        Some(text.to_string())
    }
}

pub fn set_clipboard_text(text: &str) {
    unsafe {
        let pasteboard = NSPasteboard::generalPasteboard();
        pasteboard.clearContents();
        let ns_str = NSString::from_str(text);
        pasteboard.setString_forType(&ns_str, NSPasteboardTypeString);
    }
}

pub fn get_frontmost_app() -> Option<Retained<NSRunningApplication>> {
    let workspace = NSWorkspace::sharedWorkspace();
    workspace.frontmostApplication()
}

#[allow(deprecated)]
pub fn activate_app(app: &NSRunningApplication) {
    app.activateWithOptions(
        objc2_app_kit::NSApplicationActivationOptions::ActivateIgnoringOtherApps,
    );
}

pub fn simulate_paste() {
    use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    const KEYCODE_V: CGKeyCode = 0x09;

    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .expect("failed to create CGEventSource");

    let v_down = CGEvent::new_keyboard_event(source.clone(), KEYCODE_V, true)
        .expect("failed to create key down event");
    v_down.set_flags(CGEventFlags::CGEventFlagCommand);

    let v_up = CGEvent::new_keyboard_event(source, KEYCODE_V, false)
        .expect("failed to create key up event");
    v_up.set_flags(CGEventFlags::CGEventFlagCommand);

    v_down.post(CGEventTapLocation::HID);
    v_up.post(CGEventTapLocation::HID);
}

pub fn paste_text_and_restore(text: &str, previous_app: Option<&NSRunningApplication>) {
    let original_clipboard = get_clipboard_text();
    set_clipboard_text(text);

    if let Some(app) = previous_app {
        std::thread::sleep(std::time::Duration::from_millis(100));
        activate_app(app);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    simulate_paste();

    if let Some(original) = original_clipboard {
        std::thread::sleep(std::time::Duration::from_millis(300));
        set_clipboard_text(&original);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get_clipboard_roundtrip() {
        let original = get_clipboard_text();

        set_clipboard_text("termpop_test_value");
        let result = get_clipboard_text();
        assert_eq!(result, Some("termpop_test_value".to_string()));

        if let Some(orig) = original {
            set_clipboard_text(&orig);
        }
    }
}
