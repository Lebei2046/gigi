use tauri::State;

use crate::{models::PluginState, Error, Result};

/// Get the local peer ID
#[tauri::command]
pub(crate) async fn get_peer_id(state: State<'_, PluginState>) -> Result<String> {
    let p2p_client = state.p2p_client.lock().await;
    match p2p_client.as_ref() {
        Some(client) => Ok(client.local_peer_id().to_string()),
        None => Err(Error::P2pNotInitialized),
    }
}

/// Try to get peer ID from private key bytes
#[tauri::command]
pub(crate) fn try_get_peer_id(priv_key: Vec<u8>) -> Result<String> {
    use libp2p::identity;
    match identity::Keypair::ed25519_from_bytes(priv_key) {
        Ok(id_keys) => {
            let peer_id = id_keys.public().to_peer_id();
            Ok(peer_id.to_string())
        }
        Err(e) => Err(Error::CommandFailed(format!(
            "Failed to create keypair: {}",
            e
        ))),
    }
}
