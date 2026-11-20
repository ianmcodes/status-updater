/// Returns whether a visible window title belongs to an active Zoom meeting.
/// Zoom's main window may remain open while idle, so it is deliberately not
/// treated as a call.
pub fn is_zoom_meeting_window_title(title: &str) -> bool {
    let title = title.trim().to_ascii_lowercase();
    title.contains("zoom meeting") || title.contains("zoom webinar")
}

#[cfg(target_os = "windows")]
pub fn is_on_zoom_call() -> bool {
    use windows::core::BOOL;
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible};

    unsafe extern "system" fn check_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }

        let length = GetWindowTextLengthW(hwnd);
        if length <= 0 {
            return BOOL(1);
        }

        let mut buffer = vec![0u16; length as usize + 1];
        let copied = GetWindowTextW(hwnd, &mut buffer);
        let title = String::from_utf16_lossy(&buffer[..copied as usize]);
        if super::zoom::is_zoom_meeting_window_title(&title) {
            *(lparam.0 as *mut bool) = true;
            return BOOL(0);
        }

        BOOL(1)
    }

    let mut found = false;
    unsafe {
        let _ = EnumWindows(Some(check_window), LPARAM(&mut found as *mut bool as isize));
    }
    found
}

#[cfg(target_os = "macos")]
pub fn is_on_zoom_call() -> bool {
    // Use CoreGraphics to query windows across all spaces/desktops
    // This avoids the limitation of AppleScript/System Events which only
    // see windows on the current desktop
    use core_foundation::array::CFArray;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::base::TCFType;
    use core_foundation::string::CFString;
    
    extern "C" {
        // CGWindowListCopyWindowInfo gets all windows including those on other spaces
        fn CGWindowListCopyWindowInfo(option: u32, relativeToWindow: u32) -> *mut std::ffi::c_void;
    }
    
    const CGWINDOW_LIST_OPTION_ALL: u32 = 0;
    
    unsafe {
        let window_list = CGWindowListCopyWindowInfo(CGWINDOW_LIST_OPTION_ALL, 0);
        if window_list.is_null() {
            return false;
        }
        
        let windows = CFArray::<CFDictionary>::wrap_under_create_rule(window_list as *mut _);
        
        for i in 0..windows.len() {
            if let Some(window_dict) = windows.get(i) {
                // Try to get the window name
                let name_key = CFString::new("kCGWindowName");
                // The dictionary returns ItemRef to a *const c_void
                let name_value_ref = window_dict.get(name_key.as_concrete_TypeRef() as *const _);
                // ItemRef dereferences to the actual pointer
                let name_ptr = *name_value_ref as *mut std::ffi::c_void;
                if !name_ptr.is_null() {
                    let name_str = CFString::wrap_under_create_rule(name_ptr as *mut _);
                    let title = name_str.to_string();
                    if is_zoom_meeting_window_title(&title) {
                        return true;
                    }
                }
            }
        }
    }
    
    false
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn is_on_zoom_call() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_zoom_meeting_and_webinar_windows() {
        assert!(is_zoom_meeting_window_title("Zoom Meeting"));
        assert!(is_zoom_meeting_window_title("Quarterly review - Zoom Webinar"));
    }

    #[test]
    fn ignores_idle_zoom_and_other_windows() {
        assert!(!is_zoom_meeting_window_title("Zoom Workplace"));
        assert!(!is_zoom_meeting_window_title("Microsoft Teams"));
    }
}
