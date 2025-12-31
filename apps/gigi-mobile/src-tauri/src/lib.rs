// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use base64::Engine;
use futures::channel::mpsc;
use futures::StreamExt;
use gigi_p2p::{FileInfo as P2pFileInfo, P2pClient, P2pEvent};
use hex;
use libp2p::identity;
#[cfg(target_os = "android")]
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

/// Initialize logging for the mobile application
fn init_logging() {
    gigi_p2p::init_tracing();
}

/// Download folder constant for file storage
/// On Android 10+, use app-specific storage to avoid scoped storage restrictions
#[cfg(target_os = "android")]
const DOWNLOAD_FOLDER: &str = "/data/data/app.gigi/files/downloads";

#[cfg(not(target_os = "android"))]
const DOWNLOAD_FOLDER: &str = "./gigi/Download";

// Types that match the frontend expectations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub mime_type: String,
    pub peer_id: String,
}

impl From<P2pFileInfo> for FileInfo {
    fn from(p2p_info: P2pFileInfo) -> Self {
        let name = p2p_info.name.clone();
        Self {
            id: p2p_info.id,
            name: name.clone(),
            size: p2p_info.size,
            mime_type: mime_guess::from_path(&name)
                .first_or_octet_stream()
                .to_string(),
            peer_id: "".to_string(), // Will be set when sharing
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub download_id: String,
    pub progress: f32,
    pub speed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub nickname: String,
    pub auto_accept_files: bool,
    pub download_folder: String,
    pub max_concurrent_downloads: usize,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub nickname: String,
    pub capabilities: Vec<String>,
}

impl From<gigi_p2p::PeerInfo> for Peer {
    fn from(p2p_peer: gigi_p2p::PeerInfo) -> Self {
        Self {
            id: p2p_peer.peer_id.to_string(),
            nickname: p2p_peer.nickname,
            capabilities: vec!["messaging".to_string(), "file_transfer".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from_peer_id: String,
    pub from_nickname: String,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    pub id: String,
    pub group_id: String,
    pub from_peer_id: String,
    pub from_nickname: String,
    pub content: String,
    pub timestamp: u64,
}

// App State
pub struct AppState {
    pub p2p_client: Arc<Mutex<Option<P2pClient>>>,
    pub event_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<P2pEvent>>>>,
    pub config: Arc<RwLock<Config>>,
    pub active_downloads: Arc<Mutex<HashMap<String, DownloadProgress>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self::with_download_folder(DOWNLOAD_FOLDER)
    }

    pub fn with_download_folder(download_folder: &str) -> Self {
        let base_path = PathBuf::from(download_folder);

        // Don't create directory at startup - defer until actually needed
        // This avoids permission issues on Android at startup

        Self {
            p2p_client: Arc::new(Mutex::new(None)),
            event_receiver: Arc::new(Mutex::new(None)),
            config: Arc::new(RwLock::new(Config {
                nickname: "Anonymous".to_string(),
                auto_accept_files: false,
                download_folder: base_path.to_string_lossy().to_string(),
                max_concurrent_downloads: 3,
                port: 0,
            })),
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// Commands

#[tauri::command]
async fn get_peer_id(state: State<'_, AppState>) -> Result<String, String> {
    let client_guard = state.p2p_client.lock().await;
    match client_guard.as_ref() {
        Some(client) => Ok(client.local_peer_id().to_string()),
        None => Err("P2P client not initialized".to_string()),
    }
}

#[tauri::command]
fn try_get_peer_id(priv_key: Vec<u8>) -> Result<String, String> {
    match identity::Keypair::ed25519_from_bytes(priv_key) {
        Ok(id_keys) => {
            let peer_id = id_keys.public().to_peer_id();
            Ok(peer_id.to_string())
        }
        Err(e) => Err(format!("Failed to create keypair: {}", e)),
    }
}

#[tauri::command]
async fn messaging_initialize_with_key(
    private_key: Vec<u8>,
    nickname: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let keypair = identity::Keypair::ed25519_from_bytes(private_key)
        .map_err(|e| format!("Failed to create keypair: {}", e))?;

    let peer_id = keypair.public().to_peer_id();
    let public_key_hex = hex::encode(keypair.to_protobuf_encoding().unwrap());

    // Update nickname in config before creating P2P client
    {
        let mut config_guard = state.config.write().await;
        config_guard.nickname = nickname.clone();
    }

    let config_guard = state.config.read().await;
    let output_dir = PathBuf::from(&config_guard.download_folder);
    let shared_files_path = PathBuf::from(&config_guard.download_folder).join("shared.json");
    let final_nickname = config_guard.nickname.clone();
    drop(config_guard);

    // Create downloads directory at runtime when initializing
    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create download directory: {}", e))?;

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
                        if let Err(e) = handle_p2p_event_with_fields(
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
fn messaging_send_message(
    _to_peer_id: &str,
    _message: &str,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    // This would need the peer nickname, which we don't have in this command
    // For now, return an error suggesting to use a different API
    Err("Use peer nickname instead of peer ID".to_string())
}

#[tauri::command]
async fn messaging_send_message_to_nickname(
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
async fn messaging_send_direct_share_group_message(
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
async fn messaging_get_peers(state: State<'_, AppState>) -> Result<Vec<Peer>, String> {
    let client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_ref() {
        let peers = client.list_peers();
        Ok(peers.into_iter().map(|p| p.clone().into()).collect())
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
async fn messaging_set_nickname(
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
async fn messaging_join_group(group_id: &str, state: State<'_, AppState>) -> Result<(), String> {
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
async fn messaging_send_group_message(
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
async fn messaging_share_file(
    file_path: &str,
    state: State<'_, AppState>,
) -> Result<String, String> {
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
fn messaging_request_file(
    _file_id: &str,
    _from_peer_id: &str,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    // For now, we need the nickname, not peer_id
    Err("Use peer nickname instead of peer ID for file requests".to_string())
}

#[tauri::command]
async fn messaging_request_file_from_nickname(
    nickname: &str,
    share_code: &str,
    state: State<'_, AppState>,
) -> Result<String, String> {
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
fn messaging_cancel_download(download_id: &str, _state: State<'_, AppState>) -> Result<(), String> {
    // Placeholder - would need to implement in gigi-p2p
    info!("Cancelled download: {}", download_id);
    Ok(())
}

#[tauri::command]
async fn messaging_get_shared_files(state: State<'_, AppState>) -> Result<Vec<FileInfo>, String> {
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
async fn messaging_get_public_key(state: State<'_, AppState>) -> Result<String, String> {
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
async fn messaging_get_active_downloads(
    state: State<'_, AppState>,
) -> Result<Vec<DownloadProgress>, String> {
    let downloads_guard = state.active_downloads.lock().await;
    Ok(downloads_guard.values().cloned().collect())
}

#[tauri::command]
async fn messaging_send_file_message_with_path(
    #[allow(unused_variables)] app: AppHandle,
    nickname: &str,
    file_path: &str,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!(
        "🎯 messaging_send_file_message_with_path called with nickname: {}, file_path: {}",
        nickname, file_path
    );

    let mut client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_mut() {
        let is_content_uri = file_path.starts_with("content://");

        // Read file data and convert to base64 for immediate display
        let image_data = if is_content_uri {
            // Android content URI: Use special handling
            // On Android, the dialog plugin may return content URIs
            // The standard fs plugin handles content URI translation
            let uri_str = file_path;
            info!("📱 Detected content URI: {}", uri_str);

            // On Android, we need to use a workaround to read content URIs
            #[cfg(target_os = "android")]
            {
                // For content URIs, use the android-fs plugin
                info!("📱 Attempting to read content URI: {}", uri_str);

                use tauri_plugin_android_fs::{AndroidFsExt, FileAccessMode, FileUri};
                use tauri_plugin_fs::FilePath;

                let android_api = app.android_fs();

                // Convert URI string to FileUri via FilePath::Url
                let url = tauri::Url::parse(uri_str)
                    .map_err(|e| format!("Failed to parse content URI: {}", e))?;
                let file_uri = FileUri::from(FilePath::Url(url));

                // Open the file and read its contents
                match android_api.open_file(&file_uri, FileAccessMode::Read) {
                    Ok(mut file) => {
                        let mut buffer = Vec::new();
                        use std::io::Read;
                        file.read_to_end(&mut buffer)
                            .map_err(|e| format!("Failed to read content URI data: {}", e))?;
                        buffer
                    }
                    Err(e) => {
                        error!("Failed to open content URI: {}", e);
                        return Err(format!("Failed to open content URI: {}", e));
                    }
                }
            }

            #[cfg(not(target_os = "android"))]
            {
                return Err("Content URIs are only supported on Android".to_string());
            }
        } else {
            // Regular filesystem path
            let path = PathBuf::from(file_path);

            // Verify file exists
            if !path.exists() {
                error!("❌ File does not exist: {}", file_path);
                return Err("File does not exist".to_string());
            }

            info!("✅ File exists, reading image data...");
            fs::read(&path).map_err(|e| format!("Failed to read image file: {}", e))?
        };

        let base64_data = base64::engine::general_purpose::STANDARD.encode(&image_data);
        info!(
            "✅ Successfully converted image to base64, size: {} bytes",
            image_data.len()
        );

        info!("🔄 Sending image via P2P...");

        // Send the image using P2P functionality
        // For content URIs, save to app-specific location first, then share
        // For regular paths, share directly
        let actual_path = if is_content_uri {
            // Content URI: Save to app-specific location
            #[cfg(target_os = "android")]
            {
                info!("📱 Saving content URI to app directory for sharing");

                // Get app download directory
                let config_guard = state.config.read().await;
                let download_dir = PathBuf::from(&config_guard.download_folder);
                drop(config_guard);

                // Create directory if needed
                fs::create_dir_all(&download_dir)
                    .map_err(|e| format!("Failed to create download dir: {}", e))?;

                // Try to extract filename from content URI
                let filename_from_uri = if let Some(display_name) =
                    file_path.split('=').last().and_then(|s| {
                        let decoded = percent_decode_str(s).decode_utf8().ok()?;
                        let name = decoded.split('/').last()?.to_string();
                        if name.is_empty() {
                            None
                        } else {
                            Some(name)
                        }
                    }) {
                    display_name
                } else {
                    // Fallback: extract from URI path
                    file_path
                        .split('/')
                        .last()
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| {
                            // Final fallback: use hash
                            let uri_hash = blake3::hash(file_path.as_bytes()).to_hex();
                            format!("gigi_share_{}", &uri_hash[..16])
                        })
                };

                // Detect file type from data and get appropriate extension
                let file_ext = if image_data.len() >= 8 {
                    // Check common signatures
                    if image_data.starts_with(&[0xFF, 0xD8, 0xFF]) {
                        ".jpg"
                    } else if image_data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                        ".png"
                    } else if image_data.starts_with(&[0x47, 0x49, 0x46]) {
                        ".gif"
                    } else if image_data.starts_with(b"WEBP")
                        || image_data.starts_with(&[0x52, 0x49, 0x46, 0x46])
                    {
                        ".webp"
                    } else if image_data.starts_with(b"BM") {
                        ".bmp"
                    } else if image_data.starts_with(&[0x00, 0x00, 0x00])
                        || image_data.starts_with(&[0x1A, 0x45, 0xDF, 0xA3])
                    {
                        // Video signatures: MP4/AVI or WebM
                        ".mp4"
                    } else {
                        // If filename already has extension, use it; otherwise use .dat
                        if filename_from_uri.contains('.') {
                            ""
                        } else {
                            ".dat"
                        }
                    }
                } else {
                    // For small files, trust the filename extension
                    ""
                };

                // Combine filename and extension
                let filename = if !file_ext.is_empty() && !filename_from_uri.contains('.') {
                    format!("{}{}", filename_from_uri, file_ext)
                } else {
                    filename_from_uri
                };

                let save_path = download_dir.join(&filename);

                info!("📝 Saving content URI to: {:?}", save_path);
                // Save image data to file
                fs::write(&save_path, &image_data)
                    .map_err(|e| format!("Failed to save file: {}", e))?;

                info!("✅ Saved {} bytes to disk", image_data.len());
                save_path
            }

            #[cfg(not(target_os = "android"))]
            {
                return Err("Content URIs are only supported on Android".to_string());
            }
        } else {
            // Regular path: use directly
            PathBuf::from(file_path)
        };

        client
            .send_direct_file(nickname, &actual_path)
            .await
            .map_err(|e| format!("Failed to send image: {}", e))?;

        info!("✅ Image sent successfully");

        // Return message ID and base64 data separated by a delimiter
        let message_id = uuid::Uuid::new_v4().to_string();
        info!(
            "🎯 Returning response: {}|base64_data({} bytes)",
            message_id,
            base64_data.len()
        );
        Ok(format!("{}|{}", message_id, base64_data))
    } else {
        error!("❌ P2P client not initialized");
        Err("P2P client not initialized".to_string())
    }
}

#[tauri::command]
async fn messaging_send_group_file_message_with_path(
    #[allow(unused_variables)] app: AppHandle,
    group_id: &str,
    file_path: &str,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!(
        "🎯 messaging_send_group_file_message_with_path called with group_id: {}, file_path: {}",
        group_id, file_path
    );

    let mut client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_mut() {
        let is_content_uri = file_path.starts_with("content://");

        // Read file data and convert to base64 for immediate display
        let image_data = if is_content_uri {
            // Android content URI: Use special handling
            // On Android, the dialog plugin may return content URIs
            // The standard fs plugin handles content URI translation
            let uri_str = file_path;
            info!("📱 Detected content URI (group): {}", uri_str);

            // On Android, we need to use a workaround to read content URIs
            #[cfg(target_os = "android")]
            {
                // For content URIs, use the android-fs plugin
                info!("📱 Attempting to read content URI (group): {}", uri_str);

                use tauri_plugin_android_fs::{AndroidFsExt, FileAccessMode, FileUri};
                use tauri_plugin_fs::FilePath;

                let android_api = app.android_fs();

                // Convert URI string to FileUri via FilePath::Url
                let url = tauri::Url::parse(uri_str)
                    .map_err(|e| format!("Failed to parse content URI: {}", e))?;
                let file_uri = FileUri::from(FilePath::Url(url));

                // Open the file and read its contents
                match android_api.open_file(&file_uri, FileAccessMode::Read) {
                    Ok(mut file) => {
                        let mut buffer = Vec::new();
                        use std::io::Read;
                        file.read_to_end(&mut buffer)
                            .map_err(|e| format!("Failed to read content URI data: {}", e))?;
                        buffer
                    }
                    Err(e) => {
                        error!("Failed to open content URI: {}", e);
                        return Err(format!("Failed to open content URI: {}", e));
                    }
                }
            }

            #[cfg(not(target_os = "android"))]
            {
                return Err("Content URIs are only supported on Android".to_string());
            }
        } else {
            // Regular filesystem path
            let path = PathBuf::from(file_path);

            // Verify file exists
            if !path.exists() {
                error!("❌ File does not exist: {}", file_path);
                return Err("File does not exist".to_string());
            }

            info!("✅ File exists, reading image data...");
            fs::read(&path).map_err(|e| format!("Failed to read image file: {}", e))?
        };

        let base64_data = base64::engine::general_purpose::STANDARD.encode(&image_data);
        info!(
            "✅ Successfully converted group image to base64, size: {} bytes",
            image_data.len()
        );

        info!("🔄 Sending group image via P2P...");

        // Send the group image using P2P functionality
        // For content URIs, save to app-specific location first, then share
        // For regular paths, share directly
        let actual_path = if is_content_uri {
            // Content URI: Save to app-specific location
            #[cfg(target_os = "android")]
            {
                info!("📱 Saving content URI (group) to app directory for sharing");

                // Get app download directory
                let config_guard = state.config.read().await;
                let download_dir = PathBuf::from(&config_guard.download_folder);
                drop(config_guard);

                // Create directory if needed
                fs::create_dir_all(&download_dir)
                    .map_err(|e| format!("Failed to create download dir: {}", e))?;

                // Try to extract filename from content URI
                let filename_from_uri = if let Some(display_name) =
                    file_path.split('=').last().and_then(|s| {
                        let decoded = percent_decode_str(s).decode_utf8().ok()?;
                        let name = decoded.split('/').last()?.to_string();
                        if name.is_empty() {
                            None
                        } else {
                            Some(name)
                        }
                    }) {
                    display_name
                } else {
                    // Fallback: extract from URI path
                    file_path
                        .split('/')
                        .last()
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| {
                            // Final fallback: use hash
                            let uri_hash = blake3::hash(file_path.as_bytes()).to_hex();
                            format!("gigi_group_{}", &uri_hash[..16])
                        })
                };

                // Detect file type from data and get appropriate extension
                let file_ext = if image_data.len() >= 8 {
                    // Check common signatures
                    if image_data.starts_with(&[0xFF, 0xD8, 0xFF]) {
                        ".jpg"
                    } else if image_data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                        ".png"
                    } else if image_data.starts_with(&[0x47, 0x49, 0x46]) {
                        ".gif"
                    } else if image_data.starts_with(b"WEBP")
                        || image_data.starts_with(&[0x52, 0x49, 0x46, 0x46])
                    {
                        ".webp"
                    } else if image_data.starts_with(b"BM") {
                        ".bmp"
                    } else if image_data.starts_with(&[0x00, 0x00, 0x00])
                        || image_data.starts_with(&[0x1A, 0x45, 0xDF, 0xA3])
                    {
                        // Video signatures: MP4/AVI or WebM
                        ".mp4"
                    } else {
                        // If filename already has extension, use it; otherwise use .dat
                        if filename_from_uri.contains('.') {
                            ""
                        } else {
                            ".dat"
                        }
                    }
                } else {
                    // For small files, trust the filename extension
                    ""
                };

                // Combine filename and extension
                let filename = if !file_ext.is_empty() && !filename_from_uri.contains('.') {
                    format!("{}{}", filename_from_uri, file_ext)
                } else {
                    filename_from_uri
                };

                let save_path = download_dir.join(&filename);

                info!("📝 Saving group content URI to: {:?}", save_path);
                // Save image data to file
                fs::write(&save_path, &image_data)
                    .map_err(|e| format!("Failed to save file: {}", e))?;

                info!("✅ Saved {} bytes (group) to disk", image_data.len());
                save_path
            }

            #[cfg(not(target_os = "android"))]
            {
                return Err("Content URIs are only supported on Android".to_string());
            }
        } else {
            // Regular path: use directly
            PathBuf::from(file_path)
        };

        client
            .send_group_file(group_id, &actual_path)
            .await
            .map_err(|e| format!("Failed to send group image: {}", e))?;

        info!("✅ Group image sent successfully");

        // Return message ID and base64 data separated by a delimiter
        let message_id = uuid::Uuid::new_v4().to_string();
        info!(
            "🎯 Returning group response: {}|base64_data({} bytes)",
            message_id,
            base64_data.len()
        );
        Ok(format!("{}|{}", message_id, base64_data))
    } else {
        error!("❌ P2P client not initialized");
        Err("P2P client not initialized".to_string())
    }
}

#[tauri::command]
async fn messaging_update_config(
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
async fn messaging_get_config(state: State<'_, AppState>) -> Result<Config, String> {
    let config_guard = state.config.read().await;
    Ok(config_guard.clone())
}

#[tauri::command]
async fn messaging_remove_shared_file(
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
async fn messaging_get_image_data(file_path: &str) -> Result<String, String> {
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
async fn messaging_select_any_file(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

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
async fn messaging_get_file_info(file_path: &str) -> Result<FileInfo, String> {
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
        peer_id: "".to_string(), // Will be set when sharing
    })
}

// Event handler
async fn handle_p2p_event_with_fields(
    event: P2pEvent,
    _p2p_client: &Arc<Mutex<Option<P2pClient>>>,
    _config: &Arc<RwLock<Config>>,
    active_downloads: &Arc<Mutex<HashMap<String, DownloadProgress>>>,
    app_handle: &AppHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match event {
        P2pEvent::PeerDiscovered {
            peer_id, nickname, ..
        } => {
            info!(
                "Emitting peer-discovered event for {} ({})",
                nickname, peer_id
            );
            let peer = Peer {
                id: peer_id.to_string(),
                nickname: nickname.clone(),
                capabilities: vec!["messaging".to_string(), "file_transfer".to_string()],
            };
            app_handle
                .emit("peer-discovered", &peer)
                .map_err(|e| format!("Failed to emit peer-discovered: {}", e))?;
        }
        P2pEvent::PeerExpired { peer_id, nickname } => {
            app_handle.emit(
                "peer-expired",
                &json!({
                    "peer_id": peer_id.to_string(),
                    "nickname": nickname
                }),
            )?;
        }
        P2pEvent::NicknameUpdated { peer_id, nickname } => {
            app_handle.emit(
                "nickname-updated",
                &json!({
                    "peer_id": peer_id.to_string(),
                    "nickname": nickname
                }),
            )?;
        }
        P2pEvent::DirectFileShareMessage {
            from,
            from_nickname,
            share_code,
            filename,
            file_size,
            file_type,
        } => {
            // Automatically download the image when receiving a file share message
            info!(
                "Received direct file share message from {} ({}) for file {} with share code {}",
                from_nickname, from, filename, share_code
            );

            // Check if it's an image file
            if file_type.starts_with("image/") {
                info!("Auto-downloading image file: {}", filename);
                if let Some(client) = _p2p_client.lock().await.as_mut() {
                    match client.download_file(&from_nickname, &share_code) {
                        Ok(()) => {
                            info!("Started auto-download for image: {}", filename);
                            // Emit image-message-received event so UI can show the image
                            app_handle.emit(
                                "image-message-received",
                                &json!({
                                    "from_peer_id": from.to_string(),
                                    "from_nickname": from_nickname,
                                    "share_code": share_code,
                                    "filename": filename,
                                    "file_size": file_size,
                                    "file_type": file_type,
                                    "timestamp": std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)?
                                        .as_secs(),
                                }),
                            )?;
                        }
                        Err(e) => {
                            error!(
                                "Failed to start auto-download for image {}: {}",
                                filename, e
                            );
                            // Still emit the event so UI can show error
                            app_handle.emit(
                                "image-message-received",
                                &json!({
                                    "from_peer_id": from.to_string(),
                                    "from_nickname": from_nickname,
                                    "share_code": share_code,
                                    "filename": filename,
                                    "file_size": file_size,
                                    "file_type": file_type,
                                    "timestamp": std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)?
                                        .as_secs(),
                                    "download_error": Some(format!("Failed to download: {}", e)),
                                }),
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
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)?
                            .as_secs(),
                    }),
                )?;
            }
        }
        P2pEvent::DirectMessage {
            from,
            from_nickname,
            message,
        } => {
            let msg = Message {
                id: uuid::Uuid::new_v4().to_string(),
                from_peer_id: from.to_string(),
                from_nickname,
                content: message,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs(),
            };
            app_handle.emit("message-received", &msg)?;
        }

        P2pEvent::DirectGroupShareMessage {
            from,
            from_nickname,
            group_id,
            group_name,
        } => {
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
        }
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
            info!(
                "Received group file share message from {} ({}) for file {} in group {} with share code {}",
                from_nickname, from, filename, group, share_code
            );

            // Check if it's an image file
            if file_type.starts_with("image/") {
                info!("Auto-downloading group image file: {}", filename);
                if let Some(client) = _p2p_client.lock().await.as_mut() {
                    match client.download_file(&from_nickname, &share_code) {
                        Ok(()) => {
                            info!("Started auto-download for group image: {}", filename);
                        }
                        Err(e) => {
                            error!(
                                "Failed to start auto-download for group image {}: {}",
                                filename, e
                            );
                            // Still emit the event so UI can show error
                            app_handle.emit(
                                "group-image-message-received",
                                &json!({
                                    "from_peer_id": from.to_string(),
                                    "from_nickname": from_nickname,
                                    "group_id": group,
                                    "share_code": share_code,
                                    "filename": filename,
                                    "file_size": file_size,
                                    "file_type": file_type,
                                    "timestamp": std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)?
                                        .as_secs(),
                                    "download_error": Some(format!("Failed to download: {}", e)),
                                }),
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
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)?
                            .as_secs(),
                    }),
                )?;
            }
        }
        P2pEvent::GroupMessage {
            from,
            from_nickname,
            group,
            message,
        } => {
            info!("Processing GroupMessage event:");
            info!("   - From: {} ({})", from_nickname, from);
            info!("   - Group: {}", group);
            info!("   - Message: {}", message);

            let msg = GroupMessage {
                id: uuid::Uuid::new_v4().to_string(),
                group_id: group,
                from_peer_id: from.to_string(),
                from_nickname,
                content: message,
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
        }

        P2pEvent::FileShareRequest {
            from,
            from_nickname,
            share_code,
            filename,
            size,
        } => {
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
        }
        P2pEvent::FileDownloadProgress {
            download_id,
            filename,
            share_code,
            from_peer_id,
            from_nickname,
            downloaded_chunks,
            total_chunks,
        } => {
            let progress = (downloaded_chunks as f32 / total_chunks as f32) * 100.0;
            let download_progress = DownloadProgress {
                download_id: download_id.clone(),
                progress,
                speed: 0, // TODO: Calculate speed
            };

            {
                let mut downloads_guard = active_downloads.lock().await;
                downloads_guard.insert(download_id.clone(), download_progress.clone());
            }

            // Emit progress with all required fields for frontend
            app_handle.emit(
                "file-download-progress",
                &json!({
                    "download_id": download_id,
                    "filename": filename,
                    "share_code": share_code,
                    "from_peer_id": from_peer_id.to_string(),
                    "from_nickname": from_nickname,
                    "progress_percent": progress,
                    "speed": 0 // TODO: Calculate actual speed
                }),
            )?;
        }
        P2pEvent::FileDownloadCompleted {
            download_id,
            filename,
            share_code,
            from_peer_id,
            from_nickname,
            path,
        } => {
            {
                let mut downloads_guard = active_downloads.lock().await;
                downloads_guard.remove(&download_id);
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
        }
        P2pEvent::FileDownloadStarted {
            from,
            from_nickname,
            filename,
        } => {
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
        }
        P2pEvent::FileDownloadFailed {
            download_id,
            filename,
            share_code,
            from_peer_id,
            from_nickname,
            error,
        } => {
            {
                let mut downloads_guard = active_downloads.lock().await;
                downloads_guard.remove(&download_id);
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
        }
        P2pEvent::Connected { peer_id, nickname } => {
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
        }
        P2pEvent::Disconnected { peer_id, nickname } => {
            app_handle.emit(
                "peer-disconnected",
                &json!({
                    "peer_id": peer_id.to_string(),
                    "nickname": nickname
                }),
            )?;
        }
        P2pEvent::Error(error) => {
            app_handle.emit("p2p-error", &error)?;
        }
        _ => {}
    }
    Ok(())
}

#[tauri::command]
async fn emit_current_state(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("emit_current_state called");

    // Clone the client to avoid holding the mutex lock during emit operations
    let client_clone = {
        debug!("Attempting to lock P2P client...");

        // Add timeout to prevent deadlock
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
                // Return success instead of error to avoid frontend issues
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

#[tauri::command]
async fn clear_app_data(app_handle: AppHandle) -> Result<(), String> {
    // Get app data directory
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // Remove app data directory if it exists
    if app_data_dir.exists() {
        fs::remove_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to remove app data directory: {}", e))?;
    }

    // Get local app data directory
    let local_data_dir = app_handle
        .path()
        .local_data_dir()
        .map_err(|e| format!("Failed to get local data directory: {}", e))?;

    // Remove local app data if it exists
    if local_data_dir.exists() {
        fs::remove_dir_all(&local_data_dir)
            .map_err(|e| format!("Failed to remove local data directory: {}", e))?;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging first
    init_logging();

    // Create initial state
    let app_state = AppState::default();

    let builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());

    #[cfg(target_os = "android")]
    let builder = { builder.plugin(tauri_plugin_android_fs::init()) };

    builder
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_peer_id,
            try_get_peer_id,
            messaging_initialize_with_key,
            messaging_send_message,
            messaging_send_message_to_nickname,
            messaging_send_direct_share_group_message,
            messaging_send_file_message_with_path,
            messaging_send_group_file_message_with_path,
            messaging_get_peers,
            messaging_set_nickname,
            messaging_join_group,
            messaging_send_group_message,
            messaging_share_file,
            messaging_request_file,
            messaging_request_file_from_nickname,
            messaging_cancel_download,
            messaging_get_shared_files,
            messaging_remove_shared_file,
            messaging_get_image_data,
            messaging_get_file_info,
            messaging_select_any_file,
            messaging_get_public_key,
            messaging_get_active_downloads,
            messaging_update_config,
            messaging_get_config,
            emit_current_state,
            clear_app_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
