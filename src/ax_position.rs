//! 포커스된 앱의 윈도우 프레임을 가져온다.
//! CGWindowListCopyWindowInfo를 사용하여 CG 좌표(top-left origin)로 윈도우 bounds를 얻는다.

use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use core_graphics::display::{
    kCGNullWindowID, kCGWindowListExcludeDesktopElements, kCGWindowListOptionOnScreenOnly,
    CGWindowListCopyWindowInfo,
};
use std::ffi::c_void;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGRectMakeWithDictionaryRepresentation(
        dict: *const c_void,
        rect: *mut core_graphics::geometry::CGRect,
    ) -> bool;
}

/// frontmost 앱의 메인 윈도우 bounds를 CG 좌표(top-left origin)로 반환한다.
/// 반환: (x, y, width, height)
pub fn get_frontmost_window_bounds() -> Option<(f64, f64, f64, f64)> {
    let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
    let front_app = workspace.frontmostApplication()?;
    let pid = front_app.processIdentifier();

    unsafe {
        let info_list = CGWindowListCopyWindowInfo(
            kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
            kCGNullWindowID,
        );
        if info_list.is_null() {
            return None;
        }

        let count = core_foundation::array::CFArrayGetCount(info_list);

        let pid_key = CFString::new("kCGWindowOwnerPID");
        let bounds_key = CFString::new("kCGWindowBounds");
        let layer_key = CFString::new("kCGWindowLayer");

        for i in 0..count {
            let dict_ref = core_foundation::array::CFArrayGetValueAtIndex(info_list, i);
            if dict_ref.is_null() {
                continue;
            }

            // PID 매칭
            let pid_ref = core_foundation::dictionary::CFDictionaryGetValue(
                dict_ref as _,
                pid_key.as_concrete_TypeRef() as *const c_void,
            );
            if pid_ref.is_null() {
                continue;
            }
            let mut win_pid: i32 = 0;
            if !core_foundation::number::CFNumberGetValue(
                pid_ref as _,
                core_foundation::number::kCFNumberSInt32Type,
                &mut win_pid as *mut _ as *mut c_void,
            ) {
                continue;
            }
            if win_pid != pid {
                continue;
            }

            // layer 0 = 일반 윈도우
            let layer_ref = core_foundation::dictionary::CFDictionaryGetValue(
                dict_ref as _,
                layer_key.as_concrete_TypeRef() as *const c_void,
            );
            if !layer_ref.is_null() {
                let mut layer: i32 = -1;
                core_foundation::number::CFNumberGetValue(
                    layer_ref as _,
                    core_foundation::number::kCFNumberSInt32Type,
                    &mut layer as *mut _ as *mut c_void,
                );
                if layer != 0 {
                    continue;
                }
            }

            // bounds 파싱
            let bounds_ref = core_foundation::dictionary::CFDictionaryGetValue(
                dict_ref as _,
                bounds_key.as_concrete_TypeRef() as *const c_void,
            );
            if bounds_ref.is_null() {
                continue;
            }

            let mut rect = core_graphics::geometry::CGRect::new(
                &core_graphics::geometry::CGPoint::new(0.0, 0.0),
                &core_graphics::geometry::CGSize::new(0.0, 0.0),
            );
            if CGRectMakeWithDictionaryRepresentation(bounds_ref, &mut rect) {
                core_foundation::base::CFRelease(info_list as _);
                return Some((
                    rect.origin.x,
                    rect.origin.y,
                    rect.size.width,
                    rect.size.height,
                ));
            }
        }

        core_foundation::base::CFRelease(info_list as _);
    }

    None
}
