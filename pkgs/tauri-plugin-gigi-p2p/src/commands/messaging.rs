use gigi_store::PersistenceConfig;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{events::handle_p2p_event, models::PluginState, Error, Result};
use chrono::Duration as ChronoDuration;
use futures::StreamExt;
use gigi_store::{MessageContent, MessageDirection, MessageType, StoredMessage};
use sea_orm_migration::MigratorTrait;

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
    let db_path = output_dir.join(format!("{}.db", config_guard.nickname));
    let db_path_clone = db_path.clone();
    let final_nickname = config_guard.nickname.clone();
    drop(config_guard);

    // Create downloads directory at runtime when initializing
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| Error::Io(format!("Failed to create download directory: {}", e)))?;

    tracing::info!("Download directory set to: {:?}", output_dir);

    // Create persistence config for file sharing storage
    let persistence_config = PersistenceConfig {
        db_path,
        ..Default::default()
    };

    match P2pClient::new_with_config_and_persistence(
        keypair,
        final_nickname,
        output_dir,
        Some(persistence_config),
    ) {
        Ok((mut client, event_receiver)) => {
            // Run migrations for the database to add thumbnail_path column
            let db_conn = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    sea_orm::Database::connect(format!(
                        "sqlite://{}?mode=rwc",
                        db_path_clone.display()
                    ))
                    .await
                })
            })
            .map_err(|e| Error::Io(format!("Failed to connect to database: {}", e)))?;

            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    gigi_store::migration::Migrator::up(&db_conn, None)
                        .await
                        .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))
                })
            })
            .map_err(|e| Error::Io(format!("Failed to run migrations: {}", e)))?;

            // Initialize file_sharing_store for thumbnail commands
            let db_conn_clone = db_conn.clone();
            let file_sharing_store = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { gigi_store::FileSharingStore::new(db_conn_clone).await })
            })
            .map_err(|e| Error::Io(format!("Failed to create file sharing store: {}", e)))?;

            let mut file_sharing_store_guard = state.file_sharing_store.write().await;
            *file_sharing_store_guard = Some(file_sharing_store);
            drop(file_sharing_store_guard);

            // Initialize thumbnail_store for mapping file paths to thumbnails
            let thumbnail_store = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { gigi_store::ThumbnailStore::new(db_conn).await })
            })
            .map_err(|e| Error::Io(format!("Failed to create thumbnail store: {}", e)))?;

            let mut thumbnail_store_guard = state.thumbnail_store.write().await;
            *thumbnail_store_guard = Some(thumbnail_store);
            drop(thumbnail_store_guard);

            // Initialize message_store for get_messages and search_messages commands
            let message_store = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { gigi_store::MessageStore::new(db_path_clone.clone()).await })
            })
            .map_err(|e| Error::Io(format!("Failed to create message store: {}", e)))?;

            let mut message_store_guard = state.message_store.write().await;
            *message_store_guard = Some(message_store);
            drop(message_store_guard);

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
            let state_for_events = (*state).clone();
            tokio::spawn(async move {
                let mut receiver = receiver;
                while let Some(event) = receiver.next().await {
                    if let Err(e) = handle_p2p_event(
                        event,
                        &p2p_client,
                        &active_downloads,
                        &app_handle_clone,
                        &state_for_events,
                    )
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
    let msg_id = uuid::Uuid::new_v4().to_string();
    let mut client_guard = state.p2p_client.lock().await;

    // Get peer_id before consuming client
    let peer_id = client_guard
        .as_ref()
        .and_then(|c| c.get_peer_id_by_nickname(nickname))
        .map(|id| id.to_string());

    // Get local nickname
    let local_nickname = {
        let config = state.config.read().await;
        config.nickname.clone()
    };

    if let Some(client) = client_guard.as_mut() {
        client
            .send_direct_message(nickname, message.to_string())
            .map_err(|e| Error::CommandFailed(format!("Failed to send message: {}", e)))?;

        // Save message to database asynchronously (non-blocking)
        let message_store = state.message_store.clone();
        let message_copy = message.to_string();
        let nickname_copy = nickname.to_string();
        let msg_id_clone = msg_id.clone();
        let peer_id_for_async = peer_id.clone();

        tokio::spawn(async move {
            if let Some(store) = message_store.read().await.as_ref() {
                if let Some(peer_id) = peer_id_for_async {
                    let stored_msg = StoredMessage {
                        id: msg_id_clone,
                        msg_type: MessageType::Direct,
                        direction: MessageDirection::Sent,
                        content: MessageContent::Text {
                            text: message_copy.clone(),
                        },
                        sender_nickname: local_nickname.clone(),
                        recipient_nickname: Some(nickname_copy.clone()),
                        group_name: None,
                        peer_id,
                        timestamp: chrono::Utc::now(),
                        created_at: chrono::Utc::now(),
                        delivered: false,
                        delivered_at: None,
                        read: false,
                        read_at: None,
                        sync_status: gigi_store::SyncStatus::Synced,
                        sync_attempts: 0,
                        last_sync_attempt: None,
                        expires_at: chrono::Utc::now() + ChronoDuration::days(30),
                    };
                    if let Err(e) = store.store_message(stored_msg).await {
                        tracing::error!("Failed to store sent message to database: {}", e);
                    }
                }
            }
        });

        Ok(msg_id)
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
    let msg_id = uuid::Uuid::new_v4().to_string();
    let mut client_guard = state.p2p_client.lock().await;

    // Get peer_id before consuming client
    let peer_id = client_guard
        .as_ref()
        .and_then(|c| c.get_peer_id_by_nickname(nickname))
        .map(|id| id.to_string());

    // Get local nickname
    let local_nickname = {
        let config = state.config.read().await;
        config.nickname.clone()
    };

    if let Some(client) = client_guard.as_mut() {
        client
            .send_direct_share_group_message(nickname, group_id.to_string(), group_name.to_string())
            .map_err(|e| {
                Error::CommandFailed(format!("Failed to send group share message: {}", e))
            })?;

        // Save message to database asynchronously (non-blocking)
        let message_store = state.message_store.clone();
        let nickname_copy = nickname.to_string();
        let group_id_copy = group_id.to_string();
        let group_name_copy = group_name.to_string();
        let msg_id_clone = msg_id.clone();
        let peer_id_for_async = peer_id.clone();

        tokio::spawn(async move {
            if let Some(store) = message_store.read().await.as_ref() {
                if let Some(peer_id) = peer_id_for_async {
                    let stored_msg = StoredMessage {
                        id: msg_id_clone,
                        msg_type: MessageType::Direct,
                        direction: MessageDirection::Sent,
                        content: MessageContent::ShareGroup {
                            group_id: group_id_copy.clone(),
                            group_name: group_name_copy.clone(),
                            inviter_nickname: local_nickname.clone(),
                        },
                        sender_nickname: local_nickname.clone(),
                        recipient_nickname: Some(nickname_copy.clone()),
                        group_name: Some(group_name_copy.clone()),
                        peer_id,
                        timestamp: chrono::Utc::now(),
                        created_at: chrono::Utc::now(),
                        delivered: false,
                        delivered_at: None,
                        read: false,
                        read_at: None,
                        sync_status: gigi_store::SyncStatus::Synced,
                        sync_attempts: 0,
                        last_sync_attempt: None,
                        expires_at: chrono::Utc::now() + ChronoDuration::days(30),
                    };
                    if let Err(e) = store.store_message(stored_msg).await {
                        tracing::error!("Failed to store group share message to database: {}", e);
                    }
                }
            }
        });

        Ok(msg_id)
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
    let msg_id = uuid::Uuid::new_v4().to_string();
    let mut client_guard = state.p2p_client.lock().await;

    // Get local nickname
    let local_nickname = {
        let config = state.config.read().await;
        config.nickname.clone()
    };

    if let Some(client) = client_guard.as_mut() {
        client
            .send_group_message(group_id, message.to_string())
            .map_err(|e| Error::CommandFailed(format!("Failed to send group message: {}", e)))?;

        // Save message to database asynchronously (non-blocking)
        let message_store = state.message_store.clone();
        let group_id_copy = group_id.to_string();
        let message_copy = message.to_string();
        let msg_id_clone = msg_id.clone();

        tokio::spawn(async move {
            if let Some(store) = message_store.read().await.as_ref() {
                let stored_msg = StoredMessage {
                    id: msg_id_clone,
                    msg_type: MessageType::Group,
                    direction: MessageDirection::Sent,
                    content: MessageContent::Text {
                        text: message_copy.clone(),
                    },
                    sender_nickname: local_nickname.clone(),
                    recipient_nickname: None,
                    group_name: Some(group_id_copy.clone()),
                    peer_id: "local".to_string(),
                    timestamp: chrono::Utc::now(),
                    created_at: chrono::Utc::now(),
                    delivered: false,
                    delivered_at: None,
                    read: false,
                    read_at: None,
                    sync_status: gigi_store::SyncStatus::Synced,
                    sync_attempts: 0,
                    last_sync_attempt: None,
                    expires_at: chrono::Utc::now() + ChronoDuration::days(30),
                };
                if let Err(e) = store.store_message(stored_msg).await {
                    tracing::error!("Failed to store group message to database: {}", e);
                }
            }
        });

        Ok(msg_id)
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

/// Get thumbnail for a file by its path
#[tauri::command]
pub(crate) async fn get_file_thumbnail(
    file_path: String,
    state: State<'_, PluginState>,
) -> Result<String> {
    use base64::prelude::*;
    use gigi_store::thumbnail;

    let original_file_path = std::path::PathBuf::from(&file_path);

    // Check if the file exists first
    if !original_file_path.exists() {
        tracing::warn!("File not found for thumbnail: {}", file_path);
        return Ok(String::new());
    }

    // Get download directory
    let config_guard = state.config.read().await;
    let download_dir = std::path::PathBuf::from(&config_guard.download_folder);
    let thumbnail_dir = download_dir.join("thumbnails");
    drop(config_guard);

    // Ensure thumbnail directory exists
    if let Err(e) = tokio::fs::create_dir_all(&thumbnail_dir).await {
        tracing::warn!("Failed to create thumbnail directory: {}", e);
        return Ok(String::new());
    }

    // Get thumbnail store
    let thumbnail_store = state.thumbnail_store.read().await;
    let _store = thumbnail_store
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Thumbnail store not initialized".to_string()))?;

    // Try to get existing thumbnail path from database
    let thumbnail_path = match _store.get_thumbnail(&file_path).await {
        Ok(Some(path)) => {
            // Thumbnail exists in database
            path
        }
        Ok(None) => {
            // Thumbnail not found in database, generate it on-demand
            tracing::info!("Thumbnail not in database, generating for: {}", file_path);

            // Generate thumbnail
            match thumbnail::generate_thumbnail(&original_file_path, &thumbnail_dir, (200, 200), 70).await {
                Ok(thumbnail_filename) => {
                    let _full_thumbnail_path = thumbnail_dir.join(&thumbnail_filename);

                    // Store the mapping in database
                    let thumbnail_store_clone = state.thumbnail_store.clone();
                    let file_path_clone = file_path.clone();
                    let thumbnail_filename_clone = thumbnail_filename.clone();
                    tokio::spawn(async move {
                        if let Some(store) = thumbnail_store_clone.read().await.as_ref() {
                            if let Err(e) = store.store_thumbnail(&file_path_clone, &thumbnail_filename_clone).await {
                                tracing::error!("Failed to store thumbnail mapping: {}", e);
                            }
                        }
                    });

                    thumbnail_filename
                }
                Err(e) => {
                    tracing::warn!("Failed to generate thumbnail: {}", e);
                    return Ok(String::new());
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to get thumbnail path: {}", e);
            return Ok(String::new());
        }
    };

    drop(thumbnail_store);

    let thumb_file_path = thumbnail_dir.join(&thumbnail_path);

    // Read thumbnail file
    let thumbnail_bytes = match tokio::fs::read(&thumb_file_path).await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::warn!("Failed to read thumbnail file {:?}: {}", thumb_file_path, e);
            // Thumbnail file doesn't exist even though database entry exists
            // Try to regenerate it
            match thumbnail::generate_thumbnail(&original_file_path, &thumbnail_dir, (200, 200), 70).await {
                Ok(thumbnail_filename) => {
                    let full_thumbnail_path = thumbnail_dir.join(&thumbnail_filename);

                    // Update the mapping in database
                    let thumbnail_store_clone = state.thumbnail_store.clone();
                    let file_path_clone = file_path.clone();
                    tokio::spawn(async move {
                        if let Some(store) = thumbnail_store_clone.read().await.as_ref() {
                            if let Err(e) = store.store_thumbnail(&file_path_clone, &thumbnail_filename).await {
                                tracing::error!("Failed to store thumbnail mapping: {}", e);
                            }
                        }
                    });

                    // Read the newly generated thumbnail
                    match tokio::fs::read(&full_thumbnail_path).await {
                        Ok(bytes) => bytes,
                        Err(e) => {
                            tracing::warn!("Failed to read regenerated thumbnail: {}", e);
                            return Ok(String::new());
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to regenerate thumbnail: {}", e);
                    return Ok(String::new());
                }
            }
        }
    };

    // Convert to base64
    let base64_thumbnail = BASE64_STANDARD.encode(&thumbnail_bytes);

    Ok(format!("data:image/jpeg;base64,{}", base64_thumbnail))
}

/// Get full-size image for a shared file by file path
#[tauri::command]
pub(crate) async fn get_full_image_by_path(
    file_path: String,
    _state: State<'_, PluginState>,
) -> Result<String> {
    use base64::prelude::*;

    let original_file_path = std::path::PathBuf::from(&file_path);

    // Check if file exists
    if !original_file_path.exists() {
        return Err(Error::CommandFailed(format!("File not found: {}", file_path)));
    }

    // Read full image file directly from the provided path
    let image_bytes = tokio::fs::read(&original_file_path)
        .await
        .map_err(|e| Error::Io(format!("Failed to read image: {}", e)))?;

    // Convert to base64
    let base64_image = BASE64_STANDARD.encode(&image_bytes);

    Ok(format!("data:image/jpeg;base64,{}", base64_image))
}

/// Get full-size image for a shared file by share code (for files shared by current user)
#[tauri::command]
pub(crate) async fn get_full_image(
    share_code: String,
    state: State<'_, PluginState>,
) -> Result<String> {
    use base64::prelude::*;

    // Get file info from database
    let file_sharing_store = state.file_sharing_store.read().await;
    let _store = file_sharing_store
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("File sharing not initialized".to_string()))?;

    let file_info = _store
        .get_shared_file(&share_code)
        .await
        .map_err(|e| Error::Io(format!("Failed to get shared file: {}", e)))?
        .ok_or_else(|| Error::CommandFailed("File not found".to_string()))?;
    drop(file_sharing_store);

    // Read full image file
    let image_bytes = tokio::fs::read(&file_info.file_path)
        .await
        .map_err(|e| Error::Io(format!("Failed to read image: {}", e)))?;

    // Convert to base64
    let base64_image = BASE64_STANDARD.encode(&image_bytes);

    Ok(format!("data:image/jpeg;base64,{}", base64_image))
}

/// Get message history from backend
#[tauri::command]
pub(crate) async fn get_messages(
    peer_id: String,
    limit: usize,
    offset: usize,
    state: State<'_, PluginState>,
) -> Result<serde_json::Value> {
    let message_store = state.message_store.read().await;
    let file_sharing_store = state.file_sharing_store.read().await;
    let store = message_store
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Message store not initialized".to_string()))?;

    // Check if peer_id is a group or peer
    // Groups have a different format (typically uuid-like or specific format)
    // Try peer_id query first, then fallback to group/conversation by nickname
    let stored_messages = if let Ok(msgs) = store.get_conversation_by_peer_id(&peer_id, limit, offset).await {
        if !msgs.is_empty() {
            msgs
        } else if let Ok(group_msgs) = store.get_group_messages(&peer_id, limit, offset).await {
            if !group_msgs.is_empty() {
                group_msgs
            } else if let Ok(conv_msgs) = store.get_conversation(&peer_id, limit, offset).await {
                conv_msgs
            } else {
                return Err(Error::Io("Failed to get messages".to_string()));
            }
        } else if let Ok(conv_msgs) = store.get_conversation(&peer_id, limit, offset).await {
            conv_msgs
        } else {
            return Err(Error::Io("Failed to get messages".to_string()));
        }
    } else if let Ok(group_msgs) = store.get_group_messages(&peer_id, limit, offset).await {
        if !group_msgs.is_empty() {
            group_msgs
        } else if let Ok(conv_msgs) = store.get_conversation(&peer_id, limit, offset).await {
            conv_msgs
        } else {
            return Err(Error::Io("Failed to get messages".to_string()));
        }
    } else if let Ok(conv_msgs) = store.get_conversation(&peer_id, limit, offset).await {
        conv_msgs
    } else {
        return Err(Error::Io("Failed to get messages".to_string()));
    };

    drop(message_store);

    // Collect share codes and extract file paths from FileShare and FileShareWithThumbnail
    let mut share_codes_to_query: Vec<(usize, String)> = Vec::new();

    for (idx, msg) in stored_messages.iter().enumerate() {
        match &msg.content {
            MessageContent::FileShare {
                share_code,
                file_type,
                filename: _,
                file_size: _,
            } => {
                if file_type.starts_with("image/") {
                    share_codes_to_query.push((idx, share_code.clone()));
                }
            }
            MessageContent::FileShareWithThumbnail {
                share_code,
                file_type,
                ..
            } => {
                if file_type.starts_with("image/") {
                    share_codes_to_query.push((idx, share_code.clone()));
                }
            }
            _ => {}
        }
    }

    // Fetch file paths for shared files (outgoing messages)
    let mut file_path_map: std::collections::HashMap<usize, Option<String>> =
        std::collections::HashMap::new();
    for (idx, share_code) in &share_codes_to_query {
        if let Some(ref fs_store) = *file_sharing_store {
            match fs_store.get_shared_file(share_code).await {
                Ok(Some(file_info)) => {
                    file_path_map.insert(*idx, Some(file_info.file_path));
                }
                _ => {
                    file_path_map.insert(*idx, None);
                }
            }
        }
    }

    // For downloaded files (received messages), the file path would be in the download folder
    // We need to look up by share_code and construct the path
    for (idx, msg) in stored_messages.iter().enumerate() {
        if matches!(msg.direction, MessageDirection::Received) {
            let filename = match &msg.content {
                MessageContent::FileShare { filename, .. } => filename,
                MessageContent::FileShareWithThumbnail { filename, .. } => filename,
                _ => continue,
            };

            let config_guard = state.config.read().await;
            let download_dir = std::path::PathBuf::from(&config_guard.download_folder);
            drop(config_guard);

            // Construct the downloaded file path
            let downloaded_file_path = download_dir.join(filename);
            if downloaded_file_path.exists() {
                file_path_map.insert(idx, Some(downloaded_file_path.to_string_lossy().to_string()));
            }
        }
    }

    // Convert stored messages to JSON format compatible with frontend
    let messages_json: Vec<serde_json::Value> = stored_messages
        .into_iter()
        .enumerate()
        .map(|(idx, msg)| {
            // Get file path for thumbnail lookup
            let file_path = file_path_map.get(&idx).and_then(|v| v.clone());

            // Get thumbnail path - first check content (for new messages), then use file_path for lookup
            let thumbnail_path = match &msg.content {
                MessageContent::FileShareWithThumbnail { thumbnail_path: Some(path), .. } => Some(path.clone()),
                _ => None, // Thumbnail will be looked up by file_path from thumbnail_store
            };

            serde_json::json!({
                "id": msg.id,
                "from_peer_id": msg.peer_id,
                "from_nickname": msg.sender_nickname,
                "content": match &msg.content {
                    MessageContent::Text { text } => text.clone(),
                    MessageContent::FileShare { filename, share_code, .. } => {
                        format!("[File: {}] ({})", filename, share_code)
                    }
                    MessageContent::FileShareWithThumbnail { filename, share_code, .. } => {
                        format!("[File: {}] ({})", filename, share_code)
                    }
                    MessageContent::ShareGroup { group_name, .. } => {
                        format!("[Group Share: {}]", group_name)
                    }
                },
                "timestamp": msg.timestamp.timestamp_millis(),
                "isOutgoing": matches!(msg.direction, MessageDirection::Sent),
                "messageType": match &msg.content {
                    MessageContent::Text { .. } => "text",
                    MessageContent::FileShare { file_type, .. } if file_type.starts_with("image/") => "image",
                    MessageContent::FileShare { .. } => "file",
                    MessageContent::FileShareWithThumbnail { file_type, .. } if file_type.starts_with("image/") => "image",
                    MessageContent::FileShareWithThumbnail { .. } => "file",
                    MessageContent::ShareGroup { .. } => "group_share",
                },
                "shareCode": match &msg.content {
                    MessageContent::FileShare { share_code, .. } => Some(share_code.clone()),
                    MessageContent::FileShareWithThumbnail { share_code, .. } => Some(share_code.clone()),
                    _ => None,
                },
                "filename": match &msg.content {
                    MessageContent::FileShare { filename, .. } => Some(filename.clone()),
                    MessageContent::FileShareWithThumbnail { filename, .. } => Some(filename.clone()),
                    _ => None,
                },
                "fileSize": match &msg.content {
                    MessageContent::FileShare { file_size, .. } => Some(file_size),
                    MessageContent::FileShareWithThumbnail { file_size, .. } => Some(file_size),
                    _ => None,
                },
                "fileType": match &msg.content {
                    MessageContent::FileShare { file_type, .. } => Some(file_type.clone()),
                    MessageContent::FileShareWithThumbnail { file_type, .. } => Some(file_type.clone()),
                    _ => None,
                },
                "thumbnailPath": thumbnail_path,
                "filePath": file_path,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "messages": messages_json,
        "peer_id": peer_id,
        "limit": limit,
        "offset": offset
    }))
}

/// Search messages
#[tauri::command]
pub(crate) async fn search_messages(
    query: String,
    peer_id: Option<String>,
    state: State<'_, PluginState>,
) -> Result<serde_json::Value> {
    let message_store = state.message_store.read().await;
    let file_sharing_store = state.file_sharing_store.read().await;
    let store = message_store
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Message store not initialized".to_string()))?;

    // For now, we'll query all messages and filter on the query
    // A more efficient implementation would add SQL LIKE queries
    let limit = 100; // Limit search results
    let offset = 0;

    let stored_messages = if let Some(ref peer) = peer_id {
        // Search within specific conversation or group
        match store.get_group_messages(peer, limit, offset).await {
            Ok(group_msgs) if !group_msgs.is_empty() => group_msgs,
            Ok(_) => {
                // Empty group messages, try as a peer conversation
                store
                    .get_conversation(peer, limit, offset)
                    .await
                    .map_err(|e| Error::Io(format!("Failed to get conversation: {}", e)))?
            }
            Err(e) => {
                // Error fetching group messages, try as a peer conversation
                match store.get_conversation(peer, limit, offset).await {
                    Ok(conv_msgs) => conv_msgs,
                    Err(conv_e) => {
                        return Err(Error::Io(format!(
                            "Failed to get messages (group: {}, conversation: {})",
                            e, conv_e
                        )))
                    }
                }
            }
        }
    } else {
        // Search all messages - this requires a different query
        // For now, return empty to avoid expensive full scan
        vec![]
    };

    // Filter messages by query first
    let filtered_stored: Vec<StoredMessage> = stored_messages
        .into_iter()
        .filter(|msg| {
            // Search in content (text messages only for now)
            match &msg.content {
                MessageContent::Text { text } => {
                    text.to_lowercase().contains(&query.to_lowercase())
                }
                _ => false,
            }
        })
        .collect();

    // Collect share codes and extract file paths from FileShareWithThumbnail
    let mut share_codes_to_query: Vec<(usize, String)> = Vec::new();

    for (idx, msg) in filtered_stored.iter().enumerate() {
        match &msg.content {
            MessageContent::FileShare {
                share_code,
                file_type,
                filename: _,
                file_size: _,
            } => {
                if file_type.starts_with("image/") {
                    share_codes_to_query.push((idx, share_code.clone()));
                }
            }
            MessageContent::FileShareWithThumbnail {
                file_type,
                ..
            } => {
                if file_type.starts_with("image/") {
                    // For FileShareWithThumbnail messages, we'll extract file_path from thumbnail_store
                    share_codes_to_query.push((idx, String::new()));
                }
            }
            _ => {}
        }
    }

    // Fetch file paths for shared files
    let mut file_path_map: std::collections::HashMap<usize, Option<String>> =
        std::collections::HashMap::new();
    for (idx, share_code) in &share_codes_to_query {
        if let Some(ref fs_store) = *file_sharing_store {
            match fs_store.get_shared_file(share_code).await {
                Ok(Some(file_info)) => {
                    file_path_map.insert(*idx, Some(file_info.file_path));
                }
                _ => {
                    file_path_map.insert(*idx, None);
                }
            }
        }
    }

    // For downloaded files (received messages), the file path would be in the download folder
    // We need to look up by share_code and construct the path
    for (idx, msg) in filtered_stored.iter().enumerate() {
        if matches!(msg.direction, MessageDirection::Received) {
            if let MessageContent::FileShare { filename, .. } = &msg.content {
                let config_guard = state.config.read().await;
                let download_dir = std::path::PathBuf::from(&config_guard.download_folder);
                drop(config_guard);

                // Construct the downloaded file path
                let downloaded_file_path = download_dir.join(filename);
                if downloaded_file_path.exists() {
                    file_path_map.insert(idx, Some(downloaded_file_path.to_string_lossy().to_string()));
                }
            }
        }
    }

    // Convert filtered messages to JSON format
    let filtered_messages: Vec<serde_json::Value> = filtered_stored
        .into_iter()
        .enumerate()
        .map(|(idx, msg)| {
            // Get file path for thumbnail lookup
            let file_path = file_path_map.get(&idx).and_then(|v| v.clone());

            // Get thumbnail path - first check content (for new messages), then use file_path for lookup
            let thumbnail_path = match &msg.content {
                MessageContent::FileShareWithThumbnail { thumbnail_path: Some(path), .. } => Some(path.clone()),
                _ => None, // Thumbnail will be looked up by file_path from thumbnail_store
            };

            serde_json::json!({
                "id": msg.id,
                "from_peer_id": msg.peer_id,
                "from_nickname": msg.sender_nickname,
                "content": match &msg.content {
                    MessageContent::Text { text } => text.clone(),
                    MessageContent::FileShare { filename, share_code, .. } => {
                        format!("[File: {}] ({})", filename, share_code)
                    }
                    MessageContent::FileShareWithThumbnail { filename, share_code, .. } => {
                        format!("[File: {}] ({})", filename, share_code)
                    }
                    MessageContent::ShareGroup { group_name, .. } => {
                        format!("[Group Share: {}]", group_name)
                    }
                },
                "timestamp": msg.timestamp.timestamp_millis(),
                "isOutgoing": matches!(msg.direction, MessageDirection::Sent),
                "messageType": match &msg.content {
                    MessageContent::Text { .. } => "text",
                    MessageContent::FileShare { file_type, .. } if file_type.starts_with("image/") => "image",
                    MessageContent::FileShare { .. } => "file",
                    MessageContent::FileShareWithThumbnail { file_type, .. } if file_type.starts_with("image/") => "image",
                    MessageContent::FileShareWithThumbnail { .. } => "file",
                    MessageContent::ShareGroup { .. } => "group_share",
                },
                "shareCode": match &msg.content {
                    MessageContent::FileShare { share_code, .. } => Some(share_code.clone()),
                    MessageContent::FileShareWithThumbnail { share_code, .. } => Some(share_code.clone()),
                    _ => None,
                },
                "filename": match &msg.content {
                    MessageContent::FileShare { filename, .. } => Some(filename.clone()),
                    MessageContent::FileShareWithThumbnail { filename, .. } => Some(filename.clone()),
                    _ => None,
                },
                "fileSize": match &msg.content {
                    MessageContent::FileShare { file_size, .. } => Some(file_size),
                    MessageContent::FileShareWithThumbnail { file_size, .. } => Some(file_size),
                    _ => None,
                },
                "fileType": match &msg.content {
                    MessageContent::FileShare { file_type, .. } => Some(file_type.clone()),
                    MessageContent::FileShareWithThumbnail { file_type, .. } => Some(file_type.clone()),
                    _ => None,
                },
                "thumbnailPath": thumbnail_path,
                "filePath": file_path,
            })
        })
        .collect();

    drop(message_store);
    drop(file_sharing_store);

    Ok(serde_json::json!({
        "messages": filtered_messages,
        "query": query,
        "peer_id": peer_id
    }))
}

/// Clear messages and delete thumbnail files for both incoming and outgoing images
#[tauri::command]
pub(crate) async fn clear_messages_with_thumbnails(
    peer_id: String,
    state: State<'_, PluginState>,
) -> Result<usize> {
    let message_store = state.message_store.read().await;
    let file_sharing_store = state.file_sharing_store.read().await;
    let thumbnail_store = state.thumbnail_store.read().await;

    let store = message_store
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Message store not initialized".to_string()))?;

    let fs_store = file_sharing_store
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("File sharing store not initialized".to_string()))?;

    let thumb_store = thumbnail_store
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Thumbnail store not initialized".to_string()))?;

    let count = store
        .clear_conversation_with_thumbnails(&peer_id, fs_store, thumb_store)
        .await
        .map_err(|e| Error::Io(format!("Failed to clear messages: {}", e)))?;

    drop(message_store);
    drop(file_sharing_store);
    drop(thumbnail_store);

    Ok(count)
}
