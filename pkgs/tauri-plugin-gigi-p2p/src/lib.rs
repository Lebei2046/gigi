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
use desktop::GigiP2p;
#[cfg(mobile)]
use mobile::GigiP2p;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the gigi-p2p APIs.
pub trait GigiP2pExt<R: Runtime> {
  fn gigi_p2p(&self) -> &GigiP2p<R>;
}

impl<R: Runtime, T: Manager<R>> crate::GigiP2pExt<R> for T {
  fn gigi_p2p(&self) -> &GigiP2p<R> {
    self.state::<GigiP2p<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("gigi-p2p")
    .invoke_handler(tauri::generate_handler![commands::ping])
    .setup(|app, api| {
      #[cfg(mobile)]
      let gigi_p2p = mobile::init(app, api)?;
      #[cfg(desktop)]
      let gigi_p2p = desktop::init(app, api)?;
      app.manage(gigi_p2p);
      Ok(())
    })
    .build()
}
