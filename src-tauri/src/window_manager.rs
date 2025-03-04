use serde::Serialize;
use std::fmt;
use std::sync::Mutex;
use windows::core::Error as WindowsError;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, TRUE, WIN32_ERROR};
use windows::Win32::Graphics::Dwm::{
    DwmRegisterThumbnail, DwmUnregisterThumbnail, DwmUpdateThumbnailProperties,
    DWM_THUMBNAIL_PROPERTIES,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowLongW, GetWindowTextW, GetWindowThreadProcessId,
    IsWindowVisible, SetForegroundWindow, GWL_EXSTYLE, GWL_STYLE, WS_CAPTION, WS_EX_TOOLWINDOW,
    WS_VISIBLE,
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
    pub current_pid: u32,
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

    pub fn get_window_thumbnail(
        &self,
        source_hwnd: isize,
        dest_hwnd: isize,
        width: u32,
        height: u32,
    ) -> Result<(), WindowsError> {
        unsafe {
            let mut thumbnail_id = DwmRegisterThumbnail(HWND(dest_hwnd), HWND(source_hwnd))?;

            let props = DWM_THUMBNAIL_PROPERTIES {
                dwFlags: 0x1F,
                rcDestination: windows::Win32::Foundation::RECT {
                    left: 0,
                    top: 0,
                    right: width as i32,
                    bottom: height as i32,
                },
                rcSource: windows::Win32::Foundation::RECT::default(),
                opacity: 255,
                fVisible: true.into(),
                fSourceClientAreaOnly: false.into(),
            };

            DwmUpdateThumbnailProperties(thumbnail_id, &props)?;

            // You'll need to call DwmUnregisterThumbnail(thumbnail_id) when done
            Ok(())
        }
    }

    unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let window_manager = &mut *(lparam.0 as *mut WindowManager);

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

        let mut title = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut title);
        if len == 0 {
            return TRUE;
        }

        let mut process_id = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        if process_id == window_manager.current_pid {
            return TRUE;
        }

        let title = String::from_utf16_lossy(&title[..len as usize]);

        window_manager.windows.lock().unwrap().push(WindowInfo {
            hwnd: hwnd.0,
            title,
            process_id,
        });

        TRUE
    }
}
