use crate::types::{AppState, FileInfo, FileSendTarget};
use std::path::PathBuf;
use tauri::{AppHandle, State};

/// Helper function to send file message with optional base64 data
pub async fn send_file_message_internal(
    _app: &AppHandle,
    client: &mut gigi_p2p::P2pClient,
    _state: &State<'_, AppState>,
    target: FileSendTarget<'_>,
    file_path: &str,
) -> Result<String, String> {
    use crate::file_utils;
    use std::fs;

    let is_content_uri = file_path.starts_with("content://");

    let is_image = if is_content_uri {
        true
    } else {
        file_utils::is_image_file(file_path)
    };

    let base64_data = if is_image {
        let image_data = if is_content_uri {
            tracing::info!("📱 Detected content URI: {}", file_path);

            #[cfg(target_os = "android")]
            {
                file_utils::android::read_content_uri(_app, file_path)?
            }

            #[cfg(not(target_os = "android"))]
            {
                return Err("Content URIs are only supported on Android".to_string());
            }
        } else {
            let path = PathBuf::from(file_path);

            if !path.exists() {
                tracing::error!("❌ File does not exist: {}", file_path);
                return Err("File does not exist".to_string());
            }

            tracing::info!("✅ File exists, reading image data...");
            fs::read(&path).map_err(|e| format!("Failed to read image file: {}", e))?
        };

        file_utils::convert_to_base64_if_image(&image_data)
    } else {
        tracing::info!("⚠️ File is not an image (checked extension), skipping base64 conversion");
        None
    };

    // Save content URI to app directory if needed
    let actual_path = if is_content_uri {
        #[cfg(target_os = "android")]
        {
            tracing::info!("📱 Saving content URI to app directory for sharing");

            let config_guard = _state.config.read().await;
            let download_dir = PathBuf::from(&config_guard.download_folder);
            drop(config_guard);

            fs::create_dir_all(&download_dir)
                .map_err(|e| format!("Failed to create download dir: {}", e))?;

            let image_data = if let Some(ref b64) = base64_data {
                base64::engine::general_purpose::STANDARD
                    .decode(b64)
                    .map_err(|e| format!("Failed to decode base64: {}", e))?
            } else {
                file_utils::android::read_content_uri(_app, file_path)?
            };

            file_utils::android::save_content_uri_to_app_dir(
                file_path,
                &image_data,
                &download_dir,
                "gigi_share",
            )?
        }

        #[cfg(not(target_os = "android"))]
        {
            return Err("Content URIs are only supported on Android".to_string());
        }
    } else {
        PathBuf::from(file_path)
    };

    match target {
        FileSendTarget::Direct(nickname) => {
            client
                .send_direct_file(nickname, &actual_path)
                .await
                .map_err(|e| format!("Failed to send file: {}", e))?;
        }
        FileSendTarget::Group(group_id) => {
            client
                .send_group_file(group_id, &actual_path)
                .await
                .map_err(|e| format!("Failed to send group file: {}", e))?;
        }
    }

    tracing::info!("✅ File sent successfully");

    let message_id = uuid::Uuid::new_v4().to_string();
    let response = if let Some(b64) = base64_data {
        tracing::info!(
            "🎯 Returning response: {}|base64_data({} bytes)",
            message_id,
            b64.len()
        );
        format!("{}|{}", message_id, b64)
    } else {
        tracing::info!("🎯 Returning response: {} (no base64 data)", message_id);
        message_id
    };
    Ok(response)
}

#[tauri::command]
pub async fn messaging_share_file(
    file_path: &str,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use std::path::PathBuf;

    let path = PathBuf::from(file_path);

    let mut client_guard = state.p2p_client.lock().await;
    if let Some(ref mut client) = *client_guard {
        let share_code = client
            .share_file(&path)
            .await
            .map_err(|e| format!("Failed to share file: {}", e))?;

        // P2pClient handles persistence automatically
        Ok(share_code)
    } else {
        Err("P2P client not initialized".to_string())
    }
}

#[tauri::command]
pub fn messaging_request_file(
    _file_id: &str,
    _from_peer_id: &str,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    // For now, we need the nickname, not peer_id
    Err("Use peer nickname instead of peer ID for file requests".to_string())
}

#[tauri::command]
pub async fn messaging_request_file_from_nickname(
    nickname: &str,
    share_code: &str,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use tracing::{error, info};

    info!(
        "📥 File download request: nickname={}, share_code={}",
        nickname, share_code
    );

    let mut client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_mut() {
        match client.download_file(nickname, share_code) {
            Ok(()) => {
                let download_id = uuid::Uuid::new_v4().to_string();
                info!(
                    "✅ Download request sent successfully: download_id={}",
                    download_id
                );
                Ok(download_id)
            }
            Err(e) => {
                error!("❌ Failed to request file: {}", e);
                Err(format!("Failed to request file: {}", e))
            }
        }
    } else {
        error!("❌ P2P client not initialized");
        Err("P2P client not initialized".to_string())
    }
}

#[tauri::command]
pub fn messaging_cancel_download(
    download_id: &str,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // Placeholder - would need to implement in gigi-p2p
    use tracing::info;
    info!("Cancelled download: {}", download_id);
    Ok(())
}

#[tauri::command]
pub async fn messaging_get_shared_files(
    state: State<'_, AppState>,
) -> Result<Vec<FileInfo>, String> {
    let client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_ref() {
        let shared_files = client.list_shared_files();
        Ok(shared_files
            .into_iter()
            .map(|sf| FileInfo {
                id: sf.info.id.clone(),
                name: sf.info.name.clone(),
                size: sf.info.size,
                mime_type: mime_guess::from_path(&sf.info.name)
                    .first_or_octet_stream()
                    .to_string(),
                peer_id: client.local_peer_id().to_string(),
            })
            .collect())
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
pub async fn messaging_remove_shared_file(
    share_code: &str,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_mut() {
        client
            .unshare_file(share_code)
            .map_err(|e| format!("Failed to unshare file: {}", e))?;
        Ok(())
    } else {
        Err("P2P client not initialized".to_string())
    }
}

#[tauri::command]
pub async fn messaging_send_file_message_with_path(
    app: AppHandle,
    nickname: &str,
    file_path: &str,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use tracing::{error, info};

    info!(
        "🎯 messaging_send_file_message_with_path called with nickname: {}, file_path: {}",
        nickname, file_path
    );

    let mut client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_mut() {
        send_file_message_internal(
            &app,
            client,
            &state,
            FileSendTarget::Direct(nickname),
            file_path,
        )
        .await
    } else {
        error!("❌ P2P client not initialized");
        Err("P2P client not initialized".to_string())
    }
}

#[tauri::command]
pub async fn messaging_send_group_file_message_with_path(
    app: AppHandle,
    group_id: &str,
    file_path: &str,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use tracing::{error, info};

    info!(
        "🎯 messaging_send_group_file_message_with_path called with group_id: {}, file_path: {}",
        group_id, file_path
    );

    let mut client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_mut() {
        send_file_message_internal(
            &app,
            client,
            &state,
            FileSendTarget::Group(group_id),
            file_path,
        )
        .await
    } else {
        error!("❌ P2P client not initialized");
        Err("P2P client not initialized".to_string())
    }
}

#[tauri::command]
pub async fn messaging_get_image_data(file_path: &str) -> Result<String, String> {
    use base64::Engine;
    use std::fs;
    use std::path::PathBuf;

    let path = PathBuf::from(file_path);

    // Verify file exists and is an image
    if !path.exists() {
        return Err("File does not exist".to_string());
    }

    // Check if it's an image file
    let file_type = mime_guess::from_path(&path)
        .first_or_octet_stream()
        .to_string();

    if !file_type.starts_with("image/") {
        return Err("File is not an image".to_string());
    }

    // Read image data and convert to base64
    let image_data = fs::read(&path).map_err(|e| format!("Failed to read image file: {}", e))?;

    let base64_data = base64::engine::general_purpose::STANDARD.encode(&image_data);
    let data_url = format!("data:{};base64,{}", file_type, base64_data);

    Ok(data_url)
}

#[tauri::command]
pub async fn messaging_get_file_info(file_path: &str) -> Result<FileInfo, String> {
    use std::fs;
    use std::path::PathBuf;

    let path = PathBuf::from(file_path);

    // Verify file exists
    if !path.exists() {
        return Err("File does not exist".to_string());
    }

    // Get file metadata
    let metadata =
        fs::metadata(&path).map_err(|e| format!("Failed to get file metadata: {}", e))?;

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mime_type = mime_guess::from_path(&path)
        .first_or_octet_stream()
        .to_string();

    Ok(FileInfo {
        id: uuid::Uuid::new_v4().to_string(),
        name: file_name,
        size: metadata.len(),
        mime_type,
        peer_id: "".to_string(),
    })
}

#[tauri::command]
pub async fn messaging_select_any_file(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    use tracing::info;

    match app
        .dialog()
        .file()
        .set_title("Select File")
        .add_filter("All Files", &["*"])
        .add_filter("Documents", &["pdf", "doc", "docx", "txt", "rtf"])
        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "bmp", "webp"])
        .add_filter("Videos", &["mp4", "avi", "mov", "mkv", "webm"])
        .add_filter("Audio", &["mp3", "wav", "flac", "aac", "ogg"])
        .add_filter("Archives", &["zip", "rar", "7z", "tar", "gz"])
        .blocking_pick_file()
    {
        Some(path) => {
            info!("User selected file: {}", path);
            Ok(Some(path.to_string()))
        }
        None => {
            info!("User cancelled file selection");
            Ok(None)
        }
    }
}

#[tauri::command]
pub async fn messaging_share_content_uri(
    uri: &str,
    name: &str,
    size: u64,
    _app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use tracing::{error, info};

    info!(
        "📤 Sharing content URI: uri={}, name={}, size={}",
        uri, name, size
    );

    let mut client_guard = state.p2p_client.lock().await;
    if let Some(ref mut client) = *client_guard {
        // Share the content URI using the new method
        match client.share_content_uri(uri, name, size) {
            Ok(share_code) => {
                info!(
                    "✅ Successfully shared content URI '{}' with code: {}",
                    name, share_code
                );
                Ok(share_code)
            }
            Err(e) => {
                error!("❌ Failed to share content URI: {}", e);
                Err(format!("Failed to share content URI: {}", e))
            }
        }
    } else {
        Err("P2P client not initialized".to_string())
    }
}
