// Event handlers for P2P events

use crate::types::Peer;
use crate::{Config, DownloadProgress, GroupMessage, Message};
use gigi_p2p::P2pClient;
use gigi_p2p::P2pEvent;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info};

type ActiveDownloads = Arc<Mutex<HashMap<String, DownloadProgress>>>;

/// Handle P2P events and emit corresponding frontend events
pub async fn handle_p2p_event_with_fields(
    event: P2pEvent,
    _p2p_client: &Arc<Mutex<Option<P2pClient>>>,
    _config: &Arc<RwLock<Config>>,
    active_downloads: &ActiveDownloads,
    app_handle: &AppHandle,
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
                _p2p_client,
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
        } => handle_direct_message(app_handle, &from, &from_nickname, &message)?,
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
                _p2p_client,
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
        } => handle_group_message(app_handle, &from, &from_nickname, &group, &message)?,
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
            )
            .await?
        }
        P2pEvent::FileDownloadStarted {
            from,
            from_nickname,
            filename,
        } => handle_file_download_started(app_handle, &from, &from_nickname, &filename)?,
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

fn handle_peer_discovered(
    app_handle: &AppHandle,
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

fn handle_peer_expired(
    app_handle: &AppHandle,
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

fn handle_nickname_updated(
    app_handle: &AppHandle,
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

async fn handle_direct_file_share_message(
    app_handle: &AppHandle,
    _p2p_client: &Arc<Mutex<Option<P2pClient>>>,
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

    // Check if it's an image file and auto-download
    if file_type.starts_with("image/") {
        info!("Auto-downloading image file: {}", filename);
        if let Some(client) = _p2p_client.lock().await.as_mut() {
            match client.download_file(from_nickname, share_code) {
                Ok(()) => {
                    info!("Started auto-download for image: {}", filename);
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

fn handle_direct_message(
    app_handle: &AppHandle,
    from: &libp2p::PeerId,
    from_nickname: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let msg = Message {
        id: uuid::Uuid::new_v4().to_string(),
        from_peer_id: from.to_string(),
        from_nickname: from_nickname.to_string(),
        content: message.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
    };
    app_handle.emit("message-received", &msg)?;
    Ok(())
}

fn handle_direct_group_share_message(
    app_handle: &AppHandle,
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

async fn handle_group_file_share_message(
    app_handle: &AppHandle,
    _p2p_client: &Arc<Mutex<Option<P2pClient>>>,
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

    // Check if it's an image file and auto-download
    if file_type.starts_with("image/") {
        info!("Auto-downloading group image file: {}", filename);
        if let Some(client) = _p2p_client.lock().await.as_mut() {
            match client.download_file(from_nickname, share_code) {
                Ok(()) => {
                    info!("Started auto-download for group image: {}", filename);
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

fn handle_group_message(
    app_handle: &AppHandle,
    from: &libp2p::PeerId,
    from_nickname: &str,
    group: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Processing GroupMessage event:");
    info!("   - From: {} ({})", from_nickname, from);
    info!("   - Group: {}", group);
    info!("   - Message: {}", message);

    let msg = GroupMessage {
        id: uuid::Uuid::new_v4().to_string(),
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
    Ok(())
}

fn handle_file_share_request(
    app_handle: &AppHandle,
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

async fn handle_file_download_progress(
    app_handle: &AppHandle,
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

async fn handle_file_download_completed(
    app_handle: &AppHandle,
    active_downloads: &ActiveDownloads,
    download_id: &str,
    filename: &str,
    share_code: &str,
    from_peer_id: &libp2p::PeerId,
    from_nickname: &str,
    path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    {
        let mut downloads_guard = active_downloads.lock().await;
        downloads_guard.remove(download_id);
    }

    app_handle.emit(
        "file-download-completed",
        &json!({
            "download_id": download_id,
            "filename": filename,
            "share_code": share_code,
            "from_peer_id": from_peer_id.to_string(),
            "from_nickname": from_nickname,
            "path": path.to_string_lossy()
        }),
    )?;
    Ok(())
}

fn handle_file_download_started(
    app_handle: &AppHandle,
    from: &libp2p::PeerId,
    from_nickname: &str,
    filename: &str,
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
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        }),
    )?;
    Ok(())
}

async fn handle_file_download_failed(
    app_handle: &AppHandle,
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

fn handle_connected(
    app_handle: &AppHandle,
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

fn handle_disconnected(
    app_handle: &AppHandle,
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

fn handle_error(
    app_handle: &AppHandle,
    error: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    app_handle.emit("p2p-error", &error)?;
    Ok(())
}

/// Helper function to emit image-message-received event
fn emit_image_message_received(
    app_handle: &AppHandle,
    from: &libp2p::PeerId,
    from_nickname: &str,
    share_code: &str,
    filename: &str,
    file_size: u64,
    file_type: &str,
    timestamp: u64,
    download_error: Option<String>,
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

    app_handle.emit("image-message-received", &payload)?;
    Ok(())
}

/// Helper function to emit group-image-message-received event
fn emit_group_image_message_received(
    app_handle: &AppHandle,
    from: &libp2p::PeerId,
    from_nickname: &str,
    group_id: &str,
    share_code: &str,
    filename: &str,
    file_size: u64,
    file_type: &str,
    timestamp: u64,
    download_error: Option<String>,
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

    app_handle.emit("group-image-message-received", &payload)?;
    Ok(())
}
