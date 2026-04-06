// Event handlers for P2P events

use crate::models::{DownloadProgress, GroupMessage, Message, Peer};
use gigi_p2p::P2pEvent;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::models::PluginState;
use chrono::Duration as ChronoDuration;
use gigi_store::{MessageContent, MessageDirection, MessageType, StoredMessage};

type ActiveDownloads = Arc<Mutex<HashMap<String, DownloadProgress>>>;

/// Generate thumbnail for downloaded image file
async fn generate_thumbnail_for_image<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
    file_path: &std::path::PathBuf,
    _share_code: &str,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    use gigi_store::thumbnail;

    // Check if it's an image file
    if !thumbnail::is_image_file(file_path) {
        return Ok(None);
    }

    info!("Generating thumbnail for image: {}", file_path.display());

    // Get download directory from app config
    let download_dir = app_handle
        .path()
        .download_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get download directory: {}", e))?;

    #[cfg(target_os = "android")]
    let thumbnail_dir = download_dir.join("thumbnails");

    #[cfg(not(target_os = "android"))]
    let thumbnail_dir = download_dir.join("gigi/thumbnails");

    // Ensure thumbnail directory exists
    if let Err(e) = tokio::fs::create_dir_all(&thumbnail_dir).await {
        tracing::warn!("Failed to create thumbnail directory: {}", e);
        return Ok(None);
    }

    // Generate thumbnail
    match thumbnail::generate_thumbnail(file_path, &thumbnail_dir, (200, 200), 70).await {
        Ok(thumbnail_filename) => {
            let full_thumbnail_path = thumbnail_dir.join(&thumbnail_filename);
            info!("Thumbnail generated: {}", full_thumbnail_path.display());

            // Update thumbnail path in database
            // We need access to the file_sharing_store state
            // For now, we'll emit an event that the frontend can use to update the thumbnail path
            // The actual database update should be done via a dedicated command
            Ok(Some(full_thumbnail_path.to_string_lossy().to_string()))
        }
        Err(e) => {
            tracing::warn!(
                "Failed to generate thumbnail for {}: {}",
                file_path.display(),
                e
            );
            Ok(None)
        }
    }
}

/// Handle P2P events and emit corresponding frontend events
pub async fn handle_p2p_event<R: tauri::Runtime>(
    event: P2pEvent,
    p2p_client: &Arc<Mutex<Option<gigi_p2p::P2pClient>>>,
    active_downloads: &ActiveDownloads,
    app_handle: &AppHandle<R>,
    state: &PluginState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match event {
        P2pEvent::PeerDiscovered {
            peer_id, nickname, ..
        } => handle_peer_discovered(app_handle, &peer_id, &nickname)?,
        P2pEvent::PeerExpired { peer_id, nickname } => {
            handle_peer_expired(app_handle, &peer_id, &nickname)?
        }
        P2pEvent::NicknameUpdated { peer_id, nickname } => {
            handle_nickname_updated(app_handle, &peer_id, &nickname)?
        }
        P2pEvent::DirectFileShareMessage {
            from,
            from_nickname,
            share_code,
            filename,
            file_size,
            file_type,
        } => {
            handle_direct_file_share_message(
                app_handle,
                p2p_client,
                &from,
                &from_nickname,
                &share_code,
                &filename,
                file_size,
                &file_type,
            )
            .await?
        }
        P2pEvent::DirectMessage {
            from,
            from_nickname,
            message,
        } => handle_direct_message(app_handle, &from, &from_nickname, &message).await?,
        P2pEvent::DirectGroupShareMessage {
            from,
            from_nickname,
            group_id,
            group_name,
        } => handle_direct_group_share_message(
            app_handle,
            &from,
            &from_nickname,
            &group_id,
            &group_name,
        )?,
        P2pEvent::GroupFileShareMessage {
            from,
            from_nickname,
            group,
            share_code,
            filename,
            file_size,
            file_type,
            message: _,
        } => {
            handle_group_file_share_message(
                app_handle,
                p2p_client,
                &from,
                &from_nickname,
                &group,
                &share_code,
                &filename,
                file_size,
                &file_type,
            )
            .await?
        }
        P2pEvent::GroupMessage {
            from,
            from_nickname,
            group,
            message,
        } => handle_group_message(app_handle, &from, &from_nickname, &group, &message).await?,
        P2pEvent::FileShareRequest {
            from,
            from_nickname,
            share_code,
            filename,
            size,
        } => handle_file_share_request(
            app_handle,
            &from,
            &from_nickname,
            &share_code,
            &filename,
            size,
        )?,
        P2pEvent::FileDownloadProgress {
            download_id,
            filename,
            share_code,
            from_peer_id,
            from_nickname,
            downloaded_chunks,
            total_chunks,
        } => {
            handle_file_download_progress(
                app_handle,
                active_downloads,
                &download_id,
                &filename,
                &share_code,
                &from_peer_id,
                &from_nickname,
                downloaded_chunks,
                total_chunks,
            )
            .await?
        }
        P2pEvent::FileDownloadCompleted {
            download_id,
            filename,
            share_code,
            from_peer_id,
            from_nickname,
            path,
        } => {
            handle_file_download_completed(
                app_handle,
                active_downloads,
                &download_id,
                &filename,
                &share_code,
                &from_peer_id,
                &from_nickname,
                &path,
                state,
            )
            .await?
        }
        P2pEvent::FileDownloadStarted {
            from,
            from_nickname,
            filename,
            download_id,
            share_code,
        } => handle_file_download_started(
            app_handle,
            &from,
            &from_nickname,
            &filename,
            &download_id,
            &share_code,
        )?,
        P2pEvent::FileDownloadFailed {
            download_id,
            filename,
            share_code,
            from_peer_id,
            from_nickname,
            error,
        } => {
            handle_file_download_failed(
                app_handle,
                active_downloads,
                &download_id,
                &filename,
                &share_code,
                &from_peer_id,
                &from_nickname,
                &error,
            )
            .await?
        }
        P2pEvent::Connected { peer_id, nickname } => {
            handle_connected(app_handle, &peer_id, &nickname)?
        }
        P2pEvent::Disconnected { peer_id, nickname } => {
            handle_disconnected(app_handle, &peer_id, &nickname)?
        }
        P2pEvent::Error(error) => handle_error(app_handle, &error)?,
        _ => {}
    }
    Ok(())
}

fn handle_peer_discovered<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    peer_id: &libp2p::PeerId,
    nickname: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Emitting peer-discovered event for {} ({})",
        nickname, peer_id
    );
    let peer = Peer {
        id: peer_id.to_string(),
        nickname: nickname.to_string(),
        capabilities: vec!["messaging".to_string(), "file_transfer".to_string()],
    };
    app_handle
        .emit("peer-discovered", &peer)
        .map_err(|e| format!("Failed to emit peer-discovered: {}", e))?;
    Ok(())
}

fn handle_peer_expired<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    peer_id: &libp2p::PeerId,
    nickname: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    app_handle.emit(
        "peer-expired",
        &json!({
            "peer_id": peer_id.to_string(),
            "nickname": nickname
        }),
    )?;
    Ok(())
}

fn handle_nickname_updated<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    peer_id: &libp2p::PeerId,
    nickname: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    app_handle.emit(
        "nickname-updated",
        &json!({
            "peer_id": peer_id.to_string(),
            "nickname": nickname
        }),
    )?;
    Ok(())
}

async fn handle_direct_file_share_message<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    p2p_client: &Arc<Mutex<Option<gigi_p2p::P2pClient>>>,
    from: &libp2p::PeerId,
    from_nickname: &str,
    share_code: &str,
    filename: &str,
    file_size: u64,
    file_type: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Received direct file share message from {} ({}) for file {} with share code {}",
        from_nickname, from, filename, share_code
    );

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    // Save file message to database asynchronously (non-blocking)
    // We save metadata only, not file content or thumbnails
    let state = app_handle.state::<PluginState>();
    let message_store = state.message_store.clone();
    let msg_id = uuid::Uuid::new_v4().to_string();
    let from_peer_id = from.to_string();
    let from_nickname_copy = from_nickname.to_string();
    let share_code_copy = share_code.to_string();
    let filename_copy = filename.to_string();
    let file_type_copy = file_type.to_string();

    // Get local nickname before spawning async task
    let local_nickname = {
        let config = state.config.read().await;
        config.nickname.clone()
    };

    tokio::spawn(async move {
        if let Some(store) = message_store.read().await.as_ref() {
            let stored_msg = StoredMessage {
                id: msg_id.clone(),
                msg_type: MessageType::Direct,
                direction: MessageDirection::Received,
                content: MessageContent::FileShare {
                    share_code: share_code_copy.clone(),
                    filename: filename_copy.clone(),
                    file_size,
                    file_type: file_type_copy.clone(),
                },
                sender_nickname: from_nickname_copy.clone(),
                recipient_nickname: Some(local_nickname.clone()),
                group_name: None,
                peer_id: from_peer_id.clone(),
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
                tracing::error!("Failed to store file message to database: {}", e);
            }
        }
    });

    // Check if it's an image file and auto-download
    if file_type.starts_with("image/") {
        info!("Auto-downloading image file: {}", filename);
        if let Some(client) = p2p_client.lock().await.as_mut() {
            match client.download_file(from_nickname, share_code) {
                Ok(download_id) => {
                    info!(
                        "Started auto-download for image: {} (download_id: {})",
                        filename, download_id
                    );
                    emit_image_message_received(
                        app_handle,
                        from,
                        from_nickname,
                        share_code,
                        filename,
                        file_size,
                        file_type,
                        timestamp,
                        None,
                        Some(download_id.clone()),
                    )?;
                }
                Err(e) => {
                    error!(
                        "Failed to start auto-download for image {}: {}",
                        filename, e
                    );
                    emit_image_message_received(
                        app_handle,
                        from,
                        from_nickname,
                        share_code,
                        filename,
                        file_size,
                        file_type,
                        timestamp,
                        Some(format!("Failed to download: {}", e)),
                        None,
                    )?;
                }
            }
        }
    } else {
        // For non-image files, just emit the event without downloading
        app_handle.emit(
            "file-message-received",
            &json!({
                "from_peer_id": from.to_string(),
                "from_nickname": from_nickname,
                "share_code": share_code,
                "filename": filename,
                "file_size": file_size,
                "file_type": file_type,
                "timestamp": timestamp,
            }),
        )?;
    }
    Ok(())
}

async fn handle_direct_message<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    from: &libp2p::PeerId,
    from_nickname: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let msg_id = uuid::Uuid::new_v4().to_string();
    let msg = Message {
        id: msg_id.clone(),
        from_peer_id: from.to_string(),
        from_nickname: from_nickname.to_string(),
        content: message.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
    };
    app_handle.emit("message-received", &msg)?;

    // Save message to database asynchronously (non-blocking)
    // Get local peer nickname from config if available
    let state = app_handle.state::<PluginState>();
    let message_store = state.message_store.clone();

    // Clone data for async task
    let message_copy = message.to_string();
    let from_id = from.to_string();
    let from_nick = from_nickname.to_string();

    // Get local nickname before spawning async task
    let local_nickname = {
        let config = state.config.read().await;
        config.nickname.clone()
    };

    tokio::spawn(async move {
        if let Some(store) = message_store.read().await.as_ref() {
            let stored_msg = StoredMessage {
                id: msg_id.clone(),
                msg_type: MessageType::Direct,
                direction: MessageDirection::Received,
                content: MessageContent::Text {
                    text: message_copy.clone(),
                },
                sender_nickname: from_nick.clone(),
                recipient_nickname: Some(local_nickname.clone()),
                group_name: None,
                peer_id: from_id.clone(),
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
                tracing::error!("Failed to store message to database: {}", e);
            }
        }
    });

    Ok(())
}

fn handle_direct_group_share_message<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    from: &libp2p::PeerId,
    from_nickname: &str,
    group_id: &str,
    group_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    app_handle.emit(
        "group-share-received",
        &json!({
            "from_peer_id": from.to_string(),
            "from_nickname": from_nickname,
            "group_id": group_id,
            "group_name": group_name,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        }),
    )?;
    Ok(())
}

async fn handle_group_file_share_message<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    p2p_client: &Arc<Mutex<Option<gigi_p2p::P2pClient>>>,
    from: &libp2p::PeerId,
    from_nickname: &str,
    group: &str,
    share_code: &str,
    filename: &str,
    file_size: u64,
    file_type: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Received group file share message from {} ({}) for file {} in group {} with share code {}",
        from_nickname, from, filename, group, share_code
    );

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    // Save group file message to database asynchronously (non-blocking)
    // We save metadata only, not file content or thumbnails
    let state = app_handle.state::<PluginState>();
    let message_store = state.message_store.clone();
    let msg_id = uuid::Uuid::new_v4().to_string();
    let from_peer_id = from.to_string();
    let from_nickname_copy = from_nickname.to_string();
    let share_code_copy = share_code.to_string();
    let filename_copy = filename.to_string();
    let file_type_copy = file_type.to_string();
    let group_name_copy = group.to_string();

    // Get local nickname before spawning async task
    let local_nickname = {
        let config = state.config.read().await;
        config.nickname.clone()
    };

    tokio::spawn(async move {
        if let Some(store) = message_store.read().await.as_ref() {
            let stored_msg = StoredMessage {
                id: msg_id.clone(),
                msg_type: MessageType::Group,
                direction: MessageDirection::Received,
                content: MessageContent::FileShare {
                    share_code: share_code_copy.clone(),
                    filename: filename_copy.clone(),
                    file_size,
                    file_type: file_type_copy.clone(),
                },
                sender_nickname: from_nickname_copy.clone(),
                recipient_nickname: Some(local_nickname.clone()),
                group_name: Some(group_name_copy.clone()),
                peer_id: from_peer_id.clone(),
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
                tracing::error!("Failed to store group file message to database: {}", e);
            }
        }
    });

    // Check if it's an image file and auto-download
    if file_type.starts_with("image/") {
        info!("Auto-downloading group image file: {}", filename);
        if let Some(client) = p2p_client.lock().await.as_mut() {
            match client.download_file(from_nickname, share_code) {
                Ok(download_id) => {
                    info!(
                        "Started auto-download for group image: {} (download_id: {})",
                        filename, download_id
                    );
                    emit_group_image_message_received(
                        app_handle,
                        from,
                        from_nickname,
                        group,
                        share_code,
                        filename,
                        file_size,
                        file_type,
                        timestamp,
                        None,
                        Some(download_id.clone()),
                    )?;
                }
                Err(e) => {
                    error!(
                        "Failed to start auto-download for group image {}: {}",
                        filename, e
                    );
                    emit_group_image_message_received(
                        app_handle,
                        from,
                        from_nickname,
                        group,
                        share_code,
                        filename,
                        file_size,
                        file_type,
                        timestamp,
                        Some(format!("Failed to download: {}", e)),
                        None,
                    )?;
                }
            }
        }
    } else {
        // For non-image files, just emit the event without downloading
        app_handle.emit(
            "group-file-message-received",
            &json!({
                "from_peer_id": from.to_string(),
                "from_nickname": from_nickname,
                "group_id": group,
                "share_code": share_code,
                "filename": filename,
                "file_size": file_size,
                "file_type": file_type,
                "timestamp": timestamp,
            }),
        )?;
    }
    Ok(())
}

async fn handle_group_message<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    from: &libp2p::PeerId,
    from_nickname: &str,
    group: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Processing GroupMessage event:");
    info!("   - From: {} ({})", from_nickname, from);
    info!("   - Group: {}", group);
    info!("   - Message: {}", message);

    let msg_id = uuid::Uuid::new_v4().to_string();
    let msg = GroupMessage {
        id: msg_id.clone(),
        group_id: group.to_string(),
        from_peer_id: from.to_string(),
        from_nickname: from_nickname.to_string(),
        content: message.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
    };

    info!("Emitting 'group-message' event to frontend:");
    info!("   - Message ID: {}", msg.id);
    info!("   - Group ID: {}", msg.group_id);
    info!("   - From: {} ({})", msg.from_nickname, msg.from_peer_id);
    info!("   - Content: {}", msg.content);

    app_handle.emit("group-message", &msg)?;
    info!("'group-message' event emitted successfully");

    // Save message to database asynchronously (non-blocking)
    let state = app_handle.state::<PluginState>();
    let message_store = state.message_store.clone();
    let group_name = group.to_string();

    // Clone data for async task
    let message_copy = message.to_string();
    let from_id = from.to_string();
    let from_nick = from_nickname.to_string();

    // Clone local nickname before moving into async
    let local_nickname = {
        let config = state.config.read().await;
        config.nickname.clone()
    };

    tokio::spawn(async move {
        if let Some(store) = message_store.read().await.as_ref() {
            let stored_msg = StoredMessage {
                id: msg_id.clone(),
                msg_type: MessageType::Group,
                direction: MessageDirection::Received,
                content: MessageContent::Text {
                    text: message_copy.clone(),
                },
                sender_nickname: from_nick.clone(),
                recipient_nickname: Some(local_nickname.clone()),
                group_name: Some(group_name.clone()),
                peer_id: from_id.clone(),
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

    Ok(())
}

fn handle_file_share_request<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    from: &libp2p::PeerId,
    from_nickname: &str,
    share_code: &str,
    filename: &str,
    size: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    app_handle.emit(
        "file-share-request",
        &json!({
            "from": from.to_string(),
            "from_nickname": from_nickname,
            "share_code": share_code,
            "filename": filename,
            "size": size
        }),
    )?;
    Ok(())
}

async fn handle_file_download_progress<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    active_downloads: &ActiveDownloads,
    download_id: &str,
    filename: &str,
    share_code: &str,
    from_peer_id: &libp2p::PeerId,
    from_nickname: &str,
    downloaded_chunks: usize,
    total_chunks: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let progress = (downloaded_chunks as f32 / total_chunks as f32) * 100.0;
    let download_progress = DownloadProgress {
        download_id: download_id.to_string(),
        progress,
        speed: 0,
    };

    {
        let mut downloads_guard = active_downloads.lock().await;
        downloads_guard.insert(download_id.to_string(), download_progress.clone());
    }

    app_handle.emit(
        "file-download-progress",
        &json!({
            "download_id": download_id,
            "filename": filename,
            "share_code": share_code,
            "from_peer_id": from_peer_id.to_string(),
            "from_nickname": from_nickname,
            "progress_percent": progress,
            "speed": 0
        }),
    )?;
    Ok(())
}

async fn handle_file_download_completed<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    active_downloads: &ActiveDownloads,
    download_id: &str,
    filename: &str,
    share_code: &str,
    from_peer_id: &libp2p::PeerId,
    from_nickname: &str,
    path: &std::path::PathBuf,
    state: &PluginState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    {
        let mut downloads_guard = active_downloads.lock().await;
        downloads_guard.remove(download_id);
    }

    // Generate thumbnail for image files
    let thumbnail_filename =
        if let Some(thumb) = generate_thumbnail_for_image(app_handle, path, share_code).await? {
            // Store thumbnail path in thumbnail_store using downloaded file path as key
            if let Some(thumbnail_path_only) = thumb.split('/').last() {
                let thumbnail_store = state.thumbnail_store.clone();
                let file_path_str = path.to_string_lossy().to_string();
                let thumbnail_path_clone = thumbnail_path_only.to_string();

                tokio::spawn(async move {
                    if let Some(store) = thumbnail_store.read().await.as_ref() {
                        if let Err(e) = store
                            .store_thumbnail(&file_path_str, &thumbnail_path_clone)
                            .await
                        {
                            tracing::error!("Failed to store thumbnail mapping: {}", e);
                        }
                    }
                });
            }
            Some(thumb)
        } else {
            None
        };

    app_handle.emit(
        "file-download-completed",
        &json!({
            "download_id": download_id,
            "filename": filename,
            "share_code": share_code,
            "from_peer_id": from_peer_id.to_string(),
            "from_nickname": from_nickname,
            "path": path.to_string_lossy(),
            "thumbnail_filename": thumbnail_filename
        }),
    )?;
    Ok(())
}

fn handle_file_download_started<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    from: &libp2p::PeerId,
    from_nickname: &str,
    filename: &str,
    download_id: &str,
    share_code: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Download started for {} from {} ({})",
        filename, from_nickname, from
    );
    app_handle.emit(
        "file-download-started",
        &json!({
            "from_peer_id": from.to_string(),
            "from_nickname": from_nickname,
            "filename": filename,
            "download_id": download_id,
            "share_code": share_code,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        }),
    )?;
    Ok(())
}

async fn handle_file_download_failed<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    active_downloads: &ActiveDownloads,
    download_id: &str,
    filename: &str,
    share_code: &str,
    from_peer_id: &libp2p::PeerId,
    from_nickname: &str,
    error: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    {
        let mut downloads_guard = active_downloads.lock().await;
        downloads_guard.remove(download_id);
    }

    app_handle.emit(
        "file-download-failed",
        &json!({
            "download_id": download_id,
            "filename": filename,
            "share_code": share_code,
            "from_peer_id": from_peer_id.to_string(),
            "from_nickname": from_nickname,
            "error": error
        }),
    )?;
    Ok(())
}

fn handle_connected<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    peer_id: &libp2p::PeerId,
    nickname: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Emitting peer-connected event for {} ({})",
        nickname, peer_id
    );
    app_handle
        .emit(
            "peer-connected",
            &json!({
                "peer_id": peer_id.to_string(),
                "nickname": nickname
            }),
        )
        .map_err(|e| format!("Failed to emit peer-connected: {}", e))?;
    Ok(())
}

fn handle_disconnected<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    peer_id: &libp2p::PeerId,
    nickname: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    app_handle.emit(
        "peer-disconnected",
        &json!({
            "peer_id": peer_id.to_string(),
            "nickname": nickname
        }),
    )?;
    Ok(())
}

fn handle_error<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    error: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    app_handle.emit("p2p-error", &error)?;
    Ok(())
}

/// Helper function to emit image-message-received event
fn emit_image_message_received<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    from: &libp2p::PeerId,
    from_nickname: &str,
    share_code: &str,
    filename: &str,
    file_size: u64,
    file_type: &str,
    timestamp: u64,
    download_error: Option<String>,
    download_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut payload = json!({
        "from_peer_id": from.to_string(),
        "from_nickname": from_nickname,
        "share_code": share_code,
        "filename": filename,
        "file_size": file_size,
        "file_type": file_type,
        "timestamp": timestamp,
    });

    if let Some(error) = download_error {
        payload["download_error"] = json!(error);
    }

    if let Some(dl_id) = download_id {
        payload["download_id"] = json!(dl_id);
    }

    app_handle.emit("image-message-received", &payload)?;
    Ok(())
}

/// Helper function to emit group-image-message-received event
fn emit_group_image_message_received<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    from: &libp2p::PeerId,
    from_nickname: &str,
    group_id: &str,
    share_code: &str,
    filename: &str,
    file_size: u64,
    file_type: &str,
    timestamp: u64,
    download_error: Option<String>,
    download_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut payload = json!({
        "from_peer_id": from.to_string(),
        "from_nickname": from_nickname,
        "group_id": group_id,
        "share_code": share_code,
        "filename": filename,
        "file_size": file_size,
        "file_type": file_type,
        "timestamp": timestamp,
    });

    if let Some(error) = download_error {
        payload["download_error"] = json!(error);
    }

    if let Some(dl_id) = download_id {
        payload["download_id"] = json!(dl_id);
    }

    app_handle.emit("group-image-message-received", &payload)?;
    Ok(())
}
