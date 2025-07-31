use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};
use tokio::sync::{mpsc, oneshot};

mod commands;
mod desktop;
mod error;
mod models;

pub use desktop::Libp2pMessaging;
pub use error::Error;
pub use models::{MessageReceivedEvent, PeerDiscoveredEvent};

pub enum Libp2pCommand {
  Subscribe(String),
  Unsubscribe(String),
  SendMessage(String, Vec<u8>),
  GetPeers(oneshot::Sender<Vec<(String, Vec<String>)>>), // Updated line
}

pub struct AppState {
  command_sender: mpsc::Sender<Libp2pCommand>,
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("libp2p-messaging")
    .setup(|app, _api| {
      let (command_sender, command_receiver) = tokio::sync::mpsc::channel(100);
      let state = AppState { command_sender };
      app.manage(state);

      let app_handle = app.clone();

      // Spawn the libp2p task on the existing tokio runtime
      tauri::async_runtime::spawn(async move {
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
