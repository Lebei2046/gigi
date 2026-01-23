// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Initialize logging for mobile application
fn init_logging() {
    // Check if RUST_LOG is set, if not use default level
    if std::env::var("RUST_LOG").is_err() {
        // Default: only warnings and errors, disable INFO logs
        gigi_p2p::init_tracing_with_level(tracing::Level::WARN);
    } else {
        // Use the level specified by RUST_LOG
        gigi_p2p::init_tracing();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_gigi::init());

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
