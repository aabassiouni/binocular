use crate::window_manager::WindowManager;
use ::windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, GetWindowLongPtrW, RegisterClassW,
            RegisterShellHookWindow, RegisterWindowMessageW, SetWindowLongPtrW, CS_HREDRAW,
            CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, HSHELL_WINDOWCREATED, HSHELL_WINDOWDESTROYED,
            HWND_MESSAGE, WNDCLASSW, WS_OVERLAPPEDWINDOW,
        },
    },
};
use std::mem::zeroed;
use tauri::{Emitter, Manager};

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    static mut SHELL_MESSAGE: u32 = 0;

    let app_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut tauri::App;
    if !app_ptr.is_null() {
        let app = app_ptr.as_mut().unwrap();
        let state = app.state::<WindowManager>();

        // Initialize the shell message ID if we haven't yet
        if SHELL_MESSAGE == 0 {
            SHELL_MESSAGE = RegisterWindowMessageW(w!("SHELLHOOK"));
        }

        if msg == SHELL_MESSAGE {
            let event = wparam.0 as u32;

            // Only process visible windows
            match event {
                HSHELL_WINDOWCREATED => {
                    state.inner().refresh_window_list();

                    app.emit("windows-updated", state.windows.lock().unwrap().clone())
                        .unwrap();
                }
                HSHELL_WINDOWDESTROYED => {
                    state.inner().refresh_window_list();

                    app.emit("windows-updated", state.windows.lock().unwrap().clone())
                        .unwrap();
                }
                _ => {}
            }
        }
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}

pub fn setup_window_event_listener(app: &tauri::App) {
    unsafe {
        let class_name = w!("ShellHookWindow");
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            hInstance: HINSTANCE(0),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..zeroed()
        };

        RegisterClassW(&wc);

        // Create a message-only window
        let hwnd = CreateWindowExW(
            Default::default(),
            class_name,
            w!("Binocular Shell Hook Window"),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            HWND_MESSAGE,
            None,
            None,
            None,
        );

        // Pass app to window_proc
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, app as *const _ as isize);

        // Register for shell hooks
        RegisterShellHookWindow(hwnd);
    }
}

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
