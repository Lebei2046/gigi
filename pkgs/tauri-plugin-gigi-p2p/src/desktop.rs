use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<GigiP2p<R>> {
  Ok(GigiP2p(app.clone()))
}

/// Access to the gigi-p2p APIs.
pub struct GigiP2p<R: Runtime>(AppHandle<R>);

impl<R: Runtime> GigiP2p<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    Ok(PingResponse {
      value: payload.value,
    })
  }
}
