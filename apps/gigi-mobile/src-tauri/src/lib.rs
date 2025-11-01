use gigi_messaging::{commands, AppState, Libp2pMessaging};
use tauri::{
    async_runtime::{channel, spawn},
    Manager,
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let (command_sender, command_receiver) = channel(100);
            app.manage(AppState { command_sender });

            let app_handle = app.handle().clone();
            // Spawn the libp2p task
            spawn(async move {
                match Libp2pMessaging::new(app_handle, command_receiver) {
                    Ok(mut messaging) => {
                        messaging.run().await;
                    }
                    Err(e) => {
                        eprintln!("Failed to initialize Libp2pMessaging: {:?}", e);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::subscribe_topic,
            commands::unsubscribe_topic,
            commands::send_message,
            commands::get_peers
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
