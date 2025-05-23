mod utils;
mod window;
mod window_manager;

use std::process;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use utils::{
    display::center_window_in_display,
    setup::{setup_autostart, setup_window_event_listener},
};
use window_manager::{Window, WindowManager};

fn get_windows(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<WindowManager>();

    state.inner().refresh_window_list();

    app.emit("windows-updated", state.windows.lock().unwrap().clone())
        .unwrap();

    Ok(())
}

#[tauri::command]
fn focus_window(app_handle: tauri::AppHandle, window: Window) {
    let main_window = app_handle.get_webview_window("main").unwrap();
    main_window.hide().unwrap();
    window.focus_window();
}

#[tauri::command]
fn close_window(window: Window) {
    window.close_window();
}

fn disable_dev_tools_in_dev() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    use tauri_plugin_prevent_default::Flags;

    tauri_plugin_prevent_default::Builder::new()
        .with_flags(Flags::all().difference({
            #[cfg(debug_assertions)]
            {
                Flags::DEV_TOOLS | Flags::RELOAD
            }
            #[cfg(not(debug_assertions))]
            {
                Flags::empty()
            }
        }))
        .build()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let current_pid = process::id();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(disable_dev_tools_in_dev())
        .manage(WindowManager {
            windows: Default::default(),
            current_pid,
        })
        .setup(|app| {
            setup_autostart(app);
            setup_window_event_listener(app);

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

                                        if let Err(e) = center_window_in_display(&main_window) {
                                            println!("Error centering window: {e}");
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
        .invoke_handler(tauri::generate_handler![focus_window, close_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
