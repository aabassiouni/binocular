mod window_manager;

use std::process;
use tauri::{App, Manager, State};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use window_manager::{WindowInfo, WindowManager};

#[tauri::command]
fn get_windows(state: State<WindowManager>) -> Result<Vec<WindowInfo>, String> {
    println!("getting windows");

    state
        .inner()
        .refresh_window_list()
        .map_err(|e| e.to_string())?;

    state
        .get_windows()
        .lock()
        .unwrap()
        .iter()
        .for_each(|window| {
            println!("window: {:?}", window);
            // window_titles.push(window.clone());
        });

    Ok(state.windows.lock().unwrap().clone())
    // window_titles
    // Err("not implemented".to_string())
    //
}

#[tauri::command]
fn focus_window(state: State<WindowManager>, hwnd: isize) -> Result<(), String> {
    state.inner().focus_window(hwnd).unwrap();
    Ok(())
}

fn setup_main_window(app: &App) {
    let main_window = app.get_webview_window("main").unwrap();

    main_window.center().expect("failed to center window");

    main_window
        .set_decorations(false)
        .expect("failed to set decorations");

    main_window
        .set_skip_taskbar(true)
        .expect("failed to set skip taskbar");

    main_window
        .set_always_on_top(true)
        .expect("failed to set always on top");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let current_pid = process::id();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(WindowManager {
            windows: Default::default(),
            current_pid,
        })
        .setup(|app| {
            setup_main_window(app);
            let ctrl_n_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::KeyM);

            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |app, shortcut, event| {
                        let main_window = app.get_webview_window("main").unwrap();

                        // println!("{:?}", shortcut);
                        if shortcut == &ctrl_n_shortcut {
                            match event.state() {
                                ShortcutState::Pressed => {
                                    if main_window.is_visible().unwrap() {
                                        println!("hiding");
                                        main_window.hide().unwrap();
                                    } else {
                                        println!("showing");
                                        main_window.show().unwrap();
                                    }
                                }
                                ShortcutState::Released => {
                                    println!("Ctrl-N Released!");
                                }
                            }
                        }
                    })
                    .build(),
            )?;

            app.global_shortcut().register(ctrl_n_shortcut).unwrap();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_windows, focus_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
