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

pub fn run_editor(config: EditorConfig) -> EditorResult {
    use objc2::rc::Retained;
    use objc2::MainThreadOnly;
    use objc2_app_kit::*;
    use objc2_foundation::*;

    unsafe {
        let mtm = MainThreadMarker::new().expect("must be called from main thread");

        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

        let style =
            NSWindowStyleMask::Titled | NSWindowStyleMask::Closable | NSWindowStyleMask::Resizable;

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
        window.setLevel(NSFloatingWindowLevel);
        window.center();

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

        window.setContentView(Some(&scroll_view));
        window.makeKeyAndOrderFront(None);
        window.makeFirstResponder(Some(&text_view));

        app.activate();

        let text_view_ref = text_view.clone();
        let window_ref = window.clone();

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
                    let has_cmd = event
                        .modifierFlags()
                        .contains(NSEventModifierFlags::Command);

                    if has_cmd && keycode == KEYCODE_RETURN {
                        let text = text_view_ref.string().to_string();
                        window_ref.close();
                        return EditorResult::Submitted(text);
                    }

                    if keycode == KEYCODE_ESCAPE {
                        window_ref.close();
                        return EditorResult::Cancelled;
                    }
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
}
