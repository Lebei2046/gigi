// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod event_handlers;
mod file_utils;
mod types;

use types::{AppState, Config, DownloadProgress, GroupMessage, Message};

/// Initialize logging for the mobile application
fn init_logging() {
    gigi_p2p::init_tracing();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();

    let app_state = AppState::default();

    let builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());

    #[cfg(target_os = "android")]
    let builder = { builder.plugin(tauri_plugin_android_fs::init()) };

    builder
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::peer::get_peer_id,
            commands::peer::try_get_peer_id,
            commands::messaging::messaging_initialize_with_key,
            commands::messaging::messaging_send_message,
            commands::messaging::messaging_send_message_to_nickname,
            commands::messaging::messaging_send_direct_share_group_message,
            commands::file::messaging_send_file_message_with_path,
            commands::file::messaging_send_group_file_message_with_path,
            commands::config::messaging_get_peers,
            commands::config::messaging_set_nickname,
            commands::messaging::messaging_join_group,
            commands::messaging::messaging_send_group_message,
            commands::file::messaging_share_file,
            commands::file::messaging_request_file,
            commands::file::messaging_request_file_from_nickname,
            commands::file::messaging_cancel_download,
            commands::file::messaging_get_shared_files,
            commands::file::messaging_remove_shared_file,
            commands::file::messaging_get_image_data,
            commands::file::messaging_get_file_info,
            commands::file::messaging_select_any_file,
            commands::config::messaging_get_public_key,
            commands::config::messaging_get_active_downloads,
            commands::config::messaging_update_config,
            commands::config::messaging_get_config,
            commands::messaging::emit_current_state,
            commands::utils::clear_app_data,
            commands::file::messaging_share_content_uri,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
