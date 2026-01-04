use crate::types::{AppState, Peer};
use tauri::{AppHandle, Emitter, Manager, State};
use tracing::{debug, error, info};

/// Setup file sharing chunk reader for Android
#[cfg(target_os = "android")]
pub async fn setup_file_sharing(
    app_handle: &AppHandle,
    client: &mut gigi_p2p::P2pClient,
) -> anyhow::Result<()> {
    use gigi_p2p::events::FilePath;

    let chunk_reader = std::sync::Arc::new({
        let app_handle = app_handle.clone();
        move |file_path: &FilePath, offset: u64, size: usize| -> anyhow::Result<Vec<u8>> {
            match file_path {
                FilePath::Path(path) => {
                    use std::io::{Read, Seek};
                    let mut file = std::fs::File::open(path)?;
                    file.seek(SeekFrom::Start(offset))?;
                    let mut buffer = vec![0u8; size];
                    let bytes_read = file.read(&mut buffer)?;
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
                            file.seek(SeekFrom::Start(offset))?;
                            let mut buffer = vec![0u8; size];
                            let bytes_read = file.read(&mut buffer)?;
                            buffer.truncate(bytes_read);
                            Ok(buffer)
                        }
                        Err(e) => Err(anyhow::anyhow!("Failed to open content URI: {}", e)),
                    }
                }
            }
        }
    });

    client.set_chunk_reader(chunk_reader.clone());
    Ok(())
}

#[tauri::command]
pub async fn messaging_initialize_with_key(
    private_key: Vec<u8>,
    nickname: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use futures::StreamExt;
    use gigi_p2p::P2pClient;
    use hex;
    use libp2p::identity;
    use std::fs;
    use std::path::PathBuf;

    let keypair = identity::Keypair::ed25519_from_bytes(private_key)
        .map_err(|e| format!("Failed to create keypair: {}", e))?;

    let peer_id = keypair.public().to_peer_id();
    let public_key_hex = hex::encode(keypair.to_protobuf_encoding().unwrap());

    // Update nickname in config before creating P2P client
    {
        let mut config_guard = state.config.write().await;
        config_guard.nickname = nickname.clone();
    }

    // Update config with the actual download directory
    {
        let download_dir = app_handle
            .path()
            .download_dir()
            .map_err(|e| format!("Failed to get app data directory: {}", e))?;

        #[cfg(target_os = "android")]
        let gigi_download_dir = download_dir;

        #[cfg(not(target_os = "android"))]
        let gigi_download_dir = download_dir.join("gigi");

        let mut config_guard = state.config.write().await;
        config_guard.download_folder = gigi_download_dir.to_string_lossy().to_string();
    }

    let config_guard = state.config.read().await;
    let output_dir = PathBuf::from(&config_guard.download_folder);
    let shared_files_path = PathBuf::from(&config_guard.download_folder).join("shared.json");
    let final_nickname = config_guard.nickname.clone();
    drop(config_guard);

    // Create downloads directory at runtime when initializing
    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create download directory: {}", e))?;

    info!("📁 Download directory set to: {:?}", output_dir);

    match P2pClient::new_with_config(keypair, final_nickname, output_dir, shared_files_path) {
        Ok((mut client, event_receiver)) => {
            // Start listening on a random port
            let addr = "/ip4/0.0.0.0/tcp/0".parse().unwrap();
            if let Err(e) = client.start_listening(addr) {
                return Err(format!("Failed to start listening: {}", e));
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
                    if let Err(e) = setup_file_sharing(&app_handle, client).await {
                        error!("Failed to setup file sharing: {}", e);
                    }
                }
            }

            // Start event handling loop
            let p2p_client = state.p2p_client.clone();
            let config = state.config.clone();
            let active_downloads = state.active_downloads.clone();
            let app_handle_clone = app_handle.clone();
            let receiver = {
                let mut guard = state.event_receiver.lock().await;
                guard.take().unwrap()
            };

            // Spawn two separate tasks to avoid deadlock:
            // 1. Swarm event polling - briefly locks the client to handle events
            // 2. P2P event handling - processes events from the receiver

            let p2p_client_for_events = p2p_client.clone();

            tokio::spawn(async move {
                // Task 1: Poll swarm events and handle them
                info!("Starting swarm event polling task");
                loop {
                    // Check if client is initialized and poll for events with timeout
                    let client_ready = {
                        let client_guard = p2p_client_for_events.lock().await;
                        client_guard.as_ref().is_some()
                    };

                    if client_ready {
                        // Use the public API to handle next swarm event with timeout
                        let result =
                            tokio::time::timeout(tokio::time::Duration::from_millis(100), async {
                                let mut client_guard = p2p_client_for_events.lock().await;
                                if let Some(client) = client_guard.as_mut() {
                                    client.handle_next_swarm_event().await
                                } else {
                                    Ok(()) // No error, just return
                                }
                            })
                            .await;

                        match result {
                            Ok(Ok(())) => {
                                // Event handled successfully
                                debug!("Swarm event processed successfully");
                            }
                            Ok(Err(e)) => {
                                error!("Error handling swarm event: {}", e);
                            }
                            Err(_) => {
                                // Timeout occurred - this is expected, gives other tasks chance to run
                                // Don't print anything to avoid spam
                                debug!("Swarm event polling timeout (expected)");
                            }
                        }
                    } else {
                        // Client not ready, wait and retry
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            });

            tokio::spawn(async move {
                // Task 2: Handle P2P events from the receiver
                let mut receiver = receiver;
                info!("Starting P2P event handling task");

                loop {
                    debug!("Waiting for P2P event...");
                    if let Some(event) = receiver.next().await {
                        debug!("Received P2P event: {:?}", std::mem::discriminant(&event));
                        if let Err(e) = crate::event_handlers::handle_p2p_event_with_fields(
                            event,
                            &p2p_client,
                            &config,
                            &active_downloads,
                            &app_handle_clone,
                        )
                        .await
                        {
                            error!("Error handling P2P event: {}", e);
                        }
                    }
                }
            });

            app_handle
                .emit("peer-id-changed", &peer_id.to_string())
                .unwrap();
            app_handle
                .emit("public-key-changed", &public_key_hex)
                .unwrap();

            // Emit nickname-changed event to ensure frontend is in sync
            app_handle.emit("nickname-changed", &nickname).unwrap();

            Ok(peer_id.to_string())
        }
        Err(e) => Err(format!("Failed to create P2P client: {}", e)),
    }
}

#[tauri::command]
pub fn messaging_send_message(
    _to_peer_id: &str,
    _message: &str,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    // This would need the peer nickname, which we don't have in this command
    // For now, return an error suggesting to use a different API
    Err("Use peer nickname instead of peer ID".to_string())
}

#[tauri::command]
pub async fn messaging_send_message_to_nickname(
    nickname: &str,
    message: &str,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_mut() {
        client
            .send_direct_message(nickname, message.to_string())
            .map_err(|e| format!("Failed to send message: {}", e))?;
        Ok(uuid::Uuid::new_v4().to_string())
    } else {
        Err("P2P client not initialized".to_string())
    }
}

#[tauri::command]
pub async fn messaging_send_direct_share_group_message(
    nickname: &str,
    group_id: &str,
    group_name: &str,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_mut() {
        client
            .send_direct_share_group_message(nickname, group_id.to_string(), group_name.to_string())
            .map_err(|e| format!("Failed to send group share message: {}", e))?;
        Ok(uuid::Uuid::new_v4().to_string())
    } else {
        Err("P2P client not initialized".to_string())
    }
}

#[tauri::command]
pub async fn messaging_join_group(
    group_id: &str,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_mut() {
        client
            .join_group(group_id)
            .map_err(|e| format!("Failed to join group: {}", e))?;
        Ok(())
    } else {
        Err("P2P client not initialized".to_string())
    }
}

#[tauri::command]
pub async fn messaging_send_group_message(
    group_id: &str,
    message: &str,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_mut() {
        client
            .send_group_message(group_id, message.to_string())
            .map_err(|e| format!("Failed to send group message: {}", e))?;
        Ok(uuid::Uuid::new_v4().to_string())
    } else {
        Err("P2P client not initialized".to_string())
    }
}

#[tauri::command]
pub async fn emit_current_state(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use serde_json::json;
    use tracing::{debug, info, warn};

    info!("emit_current_state called");

    // Clone the client to avoid holding the mutex lock during emit operations
    let client_clone = {
        debug!("Attempting to lock P2P client...");

        match tokio::time::timeout(tokio::time::Duration::from_secs(2), state.p2p_client.lock())
            .await
        {
            Ok(client_guard) => {
                debug!("Successfully locked P2P client");

                if let Some(client) = client_guard.as_ref() {
                    // Clone what we need and immediately release the lock
                    let peer_id = client.local_peer_id().to_string();
                    let peers = client.list_peers();
                    let peer_data: Vec<Peer> = peers
                        .iter()
                        .map(|peer| Peer {
                            id: peer.peer_id.to_string(),
                            nickname: peer.nickname.clone(),
                            capabilities: vec![
                                "messaging".to_string(),
                                "file_transfer".to_string(),
                            ],
                        })
                        .collect();

                    info!("Collected {} peers, releasing lock", peers.len());
                    Some((peer_id, peer_data))
                } else {
                    None
                }
            }
            Err(_) => {
                warn!("Timeout: Failed to acquire P2P client lock within 2 seconds, but continuing...");
                return Ok(());
            }
        }
    };

    // Now emit events without holding the mutex lock
    match client_clone {
        Some((peer_id, peers)) => {
            info!("Emitting peer-id-changed: {}", peer_id);
            app_handle
                .emit("peer-id-changed", &peer_id)
                .map_err(|e| format!("Failed to emit peer-id-changed: {}", e))?;
            info!("peer-id-changed emitted successfully");

            for peer in peers {
                info!(
                    "Emitting peer-discovered for {} ({})",
                    peer.nickname, peer.id
                );
                app_handle
                    .emit("peer-discovered", &peer)
                    .map_err(|e| format!("Failed to emit peer-discovered: {}", e))?;
                info!("peer-discovered emitted successfully");

                info!(
                    "Emitting peer-connected for {} ({})",
                    peer.nickname, peer.id
                );
                app_handle
                    .emit(
                        "peer-connected",
                        &json!({
                            "peer_id": peer.id,
                            "nickname": peer.nickname
                        }),
                    )
                    .map_err(|e| format!("Failed to emit peer-connected: {}", e))?;
                info!("peer-connected emitted successfully");
            }

            info!("emit_current_state completed successfully");
            Ok(())
        }
        None => {
            error!("P2P client not initialized");
            Err("P2P client not initialized".to_string())
        }
    }
}
