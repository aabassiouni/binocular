use windows::Win32::Foundation::{BOOL, HWND, LPARAM, TRUE};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowLongPtrW, GetWindowTextW, GetWindowThreadProcessId, IsIconic,
    IsWindowVisible, SendMessageW, SetForegroundWindow, ShowWindow, GWL_EXSTYLE, GWL_STYLE,
    SW_RESTORE, WM_CLOSE, WS_CAPTION, WS_EX_TOOLWINDOW, WS_VISIBLE,
};

use crate::utils::icon;
use crate::utils::process::get_process_name;
use crate::window_manager::{Window, WindowManager};

pub fn focus_window(hwnd: isize) {
    unsafe {
        let hwnd = HWND(hwnd);

        // Restore window if minimized
        if IsIconic(hwnd).as_bool() {
            match ShowWindow(hwnd, SW_RESTORE).as_bool() {
                true => {}
                false => println!("Failed to restore window"),
            }
        }

        // Try to bring it to the foreground
        match SetForegroundWindow(hwnd).as_bool() {
            true => {}
            false => println!("Failed to bring window to the foreground"),
        }
    }
}

pub fn close_window(hwnd: isize) {
    unsafe {
        let hwnd = HWND(hwnd);
        SendMessageW(hwnd, WM_CLOSE, None, None);
    }
}

pub fn get_windows(window_manager: &WindowManager) {
    unsafe {
        // Fetch windows and call the callback function for each window
        match EnumWindows(
            Some(enum_window_proc),
            LPARAM(window_manager as *const _ as isize),
        ) {
            Ok(_) => (),
            Err(e) => println!("Error: {}", e),
        }
    }

    return;
}

unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let window_manager = &mut *(lparam.0 as *mut WindowManager);

    // if the window is the binocular process, skip it
    let mut process_id = 0u32;

    GetWindowThreadProcessId(hwnd, Some(&mut process_id));

    if process_id == window_manager.current_pid {
        return TRUE;
    }

    // if the window is not visible, skip it
    if !IsWindowVisible(hwnd).as_bool() {
        return TRUE;
    }

    let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
    let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);

    if (style & ((WS_VISIBLE | WS_CAPTION).0 as isize)) != ((WS_VISIBLE | WS_CAPTION).0 as isize) {
        return TRUE;
    }

    if (ex_style & (WS_EX_TOOLWINDOW.0 as isize)) != 0 {
        return TRUE;
    }

    // get the window title
    let mut title = [0u16; 512];
    let len = GetWindowTextW(hwnd, &mut title);
    if len == 0 {
        return TRUE;
    }

    let title = String::from_utf16_lossy(&title[..len as usize]);
    let icon_base64 = icon::get_window_icon(hwnd);
    let process_name = get_process_name(process_id);

    window_manager.windows.lock().unwrap().push(Window {
        hwnd: hwnd.0,
        title,
        process_id,
        process_name,
        icon_base64,
    });

    TRUE
}
