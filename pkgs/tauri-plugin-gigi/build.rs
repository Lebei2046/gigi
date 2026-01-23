const COMMANDS: &[&str] = &[
    // Peer commands
    "get_peer_id",
    "try_get_peer_id",
    // Config commands
    "messaging_get_peers",
    "messaging_set_nickname",
    "messaging_get_public_key",
    "messaging_get_active_downloads",
    "messaging_update_config",
    "messaging_get_config",
    // Messaging commands
    "messaging_send_message",
    "messaging_send_message_to_nickname",
    "messaging_send_direct_share_group_message",
    "messaging_join_group",
    "messaging_send_group_message",
    "emit_current_state",
    "messaging_get_message_history",
    "messaging_save_shared_files",
    "get_messages",
    "search_messages",
    "clear_messages_with_thumbnails",
    "get_file_thumbnail",
    "get_full_image_by_path",
    "get_full_image",
    // File commands
    "messaging_send_file_message_with_path",
    "messaging_send_group_file_message_with_path",
    "messaging_share_file",
    "messaging_request_file",
    "messaging_request_file_from_nickname",
    "messaging_cancel_download",
    "messaging_get_shared_files",
    "messaging_remove_shared_file",
    "messaging_get_image_data",
    "messaging_get_file_info",
    "messaging_select_any_file",
    "messaging_share_content_uri",
    // Utils commands
    "clear_app_data",
    // Auth commands
    "auth_check_account",
    "auth_signup",
    "auth_login_with_p2p",
    "auth_get_account_info",
    "auth_delete_account",
    "auth_verify_password",
    // Conversation commands
    "get_conversations",
    "get_conversation",
    "upsert_conversation",
    "update_conversation_last_message",
    "increment_conversation_unread",
    "mark_conversation_as_read",
    "delete_conversation",
    // Contact commands
    "contact_add",
    "contact_remove",
    "contact_update",
    "contact_get",
    "contact_get_all",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();
}
