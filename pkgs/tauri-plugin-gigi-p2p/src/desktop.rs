use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Manager, Runtime};

use crate::models::PluginState;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<GigiP2p<R>> {
    let state = PluginState::new();
    app.manage(state);
    Ok(GigiP2p(app.clone()))
}

/// Access to the gigi-p2p APIs.
pub struct GigiP2p<R: Runtime>(AppHandle<R>);

impl<R: Runtime> GigiP2p<R> {
    pub fn get_state(&self) -> PluginState {
        self.0.state::<PluginState>().inner().clone()
    }
}
