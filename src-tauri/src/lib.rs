use tauri::Manager;

#[tauri::command]
fn get_windows() -> Vec<String> {
    println!("getting windows");

    let mut window_titles = Vec::new();

    window_titles.push("window 1".to_string());
    window_titles.push("window 2".to_string());

    window_titles
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();

            main_window.center().expect("failed to center window");

            main_window
                .set_title("ali bassiouni")
                .expect("failed to set title");
            main_window
                .set_closable(false)
                .expect("failed to set closable");

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_windows])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
