use crate::types::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_peer_id(state: State<'_, AppState>) -> Result<String, String> {
    let client_guard = state.p2p_client.lock().await;
    match client_guard.as_ref() {
        Some(client) => Ok(client.local_peer_id().to_string()),
        None => Err("P2P client not initialized".to_string()),
    }
}

#[tauri::command]
pub fn try_get_peer_id(priv_key: Vec<u8>) -> Result<String, String> {
    use libp2p::identity;
    match identity::Keypair::ed25519_from_bytes(priv_key) {
        Ok(id_keys) => {
            let peer_id = id_keys.public().to_peer_id();
            Ok(peer_id.to_string())
        }
        Err(e) => Err(format!("Failed to create keypair: {}", e)),
    }
}
