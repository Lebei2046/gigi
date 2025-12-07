/// Provides functionality modules related to libp2p messaging.
///
/// This module contains the following main components:
/// - `Libp2pMessaging`: Core functionality for managing libp2p messaging.
/// - `Error`: Defines error types that may occur in libp2p operations.
/// - `MessageReceivedEvent` and `PeerDiscoveredEvent`: Define event models for message reception and peer discovery.
///
/// The module also provides the following features:
/// - Subscribe and unsubscribe to topics.
/// - Send messages to specified topics.
/// - Get the list of currently connected peers.
///
/// Use the `init` function to initialize the plugin and integrate it into a Tauri application.
use tauri::{
    Manager, Runtime,
    async_runtime::{Sender, channel, spawn},
    plugin::{Builder, TauriPlugin},
};

mod error;
mod models;
mod network;

pub use error::Error;
pub use models::{MessageReceivedEvent, PeerDiscoveredEvent};
pub use network::Libp2pMessaging;
pub mod commands;

/// Enum type representing libp2p commands.
///
/// Contains the following commands:
/// - `Subscribe`: Subscribe to a specified topic.
/// - `Unsubscribe`: Unsubscribe from a specified topic.
/// - `SendMessage`: Send message to a specified topic.
/// - `GetPeers`: Get the list of currently connected peers.
pub enum Libp2pCommand {
    Subscribe(String),
    Unsubscribe(String),
    SendMessage(String, Vec<u8>),
    GetPeers(Sender<Vec<(String, Vec<String>)>>),
}

/// Application state management structure.
///
/// Contains a command sender (`command_sender`) for sending commands to libp2p tasks.
pub struct AppState {
    pub command_sender: Sender<Libp2pCommand>,
}

// Initialize libp2p messaging plugin.
///
/// This function will:
/// 1. Create a command channel for communication with libp2p tasks.
/// 2. Register the application state (`AppState`) to the Tauri application.
/// 3. Start libp2p tasks to handle messaging logic.
///
/// Returns a configured `TauriPlugin` instance that can be integrated into a Tauri application.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("libp2p-messaging")
        .setup(|app, _api| {
            let (command_sender, command_receiver) = channel(100);
            app.manage(AppState { command_sender });

            let app_handle = app.clone();
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
            commands::subscribe_topic,
            commands::unsubscribe_topic,
            commands::send_message,
            commands::get_peers
        ])
        .build()
}
