use tauri::{PhysicalPosition, Position, WebviewWindow};
use windows::Win32::{
    Foundation::RECT,
    Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST},
    UI::WindowsAndMessaging::GetForegroundWindow,
};

pub fn center_window_in_display(main_window: &WebviewWindow) -> Result<(), String> {
    unsafe {
        // Get the currently active window
        let foreground_window = GetForegroundWindow();
        if foreground_window.0 == 0 {
            return Err("Failed to get foreground window".to_string());
        }

        // Get the monitor that contains this window
        let monitor = MonitorFromWindow(foreground_window, MONITOR_DEFAULTTONEAREST);
        if monitor.is_invalid() {
            return Err("Failed to get monitor".to_string());
        }
        // Get monitor information
        let mut monitor_info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            rcMonitor: RECT::default(),
            rcWork: RECT::default(),
            dwFlags: 0,
        };

        if GetMonitorInfoW(monitor, &mut monitor_info).as_bool() {
            let center_x = (monitor_info.rcMonitor.left + monitor_info.rcMonitor.right) / 2;
            let center_y = (monitor_info.rcMonitor.top + monitor_info.rcMonitor.bottom) / 2;

            main_window
                .set_position(Position::Physical(PhysicalPosition::new(
                    center_x, center_y,
                )))
                .unwrap();
            main_window.center().unwrap();
        }

        Ok(())
    }
}
