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
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Libp2pMessaging;
#[cfg(mobile)]
use mobile::Libp2pMessaging;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the libp2p-messaging APIs.
pub trait Libp2pMessagingExt<R: Runtime> {
  fn libp2p_messaging(&self) -> &Libp2pMessaging<R>;
}

impl<R: Runtime, T: Manager<R>> crate::Libp2pMessagingExt<R> for T {
  fn libp2p_messaging(&self) -> &Libp2pMessaging<R> {
    self.state::<Libp2pMessaging<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("libp2p-messaging")
    .invoke_handler(tauri::generate_handler![commands::ping])
    .setup(|app, api| {
      #[cfg(mobile)]
      let libp2p_messaging = mobile::init(app, api)?;
      #[cfg(desktop)]
      let libp2p_messaging = desktop::init(app, api)?;
      app.manage(libp2p_messaging);
      Ok(())
    })
    .build()
}
