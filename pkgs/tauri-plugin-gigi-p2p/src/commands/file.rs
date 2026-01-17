use tauri::{AppHandle, Manager, State};

use crate::file_utils;
use crate::{models::FileInfo, models::FileSendTarget, Error, PluginState, Result};
use base64::Engine;
use gigi_store::{MessageContent, MessageDirection, MessageType, StoredMessage};
use chrono::Utc;

/// Helper function to send file message with optional base64 data
pub async fn send_file_message_internal<R: tauri::Runtime>(
    app: &AppHandle<R>,
    client: &mut gigi_p2p::P2pClient,
    state: &State<'_, PluginState>,
    target: FileSendTarget<'_>,
    file_path: &str,
) -> Result<String> {
    use std::fs;

    let is_content_uri = file_path.starts_with("content://");

    let is_image = if is_content_uri {
        true
    } else {
        file_utils::is_image_file(file_path)
    };

    let base64_data = if is_image {
        let image_data = if is_content_uri {
            #[cfg(target_os = "android")]
            {
                file_utils::android::read_content_uri(app, file_path)
                    .map_err(|e| Error::Io(format!("Failed to read content URI: {}", e)))?
            }

            #[cfg(not(target_os = "android"))]
            {
                return Err(Error::CommandFailed(
                    "Content URIs are only supported on Android".to_string(),
                ));
            }
        } else {
            let path = std::path::PathBuf::from(file_path);

            if !path.exists() {
                tracing::error!("File does not exist: {}", file_path);
                return Err(Error::Io("File does not exist".to_string()));
            }

            fs::read(&path).map_err(|e| Error::Io(format!("Failed to read image file: {}", e)))?
        };

        file_utils::convert_to_base64_if_image(&image_data)
    } else {
        None
    };

    // Save content URI to app directory if needed
    let actual_path = if is_content_uri {
        #[cfg(target_os = "android")]
        {
            let config_guard = state.config.read().await;
            let download_dir = std::path::PathBuf::from(&config_guard.download_folder);
            drop(config_guard);

            fs::create_dir_all(&download_dir)
                .map_err(|e| Error::Io(format!("Failed to create download dir: {}", e)))?;

            let image_data = if let Some(ref b64) = base64_data {
                base64::engine::general_purpose::STANDARD
                    .decode(b64)
                    .map_err(|e| Error::Io(format!("Failed to decode base64: {}", e)))?
            } else {
                file_utils::android::read_content_uri(app, file_path)
                    .map_err(|e| Error::Io(format!("Failed to read content URI: {}", e)))?
            };

            file_utils::android::save_content_uri_to_app_dir(
                file_path,
                &image_data,
                &download_dir,
                "gigi_share",
            )
            .map_err(|e| Error::Io(format!("Failed to save content URI: {}", e)))?
        }

        #[cfg(not(target_os = "android"))]
        {
            return Err(Error::CommandFailed(
                "Content URIs are only supported on Android".to_string(),
            ));
        }
    } else {
        std::path::PathBuf::from(file_path)
    };

    // Share file to get share code BEFORE sending
    let share_code = client
        .share_file(&actual_path)
        .await
        .map_err(|e| Error::P2p(e.to_string()))?;

    // Get file metadata for database storage
    let file_metadata = std::fs::metadata(&actual_path).ok();
    let file_size = file_metadata.map(|m| m.len()).unwrap_or(0);
    let file_name = actual_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    let file_type = mime_guess::from_path(&actual_path)
        .first_or_octet_stream()
        .to_string();

    // Send file to target
    match target {
        FileSendTarget::Direct(nickname) => {
            client
                .send_direct_file(nickname, &actual_path)
                .await
                .map_err(|e| Error::P2p(e.to_string()))?;
        }
        FileSendTarget::Group(group_id) => {
            client
                .send_group_file(group_id, &actual_path)
                .await
                .map_err(|e| Error::P2p(e.to_string()))?;
        }
    }

    tracing::info!("File sent successfully");

    // Generate thumbnail for image files
    let thumbnail_path_str = if is_image {
        let download_dir = app.path().download_dir()
            .map_err(|e| Error::Io(format!("Failed to get download directory: {}", e)))?;

        #[cfg(target_os = "android")]
        let thumbnail_dir = download_dir.join("thumbnails");

        #[cfg(not(target_os = "android"))]
        let thumbnail_dir = download_dir.join("gigi/thumbnails");

        // Ensure thumbnail directory exists
        if let Err(e) = tokio::fs::create_dir_all(&thumbnail_dir).await {
            tracing::warn!("Failed to create thumbnail directory: {}", e);
            None
        } else {
            use gigi_store::thumbnail;

            match thumbnail::generate_thumbnail(
                &actual_path,
                &thumbnail_dir,
                (200, 200),
                70
            ).await {
                Ok(thumbnail_filename) => {
                    let full_thumbnail_path = thumbnail_dir.join(&thumbnail_filename);
                    tracing::info!("Generated thumbnail for sent image: {}", full_thumbnail_path.display());

                    // Store thumbnail mapping in thumbnail_store
                    let thumbnail_store = state.thumbnail_store.clone();
                    let actual_path_str = actual_path.to_string_lossy().to_string();
                    let thumbnail_filename_clone = thumbnail_filename.clone();
                    tokio::spawn(async move {
                        if let Some(store) = thumbnail_store.read().await.as_ref() {
                            if let Err(e) = store
                                .store_thumbnail(&actual_path_str, &thumbnail_filename_clone)
                                .await
                            {
                                tracing::error!("Failed to store thumbnail mapping: {}", e);
                            }
                        }
                    });

                    Some(full_thumbnail_path.to_string_lossy().to_string())
                }
                Err(e) => {
                    tracing::warn!("Failed to generate thumbnail for sent image: {}", e);
                    None
                }
            }
        }
    } else {
        None
    };

    let message_id = uuid::Uuid::new_v4().to_string();
    let message_id_for_db = message_id.clone();

    // Save message to database asynchronously
    let is_group = matches!(target, FileSendTarget::Group(_));
    let recipient_nickname = match target {
        FileSendTarget::Direct(nickname) => Some(nickname.to_string()),
        FileSendTarget::Group(_) => None,
    };

    // Get peer_id for direct messages
    let peer_id = match target {
        FileSendTarget::Direct(nickname) => {
            client
                .get_peer_id_by_nickname(nickname)
                .map(|id| id.to_string())
                .unwrap_or_default()
        }
        FileSendTarget::Group(group_id) => group_id.to_string(),
    };

    let message_store = state.message_store.clone();
    let share_code_clone = share_code.clone();
    let filename_clone = file_name.clone();
    let file_type_clone = file_type.clone();
    let local_nickname = {
        let config = state.config.read().await;
        config.nickname.clone()
    };
    let peer_id_clone = peer_id.clone();

    // Prepare thumbnail path for storage in message
    let thumbnail_path_for_msg = thumbnail_path_str.as_ref().and_then(|p| p.split('/').last().map(String::from));

    tokio::spawn(async move {
        if let Some(store) = message_store.read().await.as_ref() {
            let content = if let Some(thumb) = thumbnail_path_for_msg {
                MessageContent::FileShareWithThumbnail {
                    share_code: share_code_clone.clone(),
                    filename: filename_clone.clone(),
                    file_size,
                    file_type: file_type_clone.clone(),
                    thumbnail_path: Some(thumb),
                }
            } else {
                MessageContent::FileShare {
                    share_code: share_code_clone.clone(),
                    filename: filename_clone.clone(),
                    file_size,
                    file_type: file_type_clone.clone(),
                }
            };

            let stored_msg = StoredMessage {
                id: message_id_for_db.clone(),
                msg_type: if is_group { MessageType::Group } else { MessageType::Direct },
                direction: MessageDirection::Sent,
                content,
                sender_nickname: local_nickname.clone(),
                recipient_nickname,
                group_name: None,
                peer_id: peer_id_clone,
                timestamp: Utc::now(),
                created_at: Utc::now(),
                delivered: false,
                delivered_at: None,
                read: false,
                read_at: None,
                sync_status: gigi_store::SyncStatus::Synced,
                sync_attempts: 0,
                last_sync_attempt: None,
                expires_at: Utc::now() + chrono::Duration::days(30),
            };
            if let Err(e) = store.store_message(stored_msg).await {
                tracing::error!("Failed to store sent file message to database: {}", e);
            }
        }
    });

    let response = if let Some(b64) = base64_data {
        if let Some(thumb) = thumbnail_path_str {
            format!("{}|{}|thumbnail:{}", message_id, b64, thumb)
        } else {
            format!("{}|{}", message_id, b64)
        }
    } else {
        if let Some(thumb) = thumbnail_path_str {
            format!("{}|thumbnail:{}", message_id, thumb)
        } else {
            message_id
        }
    };

    Ok(response)
}

/// Send file message to a peer
#[tauri::command]
pub(crate) async fn messaging_send_file_message_with_path<R: tauri::Runtime>(
    app: AppHandle<R>,
    state: State<'_, PluginState>,
    nickname: &str,
    file_path: &str,
) -> Result<String> {
    let mut p2p_client = state.p2p_client.lock().await;
    let client = p2p_client.as_mut().ok_or(Error::P2pNotInitialized)?;

    send_file_message_internal(
        &app,
        client,
        &state,
        FileSendTarget::Direct(nickname),
        file_path,
    )
    .await
}

/// Send file message to a group
#[tauri::command]
pub(crate) async fn messaging_send_group_file_message_with_path<R: tauri::Runtime>(
    app: AppHandle<R>,
    state: State<'_, PluginState>,
    group_id: &str,
    file_path: &str,
) -> Result<String> {
    let mut p2p_client = state.p2p_client.lock().await;
    let client = p2p_client.as_mut().ok_or(Error::P2pNotInitialized)?;

    send_file_message_internal(
        &app,
        client,
        &state,
        FileSendTarget::Group(group_id),
        file_path,
    )
    .await
}

/// Share a file for P2P transfer
#[tauri::command]
pub(crate) async fn messaging_share_file<R: tauri::Runtime>(
    _app: AppHandle<R>,
    state: State<'_, PluginState>,
    file_path: &str,
) -> Result<String> {
    use std::path::PathBuf;

    let path = PathBuf::from(file_path);

    let mut p2p_client = state.p2p_client.lock().await;
    let client = p2p_client.as_mut().ok_or(Error::P2pNotInitialized)?;

    let share_code = client
        .share_file(&path)
        .await
        .map_err(|e| Error::P2p(e.to_string()))?;

    Ok(share_code)
}

/// Request file download from a peer
#[tauri::command]
pub(crate) async fn messaging_request_file_from_nickname<R: tauri::Runtime>(
    _app: AppHandle<R>,
    state: State<'_, PluginState>,
    nickname: &str,
    share_code: &str,
) -> Result<String> {
    let mut p2p_client = state.p2p_client.lock().await;
    let client = p2p_client.as_mut().ok_or(Error::P2pNotInitialized)?;

    client
        .download_file(nickname, share_code)
        .map_err(|e| Error::P2p(format!("Failed to request file: {}", e)))
}

/// Cancel an in-progress download
#[tauri::command]
pub(crate) async fn messaging_cancel_download<R: tauri::Runtime>(
    _app: AppHandle<R>,
    _state: State<'_, PluginState>,
    _download_id: &str,
) -> Result<()> {
    Ok(())
}

/// Get list of shared files
#[tauri::command]
pub(crate) async fn messaging_get_shared_files<R: tauri::Runtime>(
    _app: AppHandle<R>,
    state: State<'_, PluginState>,
) -> Result<Vec<FileInfo>> {
    let p2p_client = state.p2p_client.lock().await;
    let client = p2p_client.as_ref().ok_or(Error::P2pNotInitialized)?;

    let shared_files = client.list_shared_files();
    let peer_id = client.local_peer_id().to_string();

    Ok(shared_files
        .into_iter()
        .map(|sf| FileInfo {
            id: sf.info.id.clone(),
            name: sf.info.name.clone(),
            size: sf.info.size,
            mime_type: mime_guess::from_path(&sf.info.name)
                .first_or_octet_stream()
                .to_string(),
            peer_id: peer_id.clone(),
        })
        .collect())
}

/// Remove a file from sharing
#[tauri::command]
pub(crate) async fn messaging_remove_shared_file<R: tauri::Runtime>(
    _app: AppHandle<R>,
    state: State<'_, PluginState>,
    share_code: &str,
) -> Result<()> {
    let mut p2p_client = state.p2p_client.lock().await;
    let client = p2p_client.as_mut().ok_or(Error::P2pNotInitialized)?;

    client
        .unshare_file(share_code)
        .map_err(|e| Error::P2p(e.to_string()))?;

    Ok(())
}

/// Get image data as base64
#[tauri::command]
pub(crate) async fn messaging_get_image_data<R: tauri::Runtime>(
    _app: AppHandle<R>,
    file_path: &str,
) -> Result<String> {
    use std::fs;
    use std::path::Path;

    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(Error::Io("File does not exist".to_string()));
    }

    let file_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    if !file_type.starts_with("image/") {
        return Err(Error::CommandFailed("File is not an image".to_string()));
    }

    let image_data =
        fs::read(path).map_err(|e| Error::Io(format!("Failed to read image file: {}", e)))?;

    let base64_data = base64::prelude::BASE64_STANDARD.encode(&image_data);
    let data_url = format!("data:{};base64,{}", file_type, base64_data);

    Ok(data_url)
}

/// Get file metadata
#[tauri::command]
pub(crate) async fn messaging_get_file_info<R: tauri::Runtime>(
    _app: AppHandle<R>,
    file_path: &str,
) -> Result<FileInfo> {
    use std::fs;
    use std::path::Path;

    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(Error::Io("File does not exist".to_string()));
    }

    let metadata = fs::metadata(path)
        .map_err(|e| Error::Io(format!("Failed to read file metadata: {}", e)))?;

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let mime_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    Ok(FileInfo {
        id: uuid::Uuid::new_v4().to_string(),
        name: filename,
        size: metadata.len(),
        mime_type,
        peer_id: String::new(),
    })
}

/// Open file picker dialog (desktop only)
#[tauri::command]
pub(crate) async fn messaging_select_any_file<R: tauri::Runtime>(
    _app: AppHandle<R>,
) -> Result<Option<String>> {
    Ok(None)
}

/// Share Android content URI
#[tauri::command]
pub(crate) async fn messaging_share_content_uri<R: tauri::Runtime>(
    _app: AppHandle<R>,
    _state: State<'_, PluginState>,
    _uri: &str,
    _name: &str,
    _size: u64,
) -> Result<String> {
    #[cfg(target_os = "android")]
    {
        let mut p2p_client = state.p2p_client.lock().await;
        let client = p2p_client.as_mut().ok_or(Error::P2pNotInitialized)?;

        let share_code = client
            .share_content_uri(&_uri, &_name, _size)
            .await
            .map_err(|e| Error::P2p(e.to_string()))?;

        Ok(share_code)
    }

    #[cfg(not(target_os = "android"))]
    {
        Err(Error::CommandFailed(
            "Content URIs are only supported on Android".to_string(),
        ))
    }
}
