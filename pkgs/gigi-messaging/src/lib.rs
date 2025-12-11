use gigi_p2p::{P2pClient, P2pEvent};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc as tokio_mpsc;
use futures::StreamExt;
use base64::Engine;

#[cfg(feature = "tauri")]
use tauri::{State, Manager};
#[cfg(feature = "tauri")]
use std::sync::mpsc;

mod models;
mod error;

pub use models::*;
pub use error::*;

// Inner state structure (without the Mutex)
#[derive(Default)]
pub struct MessagingStateInner {
    pub client: Option<MessagingClient>,
    pub event_sender: Option<tokio_mpsc::UnboundedSender<MessagingEvent>>,
    pub config: MessagingConfig,
}

// Type alias for the managed state
pub type MessagingState = Mutex<MessagingStateInner>;

impl MessagingClient {
    /// Create client from private key provided by frontend
    pub async fn from_private_key(
        private_key_base64: String,
        nickname: String,
        config: MessagingConfig,
    ) -> Result<Self, MessagingError> {
        // Decode base64 private key
        let private_key_bytes = base64::engine::general_purpose::STANDARD.decode(private_key_base64)
            .map_err(|_| MessagingError::InvalidPrivateKey)?;
        
        // Create keypair from private key bytes using ed25519_from_bytes
        let keypair = libp2p::identity::Keypair::ed25519_from_bytes(private_key_bytes[..32].to_vec())
            .map_err(|_| MessagingError::InvalidPrivateKey)?;
        
        let (p2p_client, p2p_event_receiver) = P2pClient::new_with_config(
            keypair,
            nickname,
            config.downloads_dir.clone(),
            config.share_json_path.clone(),
        )?;
        
        let client = Self {
            p2p_client: Arc::new(Mutex::new(p2p_client)),
            event_sender: tokio_mpsc::unbounded_channel().0,
            event_receiver: Some(tokio_mpsc::unbounded_channel().1),
            config,
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Start event translation task
        client.start_event_translation(p2p_event_receiver);
        
        Ok(client)
    }
    
    /// Generate new keypair for frontend
    pub fn generate_keypair() -> Result<KeyPair, MessagingError> {
        let keypair = libp2p::identity::Keypair::generate_ed25519();
        let private_bytes = keypair.to_protobuf_encoding()
            .map_err(|_| MessagingError::KeyGenerationFailed)?;
        let public_bytes = keypair.to_protobuf_encoding()
            .map_err(|_| MessagingError::KeyGenerationFailed)?;
        
        Ok(KeyPair {
            private_key: private_bytes,
            public_key: public_bytes,
        })
    }
    
    /// Set private key bytes before creating the swarm
    pub async fn set_private_key(&mut self, private_key_base64: String) -> Result<(), MessagingError> {
        // Decode base64 private key
        let private_key_bytes = base64::engine::general_purpose::STANDARD.decode(private_key_base64)
            .map_err(|_| MessagingError::InvalidPrivateKey)?;
        
        // Create keypair from private key bytes
        let keypair = libp2p::identity::Keypair::ed25519_from_bytes(private_key_bytes[..32].to_vec())
            .map_err(|_| MessagingError::InvalidPrivateKey)?;
        
        // Since P2pClient doesn't have update_keypair, we need to recreate it
        let nickname = self.get_nickname();
        let config = self.config.clone();
        let (new_p2p_client, p2p_event_receiver) = P2pClient::new_with_config(
            keypair,
            nickname,
            config.downloads_dir.clone(),
            config.share_json_path.clone(),
        )?;
        
        self.p2p_client = Arc::new(Mutex::new(new_p2p_client));
        
        // Start event translation for the new client
        self.start_event_translation(p2p_event_receiver);
        
        Ok(())
    }
    
    pub async fn new(nickname: String) -> Result<Self, MessagingError> {
        Self::with_config(nickname, MessagingConfig::default()).await
    }
    
    pub async fn with_config(nickname: String, config: MessagingConfig) -> Result<Self, MessagingError> {
        let keypair = libp2p::identity::Keypair::generate_ed25519();
        let (p2p_client, p2p_event_receiver) = P2pClient::new_with_config(
            keypair,
            nickname,
            config.downloads_dir.clone(),
            config.share_json_path.clone(),
        )?;
        
        let client = Self {
            p2p_client: Arc::new(Mutex::new(p2p_client)),
            event_sender: tokio_mpsc::unbounded_channel().0,
            event_receiver: Some(tokio_mpsc::unbounded_channel().1),
            config,
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Start event translation task
        client.start_event_translation(p2p_event_receiver);
        
        Ok(client)
    }
    
    fn start_event_translation(&self, mut p2p_event_receiver: futures::channel::mpsc::UnboundedReceiver<P2pEvent>) {
        let event_sender = self.event_sender.clone();
        
        tokio::spawn(async move {
            while let Some(p2p_event) = p2p_event_receiver.next().await {
                let messaging_event = match p2p_event {
                    P2pEvent::PeerDiscovered { peer_id, nickname, address: _ } => {
                        MessagingEvent::PeerJoined {
                            peer_id: peer_id.to_string(),
                            nickname,
                        }
                    }
                    P2pEvent::PeerExpired { peer_id, nickname } => {
                        MessagingEvent::PeerLeft {
                            peer_id: peer_id.to_string(),
                            nickname,
                        }
                    }
                    P2pEvent::NicknameUpdated { peer_id, nickname } => {
                        MessagingEvent::NicknameChanged {
                            peer_id: peer_id.to_string(),
                            nickname,
                        }
                    }
                    P2pEvent::DirectMessage { from, from_nickname: _, message } => {
                        MessagingEvent::MessageReceived {
                            from: from.to_string(),
                            content: message,
                        }
                    }
                    P2pEvent::DirectImageMessage { from, from_nickname: _, filename, data } => {
                        MessagingEvent::ImageReceived {
                            from: from.to_string(),
                            filename,
                            data,
                        }
                    }
                    P2pEvent::GroupMessage { from, from_nickname: _, group, message } => {
                        MessagingEvent::GroupMessageReceived {
                            from: from.to_string(),
                            group,
                            content: message,
                        }
                    }
                    P2pEvent::GroupImageMessage { from, from_nickname: _, group: _, filename, data, message: _ } => {
                        MessagingEvent::ImageReceived {
                            from: from.to_string(),
                            filename,
                            data,
                        }
                    }
                    P2pEvent::FileShared { file_id, info } => {
                        MessagingEvent::FileShared {
                            file_id: file_id.clone(),
                            filename: info.name,
                            share_code: file_id, // In P2pClient, file_id is the share_code
                        }
                    }
                    P2pEvent::FileRevoked { file_id } => {
                        MessagingEvent::FileRevoked { file_id }
                    }
                    P2pEvent::FileDownloadStarted { from: _, from_nickname: _, filename } => {
                        MessagingEvent::FileTransferStarted {
                            file_id: format!("download_{}", filename),
                            filename,
                            total_size: 0, // Will be updated when FileInfo is received
                        }
                    }
                    P2pEvent::FileDownloadProgress { file_id, downloaded_chunks, total_chunks } => {
                        MessagingEvent::FileTransferProgress {
                            file_id,
                            downloaded_size: (downloaded_chunks * 256 * 1024) as u64, // Approximate
                            total_size: (total_chunks * 256 * 1024) as u64,
                            speed: 0.0, // TODO: Calculate actual speed
                        }
                    }
                    P2pEvent::FileDownloadCompleted { file_id, path } => {
                        MessagingEvent::FileTransferCompleted {
                            file_id,
                            filename: path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string(),
                            final_path: path,
                        }
                    }
                    P2pEvent::FileDownloadFailed { file_id, error } => {
                        MessagingEvent::FileTransferFailed { file_id, error }
                    }
                    P2pEvent::Error(error) => {
                        MessagingEvent::Error { message: error }
                    }
                    _ => {
                        // Skip events we don't translate
                        continue;
                    }
                };
                
                let _ = event_sender.send(messaging_event);
            }
        });
    }
    
    pub async fn start(&mut self) -> Result<(), MessagingError> {
        let mut client = self.p2p_client.lock().unwrap();
        // Start listening on default address
        let addr = "/ip4/0.0.0.0/tcp/0".parse().unwrap();
        client.start_listening(addr)?;
        Ok(())
    }
    
    pub async fn shutdown(&mut self) -> Result<(), MessagingError> {
        let mut client = self.p2p_client.lock().unwrap();
        client.shutdown()?;
        Ok(())
    }
    
    pub async fn set_nickname(&mut self, nickname: String) -> Result<(), MessagingError> {
        let mut client = self.p2p_client.lock().unwrap();
        client.local_nickname = nickname.clone();
        Ok(())
    }
    
    pub fn get_nickname(&self) -> String {
        let client = self.p2p_client.lock().unwrap();
        client.local_nickname.clone()
    }
    
    pub fn get_peer_id(&self) -> String {
        let client = self.p2p_client.lock().unwrap();
        client.local_peer_id().to_string()
    }
    
    pub fn get_public_key(&self) -> String {
        let client = self.p2p_client.lock().unwrap();
        base64::engine::general_purpose::STANDARD.encode(client.local_peer_id().to_bytes())
    }
    
    pub async fn get_connected_peers(&self) -> Result<Vec<PeerInfo>, MessagingError> {
        let client = self.p2p_client.lock().unwrap();
        let p2p_peers = client.list_peers();
        Ok(p2p_peers.into_iter().map(|p| PeerInfo {
            id: p.peer_id.to_string(),
            nickname: p.nickname.clone(),
            address: p.addresses.first().map(|a| a.to_string()),
            last_seen: Some(p.last_seen.elapsed().as_secs()),
        }).collect())
    }
    
    pub async fn send_message(&self, to_peer: String, message: String) -> Result<(), MessagingError> {
        let mut client = self.p2p_client.lock().unwrap();
        client.send_direct_message(&to_peer, message)?;
        Ok(())
    }
    
    pub async fn send_image(&self, to_peer: String, image_data: Vec<u8>, filename: String) -> Result<(), MessagingError> {
        // Create temporary file for image
        let temp_path = std::env::temp_dir().join(&filename);
        std::fs::write(&temp_path, image_data)?;
        
        let mut client = self.p2p_client.lock().unwrap();
        client.send_direct_image(&to_peer, &temp_path)?;
        
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);
        
        Ok(())
    }
    
    pub async fn join_group(&mut self, group_name: String) -> Result<(), MessagingError> {
        let mut client = self.p2p_client.lock().unwrap();
        client.join_group(&group_name)?;
        Ok(())
    }
    
    pub async fn leave_group(&mut self, group_name: String) -> Result<(), MessagingError> {
        let mut client = self.p2p_client.lock().unwrap();
        client.leave_group(&group_name)?;
        Ok(())
    }
    
    pub async fn send_group_message(&self, group: String, message: String) -> Result<(), MessagingError> {
        let mut client = self.p2p_client.lock().unwrap();
        client.send_group_message(&group, message)?;
        Ok(())
    }
    
    pub async fn send_group_image(&self, group: String, image_data: Vec<u8>, filename: String) -> Result<(), MessagingError> {
        // Create temporary file for image
        let temp_path = std::env::temp_dir().join(&filename);
        std::fs::write(&temp_path, image_data)?;
        
        let mut client = self.p2p_client.lock().unwrap();
        client.send_group_image(&group, &temp_path).await?;
        
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);
        
        Ok(())
    }
    
    pub async fn share_file(&mut self, file_path: String) -> Result<String, MessagingError> {
        let mut client = self.p2p_client.lock().unwrap();
        let share_code = client.share_file(&std::path::Path::new(&file_path)).await?;
        Ok(share_code)
    }
    
    pub async fn request_file(&self, share_code: String) -> Result<String, MessagingError> {
        self.handle_file_download(share_code).await
    }
    
    pub async fn cancel_download(&self, download_id: String) -> Result<(), MessagingError> {
        self.active_downloads.lock().unwrap().remove(&download_id);
        Ok(())
    }
    
    pub async fn unshare_file(&mut self, share_code: String) -> Result<(), MessagingError> {
        let mut client = self.p2p_client.lock().unwrap();
        client.unshare_file(&share_code)?;
        Ok(())
    }
    
    pub async fn get_shared_files(&self) -> Result<Vec<SharedFileInfo>, MessagingError> {
        let client = self.p2p_client.lock().unwrap();
        let files = client.list_shared_files();
        Ok(files.into_iter().map(|f| SharedFileInfo {
            id: f.info.id.clone(),
            name: f.info.name.clone(),
            size: f.info.size,
            share_code: f.share_code.clone(),
            created_at: f.info.created_at,
        }).collect())
    }
    
    pub async fn get_active_downloads(&self) -> Result<Vec<DownloadInfo>, MessagingError> {
        let downloads = self.active_downloads.lock().unwrap();
        Ok(downloads.values().cloned().collect())
    }
    
    pub async fn update_config(&mut self, updates: MessagingConfigUpdates) -> Result<(), MessagingError> {
        if let Some(share_json_path) = updates.share_json_path {
            self.config.share_json_path = share_json_path;
        }
        if let Some(downloads_dir) = updates.downloads_dir {
            self.config.downloads_dir = downloads_dir;
        }
        if let Some(temp_dir) = updates.temp_dir {
            self.config.temp_dir = temp_dir;
        }
        if let Some(max_file_size) = updates.max_file_size {
            self.config.max_file_size = max_file_size;
        }
        if let Some(chunk_size) = updates.chunk_size {
            self.config.chunk_size = chunk_size;
        }
        Ok(())
    }
    
    pub fn get_config(&self) -> &MessagingConfig {
        &self.config
    }
    
    pub async fn next_event(&mut self) -> Option<MessagingEvent> {
        if let Some(ref mut receiver) = self.event_receiver {
            receiver.recv().await
        } else {
            None
        }
    }
    
    pub fn try_event(&mut self) -> Option<MessagingEvent> {
        if let Some(ref mut receiver) = self.event_receiver {
            receiver.try_recv().ok()
        } else {
            None
        }
    }
    
    pub fn subscribe_events(&self) -> tokio_mpsc::UnboundedReceiver<MessagingEvent> {
        let (_sender, receiver) = tokio_mpsc::unbounded_channel();
        // Note: In a real implementation, you'd want to store this sender
        // to broadcast events to all subscribers
        receiver
    }
    
    async fn handle_file_download(&self, share_code: String) -> Result<String, MessagingError> {
        let download_id = uuid::Uuid::new_v4().to_string();
        
        // For now, we'll use a simplified approach - just request the file
        // from all connected peers (in a real implementation, you'd have a way to
        // know which peer has the file)
        let peers = self.get_connected_peers().await?;
        if let Some(first_peer) = peers.first() {
            let mut client = self.p2p_client.lock().unwrap();
            client.download_file(&first_peer.nickname, &share_code)?;
        }
        
        Ok(download_id)
    }
    
    async fn perform_download(
        _client: Arc<Mutex<P2pClient>>,
        _share_code: String,
        _temp_path: PathBuf,
        _download_id: String,
    ) -> Result<(), MessagingError> {
        let _start_time = std::time::Instant::now();
        // Implementation of chunked download with progress reporting
        Ok(())
    }
    
    async fn request_file_direct(&self, share_code: String, peer_nickname: String) -> Result<String, MessagingError> {
        let mut client = self.p2p_client.lock().unwrap();
        client.download_file(&peer_nickname, &share_code)?;
        Ok(share_code) // Return share_code as download_id for tracking
    }
    
    async fn get_file_info_from_share(&self, share_code: &str) -> Result<FileSharedInfo, MessagingError> {
        // This is a placeholder - in a real implementation, you'd parse the share.json
        // file to get information about the shared file
        Err(MessagingError::FileNotFound(share_code.to_string()))
    }
}

#[cfg(feature = "tauri")]
// Tauri Commands
#[tauri::command]
async fn messaging_send_message(
    state: tauri::State<'_, MessagingState>,
    to: String,
    message: String,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    if let Some(client) = state.client.as_mut() {
        client.send_message(to, message).await
            .map_err(|e| e.to_string())
    } else {
        Err("Messaging client not initialized".to_string())
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
async fn messaging_get_peers(
    state: tauri::State<'_, MessagingState>,
) -> Result<Vec<PeerInfo>, String> {
    let mut state = state.lock().unwrap();
    if let Some(client) = state.client.as_mut() {
        client.get_connected_peers().await
            .map_err(|e| e.to_string())
    } else {
        Ok(vec![])
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
async fn messaging_set_nickname(
    state: tauri::State<'_, MessagingState>,
    nickname: String,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    if let Some(client) = state.client.as_mut() {
        client.set_nickname(nickname).await
            .map_err(|e| e.to_string())
    } else {
        Err("Messaging client not initialized".to_string())
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
async fn messaging_join_group(
    state: tauri::State<'_, MessagingState>,
    group_name: String,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    if let Some(client) = state.client.as_mut() {
        client.join_group(group_name).await
            .map_err(|e| e.to_string())
    } else {
        Err("Messaging client not initialized".to_string())
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
async fn messaging_send_group_message(
    state: tauri::State<'_, MessagingState>,
    group: String,
    message: String,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    if let Some(client) = state.client.as_mut() {
        client.send_group_message(group, message).await
            .map_err(|e| e.to_string())
    } else {
        Err("Messaging client not initialized".to_string())
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
async fn messaging_share_file(
    state: tauri::State<'_, MessagingState>,
    file_path: String,
) -> Result<String, String> {
    let mut state = state.lock().unwrap();
    if let Some(client) = state.client.as_mut() {
        client.share_file(file_path).await
            .map_err(|e| e.to_string())
    } else {
        Err("Messaging client not initialized".to_string())
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
async fn messaging_get_shared_files(
    state: tauri::State<'_, MessagingState>,
) -> Result<Vec<SharedFileInfo>, String> {
    let mut state = state.lock().unwrap();
    if let Some(client) = state.client.as_mut() {
        client.get_shared_files().await
            .map_err(|e| e.to_string())
    } else {
        Ok(vec![])
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
fn messaging_get_peer_id(
    state: tauri::State<'_, MessagingState>,
) -> Result<String, String> {
    let state = state.lock().unwrap();
    if let Some(client) = state.client.as_ref() {
        Ok(client.get_peer_id())
    } else {
        Err("Messaging client not initialized".to_string())
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
async fn messaging_initialize(
    app: tauri::AppHandle,
    nickname: String,
) -> Result<(), String> {
    // Get the state from the app handle
    let state = app.state::<MessagingState>();
    
    // Initialize the messaging client with default config
    let config = {
        let state_guard = state.lock().unwrap();
        state_guard.config.clone()
    };
    
    let client = MessagingClient::with_config(nickname, config).await
        .map_err(|e| e.to_string())?;
    
    // Create event channel for real-time events
    let (event_sender, _event_receiver) = tokio_mpsc::unbounded_channel::<MessagingEvent>();
    
    // Store the client and event sender in the state
    {
        let mut state = state.lock().unwrap();
        state.client = Some(client);
        state.event_sender = Some(event_sender);
    }
    
    // Start the client
    {
        let mut state = state.lock().unwrap();
        if let Some(client) = state.client.as_mut() {
            client.start().await
                .map_err(|e| e.to_string())?;
        }
    }
    
    Ok(())
}

#[cfg(feature = "tauri")]
#[tauri::command]
async fn messaging_initialize_with_key(
    app: tauri::AppHandle,
    private_key: String,
    nickname: String,
) -> Result<(), String> {
    // Get the state from the app handle
    let state = app.state::<MessagingState>();
    
    // Initialize the messaging client with provided private key
    let config = {
        let state_guard = state.lock().unwrap();
        state_guard.config.clone()
    };
    
    let client = MessagingClient::from_private_key(private_key, nickname, config).await
        .map_err(|e| e.to_string())?;
    
    // Create event channel for real-time events
    let (event_sender, _event_receiver) = tokio_mpsc::unbounded_channel::<MessagingEvent>();
    
    // Store the client and event sender in the state
    {
        let mut state = state.lock().unwrap();
        state.client = Some(client);
        state.event_sender = Some(event_sender);
    }
    
    // Start the client
    {
        let mut state = state.lock().unwrap();
        if let Some(client) = state.client.as_mut() {
            client.start().await
                .map_err(|e| e.to_string())?;
        }
    }
    
    Ok(())
}

#[cfg(feature = "tauri")]
#[tauri::command]
fn messaging_generate_keypair() -> Result<KeyPair, String> {
    MessagingClient::generate_keypair()
        .map_err(|e| e.to_string())
}

#[cfg(feature = "tauri")]
#[tauri::command]
async fn messaging_set_private_key(
    state: tauri::State<'_, MessagingState>,
    private_key: String,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    if let Some(client) = state.client.as_mut() {
        client.set_private_key(private_key).await
            .map_err(|e| e.to_string())
    } else {
        Err("Messaging client not initialized".to_string())
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
async fn messaging_request_file(
    state: tauri::State<'_, MessagingState>,
    share_code: String,
) -> Result<String, String> {
    let mut state = state.lock().unwrap();
    if let Some(client) = state.client.as_mut() {
        client.request_file(share_code).await
            .map_err(|e| e.to_string())
    } else {
        Err("Messaging client not initialized".to_string())
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
async fn messaging_cancel_download(
    state: tauri::State<'_, MessagingState>,
    download_id: String,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    if let Some(client) = state.client.as_mut() {
        client.cancel_download(download_id).await
            .map_err(|e| e.to_string())
    } else {
        Err("Messaging client not initialized".to_string())
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
async fn messaging_update_config(
    state: tauri::State<'_, MessagingState>,
    updates: MessagingConfigUpdates,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    if let Some(client) = state.client.as_mut() {
        client.update_config(updates).await
            .map_err(|e| e.to_string())
    } else {
        Err("Messaging client not initialized".to_string())
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
fn messaging_get_config(
    state: tauri::State<'_, MessagingState>,
) -> Result<MessagingConfig, String> {
    let state = state.lock().unwrap();
    Ok(state.config.clone())
}

#[cfg(feature = "tauri")]
#[tauri::command]
fn messaging_get_public_key(
    state: tauri::State<'_, MessagingState>,
) -> Result<String, String> {
    let state = state.lock().unwrap();
    if let Some(client) = state.client.as_ref() {
        Ok(client.get_public_key())
    } else {
        Err("Messaging client not initialized".to_string())
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
async fn messaging_get_active_downloads(
    state: tauri::State<'_, MessagingState>,
) -> Result<Vec<DownloadInfo>, String> {
    let mut state = state.lock().unwrap();
    if let Some(client) = state.client.as_mut() {
        client.get_active_downloads().await
            .map_err(|e| e.to_string())
    } else {
        Ok(vec![])
    }
}

#[cfg(feature = "tauri")]
// Initialize the Tauri app with commands and state
pub fn init_messaging_plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("gigi-messaging")
        .invoke_handler(tauri::generate_handler![
            messaging_initialize,
            messaging_initialize_with_key,
            messaging_generate_keypair,
            messaging_set_private_key,
            messaging_send_message,
            messaging_get_peers,
            messaging_set_nickname,
            messaging_join_group,
            messaging_send_group_message,
            messaging_share_file,
            messaging_request_file,
            messaging_cancel_download,
            messaging_get_shared_files,
            messaging_get_peer_id,
            messaging_get_public_key,
            messaging_get_active_downloads,
            messaging_update_config,
            messaging_get_config,
        ])
        .setup(|app, _api| {
            // Initialize messaging state using Tauri's state management
            app.manage(Mutex::new(MessagingStateInner {
                client: None,
                event_sender: None,
                config: MessagingConfig::default(),
            }));
            Ok(())
        })
        .build()
}