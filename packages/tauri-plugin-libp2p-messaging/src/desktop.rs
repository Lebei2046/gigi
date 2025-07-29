use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<Libp2pMessaging<R>> {
  Ok(Libp2pMessaging(app.clone()))
}

/// Access to the libp2p-messaging APIs.
pub struct Libp2pMessaging<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Libp2pMessaging<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    Ok(PingResponse {
      value: payload.value,
    })
  }
}
