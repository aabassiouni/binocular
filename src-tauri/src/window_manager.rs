use crate::utils::icon;
use crate::utils::process::get_process_name;
use crate::window;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Window {
    pub hwnd: isize,
    pub title: String,
    pub process_id: u32,
    pub process_name: Option<String>,
    pub icon_base64: Option<String>,
}

impl Window {
    pub fn focus_window(&self) {
        #[cfg(target_os = "windows")]
        window::focus_window(self.hwnd);
    }

    pub fn close_window(&self) {
        #[cfg(target_os = "windows")]
        window::close_window(self.hwnd);
    }
}
pub struct WindowManager {
    pub windows: Mutex<Vec<Window>>,
    pub current_pid: u32,
}

impl WindowManager {
    pub fn refresh_window_list(&self) {
        self.windows.lock().unwrap().clear();

        window::get_windows(self);
    }
}
