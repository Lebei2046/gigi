//! Gigi Messaging - Tauri integration for Gigi P2P
//!
//! This crate provides Tauri plugin integration for the unified gigi-p2p library.

use tauri::{Runtime, plugin::TauriPlugin};

// Use local modules
pub mod commands;
pub mod error;
pub mod models;
pub mod state;

pub use commands::*;
pub use error::Error;
pub use state::P2pState;

/// Initialize Gigi Messaging plugin
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("gigi-messaging")
        .invoke_handler(tauri::generate_handler![
            commands::send_message,
            commands::subscribe_topic,
            commands::unsubscribe_topic,
            commands::send_image,
            commands::get_peers,
            commands::get_peer_id,
        ])
        .setup(|_app, _api| {
            // Initialize P2P state would be done here
            // For now, we'll skip state management
            Ok(())
        })
        .build()
}
