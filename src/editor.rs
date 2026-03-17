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
    pub theme: String,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            initial_text: String::new(),
            title: "TermPop".to_string(),
            width: 600.0,
            height: 300.0,
            font_size: 14.0,
            theme: "dark".to_string(),
        }
    }
}

const KEYCODE_RETURN: u16 = 36;
const KEYCODE_ESCAPE: u16 = 53;
const KEYCODE_EQUAL: u16 = 0x18;
const KEYCODE_MINUS: u16 = 0x1B;
const KEYCODE_ZERO: u16 = 0x1D;
const KEYCODE_T: u16 = 0x11;
const KEYCODE_Z: u16 = 0x06;
const FONT_SIZE_MIN: f64 = 8.0;
const FONT_SIZE_MAX: f64 = 72.0;
const FONT_SIZE_STEP: f64 = 2.0;

struct ThemeColors {
    bg: (f64, f64, f64),
    fg: (f64, f64, f64),
    hint: (f64, f64, f64),
    cursor: (f64, f64, f64),
    border: (f64, f64, f64),
    title: (f64, f64, f64),
}

fn theme_colors(is_dark: bool) -> ThemeColors {
    if is_dark {
        ThemeColors {
            bg: (0.157, 0.173, 0.204),
            fg: (0.671, 0.698, 0.749),
            hint: (0.361, 0.388, 0.439),
            cursor: (0.322, 0.545, 1.0),
            border: (0.094, 0.102, 0.122),
            title: (0.922, 0.933, 0.945),  // near-white for dark theme
        }
    } else {
        ThemeColors {
            bg: (0.98, 0.98, 0.98),
            fg: (0.220, 0.227, 0.259),
            hint: (0.627, 0.631, 0.655),
            cursor: (0.322, 0.435, 1.0),
            border: (0.859, 0.859, 0.863),
            title: (0.133, 0.137, 0.161),  // near-black for light theme
        }
    }
}

fn save_font_size(font_size: f64) {
    let mut cfg = crate::config::load_config();
    cfg.font_size = font_size;
    if let Err(e) = crate::config::save_config(&cfg) {
        eprintln!("failed to save font size: {}", e);
    }
}

fn save_theme(theme: &str) {
    let mut cfg = crate::config::load_config();
    cfg.theme = theme.to_string();
    if let Err(e) = crate::config::save_config(&cfg) {
        eprintln!("failed to save theme: {}", e);
    }
}

fn build_hint_attributed_string(is_dark: bool) -> objc2::rc::Retained<objc2_foundation::NSMutableAttributedString> {
    use objc2::AnyThread;
    use objc2_app_kit::*;
    use objc2_foundation::*;

    let colors = theme_colors(is_dark);
    let hint_font = NSFont::systemFontOfSize(11.0);
    let hint_bold_font = NSFont::boldSystemFontOfSize(11.0);
    let hint_color =
        NSColor::colorWithSRGBRed_green_blue_alpha(colors.hint.0, colors.hint.1, colors.hint.2, 1.0);
    let title_color =
        NSColor::colorWithSRGBRed_green_blue_alpha(colors.title.0, colors.title.1, colors.title.2, 1.0);

    let title_part = "TermPop";
    let rest_part = "  │  Enter: New line  │  ⌘+Enter: Submit  │  Esc: Cancel  │  ⌃+/⌃-: Font  │  ⌃T: Theme";
    let full_hint = format!("{}{}", title_part, rest_part);

    unsafe {
        let attr_str = NSMutableAttributedString::initWithString(
            NSMutableAttributedString::alloc(),
            &NSString::from_str(&full_hint),
        );
        let ns_title_len = NSString::from_str(title_part).length();
        let full_len = NSString::from_str(&full_hint).length();
        let title_range = NSRange::new(0, ns_title_len as usize);
        let rest_range = NSRange::new(ns_title_len as usize, (full_len - ns_title_len) as usize);

        attr_str.addAttribute_value_range(NSForegroundColorAttributeName, &title_color, title_range);
        attr_str.addAttribute_value_range(NSFontAttributeName, &hint_bold_font, title_range);
        attr_str.addAttribute_value_range(NSForegroundColorAttributeName, &hint_color, rest_range);
        attr_str.addAttribute_value_range(NSFontAttributeName, &hint_font, rest_range);
        attr_str
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

        const WINDOW_BOTTOM_MARGIN: f64 = 80.0;

        // 포커스된 앱 윈도우 하단 중앙에 팝업 배치, 실패 시 마우스 커서 폴백
        // CGWindowList는 CG 좌표(top-left origin), setFrameTopLeftPoint는 NS 좌표(bottom-left origin)
        // 변환: ns_y = primary_screen_height - cg_y
        let primary_screen_h = {
            let screens = NSScreen::screens(mtm);
            if screens.count() > 0 {
                let primary: Retained<NSScreen> = screens.objectAtIndex(0);
                primary.frame().size.height
            } else {
                900.0
            }
        };

        let top_left = if let Some((wx, wy, ww, wh)) = crate::ax_position::get_frontmost_window_bounds() {
            // 윈도우 하단에서 마진만큼 위, 수평 중앙
            let popup_x = wx + (ww / 2.0) - (config.width / 2.0);
            let cg_y = wy + wh - WINDOW_BOTTOM_MARGIN;
            let ns_y = primary_screen_h - cg_y;
            eprintln!("[termpop] window bounds: ({}, {}, {}, {}), primary_h={}, popup=({}, {})", wx, wy, ww, wh, primary_screen_h, popup_x, ns_y);
            NSPoint::new(popup_x, ns_y)
        } else {
            let mouse_pos = NSEvent::mouseLocation();
            eprintln!("[termpop] fallback mouse: ({}, {})", mouse_pos.x, mouse_pos.y);
            NSPoint::new(mouse_pos.x, mouse_pos.y)
        };
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
        text_view.setAllowsUndo(true);

        let font = NSFont::monospacedSystemFontOfSize_weight(config.font_size, NSFontWeightRegular);
        text_view.setFont(Some(&font));

        let mut is_dark = config.theme != "light";
        let colors = theme_colors(is_dark);

        let app_name = if is_dark {
            NSAppearanceNameDarkAqua
        } else {
            NSAppearanceNameAqua
        };
        if let Some(app_appearance) = NSAppearance::appearanceNamed(app_name) {
            window.setAppearance(Some(&app_appearance));
        }

        let bg =
            NSColor::colorWithSRGBRed_green_blue_alpha(colors.bg.0, colors.bg.1, colors.bg.2, 1.0);
        let fg =
            NSColor::colorWithSRGBRed_green_blue_alpha(colors.fg.0, colors.fg.1, colors.fg.2, 1.0);
        let cursor_color = NSColor::colorWithSRGBRed_green_blue_alpha(
            colors.cursor.0,
            colors.cursor.1,
            colors.cursor.2,
            1.0,
        );

        window.setBackgroundColor(Some(&bg));
        text_view.setBackgroundColor(&bg);
        text_view.setTextColor(Some(&fg));
        text_view.setInsertionPointColor(Some(&cursor_color));

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
            &NSString::from_str(""),
            mtm,
        );
        hint_label.setFrame(hint_frame);
        hint_label.setEditable(false);
        hint_label.setSelectable(false);
        hint_label.setBordered(false);
        hint_label.setDrawsBackground(false);

        hint_label.setAttributedStringValue(&build_hint_attributed_string(is_dark));

        hint_label.setAutoresizingMask(NSAutoresizingMaskOptions::ViewWidthSizable);

        container.addSubview(&scroll_view);
        container.addSubview(&hint_label);

        container.setWantsLayer(true);
        if let Some(layer) = container.layer() {
            layer.setCornerRadius(12.0);
            layer.setMasksToBounds(true);
            layer.setBorderWidth(1.5);
            let border_ns = NSColor::colorWithSRGBRed_green_blue_alpha(
                colors.border.0,
                colors.border.1,
                colors.border.2,
                1.0,
            );
            layer.setBorderColor(Some(&border_ns.CGColor()));
        }

        window.setContentView(Some(&container));
        window.makeKeyAndOrderFront(None);
        window.makeFirstResponder(Some(&text_view));

        app.activate();

        let text_view_ref = text_view.clone();
        let window_ref = window.clone();
        let hint_label_ref = hint_label.clone();
        let container_ref = container.clone();
        let default_font_size = config.font_size;
        let mut current_font_size = config.font_size;

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
                    let has_shift = flags.contains(NSEventModifierFlags::Shift);

                    if has_cmd && keycode == KEYCODE_Z {
                        if let Some(undo_manager) = text_view_ref.undoManager() {
                            if has_shift {
                                if undo_manager.canRedo() {
                                    undo_manager.redo();
                                }
                            } else {
                                if undo_manager.canUndo() {
                                    undo_manager.undo();
                                }
                            }
                        }
                        continue;
                    }

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
                            "Font size: {:.0}pt",
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
                            "Font size: {:.0}pt",
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
                            "Font size: {:.0}pt (default)",
                            current_font_size
                        )));
                        save_font_size(current_font_size);
                        continue;
                    }

                    if has_ctrl && keycode == KEYCODE_T {
                        is_dark = !is_dark;
                        let c = theme_colors(is_dark);

                        let toggle_app_name = if is_dark {
                            NSAppearanceNameDarkAqua
                        } else {
                            NSAppearanceNameAqua
                        };
                        if let Some(toggle_appearance) =
                            NSAppearance::appearanceNamed(toggle_app_name)
                        {
                            window_ref.setAppearance(Some(&toggle_appearance));
                        }

                        let new_bg =
                            NSColor::colorWithSRGBRed_green_blue_alpha(c.bg.0, c.bg.1, c.bg.2, 1.0);
                        let new_fg =
                            NSColor::colorWithSRGBRed_green_blue_alpha(c.fg.0, c.fg.1, c.fg.2, 1.0);
                        let new_cursor = NSColor::colorWithSRGBRed_green_blue_alpha(
                            c.cursor.0, c.cursor.1, c.cursor.2, 1.0,
                        );
                        let new_border = NSColor::colorWithSRGBRed_green_blue_alpha(
                            c.border.0, c.border.1, c.border.2, 1.0,
                        );

                        window_ref.setBackgroundColor(Some(&new_bg));
                        text_view_ref.setBackgroundColor(&new_bg);
                        text_view_ref.setTextColor(Some(&new_fg));
                        text_view_ref.setInsertionPointColor(Some(&new_cursor));

                        if let Some(layer) = container_ref.layer() {
                            layer.setBorderColor(Some(&new_border.CGColor()));
                        }

                        let theme_name = if is_dark { "dark" } else { "light" };
                        hint_label_ref.setAttributedStringValue(&build_hint_attributed_string(is_dark));
                        save_theme(theme_name);
                        continue;
                    }

                    hint_label_ref.setAttributedStringValue(&build_hint_attributed_string(is_dark));
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
