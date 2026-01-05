use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::models::PluginState;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_gigi_p2p);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<GigiP2p<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin("", "GigiP2pPlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_gigi_p2p)?;

    Ok(GigiP2p(handle))
}

/// Access to the gigi-p2p APIs.
pub struct GigiP2p<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> GigiP2p<R> {
    pub fn get_state(&self) -> crate::models::PluginState {
        // For mobile, state is managed by the plugin handle
        // We'll need to handle this differently
        unimplemented!("Mobile state access not yet implemented")
    }
}
