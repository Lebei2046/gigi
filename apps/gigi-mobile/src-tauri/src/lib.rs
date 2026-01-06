// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Initialize logging for mobile application
fn init_logging() {
    gigi_p2p::init_tracing();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_gigi_p2p::init());

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
