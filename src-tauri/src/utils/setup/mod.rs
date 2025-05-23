#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

pub fn setup_autostart(app: &tauri::App) {
    // Only enable autostart in release builds
    #[cfg(not(debug_assertions))]
    {
        use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

        app.handle()
            .plugin(tauri_plugin_autostart::init(
                MacosLauncher::LaunchAgent,
                Some(vec!["--flag1", "--flag2"]),
            ))
            .unwrap();

        // Get the autostart manager
        let autostart_manager = app.autolaunch();
        // Enable autostart
        let _ = autostart_manager.enable();
        // Check enable state
        match autostart_manager.is_enabled() {
            Ok(enabled) => {
                println!("Autostart enabled: {}", enabled);
            }
            Err(e) => {
                println!("failed to enable autostart: {}", e);
            }
        }
    }
}
