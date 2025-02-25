use serde::Serialize;
use std::fmt;
use std::sync::Mutex;
use windows::core::Error as WindowsError;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, TRUE, WIN32_ERROR};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
    SetForegroundWindow,
};

#[derive(Debug)]
pub enum WindowError {
    EnumWindowsFailed(WindowsError),
    SetForegroundFailed(WindowsError),
}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WindowError::EnumWindowsFailed(e) => write!(f, "Failed to enumerate windows: {}", e),
            WindowError::SetForegroundFailed(e) => {
                write!(f, "Failed to set foreground window: {}", e)
            }
        }
    }
}

impl std::error::Error for WindowError {}

#[derive(Debug, Serialize, Clone)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub process_id: u32,
}

pub struct WindowManager {
    pub windows: Mutex<Vec<WindowInfo>>,
}

impl WindowManager {
    pub fn refresh_window_list(&self) -> Result<(), WindowError> {
        self.windows.lock().unwrap().clear();

        unsafe {
            EnumWindows(
                Some(Self::enum_window_proc),
                LPARAM(self as *const _ as isize),
            )
            .map_err(WindowError::EnumWindowsFailed)?;
        }

        Ok(())
    }

    pub fn get_windows(&self) -> &Mutex<Vec<WindowInfo>> {
        &self.windows
    }

    pub fn focus_window(&self, hwnd: isize) -> Result<(), ()> {
        unsafe {
            if SetForegroundWindow(HWND(hwnd)).as_bool() {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let window_manager = &mut *(lparam.0 as *mut WindowManager);

        if !IsWindowVisible(hwnd).as_bool() {
            return TRUE;
        }

        let mut title = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut title);
        if len == 0 {
            // Skip if GetWindowTextW fails or window has no title
            return TRUE;
        }

        let mut process_id = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        let title = String::from_utf16_lossy(&title[..len as usize]);

        window_manager.windows.lock().unwrap().push(WindowInfo {
            hwnd: hwnd.0,
            title,
            process_id,
        });

        TRUE
    }
}
