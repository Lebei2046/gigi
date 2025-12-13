// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use futures::channel::mpsc;
use futures::StreamExt;
use gigi_p2p::{FileInfo as P2pFileInfo, P2pClient, P2pEvent};
use hex;
use libp2p::identity;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::{Mutex, RwLock};

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
    pub file_id: String,
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

impl Default for Config {
    fn default() -> Self {
        Self {
            nickname: "Anonymous".to_string(),
            auto_accept_files: false,
            download_folder: "./downloads".to_string(),
            max_concurrent_downloads: 3,
            port: 0,
        }
    }
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
    pub shared_files: Arc<Mutex<HashMap<String, FileInfo>>>,
    pub shared_files_path: PathBuf,
}

impl AppState {
    pub fn new() -> Self {
        Self::with_download_folder("./downloads")
    }

    pub fn with_download_folder(download_folder: &str) -> Self {
        let base_path = PathBuf::from(download_folder);
        Self {
            p2p_client: Arc::new(Mutex::new(None)),
            event_receiver: Arc::new(Mutex::new(None)),
            config: Arc::new(RwLock::new(Config::default())),
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
            shared_files: Arc::new(Mutex::new(HashMap::new())),
            shared_files_path: base_path.join("shared_files.json"),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions for shared files persistence

async fn load_shared_files(shared_files_path: &PathBuf) -> HashMap<String, FileInfo> {
    if shared_files_path.exists() {
        match fs::read_to_string(shared_files_path) {
            Ok(content) => match serde_json::from_str::<HashMap<String, FileInfo>>(&content) {
                Ok(files) => files,
                Err(e) => {
                    eprintln!("Failed to deserialize shared files: {}", e);
                    HashMap::new()
                }
            },
            Err(e) => {
                eprintln!("Failed to read shared files file: {}", e);
                HashMap::new()
            }
        }
    } else {
        HashMap::new()
    }
}

async fn save_shared_files(
    shared_files: &HashMap<String, FileInfo>,
    shared_files_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = shared_files_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(shared_files)?;
    fs::write(shared_files_path, content)?;
    Ok(())
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
    let final_nickname = config_guard.nickname.clone();
    drop(config_guard);

    match P2pClient::new(keypair, final_nickname, output_dir) {
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
            let shared_files = state.shared_files.clone();
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
                println!("🔥 Starting swarm event polling task");
                loop {
                    // Check if client is initialized and poll for events with timeout
                    let client_ready = {
                        let client_guard = p2p_client_for_events.lock().await;
                        client_guard.as_ref().is_some()
                    };

                    if client_ready {
                        // Use tokio::select! with timeout to avoid blocking indefinitely
                        let swarm_event_result =
                            tokio::time::timeout(tokio::time::Duration::from_millis(100), async {
                                let mut client_guard = p2p_client_for_events.lock().await;
                                if let Some(client) = client_guard.as_mut() {
                                    Some(client.swarm.select_next_some().await)
                                } else {
                                    None
                                }
                            })
                            .await;

                        match swarm_event_result {
                            Ok(Some(swarm_event)) => {
                                // Briefly lock again to handle the event
                                let mut client_guard = p2p_client_for_events.lock().await;
                                if let Some(client) = client_guard.as_mut() {
                                    println!("🎯 Processing swarm event...");
                                    if let Err(e) = client.handle_event(swarm_event) {
                                        eprintln!("Error handling swarm event: {}", e);
                                    }
                                    println!("✅ Swarm event processed");
                                }
                            }
                            Ok(None) => {
                                println!("⚠️ Client became None, waiting...");
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            }
                            Err(_) => {
                                // Timeout occurred - this is expected, gives other tasks chance to run
                                // Don't print anything to avoid spam
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
                println!("🚀 Starting P2P event handling task");

                loop {
                    println!("⏳ Waiting for P2P event...");
                    if let Some(event) = receiver.next().await {
                        println!(
                            "📦 Received P2P event: {:?}",
                            std::mem::discriminant(&event)
                        );
                        if let Err(e) = handle_p2p_event_with_fields(
                            event,
                            &p2p_client,
                            &config,
                            &active_downloads,
                            &shared_files,
                            &app_handle_clone,
                        )
                        .await
                        {
                            eprintln!("Error handling P2P event: {}", e);
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

        // Get file metadata and store it
        let metadata =
            fs::metadata(&path).map_err(|e| format!("Failed to get file metadata: {}", e))?;

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("Invalid file name")?
            .to_string();

        let file_info = FileInfo {
            id: share_code.clone(),
            name: file_name,
            size: metadata.len(),
            mime_type: mime_guess::from_path(&path)
                .first_or_octet_stream()
                .to_string(),
            peer_id: {
                let client_guard = state.p2p_client.lock().await;
                if let Some(client) = client_guard.as_ref() {
                    client.local_peer_id().to_string()
                } else {
                    "".to_string()
                }
            },
        };

        // Store the file info
        {
            let mut shared_files_guard = state.shared_files.lock().await;
            shared_files_guard.insert(share_code.clone(), file_info);
        }

        // Save to disk
        if let Err(e) =
            save_shared_files(&*state.shared_files.lock().await, &state.shared_files_path).await
        {
            eprintln!("Failed to save shared files: {}", e);
        }

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
    let mut client_guard = state.p2p_client.lock().await;
    if let Some(client) = client_guard.as_mut() {
        client
            .download_file(nickname, share_code)
            .map_err(|e| format!("Failed to request file: {}", e))?;
        Ok(uuid::Uuid::new_v4().to_string())
    } else {
        Err("P2P client not initialized".to_string())
    }
}

#[tauri::command]
fn messaging_cancel_download(download_id: &str, _state: State<'_, AppState>) -> Result<(), String> {
    // Placeholder - would need to implement in gigi-p2p
    println!("Cancelled download: {}", download_id);
    Ok(())
}

#[tauri::command]
async fn messaging_get_shared_files(state: State<'_, AppState>) -> Result<Vec<FileInfo>, String> {
    let files_guard = state.shared_files.lock().await;
    Ok(files_guard.values().cloned().collect())
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
    // Remove from memory
    {
        let mut shared_files_guard = state.shared_files.lock().await;
        shared_files_guard.remove(share_code);
    }

    // Save to disk
    save_shared_files(&*state.shared_files.lock().await, &state.shared_files_path)
        .await
        .map_err(|e| format!("Failed to save shared files: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn messaging_save_shared_files(state: State<'_, AppState>) -> Result<(), String> {
    save_shared_files(&*state.shared_files.lock().await, &state.shared_files_path)
        .await
        .map_err(|e| format!("Failed to save shared files: {}", e))
}

// Event handler
async fn handle_p2p_event_with_fields(
    event: P2pEvent,
    _p2p_client: &Arc<Mutex<Option<P2pClient>>>,
    _config: &Arc<RwLock<Config>>,
    active_downloads: &Arc<Mutex<HashMap<String, DownloadProgress>>>,
    _shared_files: &Arc<Mutex<HashMap<String, FileInfo>>>,
    app_handle: &AppHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match event {
        P2pEvent::PeerDiscovered {
            peer_id, nickname, ..
        } => {
            println!(
                "🔥 Rust emitting peer-discovered event for {} ({})",
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
            app_handle.emit("direct-message", &msg)?;
        }
        P2pEvent::GroupMessage {
            from,
            from_nickname,
            group,
            message,
        } => {
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
            app_handle.emit("group-message", &msg)?;
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
            file_id,
            downloaded_chunks,
            total_chunks,
        } => {
            let progress = (downloaded_chunks as f32 / total_chunks as f32) * 100.0;
            let download_progress = DownloadProgress {
                file_id: file_id.clone(),
                progress,
                speed: 0, // TODO: Calculate speed
            };

            {
                let mut downloads_guard = active_downloads.lock().await;
                downloads_guard.insert(file_id.clone(), download_progress.clone());
            }

            app_handle.emit("download-progress", &download_progress)?;
        }
        P2pEvent::FileDownloadCompleted { file_id, path } => {
            {
                let mut downloads_guard = active_downloads.lock().await;
                downloads_guard.remove(&file_id);
            }

            app_handle.emit(
                "download-completed",
                &json!({
                    "file_id": file_id,
                    "path": path.to_string_lossy()
                }),
            )?;
        }
        P2pEvent::FileDownloadFailed { file_id, error } => {
            {
                let mut downloads_guard = active_downloads.lock().await;
                downloads_guard.remove(&file_id);
            }

            app_handle.emit(
                "download-failed",
                &json!({
                    "file_id": file_id,
                    "error": error
                }),
            )?;
        }
        P2pEvent::Connected { peer_id, nickname } => {
            println!(
                "🔥 Rust emitting peer-connected event for {} ({})",
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
    println!("🚀 emit_current_state called");

    // Clone the client to avoid holding the mutex lock during emit operations
    let client_clone = {
        println!("🔒 Attempting to lock P2P client...");

        // Add timeout to prevent deadlock
        match tokio::time::timeout(tokio::time::Duration::from_secs(2), state.p2p_client.lock())
            .await
        {
            Ok(client_guard) => {
                println!("✅ Successfully locked P2P client");

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

                    println!("📤 Collected {} peers, releasing lock", peers.len());
                    Some((peer_id, peer_data))
                } else {
                    None
                }
            }
            Err(_) => {
                println!("⚠️ Timeout: Failed to acquire P2P client lock within 2 seconds, but continuing...");
                // Return success instead of error to avoid frontend issues
                return Ok(());
            }
        }
    };

    // Now emit events without holding the mutex lock
    match client_clone {
        Some((peer_id, peers)) => {
            println!("📤 Emitting peer-id-changed: {}", peer_id);
            app_handle
                .emit("peer-id-changed", &peer_id)
                .map_err(|e| format!("Failed to emit peer-id-changed: {}", e))?;
            println!("✅ peer-id-changed emitted successfully");

            for peer in peers {
                println!(
                    "📤 Emitting peer-discovered for {} ({})",
                    peer.nickname, peer.id
                );
                app_handle
                    .emit("peer-discovered", &peer)
                    .map_err(|e| format!("Failed to emit peer-discovered: {}", e))?;
                println!("✅ peer-discovered emitted successfully");

                println!(
                    "📤 Emitting peer-connected for {} ({})",
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
                println!("✅ peer-connected emitted successfully");
            }

            println!("🎉 emit_current_state completed successfully");
            Ok(())
        }
        None => {
            println!("❌ P2P client not initialized");
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
    // Create initial state
    let app_state = AppState::default();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .setup(|app| {
            // Load shared files from disk in async context
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Some(app_state) = app_handle.try_state::<AppState>() {
                    let shared_files = load_shared_files(&app_state.shared_files_path).await;
                    let mut files_guard = app_state.shared_files.lock().await;
                    *files_guard = shared_files;
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_peer_id,
            try_get_peer_id,
            messaging_initialize_with_key,
            messaging_send_message,
            messaging_send_message_to_nickname,
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
            messaging_save_shared_files,
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
