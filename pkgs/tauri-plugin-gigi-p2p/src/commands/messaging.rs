use tauri::{AppHandle, Emitter, Manager, State};

use crate::{events::handle_p2p_event, models::PluginState, Error, Result};
use futures::StreamExt;

/// Setup file sharing chunk reader for Android
#[cfg(target_os = "android")]
async fn setup_file_sharing<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    client: &mut gigi_p2p::P2pClient,
) -> Result<()> {
    use gigi_p2p::events::FilePath;

    let chunk_reader = std::sync::Arc::new({
        let app_handle = app_handle.clone();
        move |file_path: &FilePath, offset: u64, size: usize| -> Result<Vec<u8>> {
            match file_path {
                FilePath::Path(path) => {
                    use std::io::{Read, Seek};
                    let mut file = std::fs::File::open(path)
                        .map_err(|e| Error::Io(format!("Failed to open file: {}", e)))?;
                    file.seek(SeekFrom::Start(offset))
                        .map_err(|e| Error::Io(format!("Failed to seek file: {}", e)))?;
                    let mut buffer = vec![0u8; size];
                    let bytes_read = file
                        .read(&mut buffer)
                        .map_err(|e| Error::Io(format!("Failed to read file: {}", e)))?;
                    buffer.truncate(bytes_read);
                    Ok(buffer)
                }
                FilePath::Url(url) => {
                    use tauri_plugin_android_fs::{AndroidFsExt, FileAccessMode, FileUri};
                    use tauri_plugin_fs::FilePath as TauriFilePath;

                    let android_api = app_handle.android_fs();
                    let file_uri = FileUri::from(TauriFilePath::Url(url.clone()));

                    match android_api.open_file(&file_uri, FileAccessMode::Read) {
                        Ok(mut file) => {
                            use std::io::{Read, Seek};
                            file.seek(SeekFrom::Start(offset))
                                .map_err(|e| Error::Io(format!("Failed to seek file: {}", e)))?;
                            let mut buffer = vec![0u8; size];
                            let bytes_read = file
                                .read(&mut buffer)
                                .map_err(|e| Error::Io(format!("Failed to read file: {}", e)))?;
                            buffer.truncate(bytes_read);
                            Ok(buffer)
                        }
                        Err(e) => Err(Error::Io(format!("Failed to open content URI: {}", e))),
                    }
                }
            }
        }
    });

    client.set_chunk_reader(chunk_reader);
    Ok(())
}

/// Initialize P2P client with keypair
#[tauri::command]
pub(crate) async fn messaging_initialize_with_key<R: tauri::Runtime>(
    app: AppHandle<R>,
    state: State<'_, PluginState>,
    private_key: Vec<u8>,
    nickname: String,
) -> Result<String> {
    use gigi_p2p::P2pClient;
    use hex;
    use libp2p::identity;

    let keypair = identity::Keypair::ed25519_from_bytes(private_key)
        .map_err(|e| Error::CommandFailed(format!("Failed to create keypair: {}", e)))?;

    let peer_id = keypair.public().to_peer_id();
    let public_key_hex = hex::encode(keypair.to_protobuf_encoding().unwrap());

    // Update nickname in config before creating P2P client
    {
        let mut config_guard = state.config.write().await;
        config_guard.nickname = nickname.clone();
    }

    // Update config with actual download directory
    {
        let download_dir = app
            .path()
            .download_dir()
            .map_err(|e| Error::Io(format!("Failed to get app data directory: {}", e)))?;

        #[cfg(target_os = "android")]
        let gigi_download_dir = download_dir;

        #[cfg(not(target_os = "android"))]
        let gigi_download_dir = download_dir.join("gigi");

        let mut config_guard = state.config.write().await;
        config_guard.download_folder = gigi_download_dir.to_string_lossy().to_string();
    }

    // Get config for download directory
    let config_guard = state.config.read().await;
    let output_dir = std::path::PathBuf::from(&config_guard.download_folder);
    let shared_files_path = output_dir.join("shared.json");
    let final_nickname = config_guard.nickname.clone();
    drop(config_guard);

    // Create downloads directory at runtime when initializing
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| Error::Io(format!("Failed to create download directory: {}", e)))?;

    tracing::info!("Download directory set to: {:?}", output_dir);

    match P2pClient::new_with_config(keypair, final_nickname, output_dir, shared_files_path) {
        Ok((mut client, event_receiver)) => {
            // Start listening on a random port
            let addr = "/ip4/0.0.0.0/tcp/0"
                .parse()
                .map_err(|e| Error::P2p(format!("Failed to parse address: {}", e)))?;
            if let Err(e) = client.start_listening(addr) {
                return Err(Error::P2p(format!("Failed to start listening: {}", e)));
            }

            // Store client and receiver
            let mut client_guard = state.p2p_client.lock().await;
            *client_guard = Some(client);
            drop(client_guard);

            let mut receiver_guard = state.event_receiver.lock().await;
            *receiver_guard = Some(event_receiver);
            drop(receiver_guard);

            // Setup file sharing chunk reader for Android content URIs
            #[cfg(target_os = "android")]
            {
                let mut client_guard = state.p2p_client.lock().await;
                if let Some(ref mut client) = *client_guard {
                    if let Err(e) = setup_file_sharing(&app, client).await {
                        tracing::error!("Failed to setup file sharing: {}", e);
                    }
                }
            }

            // Start event handling loop
            let p2p_client = state.p2p_client.clone();
            let app_handle_clone = app.clone();
            let receiver = {
                let mut guard = state.event_receiver.lock().await;
                guard.take().unwrap()
            };

            // Task 1: Poll swarm events
            let p2p_client_for_events = p2p_client.clone();
            tokio::spawn(async move {
                loop {
                    let client_ready = {
                        let client_guard = p2p_client_for_events.lock().await;
                        client_guard.as_ref().is_some()
                    };

                    if client_ready {
                        let result =
                            tokio::time::timeout(tokio::time::Duration::from_millis(100), async {
                                let mut client_guard = p2p_client_for_events.lock().await;
                                if let Some(client) = client_guard.as_mut() {
                                    client.handle_next_swarm_event().await
                                } else {
                                    Ok(())
                                }
                            })
                            .await;

                        if let Err(e) = result {
                            // Ignore timeout errors - they're expected when no events are pending
                            if !matches!(e, tokio::time::error::Elapsed { .. }) {
                                tracing::error!("Error handling swarm event: {:?}", e);
                            }
                        }
                    } else {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            });

            // Task 2: Handle P2P events
            let active_downloads = state.active_downloads.clone();
            tokio::spawn(async move {
                let mut receiver = receiver;
                while let Some(event) = receiver.next().await {
                    if let Err(e) =
                        handle_p2p_event(event, &p2p_client, &active_downloads, &app_handle_clone)
                            .await
                    {
                        tracing::error!("Failed to handle P2P event: {}", e);
                    }
                }
            });

            app.emit("peer-id-changed", &peer_id.to_string())
                .map_err(|e| {
                    Error::CommandFailed(format!("Failed to emit peer-id-changed: {}", e))
                })?;
            app.emit("public-key-changed", &public_key_hex)
                .map_err(|e| {
                    Error::CommandFailed(format!("Failed to emit public-key-changed: {}", e))
                })?;
            app.emit("nickname-changed", &nickname).map_err(|e| {
                Error::CommandFailed(format!("Failed to emit nickname-changed: {}", e))
            })?;

            Ok(peer_id.to_string())
        }
        Err(e) => Err(Error::P2p(format!("Failed to create P2P client: {}", e))),
    }
}

/// Send message to a peer by peer ID (deprecated - use nickname instead)
#[tauri::command]
pub(crate) async fn messaging_send_message<R: tauri::Runtime>(
    _app: AppHandle<R>,
    _state: State<'_, PluginState>,
    _to_peer_id: String,
    _message: String,
) -> Result<String> {
    // This would need the peer nickname, which we don't have in this command
    // For now, return an error suggesting to use a different API
    Err(Error::CommandFailed(
        "Use peer nickname instead of peer ID".to_string(),
    ))
}

/// Send message to a peer by nickname
#[tauri::command]
pub(crate) async fn messaging_send_message_to_nickname<R: tauri::Runtime>(
    _app: AppHandle<R>,
    state: State<'_, PluginState>,
    nickname: &str,
    message: &str,
) -> Result<String> {
    let mut client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_mut() {
        client
            .send_direct_message(nickname, message.to_string())
            .map_err(|e| Error::CommandFailed(format!("Failed to send message: {}", e)))?;
        Ok(uuid::Uuid::new_v4().to_string())
    } else {
        Err(Error::CommandFailed(
            "P2P client not initialized".to_string(),
        ))
    }
}

/// Send group invitation to a peer
#[tauri::command]
pub(crate) async fn messaging_send_direct_share_group_message<R: tauri::Runtime>(
    _app: AppHandle<R>,
    state: State<'_, PluginState>,
    nickname: &str,
    group_id: &str,
    group_name: &str,
) -> Result<String> {
    let mut client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_mut() {
        client
            .send_direct_share_group_message(nickname, group_id.to_string(), group_name.to_string())
            .map_err(|e| {
                Error::CommandFailed(format!("Failed to send group share message: {}", e))
            })?;
        Ok(uuid::Uuid::new_v4().to_string())
    } else {
        Err(Error::CommandFailed(
            "P2P client not initialized".to_string(),
        ))
    }
}

/// Join a group
#[tauri::command]
pub(crate) async fn messaging_join_group<R: tauri::Runtime>(
    _app: AppHandle<R>,
    state: State<'_, PluginState>,
    group_id: &str,
) -> Result<()> {
    let mut client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_mut() {
        client
            .join_group(group_id)
            .map_err(|e| Error::CommandFailed(format!("Failed to join group: {}", e)))?;
        Ok(())
    } else {
        Err(Error::CommandFailed(
            "P2P client not initialized".to_string(),
        ))
    }
}

/// Send message to a group
#[tauri::command]
pub(crate) async fn messaging_send_group_message<R: tauri::Runtime>(
    _app: AppHandle<R>,
    state: State<'_, PluginState>,
    group_id: &str,
    message: &str,
) -> Result<String> {
    let mut client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_mut() {
        client
            .send_group_message(group_id, message.to_string())
            .map_err(|e| Error::CommandFailed(format!("Failed to send group message: {}", e)))?;
        Ok(uuid::Uuid::new_v4().to_string())
    } else {
        Err(Error::CommandFailed(
            "P2P client not initialized".to_string(),
        ))
    }
}

/// Emit current state to frontend
#[tauri::command]
pub(crate) async fn emit_current_state<R: tauri::Runtime>(
    app: AppHandle<R>,
    state: State<'_, PluginState>,
) -> Result<()> {
    use serde_json::json;

    tracing::info!("emit_current_state called");

    // Clone the client to avoid holding the mutex lock during emit operations
    let client_clone = {
        match tokio::time::timeout(tokio::time::Duration::from_secs(2), state.p2p_client.lock())
            .await
        {
            Ok(client_guard) => {
                if let Some(client) = client_guard.as_ref() {
                    let peer_id = client.local_peer_id().to_string();
                    let peers = client.list_peers();
                    let peer_data: Vec<crate::models::Peer> =
                        peers.into_iter().map(|peer| peer.clone().into()).collect();

                    tracing::info!("Collected {} peers, releasing lock", peer_data.len());
                    Some((peer_id, peer_data))
                } else {
                    None
                }
            }
            Err(_) => {
                tracing::warn!("Timeout: Failed to acquire P2P client lock within 2 seconds");
                return Ok(());
            }
        }
    };

    // Now emit events without holding the mutex lock
    match client_clone {
        Some((peer_id, peers)) => {
            app.emit("peer-id-changed", &peer_id).map_err(|e| {
                Error::CommandFailed(format!("Failed to emit peer-id-changed: {}", e))
            })?;

            for peer in peers {
                app.emit("peer-discovered", &peer).map_err(|e| {
                    Error::CommandFailed(format!("Failed to emit peer-discovered: {}", e))
                })?;

                app.emit(
                    "peer-connected",
                    &json!({
                        "peer_id": peer.id,
                        "nickname": peer.nickname
                    }),
                )
                .map_err(|e| {
                    Error::CommandFailed(format!("Failed to emit peer-connected: {}", e))
                })?;
            }

            tracing::info!("emit_current_state completed successfully");
            Ok(())
        }
        None => {
            tracing::error!("P2P client not initialized");
            Err(Error::CommandFailed(
                "P2P client not initialized".to_string(),
            ))
        }
    }
}
