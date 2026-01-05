use tauri::{AppHandle, command, Runtime};

use crate::models::*;
use crate::Result;
use crate::GigiP2pExt;

#[command]
pub(crate) async fn ping<R: Runtime>(
    app: AppHandle<R>,
    payload: PingRequest,
) -> Result<PingResponse> {
    app.gigi_p2p().ping(payload)
}
