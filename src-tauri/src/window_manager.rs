use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;
use std::fmt;
use std::sync::Mutex;
use tauri::image::Image;
use windows::core::Error as WindowsError;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, TRUE, WIN32_ERROR, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DwmRegisterThumbnail, DwmUnregisterThumbnail, DwmUpdateThumbnailProperties,
    DWM_THUMBNAIL_PROPERTIES,
};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, SelectObject, COLOR_GRADIENTACTIVECAPTION, SRCCOPY
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassLongPtrW, GetWindowLongPtrW, GetWindowTextW, GetWindowThreadProcessId,
    IsWindowVisible, SendMessageW, SetForegroundWindow, GCLP_HICON, GCLP_HICONSM, GWL_EXSTYLE,
    GWL_STYLE, HICON, ICON_BIG, ICON_SMALL, WM_GETICON, WS_CAPTION, WS_EX_TOOLWINDOW, WS_VISIBLE,
};

#[derive(Debug)]
pub enum WindowError {
    EnumWindowsFailed(WindowsError),
    SetForegroundFailed(WindowsError),
    GetIconFailed(WindowsError),
    IconExtractionFailed,
}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WindowError::EnumWindowsFailed(e) => write!(f, "Failed to enumerate windows: {}", e),
            WindowError::SetForegroundFailed(e) => {
                write!(f, "Failed to set foreground window: {}", e)
            }
            WindowError::GetIconFailed(e) => write!(f, "Failed to get window icon: {}", e),
            WindowError::IconExtractionFailed => write!(f, "Failed to extract window icon"),
        }
    }
}

impl std::error::Error for WindowError {}

#[derive(Debug, Serialize, Clone)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub process_id: u32,
    pub icon_base64: Option<String>,
}

pub struct WindowManager {
    pub windows: Mutex<Vec<WindowInfo>>,
    pub current_pid: u32,
}

impl WindowManager {
    pub fn refresh_window_list(&self) -> Result<(), WindowError> {
        self.windows.lock().unwrap().clear();

        unsafe {
            // Fetch windows and call the callback function for each window
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

    unsafe fn get_window_icon(hwnd: HWND) -> Option<String> {
        // Try different methods to get the icon
        let mut h_icon = HICON(SendMessageW(hwnd, WM_GETICON, WPARAM(ICON_SMALL as usize), LPARAM(0)).0);
        if h_icon.0 == 0 {
            h_icon = HICON(SendMessageW(hwnd, WM_GETICON, WPARAM(ICON_BIG as usize), LPARAM(0)).0);
        }

        if h_icon.0 == 0 {
            h_icon = HICON(GetClassLongPtrW(hwnd, GCLP_HICONSM) as isize);
        }

        if h_icon.0 == 0 {
            h_icon = HICON(GetClassLongPtrW(hwnd, GCLP_HICON) as isize);
        }

        if h_icon.0 == 0 {
            return None;
        }

        // Convert icon to bitmap and then to base64
        let icon_size = 16; // Small icon size
        let hdc = GetDC(hwnd);
        let hdc_mem = CreateCompatibleDC(hdc);
        let h_bitmap = CreateCompatibleBitmap(hdc, icon_size, icon_size);
        let old_bitmap = SelectObject(hdc_mem, h_bitmap);

        // Draw icon to bitmap
        let result = windows::Win32::UI::WindowsAndMessaging::DrawIconEx(
            hdc_mem,
            0,
            0,
            h_icon,
            icon_size,
            icon_size,
            0,
            None,
            windows::Win32::UI::WindowsAndMessaging::DI_NORMAL,
        );

        if !result.is_ok() {
            // Clean up and return None if drawing failed
            SelectObject(hdc_mem, old_bitmap);
            DeleteObject(h_bitmap);
            DeleteDC(hdc_mem);
            return None;
        }

        // Create buffer for bitmap data
        let buffer_size = (icon_size * icon_size * 4) as usize;
        let mut buffer = vec![0u8; buffer_size];

        // Get bitmap data
        let mut bitmap_info = windows::Win32::Graphics::Gdi::BITMAPINFO {
            bmiHeader: windows::Win32::Graphics::Gdi::BITMAPINFOHEADER {
                biSize: std::mem::size_of::<windows::Win32::Graphics::Gdi::BITMAPINFOHEADER>()
                    as u32,
                biWidth: icon_size,
                biHeight: -icon_size, // Negative for top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: windows::Win32::Graphics::Gdi::BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [windows::Win32::Graphics::Gdi::RGBQUAD::default()],
        };

        let result = windows::Win32::Graphics::Gdi::GetDIBits(
            hdc_mem,
            h_bitmap,
            0,
            icon_size as u32,
            Some(buffer.as_mut_ptr() as *mut std::ffi::c_void),
            &mut bitmap_info as *mut _,
            windows::Win32::Graphics::Gdi::DIB_RGB_COLORS,
        );

        // Clean up
        SelectObject(hdc_mem, old_bitmap);
        DeleteObject(h_bitmap);
        DeleteDC(hdc_mem);

        if result == 0 {
            return None;
        }

        // Encode as PNG and convert to base64
        let png_data = match image::RgbaImage::from_raw(
            icon_size as u32,
            icon_size as u32,
            buffer,
        ) {
            Some(img) => {
                let mut cursor = std::io::Cursor::new(Vec::new());
                if let Ok(_) = img.write_to(&mut cursor, image::ImageOutputFormat::Png) {
                    cursor.into_inner()
                } else {
                    return None;
                }
            }
            None => return None,
        };

        // Convert to base64
        let base64_string = general_purpose::STANDARD.encode(png_data);
        println!("{}", base64_string);
        Some(format!("data:image/png;base64,{}", base64_string))
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

        if (style & ((WS_VISIBLE | WS_CAPTION).0 as isize))
            != ((WS_VISIBLE | WS_CAPTION).0 as isize)
        {
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
        let icon_base64 = Self::get_window_icon(hwnd);

        window_manager.windows.lock().unwrap().push(WindowInfo {
            hwnd: hwnd.0,
            title,
            process_id,
            icon_base64,
        });

        TRUE
    }
}
