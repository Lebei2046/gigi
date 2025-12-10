use gigi_messaging::init;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(init())
        .setup(|_app| {
            // Setup would be done here if needed
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            gigi_messaging::commands::get_peer_id,
            gigi_messaging::commands::subscribe_topic,
            gigi_messaging::commands::unsubscribe_topic,
            gigi_messaging::commands::send_message,
            gigi_messaging::commands::get_peers
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
