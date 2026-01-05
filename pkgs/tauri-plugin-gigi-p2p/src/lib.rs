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
mod file_utils;
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
        .invoke_handler(tauri::generate_handler![
            // Add all the commands
            commands::peer::get_peer_id,
            commands::peer::try_get_peer_id,
            commands::config::messaging_get_peers,
            commands::config::messaging_set_nickname,
            commands::config::messaging_get_public_key,
            commands::config::messaging_get_active_downloads,
            commands::config::messaging_update_config,
            commands::config::messaging_get_config,
            commands::messaging::messaging_initialize_with_key,
            commands::messaging::messaging_send_message,
            commands::messaging::messaging_send_message_to_nickname,
            commands::messaging::messaging_send_direct_share_group_message,
            commands::messaging::messaging_join_group,
            commands::messaging::messaging_send_group_message,
            commands::messaging::emit_current_state,
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
            commands::utils::clear_app_data,
        ])
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
