//! Configuration-related Tauri commands.
//!
//! This module provides commands for managing the application configuration,
//! including peer information, local nickname, public key, download settings,
//! and other user preferences.

use tauri::{AppHandle, Emitter, State};

use crate::{
    models::{Config, DownloadProgress, Peer},
    PluginState, Result,
};

/// Gets a list of all discovered peers.
///
/// This command retrieves information about all peers that have been discovered
/// on the P2P network. Peers are discovered through the libp2p discovery
/// mechanism.
///
/// # Arguments
///
/// * `_app` - The Tauri app handle (unused)
/// * `state` - The plugin state containing the P2P client
///
/// # Returns
///
/// A `Result` containing a vector of `Peer` objects, or an empty vector if
/// the P2P client is not initialized
///
/// # Example
///
/// ```typescript,ignore
/// const peers = await invoke('messaging_get_peers');
/// console.log('Discovered peers:', peers);
/// ```
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

/// Sets the local nickname.
///
/// This command updates the user's display nickname used in P2P communication.
/// The nickname is visible to other peers and is stored in the plugin state.
///
/// # Arguments
///
/// * `nickname` - The new nickname to set
/// * `app` - The Tauri app handle for emitting events
/// * `state` - The plugin state containing the configuration
///
/// # Returns
///
/// A `Result` indicating success or failure
///
/// # Events
///
/// Emits a `nickname-changed` event with the new nickname when successful
///
/// # Example
///
/// ```typescript,ignore
/// await invoke('messaging_set_nickname', { nickname: 'Alice' });
///
/// // Listen for the event
/// listen('nickname-changed', (event) => {
///   console.log('Nickname changed to:', event.payload);
/// });
/// ```
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

/// Gets the public key.
///
/// This command retrieves the public key associated with the local peer.
/// Currently, this returns the peer ID string as a representation of the
/// public key. In a future implementation, this would return the actual
/// cryptographic public key bytes.
///
/// # Arguments
///
/// * `_app` - The Tauri app handle (unused)
/// * `state` - The plugin state containing the P2P client
///
/// # Returns
///
/// A `Result` containing the public key (currently peer ID) as a string,
/// or an error if the P2P client is not initialized
///
/// # Example
///
/// ```typescript,ignore
/// const publicKey = await invoke('messaging_get_public_key');
/// console.log('Public key:', publicKey);
/// ```
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

/// Gets the list of active downloads.
///
/// This command retrieves information about all currently active file downloads,
/// including their progress and speed.
///
/// # Arguments
///
/// * `_app` - The Tauri app handle (unused)
/// * `state` - The plugin state containing the active downloads map
///
/// # Returns
///
/// A `Result` containing a vector of `DownloadProgress` objects
///
/// # Example
///
/// ```typescript,ignore
/// const downloads = await invoke('messaging_get_active_downloads');
/// downloads.forEach(dl => {
///   console.log(`${dl.download_id}: ${dl.progress}% complete`);
/// });
/// ```
#[tauri::command]
pub(crate) async fn messaging_get_active_downloads<R: tauri::Runtime>(
    _app: AppHandle<R>,
    state: State<'_, PluginState>,
) -> Result<Vec<DownloadProgress>> {
    let downloads_guard = state.active_downloads.lock().await;
    Ok(downloads_guard.values().cloned().collect())
}

/// Updates the application configuration.
///
/// This command replaces the entire application configuration with the provided
/// configuration object. This allows updating multiple settings at once.
///
/// # Arguments
///
/// * `config` - The new configuration object
/// * `app` - The Tauri app handle for emitting events
/// * `state` - The plugin state containing the configuration
///
/// # Returns
///
/// A `Result` indicating success or failure
///
/// # Events
///
/// Emits a `config-changed` event with the new configuration when successful
///
/// # Example
///
/// ```typescript,ignore
/// const newConfig = {
///   nickname: 'Bob',
///   autoAcceptFiles: true,
///   downloadFolder: '/Users/bob/Downloads',
///   maxConcurrentDownloads: 5,
///   port: 0
/// };
/// await invoke('messaging_update_config', { config: newConfig });
/// ```
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

/// Gets the current application configuration.
///
/// This command retrieves the current application configuration, including all
/// user settings and preferences.
///
/// # Arguments
///
/// * `_app` - The Tauri app handle (unused)
/// * `state` - The plugin state containing the configuration
///
/// # Returns
///
/// A `Result` containing the current `Config` object
///
/// # Example
///
/// ```typescript,ignore
/// const config = await invoke('messaging_get_config');
/// console.log('Current nickname:', config.nickname);
/// console.log('Auto-accept files:', config.autoAcceptFiles);
/// ```
#[tauri::command]
pub(crate) async fn messaging_get_config<R: tauri::Runtime>(
    _app: AppHandle<R>,
    state: State<'_, PluginState>,
) -> Result<Config> {
    let config_guard = state.config.read().await;
    Ok(config_guard.clone())
}
