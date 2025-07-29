use tauri::{AppHandle, command, Runtime};

use crate::models::*;
use crate::Result;
use crate::Libp2pMessagingExt;

#[command]
pub(crate) async fn ping<R: Runtime>(
  app: AppHandle<R>,
  payload: PingRequest,
) -> Result<PingResponse> {
  app.libp2p_messaging().ping(payload)
}
