#[derive(Debug, Clone, PartialEq)]
pub enum EditorResult {
    Submitted(String),
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct EditorConfig {
    pub initial_text: String,
    pub title: String,
    pub width: f64,
    pub height: f64,
    pub font_size: f64,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            initial_text: String::new(),
            title: "TermPop".to_string(),
            width: 600.0,
            height: 300.0,
            font_size: 14.0,
        }
    }
}

const KEYCODE_RETURN: u16 = 36;
const KEYCODE_ESCAPE: u16 = 53;
const KEYCODE_EQUAL: u16 = 0x18;
const KEYCODE_MINUS: u16 = 0x1B;
const KEYCODE_ZERO: u16 = 0x1D;
const FONT_SIZE_MIN: f64 = 8.0;
const FONT_SIZE_MAX: f64 = 72.0;
const FONT_SIZE_STEP: f64 = 2.0;

fn save_font_size(font_size: f64) {
    let mut cfg = crate::config::load_config();
    cfg.font_size = font_size;
    if let Err(e) = crate::config::save_config(&cfg) {
        eprintln!("failed to save font size: {}", e);
    }
}

pub fn run_editor(config: EditorConfig) -> EditorResult {
    use objc2::rc::Retained;
    use objc2::MainThreadOnly;
    use objc2_app_kit::*;
    use objc2_foundation::*;

    unsafe {
        let mtm = MainThreadMarker::new().expect("must be called from main thread");

        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

        let style = NSWindowStyleMask::Titled
            | NSWindowStyleMask::Closable
            | NSWindowStyleMask::Resizable
            | NSWindowStyleMask::FullSizeContentView;

        let frame = NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(config.width, config.height),
        );

        let window = NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            frame,
            style,
            NSBackingStoreType::Buffered,
            false,
        );

        window.setTitle(&NSString::from_str(&config.title));
        window.setTitleVisibility(NSWindowTitleVisibility::Hidden);
        window.setTitlebarAppearsTransparent(true);
        window.setLevel(NSFloatingWindowLevel);
        window.setHasShadow(true);
        window.setOpaque(false);

        let mouse_pos = NSEvent::mouseLocation();
        let top_left = NSPoint::new(mouse_pos.x, mouse_pos.y);
        window.setFrameTopLeftPoint(top_left);

        if let Some(screen) = NSScreen::mainScreen(mtm) {
            let screen_frame = screen.visibleFrame();
            let win_frame = window.frame();

            let mut x = win_frame.origin.x;
            let mut y = win_frame.origin.y;

            if x + win_frame.size.width > screen_frame.origin.x + screen_frame.size.width {
                x = screen_frame.origin.x + screen_frame.size.width - win_frame.size.width;
            }
            if x < screen_frame.origin.x {
                x = screen_frame.origin.x;
            }
            if y < screen_frame.origin.y {
                y = screen_frame.origin.y;
            }
            if y + win_frame.size.height > screen_frame.origin.y + screen_frame.size.height {
                y = screen_frame.origin.y + screen_frame.size.height - win_frame.size.height;
            }

            window.setFrameOrigin(NSPoint::new(x, y));
        }

        let scroll_view = NSTextView::scrollableTextView(mtm);
        let text_view: Retained<NSTextView> =
            Retained::cast_unchecked(scroll_view.documentView().expect("documentView missing"));

        text_view.setEditable(true);
        text_view.setRichText(false);

        let font = NSFont::monospacedSystemFontOfSize_weight(config.font_size, NSFontWeightRegular);
        text_view.setFont(Some(&font));

        let appearance = NSApplication::sharedApplication(mtm).effectiveAppearance();
        let appearance_name = appearance.name();
        let is_dark = appearance_name.to_string().contains("Dark");

        if is_dark {
            let bg = NSColor::colorWithSRGBRed_green_blue_alpha(0.15, 0.15, 0.15, 1.0);
            let fg = NSColor::colorWithSRGBRed_green_blue_alpha(0.93, 0.93, 0.93, 1.0);
            window.setBackgroundColor(Some(&bg));
            text_view.setBackgroundColor(&bg);
            text_view.setTextColor(Some(&fg));
            text_view.setInsertionPointColor(Some(&fg));
        } else {
            let bg = NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 1.0, 1.0, 1.0);
            let fg = NSColor::colorWithSRGBRed_green_blue_alpha(0.1, 0.1, 0.1, 1.0);
            window.setBackgroundColor(Some(&bg));
            text_view.setBackgroundColor(&bg);
            text_view.setTextColor(Some(&fg));
            text_view.setInsertionPointColor(Some(&fg));
        }

        if !config.initial_text.is_empty() {
            text_view.setString(&NSString::from_str(&config.initial_text));
        }

        let hint_height: f64 = 20.0;
        let content_frame = NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(config.width, config.height),
        );

        let container = NSView::initWithFrame(NSView::alloc(mtm), content_frame);

        let scroll_frame = NSRect::new(
            NSPoint::new(0.0, hint_height),
            NSSize::new(config.width, config.height - hint_height),
        );
        scroll_view.setFrame(scroll_frame);
        scroll_view.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewWidthSizable
                | NSAutoresizingMaskOptions::ViewHeightSizable,
        );

        let hint_frame = NSRect::new(
            NSPoint::new(8.0, 0.0),
            NSSize::new(config.width - 16.0, hint_height),
        );
        let hint_label = NSTextField::wrappingLabelWithString(
            &NSString::from_str(
                "Enter: 줄바꿈  │  ⌘+Enter: 제출  │  Esc: 취소  │  ⌃+/⌃-: 글자 크기",
            ),
            mtm,
        );
        hint_label.setFrame(hint_frame);
        hint_label.setEditable(false);
        hint_label.setSelectable(false);
        hint_label.setBordered(false);
        hint_label.setDrawsBackground(false);

        let hint_font = NSFont::systemFontOfSize(11.0);
        hint_label.setFont(Some(&hint_font));

        if is_dark {
            let hint_color = NSColor::colorWithSRGBRed_green_blue_alpha(0.6, 0.6, 0.6, 1.0);
            hint_label.setTextColor(Some(&hint_color));
        } else {
            let hint_color = NSColor::colorWithSRGBRed_green_blue_alpha(0.5, 0.5, 0.5, 1.0);
            hint_label.setTextColor(Some(&hint_color));
        }

        hint_label.setAutoresizingMask(NSAutoresizingMaskOptions::ViewWidthSizable);

        container.addSubview(&scroll_view);
        container.addSubview(&hint_label);

        container.setWantsLayer(true);
        if let Some(layer) = container.layer() {
            layer.setCornerRadius(12.0);
            layer.setMasksToBounds(true);
        }

        window.setContentView(Some(&container));
        window.makeKeyAndOrderFront(None);
        window.makeFirstResponder(Some(&text_view));

        app.activate();

        let text_view_ref = text_view.clone();
        let window_ref = window.clone();
        let hint_label_ref = hint_label.clone();
        let default_font_size = config.font_size;
        let mut current_font_size = config.font_size;
        let hint_default = NSString::from_str(
            "Enter: 줄바꿈  │  ⌘+Enter: 제출  │  Esc: 취소  │  ⌃+/⌃-: 글자 크기",
        );

        loop {
            let event = app.nextEventMatchingMask_untilDate_inMode_dequeue(
                NSEventMask::Any,
                Some(&NSDate::distantFuture()),
                NSDefaultRunLoopMode,
                true,
            );

            if let Some(ref event) = event {
                let event_type = event.r#type();

                if event_type == NSEventType::KeyDown {
                    let keycode = event.keyCode();
                    let flags = event.modifierFlags();
                    let has_cmd = flags.contains(NSEventModifierFlags::Command);
                    let has_ctrl = flags.contains(NSEventModifierFlags::Control);

                    if has_cmd && keycode == KEYCODE_RETURN {
                        let text = text_view_ref.string().to_string();
                        window_ref.close();
                        return EditorResult::Submitted(text);
                    }

                    if keycode == KEYCODE_ESCAPE {
                        window_ref.close();
                        return EditorResult::Cancelled;
                    }

                    if has_ctrl && keycode == KEYCODE_EQUAL {
                        current_font_size = (current_font_size + FONT_SIZE_STEP).min(FONT_SIZE_MAX);
                        let new_font = NSFont::monospacedSystemFontOfSize_weight(
                            current_font_size,
                            NSFontWeightRegular,
                        );
                        text_view_ref.setFont(Some(&new_font));
                        hint_label_ref.setStringValue(&NSString::from_str(&format!(
                            "폰트 크기: {:.0}pt",
                            current_font_size
                        )));
                        save_font_size(current_font_size);
                        continue;
                    }

                    if has_ctrl && keycode == KEYCODE_MINUS {
                        current_font_size = (current_font_size - FONT_SIZE_STEP).max(FONT_SIZE_MIN);
                        let new_font = NSFont::monospacedSystemFontOfSize_weight(
                            current_font_size,
                            NSFontWeightRegular,
                        );
                        text_view_ref.setFont(Some(&new_font));
                        hint_label_ref.setStringValue(&NSString::from_str(&format!(
                            "폰트 크기: {:.0}pt",
                            current_font_size
                        )));
                        save_font_size(current_font_size);
                        continue;
                    }

                    if has_ctrl && keycode == KEYCODE_ZERO {
                        current_font_size = default_font_size;
                        let new_font = NSFont::monospacedSystemFontOfSize_weight(
                            current_font_size,
                            NSFontWeightRegular,
                        );
                        text_view_ref.setFont(Some(&new_font));
                        hint_label_ref.setStringValue(&NSString::from_str(&format!(
                            "폰트 크기: {:.0}pt (기본)",
                            current_font_size
                        )));
                        save_font_size(current_font_size);
                        continue;
                    }

                    hint_label_ref.setStringValue(&hint_default);
                }

                if event_type == NSEventType::AppKitDefined && !window_ref.isVisible() {
                    return EditorResult::Cancelled;
                }

                app.sendEvent(event);
            }

            if !window_ref.isVisible() {
                return EditorResult::Cancelled;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_result_submitted_holds_text() {
        let result = EditorResult::Submitted("hello world".to_string());
        assert_eq!(result, EditorResult::Submitted("hello world".to_string()));
    }

    #[test]
    fn editor_result_cancelled_is_distinct() {
        let result = EditorResult::Cancelled;
        assert_ne!(result, EditorResult::Submitted(String::new()));
    }

    #[test]
    fn default_config_has_expected_values() {
        let config = EditorConfig::default();
        assert_eq!(config.title, "TermPop");
        assert_eq!(config.width, 600.0);
        assert_eq!(config.height, 300.0);
        assert!(config.initial_text.is_empty());
    }

    #[test]
    fn config_with_initial_text() {
        let config = EditorConfig {
            initial_text: "existing text".to_string(),
            ..Default::default()
        };
        assert_eq!(config.initial_text, "existing text");
    }

    #[test]
    fn default_config_has_font_size() {
        let config = EditorConfig::default();
        assert_eq!(config.font_size, 14.0);
    }

    #[test]
    fn config_with_custom_font_size() {
        let config = EditorConfig {
            font_size: 24.0,
            ..Default::default()
        };
        assert_eq!(config.font_size, 24.0);
    }

    #[test]
    fn font_size_bounds() {
        assert!(FONT_SIZE_MIN > 0.0);
        assert!(FONT_SIZE_MAX > FONT_SIZE_MIN);
        assert!(FONT_SIZE_STEP > 0.0);
    }

    #[test]
    fn font_size_increase_clamped() {
        let size = FONT_SIZE_MAX;
        let new_size = (size + FONT_SIZE_STEP).min(FONT_SIZE_MAX);
        assert_eq!(new_size, FONT_SIZE_MAX);
    }

    #[test]
    fn font_size_decrease_clamped() {
        let size = FONT_SIZE_MIN;
        let new_size = (size - FONT_SIZE_STEP).max(FONT_SIZE_MIN);
        assert_eq!(new_size, FONT_SIZE_MIN);
    }
}
