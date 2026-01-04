use crate::types::{AppState, Config, DownloadProgress};
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn messaging_get_peers(
    state: State<'_, AppState>,
) -> Result<Vec<crate::types::Peer>, String> {
    let client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_ref() {
        let peers = client.list_peers();
        Ok(peers.into_iter().map(|p| p.clone().into()).collect())
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
pub async fn messaging_set_nickname(
    nickname: &str,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    {
        let mut config_guard = state.config.write().await;
        config_guard.nickname = nickname.to_string();
    }

    app_handle.emit("nickname-changed", nickname).unwrap();
    Ok(())
}

#[tauri::command]
pub async fn messaging_get_public_key(state: State<'_, AppState>) -> Result<String, String> {
    let client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_ref() {
        let peer_id = client.local_peer_id();
        // For now, return the peer ID as the "public key"
        // In a real implementation, we'd get the actual public key
        Ok(peer_id.to_string())
    } else {
        Err("P2P client not initialized".to_string())
    }
}

#[tauri::command]
pub async fn messaging_get_active_downloads(
    state: State<'_, AppState>,
) -> Result<Vec<DownloadProgress>, String> {
    let downloads_guard = state.active_downloads.lock().await;
    Ok(downloads_guard.values().cloned().collect())
}

#[tauri::command]
pub async fn messaging_update_config(
    config: Config,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    {
        let mut config_guard = state.config.write().await;
        *config_guard = config.clone();
    }

    app_handle.emit("config-changed", &config).unwrap();
    Ok(())
}

#[tauri::command]
pub async fn messaging_get_config(state: State<'_, AppState>) -> Result<Config, String> {
    let config_guard = state.config.read().await;
    Ok(config_guard.clone())
}
