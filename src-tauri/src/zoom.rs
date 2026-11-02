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
    // Zoom does not expose a stable public call-state API on macOS. Its meeting
    // windows are exposed through System Events; querying them requires the
    // app to be allowed under macOS Accessibility/Automation privacy settings.
    let script = r#"tell application "System Events"
        if not (exists process "zoom.us") then return ""
        tell process "zoom.us"
            set windowNames to name of every window
        end tell
        return windowNames as text
    end tell"#;
    let output = std::process::Command::new("/usr/bin/osascript")
        .args(["-e", script])
        .output();
    let Ok(output) = output else { return false };
    if !output.status.success() { return false }
    String::from_utf8_lossy(&output.stdout)
        .split(',')
        .any(is_zoom_meeting_window_title)
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
