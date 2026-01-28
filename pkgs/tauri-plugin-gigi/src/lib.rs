//! # Tauri Plugin for Gigi P2P Messaging
//!
//! This plugin provides a comprehensive P2P (Peer-to-Peer) messaging and file sharing solution
//! for Tauri applications. It integrates with the gigi-p2p and gigi-store packages to enable
//! direct peer communication, group messaging, and file transfers across different platforms.
//!
//! ## Features
//!
//! - **Direct Messaging**: Send and receive messages directly between peers
//! - **Group Messaging**: Join and participate in group conversations
//! - **File Sharing**: Share files with automatic download for images
//! - **Peer Discovery**: Automatically discover and connect to peers on the local network
//! - **Persistence**: Store messages, conversations, contacts, and groups in a SQLite database
//! - **Authentication**: Account signup, login, and password verification using P2P
//! - **Event System**: Real-time events for peer discovery, messages, file transfers, etc.
//!
//! ## Architecture
//!
//! The plugin is organized into several key modules:
//!
//! - **commands**: Tauri command handlers for frontend interaction
//!   - `auth`: Authentication commands (signup, login, account management)
//!   - `config`: Configuration management (nickname, peers, settings)
//!   - `contacts`: Contact management (add, remove, update contacts)
//!   - `conversations`: Conversation management (CRUD operations)
//!   - `file`: File sharing commands (share, download, request files)
//!   - `groups`: Group management (create, join, update groups)
//!   - `messaging`: Core messaging commands (send/receive messages)
//!   - `peer`: Peer information commands (get peer ID)
//!   - `utils`: Utility commands (clear app data)
//!
//! - **models**: Data structures and state management
//!   - `PluginState`: Global plugin state containing all managers and clients
//!   - `Config`: Application configuration
//!   - `Peer`, `Message`, `GroupMessage`: Data transfer objects
//!   - `FileInfo`, `DownloadProgress`: File transfer structures
//!
//! - **events**: Event handlers for P2P events
//!   - Handles peer discovery, connection, disconnection
//!   - Processes incoming messages (direct and group)
//!   - Manages file share events and download progress
//!   - Generates thumbnails for downloaded images
//!
//! - **file_utils**: File utility functions
//!   - Image detection and validation
//!   - Base64 encoding for images
//!   - Android content URI handling
//!
//! - **error**: Error types and handling
//!   - Custom error types for different failure scenarios
//!
//! - **desktop/mobile**: Platform-specific initialization
//!   - Database initialization and migration
//!   - Manager setup (AuthManager, GroupManager, ContactManager, etc.)
//!
//! ## Usage
//!
//! ### Basic Setup
//!
//! ```ignore
//! use tauri::Builder;
//! use tauri_plugin_gigi::init;
//!
//! fn main() {
//!     tauri::Builder::default()
//!         .plugin(init())
//!         .run(tauri::generate_context!())
//!         .expect("error while running tauri application");
//! }
//! ```
//!
//! ### Accessing the Plugin from Tauri State
//!
//! ```ignore
//! use tauri_plugin_gigi::GigiExt;
//! use tauri::Manager;
//!
//! fn get_gigi_state<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
//!     let gigi = app.gigi();
//!     let state = gigi.get_state();
//!     // Access state fields as needed
//! }
//! ```
//!
//! ## Database Schema
//!
//! The plugin uses SQLite with Sea-ORM for persistence. Key tables include:
//! - `messages`: Stored messages (direct and group)
//! - `conversations`: Chat conversations
//! - `contacts`: User contacts
//! - `groups`: Group chats
//! - `accounts`: User accounts for authentication
//! - `file_sharing`: File share metadata
//! - `thumbnails`: Image thumbnail paths
//!
//! ## Events
//!
//! The plugin emits the following events to the frontend:
//! - `peer-discovered`: New peer discovered
//! - `peer-expired`: Peer expired from peer table
//! - `peer-connected`: Peer connected
//! - `peer-disconnected`: Peer disconnected
//! - `nickname-updated`: Peer nickname updated
//! - `message-received`: Direct message received
//! - `group-message`: Group message received
//! - `file-message-received`: File share message received
//! - `image-message-received`: Image message received (auto-downloaded)
//! - `group-file-message-received`: Group file message received
//! - `group-image-message-received`: Group image message received
//! - `file-download-started`: File download started
//! - `file-download-progress`: File download progress
//! - `file-download-completed`: File download completed
//! - `file-download-failed`: File download failed
//! - `p2p-error`: P2P error occurred
//! - `nickname-changed`: Local nickname changed
//! - `config-changed`: Configuration changed

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod events;
pub mod file_utils;
mod models;

#[cfg(test)]
mod lib_tests;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Gigi;
#[cfg(mobile)]
use mobile::Gigi;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the gigi APIs.
///
/// This trait provides a convenient way to access the Gigi plugin state from any
/// Tauri component that implements the `Manager` trait.
///
/// # Example
///
/// Basic usage:
///
/// ```ignore
/// use tauri_plugin_gigi::GigiExt;
/// use tauri::Manager;
///
/// fn my_function<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
///     let gigi = app.gigi();
///     let state = gigi.get_state();
///     // Use the state...
/// }
/// ```
pub trait GigiExt<R: Runtime> {
    fn gigi(&self) -> &Gigi<R>;
}

impl<R: Runtime, T: Manager<R>> crate::GigiExt<R> for T {
    fn gigi(&self) -> &Gigi<R> {
        self.state::<Gigi<R>>().inner()
    }
}

/// Initializes the plugin.
///
/// This function creates and returns a Tauri plugin that provides all Gigi P2P
/// messaging functionality. It registers all command handlers and sets up the
/// plugin state.
///
/// # Returns
///
/// A configured `TauriPlugin` that can be added to a Tauri app.
///
/// # Example
///
/// Basic usage:
///
/// ```ignore
/// use tauri::Builder;
/// use tauri_plugin_gigi::init;
///
/// fn main() {
///     tauri::Builder::default()
///         .plugin(init())
///         .build(tauri::generate_context!())
///         .expect("error while running tauri application");
/// }
/// ```
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("gigi")
        .invoke_handler(tauri::generate_handler![
            // Peer commands
            commands::peer::get_peer_id,
            commands::peer::try_get_peer_id,
            // Config commands
            commands::config::messaging_get_peers,
            commands::config::messaging_set_nickname,
            commands::config::messaging_get_public_key,
            commands::config::messaging_get_active_downloads,
            commands::config::messaging_update_config,
            commands::config::messaging_get_config,
            // Messaging commands
            commands::messaging::messaging_send_message,
            commands::messaging::messaging_send_message_to_nickname,
            commands::messaging::messaging_send_direct_share_group_message,
            commands::messaging::messaging_join_group,
            commands::messaging::messaging_send_group_message,
            commands::messaging::emit_current_state,
            commands::messaging::get_file_thumbnail,
            commands::messaging::get_full_image_by_path,
            commands::messaging::get_full_image,
            commands::messaging::get_messages,
            commands::messaging::search_messages,
            commands::messaging::clear_messages_with_thumbnails,
            // File commands
            commands::file::messaging_send_file_message_with_path,
            commands::file::messaging_send_group_file_message_with_path,
            commands::file::messaging_share_file,
            commands::file::messaging_request_file_from_nickname,
            commands::file::messaging_cancel_download,
            commands::file::messaging_get_shared_files,
            commands::file::messaging_remove_shared_file,
            commands::file::messaging_get_image_data,
            commands::file::messaging_get_file_info,
            commands::file::messaging_select_any_file,
            commands::file::messaging_share_content_uri,
            // Utility commands
            commands::utils::clear_app_data,
            // Conversation commands
            commands::conversations::get_conversations,
            commands::conversations::get_conversation,
            commands::conversations::upsert_conversation,
            commands::conversations::update_conversation_last_message,
            commands::conversations::increment_conversation_unread,
            commands::conversations::mark_conversation_as_read,
            commands::conversations::delete_conversation,
            // Auth commands
            commands::auth::auth_check_account,
            commands::auth::auth_signup,
            commands::auth::auth_login_with_p2p,
            commands::auth::auth_get_account_info,
            commands::auth::auth_delete_account,
            commands::auth::auth_verify_password,
            // Group commands
            commands::groups::group_create,
            commands::groups::group_join,
            commands::groups::group_get_all,
            commands::groups::group_get,
            commands::groups::group_delete,
            commands::groups::group_update,
            // Contact commands
            commands::contacts::contact_add,
            commands::contacts::contact_remove,
            commands::contacts::contact_update,
            commands::contacts::contact_get,
            commands::contacts::contact_get_all,
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let gigi = mobile::init(app, api)?;
            #[cfg(desktop)]
            let gigi = desktop::init(app, api)?;
            app.manage(gigi);
            Ok(())
        })
        .build()
}
