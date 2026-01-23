use tauri::{AppHandle, Emitter, State};

use crate::{
    models::{Config, DownloadProgress, Peer},
    PluginState, Result,
};

/// Get list of discovered peers
#[tauri::command]
pub(crate) async fn messaging_get_peers<R: tauri::Runtime>(
    _app: AppHandle<R>,
    state: State<'_, PluginState>,
) -> Result<Vec<Peer>> {
    let client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_ref() {
        let peers = client.list_peers();
        Ok(peers.into_iter().map(|p| p.clone().into()).collect())
    } else {
        Ok(vec![])
    }
}

/// Set local nickname
#[tauri::command]
pub(crate) async fn messaging_set_nickname<R: tauri::Runtime>(
    nickname: &str,
    app: AppHandle<R>,
    state: State<'_, PluginState>,
) -> Result<()> {
    {
        let mut config_guard = state.config.write().await;
        config_guard.nickname = nickname.to_string();
    }

    app.emit("nickname-changed", nickname).map_err(|e| {
        crate::Error::CommandFailed(format!("Failed to emit nickname-changed: {}", e))
    })?;
    Ok(())
}

/// Get public key
#[tauri::command]
pub(crate) async fn messaging_get_public_key<R: tauri::Runtime>(
    _app: AppHandle<R>,
    state: State<'_, PluginState>,
) -> Result<String> {
    let client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_ref() {
        let peer_id = client.local_peer_id();
        // For now, return the peer ID as the "public key"
        // In a real implementation, we'd get the actual public key
        Ok(peer_id.to_string())
    } else {
        Err(crate::Error::P2pNotInitialized)
    }
}

/// Get active downloads
#[tauri::command]
pub(crate) async fn messaging_get_active_downloads<R: tauri::Runtime>(
    _app: AppHandle<R>,
    state: State<'_, PluginState>,
) -> Result<Vec<DownloadProgress>> {
    let downloads_guard = state.active_downloads.lock().await;
    Ok(downloads_guard.values().cloned().collect())
}

/// Update configuration
#[tauri::command]
pub(crate) async fn messaging_update_config<R: tauri::Runtime>(
    config: Config,
    app: AppHandle<R>,
    state: State<'_, PluginState>,
) -> Result<()> {
    {
        let mut config_guard = state.config.write().await;
        *config_guard = config.clone();
    }

    app.emit("config-changed", &config).map_err(|e| {
        crate::Error::CommandFailed(format!("Failed to emit config-changed: {}", e))
    })?;
    Ok(())
}

/// Get current configuration
#[tauri::command]
pub(crate) async fn messaging_get_config<R: tauri::Runtime>(
    _app: AppHandle<R>,
    state: State<'_, PluginState>,
) -> Result<Config> {
    let config_guard = state.config.read().await;
    Ok(config_guard.clone())
}
