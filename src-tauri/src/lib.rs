mod window_manager;

use std::process;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, State,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use window_manager::WindowManager;

fn get_windows(app: &AppHandle) -> Result<(), String> {
    println!("getting windows");

    let state = app.state::<WindowManager>();

    state
        .inner()
        .refresh_window_list()
        .map_err(|e| e.to_string())?;

    app.emit("windows-updated", state.windows.lock().unwrap().clone())
        .unwrap();

    Ok(())
}

#[tauri::command]
fn focus_window(app_handle: tauri::AppHandle, state: State<WindowManager>, hwnd: isize) {
    let main_window = app_handle.get_webview_window("main").unwrap();
    main_window.hide().unwrap();
    state.inner().focus_window(hwnd)
}

#[tauri::command]
fn hide_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    let main_window = app_handle.get_webview_window("main").unwrap();
    main_window.hide().unwrap();
    Ok(())
}

fn prevent_default() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    use tauri_plugin_prevent_default::Flags;

    tauri_plugin_prevent_default::Builder::new()
        .with_flags(Flags::all().difference(
            #[cfg(debug_assertions)]
            {
                Flags::DEV_TOOLS | Flags::RELOAD
            },
        ))
        .build()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let current_pid = process::id();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(prevent_default())
        .manage(WindowManager {
            windows: Default::default(),
            current_pid,
        })
        .setup(|app| {
            // Only enable autostart in release builds
            #[cfg(not(debug_assertions))]
            {
                use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

                app.handle().plugin(tauri_plugin_autostart::init(
                    MacosLauncher::LaunchAgent,
                    Some(vec!["--flag1", "--flag2"]),
                ))?;

                // Get the autostart manager
                let autostart_manager = app.autolaunch();
                // Enable autostart
                let _ = autostart_manager.enable();
                // Check enable state
                println!(
                    "registered for autostart? {}",
                    autostart_manager.is_enabled().unwrap()
                );
            }

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;

            let product_name = app.config().product_name.clone().unwrap();

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip(product_name)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        println!("quit menu item was clicked");
                        app.exit(0);
                    }
                    _ => {
                        println!("menu item {:?} not handled", event.id);
                    }
                })
                .build(app)?;

            let ctrl_n_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::KeyM);

            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |app, shortcut, event| {
                        let main_window = app.get_webview_window("main").unwrap();

                        if shortcut == &ctrl_n_shortcut {
                            match event.state() {
                                ShortcutState::Pressed => match main_window.is_visible() {
                                    Ok(true) => {
                                        main_window.hide().expect("failed to hide window");
                                    }
                                    Ok(false) => {
                                        if let Err(e) = get_windows(app) {
                                            println!("Error refreshing window list: {e}");
                                        }
                                        if let Err(e) = main_window.show() {
                                            println!("Error showing window: {e}");
                                        }
                                        if let Err(e) = main_window.set_focus() {
                                            println!("Error focusing window: {e}");
                                        }
                                    }
                                    Err(e) => {
                                        println!("Error checking window visibility: {e}");
                                    }
                                },
                                _ => {}
                            }
                        }
                    })
                    .build(),
            )?;

            match app.global_shortcut().register(ctrl_n_shortcut) {
                Ok(_) => println!("registered global shortcut"),
                Err(e) => println!("failed to register global shortcut: {e}"),
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![focus_window, hide_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
