# Gigi Mobile Integration Specification

## Overview

This document describes the integration of `gigi-p2p` as the network layer for `gigi-mobile`, with `gigi-messaging` providing a high-level command and event interface. The architecture separates concerns cleanly:

- **gigi-p2p**: Low-level P2P networking library (libp2p-based)
- **gigi-messaging**: High-level command/event API layer 
- **gigi-mobile**: Mobile application that consumes gigi-messaging

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   gigi-mobile   │───▶│  gigi-messaging  │───▶│   gigi-p2p      │
│  (Mobile App)   │    │ (Command API)    │    │ (Network Layer) │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │                         │
                              ▼                         ▼
                       Commands/Events            Network Events
```

## Integration Requirements

### 1. Library Dependencies

**gigi-messaging/Cargo.toml**:
```toml
[package]
name = "gigi-messaging"
version = "0.1.0"
edition = "2021"

[lib]
name = "gigi_messaging_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
gigi-p2p = { path = "../gigi-p2p" }
tauri = { version = "2", features = [] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
tracing = "0.1"
```

### 2. Core Components

#### 2.1 Configuration

Configuration for paths and settings:

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingConfig {
    /// Path to share.json file for file sharing metadata
    pub share_json_path: PathBuf,
    
    /// Directory path for saving downloaded files
    pub downloads_dir: PathBuf,
    
    /// Directory path for temporary files during transfer
    pub temp_dir: PathBuf,
    
    /// Maximum file size for sharing (in bytes)
    pub max_file_size: u64,
    
    /// Chunk size for file transfers
    pub chunk_size: usize,
}

impl Default for MessagingConfig {
    fn default() -> Self {
        let mut home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home_dir.push(".gigi");
        
        let mut downloads_dir = home_dir.clone();
        downloads_dir.push("downloads");
        
        let mut temp_dir = home_dir.clone();
        temp_dir.push("temp");
        
        let mut share_json_path = home_dir.clone();
        share_json_path.push("share.json");
        
        Self {
            share_json_path,
            downloads_dir,
            temp_dir,
            max_file_size: 100 * 1024 * 1024, // 100MB
            chunk_size: 256 * 1024, // 256KB
        }
    }
}
```

#### 2.2 Key Management

Enhanced key management with frontend-provided private keys:

```rust
use libp2p::identity::Keypair;
use serde::{Deserialize, Serialize};
use base64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    #[serde(with = "base64_serde")]
    private_key: Vec<u8>,
    #[serde(with = "base64_serde")]
    public_key: Vec<u8>,
}

mod base64_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use base64::{Engine as _, engine::general_purpose};
    
    pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoded = general_purpose::STANDARD.encode(bytes);
        serializer.serialize_str(&encoded)
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        general_purpose::STANDARD.decode(encoded.as_bytes()).map_err(serde::de::Error::custom)
    }
}

impl MessagingClient {
    /// Create client from private key provided by frontend
    pub async fn from_private_key(
        private_key_base64: String,
        nickname: String,
        config: MessagingConfig,
    ) -> Result<Self, MessagingError> {
        // Decode base64 private key
        let private_key_bytes = base64::decode(private_key_base64)
            .map_err(|_| MessagingError::InvalidPrivateKey)?;
        
        // Create keypair from private key bytes using ed25519_from_bytes
        let keypair = libp2p::identity::ed25519_from_bytes(&private_key_bytes[..32])
            .map_err(|_| MessagingError::InvalidPrivateKey)?;
        let keypair = Keypair::Ed25519(keypair);
        
        let mut client = Self {
            p2p_client: Arc::new(Mutex::new(P2pClient::new_with_keypair(keypair, nickname)?)),
            event_sender: mpsc::unbounded_channel().0,
            event_receiver: Some(mpsc::unbounded_channel().1),
            config,
            // ... other fields
        };
        
        Ok(client)
    }
    
    /// Generate new keypair for frontend
    pub fn generate_keypair() -> Result<KeyPair, MessagingError> {
        let keypair = Keypair::generate_ed25519();
        let private_bytes = keypair.to_protobuf_encoding()
            .map_err(|_| MessagingError::KeyGenerationFailed)?;
        let public_bytes = keypair.public().to_protobuf_encoding()
            .map_err(|_| MessagingError::KeyGenerationFailed)?;
        
        Ok(KeyPair {
            private_key: private_bytes,
            public_key: public_bytes,
        })
    }
    
    /// Set private key bytes before creating the swarm
    pub async fn set_private_key(&mut self, private_key_base64: String) -> Result<(), MessagingError> {
        // Decode base64 private key
        let private_key_bytes = base64::decode(private_key_base64)
            .map_err(|_| MessagingError::InvalidPrivateKey)?;
        
        // Create keypair from private key bytes
        let keypair = libp2p::identity::ed25519_from_bytes(&private_key_bytes[..32])
            .map_err(|_| MessagingError::InvalidPrivateKey)?;
        let keypair = Keypair::Ed25519(keypair);
        
        // Update the P2P client with new keypair
        {
            let mut p2p_client = self.p2p_client.lock().unwrap();
            p2p_client.update_keypair(keypair)
                .map_err(|_| MessagingError::KeyUpdateFailed)?;
        }
        
        Ok(())
    }
}
```

#### 2.3 Messaging Client (replacing NetworkManager)

The `gigi-messaging` library should provide a `MessagingClient` that wraps `gigi-p2p::P2pClient`:

```rust
use gigi_p2p::{P2pClient, P2pEvent, P2pError};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct MessagingClient {
    p2p_client: Arc<Mutex<P2pClient>>,
    event_sender: mpsc::UnboundedSender<MessagingEvent>,
    event_receiver: Option<mpsc::UnboundedReceiver<MessagingEvent>>,
    config: MessagingConfig,
    active_downloads: Arc<Mutex<std::collections::HashMap<String, DownloadInfo>>>,
}

#[derive(Debug, Clone)]
pub struct DownloadInfo {
    pub file_id: String,
    pub filename: String,
    pub total_size: u64,
    pub downloaded_size: u64,
    pub temp_path: PathBuf,
    pub final_path: PathBuf,
}

#[derive(Debug, Clone)]
pub enum MessagingEvent {
    // Discovery events
    PeerJoined { peer_id: String, nickname: String },
    PeerLeft { peer_id: String, nickname: String },
    NicknameChanged { peer_id: String, nickname: String },
    
    // Messaging events
    MessageReceived { from: String, content: String },
    GroupMessageReceived { from: String, group: String, content: String },
    ImageReceived { from: String, filename: String, data: Vec<u8> },
    
    // File sharing events with enhanced progress
    FileShared { file_id: String, filename: String, share_code: String },
    FileRevoked { file_id: String },
    FileTransferStarted { file_id: String, filename: String, total_size: u64 },
    FileTransferProgress { file_id: String, downloaded_size: u64, total_size: u64, speed: f64 },
    FileTransferCompleted { file_id: String, filename: String, final_path: PathBuf },
    FileTransferFailed { file_id: String, error: String },
    
    // Configuration events
    ConfigurationUpdated { field: String, value: serde_json::Value },
    
    // Error events
    Error { message: String },
}
```

#### 2.2 Command Interface

Provide a high-level command interface that maps to gigi-p2p operations:

```rust
impl MessagingClient {
    // Lifecycle
    pub async fn new(nickname: String) -> Result<Self, MessagingError> {
        Self::with_config(nickname, MessagingConfig::default()).await
    }
    
    pub async fn with_config(nickname: String, config: MessagingConfig) -> Result<Self, MessagingError>;
    pub async fn from_private_key(
        private_key_base64: String,
        nickname: String,
        config: MessagingConfig,
    ) -> Result<Self, MessagingError>;
    pub async fn start(&mut self) -> Result<(), MessagingError>;
    pub async fn shutdown(&mut self) -> Result<(), MessagingError>;
    
    // Discovery & Identity
    pub async fn set_nickname(&mut self, nickname: String) -> Result<(), MessagingError>;
    pub fn get_nickname(&self) -> String;
    pub fn get_peer_id(&self) -> String;
    pub fn get_public_key(&self) -> String; // Return base64 encoded public key
    pub async fn get_connected_peers(&self) -> Result<Vec<PeerInfo>, MessagingError>;
    
    // Direct Messaging
    pub async fn send_message(&self, to_peer: String, message: String) -> Result<(), MessagingError>;
    pub async fn send_image(&self, to_peer: String, image_data: Vec<u8>, filename: String) -> Result<(), MessagingError>;
    
    // Group Messaging  
    pub async fn join_group(&mut self, group_name: String) -> Result<(), MessagingError>;
    pub async fn leave_group(&mut self, group_name: String) -> Result<(), MessagingError>;
    pub async fn send_group_message(&self, group: String, message: String) -> Result<(), MessagingError>;
    pub async fn send_group_image(&self, group: String, image_data: Vec<u8>, filename: String) -> Result<(), MessagingError>;
    
    // File Sharing with enhanced features
    pub async fn share_file(&mut self, file_path: String) -> Result<String, MessagingError>; // returns share_code
    pub async fn request_file(&self, share_code: String) -> Result<String, MessagingError>; // returns download_id
    pub async fn cancel_download(&self, download_id: String) -> Result<(), MessagingError>;
    pub async fn unshare_file(&mut self, share_code: String) -> Result<(), MessagingError>;
    pub async fn get_shared_files(&self) -> Result<Vec<SharedFileInfo>, MessagingError>;
    pub async fn get_active_downloads(&self) -> Result<Vec<DownloadInfo>, MessagingError>;
    
    // Configuration Management
    pub async fn update_config(&mut self, updates: MessagingConfigUpdates) -> Result<(), MessagingError>;
    pub fn get_config(&self) -> &MessagingConfig;
    
    // Event Handling
    pub async fn next_event(&mut self) -> Option<MessagingEvent>;
    pub fn try_event(&mut self) -> Option<MessagingEvent>;
    pub fn subscribe_events(&self) -> mpsc::UnboundedReceiver<MessagingEvent>;
    
    // Key Management
    pub static fn generate_keypair() -> Result<KeyPair, MessagingError>;
    pub async fn export_private_key(&self) -> Result<String, MessagingError>; // Base64 encoded
    pub fn get_peer_id_from_public_key(public_key_base64: &str) -> Result<String, MessagingError>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessagingConfigUpdates {
    pub share_json_path: Option<PathBuf>,
    pub downloads_dir: Option<PathBuf>,
    pub temp_dir: Option<PathBuf>,
    pub max_file_size: Option<u64>,
    pub chunk_size: Option<usize>,
}

// Enhanced file transfer with progress tracking
impl MessagingClient {
    async fn handle_file_download(&self, share_code: String) -> Result<String, MessagingError> {
        // Start download and return download_id for tracking
        let download_id = uuid::Uuid::new_v4().to_string();
        
        // Get file info from share.json
        let file_info = self.get_file_info_from_share(&share_code).await?;
        
        // Create download info
        let temp_path = self.config.temp_dir.join(format!("{}.tmp", download_id));
        let final_path = self.config.downloads_dir.join(&file_info.filename);
        
        let download_info = DownloadInfo {
            file_id: download_id.clone(),
            filename: file_info.filename,
            total_size: file_info.size,
            downloaded_size: 0,
            temp_path: temp_path.clone(),
            final_path: final_path.clone(),
        };
        
        // Store download info
        self.active_downloads.lock().unwrap().insert(download_id.clone(), download_info);
        
        // Emit download started event
        let _ = self.event_sender.send(MessagingEvent::FileTransferStarted {
            file_id: download_id.clone(),
            filename: file_info.filename,
            total_size: file_info.size,
        });
        
        // Start actual download in background
        let client = self.p2p_client.clone();
        let event_sender = self.event_sender.clone();
        let active_downloads = self.active_downloads.clone();
        
        tokio::spawn(async move {
            match Self::perform_download(client, share_code, temp_path, download_id.clone()).await {
                Ok(_) => {
                    // Move temp file to final location
                    if let Err(e) = tokio::fs::rename(&temp_path, &final_path).await {
                        let _ = event_sender.send(MessagingEvent::FileTransferFailed {
                            file_id: download_id,
                            error: format!("Failed to move file: {}", e),
                        });
                        return;
                    }
                    
                    // Remove from active downloads
                    active_downloads.lock().unwrap().remove(&download_id);
                    
                    // Emit completion event
                    let _ = event_sender.send(MessagingEvent::FileTransferCompleted {
                        file_id: download_id,
                        filename: file_info.filename,
                        final_path,
                    });
                }
                Err(e) => {
                    // Clean up temp file
                    let _ = tokio::fs::remove_file(&temp_path).await;
                    active_downloads.lock().unwrap().remove(&download_id);
                    
                    // Emit error event
                    let _ = event_sender.send(MessagingEvent::FileTransferFailed {
                        file_id: download_id,
                        error: e.to_string(),
                    });
                }
            }
        });
        
        Ok(download_id)
    }
    
    async fn perform_download(
        client: Arc<Mutex<P2pClient>>,
        share_code: String,
        temp_path: PathBuf,
        download_id: String,
    ) -> Result<(), MessagingError> {
        // Implementation of chunked download with progress tracking
        let start_time = std::time::Instant::now();
        
        // ... detailed download implementation with progress reporting
        
        Ok(())
    }
}
```

#### 2.3 Data Models

Define mobile-friendly data models:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub nickname: String,
    pub address: Option<String>,
    pub last_seen: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedFileInfo {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub share_code: String,
    pub created_at: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum MessagingError {
    #[error("P2P error: {0}")]
    P2pError(#[from] P2pError),
    
    #[error("Client not initialized")]
    NotInitialized,
    
    #[error("Invalid peer ID: {0}")]
    InvalidPeerId(String),
    
    #[error("Invalid group name: {0}")]
    InvalidGroupName(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Event channel closed")]
    EventChannelClosed,
    
    #[error("Invalid private key format")]
    InvalidPrivateKey,
    
    #[error("Key generation failed")]
    KeyGenerationFailed,
    
    #[error("Key update failed")]
    KeyUpdateFailed,
}
```

### 3. Event Loop Integration

The messaging client should run an internal event loop that:

1. Polls the `gigi-p2p` client for events
2. Translates `P2pEvent` to `MessagingEvent`
3. Forwards events to registered listeners

```rust
impl MessagingClient {
    async fn run_event_loop(&mut self) {
        let mut event_receiver = self.p2p_client.lock().unwrap().get_event_receiver();
        
        loop {
            match event_receiver.recv().await {
                Some(P2pEvent::PeerDiscovered { peer_id, nickname, address }) => {
                    let event = MessagingEvent::PeerJoined {
                        peer_id: peer_id.to_string(),
                        nickname,
                    };
                    let _ = self.event_sender.send(event);
                }
                
                Some(P2pEvent::DirectMessage { from, from_nickname, message }) => {
                    let event = MessagingEvent::MessageReceived {
                        from: from.to_string(),
                        content: message,
                    };
                    let _ = self.event_sender.send(event);
                }
                
                // ... other event mappings
                
                None => break, // Channel closed
            }
        }
    }
}
```

### 4. Mobile Integration Points

#### 4.1 Tauri Commands

Based on the Tauri pattern, commands are simple functions with the `#[tauri::command]` attribute:

```rust
use tauri::{State, Manager};
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

// Inner state structure (without the Mutex)
#[derive(Default)]
struct MessagingStateInner {
    client: Option<MessagingClient>,
    event_sender: Option<mpsc::UnboundedSender<MessagingEvent>>,
    config: MessagingConfig,
}

// Type alias for the managed state
type MessagingState = Mutex<MessagingStateInner>;

// Commands follow the Tauri pattern with proper state management
// Note: Commands in lib.rs cannot be marked as pub due to Tauri limitations

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
    let (event_sender, _event_receiver) = mpsc::unbounded_channel::<MessagingEvent>();
    
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
    
    // Start event loop in a background thread and emit events to frontend
    let app_handle = app.clone();
    tokio::spawn(async move {
        let state = app_handle.state::<MessagingState>();
        let mut client_guard = state.lock().unwrap();
        if let Some(client) = client_guard.client.as_mut() {
            while let Some(event) = client.next_event().await {
                // Emit events to frontend using Tauri's event system
                match event {
                    MessagingEvent::PeerJoined { peer_id, nickname } => {
                        let _ = app_handle.emit("peer-joined", serde_json::json!({
                            "peerId": peer_id,
                            "nickname": nickname
                        }));
                    }
                    MessagingEvent::MessageReceived { from, content } => {
                        let _ = app_handle.emit("message-received", serde_json::json!({
                            "from": from,
                            "content": content
                        }));
                    }
                    MessagingEvent::GroupMessageReceived { from, group, content } => {
                        let _ = app_handle.emit("group-message-received", serde_json::json!({
                            "from": from,
                            "group": group,
                            "content": content
                        }));
                    }
                    MessagingEvent::FileTransferStarted { file_id, filename, total_size } => {
                        let _ = app_handle.emit("file-transfer-started", serde_json::json!({
                            "fileId": file_id,
                            "filename": filename,
                            "totalSize": total_size
                        }));
                    }
                    MessagingEvent::FileTransferProgress { file_id, downloaded_size, total_size, speed } => {
                        let _ = app_handle.emit("file-transfer-progress", serde_json::json!({
                            "fileId": file_id,
                            "downloadedSize": downloaded_size,
                            "totalSize": total_size,
                            "speed": speed
                        }));
                    }
                    MessagingEvent::FileTransferCompleted { file_id, filename, final_path } => {
                        let _ = app_handle.emit("file-transfer-completed", serde_json::json!({
                            "fileId": file_id,
                            "filename": filename,
                            "finalPath": final_path
                        }));
                    }
                    MessagingEvent::FileTransferFailed { file_id, error } => {
                        let _ = app_handle.emit("file-transfer-failed", serde_json::json!({
                            "fileId": file_id,
                            "error": error
                        }));
                    }
                    MessagingEvent::ConfigurationUpdated { field, value } => {
                        let _ = app_handle.emit("config-updated", serde_json::json!({
                            "field": field,
                            "value": value
                        }));
                    }
                    MessagingEvent::Error { message } => {
                        let _ = app_handle.emit("messaging-error", serde_json::json!({
                            "message": message
                        }));
                    }
                    _ => {
                        let _ = app_handle.emit("messaging-event", serde_json::json!({
                            "event": format!("{:?}", event)
                        }));
                    }
                }
            }
        }
    });
    
    Ok(())
}

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
    let (event_sender, _event_receiver) = mpsc::unbounded_channel::<MessagingEvent>();
    
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
    
    // Start event loop (same as above)
    // ... event loop implementation ...
    
    Ok(())
}

#[tauri::command]
fn messaging_generate_keypair() -> Result<KeyPair, String> {
    MessagingClient::generate_keypair()
        .map_err(|e| e.to_string())
}

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

#[tauri::command]
fn messaging_get_config(
    state: tauri::State<'_, MessagingState>,
) -> Result<MessagingConfig, String> {
    let state = state.lock().unwrap();
    Ok(state.config.clone())
}

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

#[tauri::command]
fn messaging_get_active_downloads(
    state: tauri::State<'_, MessagingState>,
) -> Result<Vec<DownloadInfo>, String> {
    let state = state.lock().unwrap();
    if let Some(client) = state.client.as_ref() {
        // This would need to be implemented in MessagingClient
        client.get_active_downloads().await
            .map_err(|e| e.to_string())
    } else {
        Ok(vec![])
    }
}

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
```

#### 4.2 Mobile Integration

For mobile integration, provide async API that can be consumed by mobile frameworks:

```rust
// Mobile-friendly async API
impl MessagingClient {
    pub async fn create_mobile_client(nickname: String) -> Result<Arc<Mutex<MessagingClient>>, MessagingError> {
        // Create client optimized for mobile usage
    }
    
    pub async fn start_background_tasks(&self) -> Result<(), MessagingError> {
        // Start event polling and background tasks for mobile
    }
    
    pub fn get_event_stream(&self) -> mpsc::UnboundedReceiver<MessagingEvent> {
        // Return event stream for mobile UI to consume
    }
}

## Implementation Steps

### Phase 1: Core Integration

1. **Update Dependencies**: Add `gigi-p2p` dependency to `gigi-messaging`
2. **Replace NetworkManager**: Implement `MessagingClient` wrapper around `P2pClient`
3. **Event Translation**: Map `P2pEvent` variants to `MessagingEvent` variants
4. **Command Mapping**: Implement all command methods by delegating to `P2pClient`

### Phase 2: Mobile-Specific Features

1. **Async/Await Support**: Ensure all APIs work with mobile async patterns
2. **Error Handling**: Implement comprehensive error mapping and handling
3. **State Management**: Add thread-safe state management for mobile contexts
4. **Event Broadcasting**: Implement efficient event distribution to mobile UI

### Phase 3: Platform Integration

1. **Tauri Commands**: Add Tauri command handlers if using Tauri mobile
2. **Mobile Async API**: Implement mobile-optimized async interface
3. **Testing**: Add comprehensive tests for the integration layer
4. **Documentation**: Provide mobile integration guides

## Usage Examples

### Rust Usage

```rust
use gigi_messaging::{MessagingClient, MessagingEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MessagingClient::new("MyNickname".to_string()).await?;
    client.start().await?;
    
    // Join a group
    client.join_group("general".to_string()).await?;
    
    // Send a message
    client.send_group_message("general".to_string(), "Hello world!".to_string()).await?;
    
    // Listen for events
    while let Some(event) = client.next_event().await {
        match event {
            MessagingEvent::GroupMessageReceived { from, group, content } => {
                println!("{} in {}: {}", from, group, content);
            }
            MessagingEvent::PeerJoined { peer_id, nickname } => {
                println!("{} joined", nickname);
            }
            _ => {}
        }
    }
    
    Ok(())
}
```

### Mobile Integration Usage

```rust
// In mobile app Rust layer
use gigi_messaging::{MessagingClient, MessagingEvent};

async fn initialize_messaging(nickname: String) -> Result<mpsc::UnboundedReceiver<MessagingEvent>, MessagingError> {
    let mut client = MessagingClient::new(nickname).await?;
    client.start().await?;
    
    // Start background tasks for mobile
    client.start_background_tasks().await?;
    
    // Get event stream for UI updates
    let event_stream = client.subscribe_events();
    
    Ok(event_stream)
}

// In UI layer (pseudocode)
let mut event_stream = initialize_messaging("MyNickname".await?;

// Handle events in UI loop
while let Some(event) = event_stream.recv().await {
    match event {
        MessagingEvent::GroupMessageReceived { from, group, content } => {
            ui_update_chat_message(&from, &content);
        }
        MessagingEvent::PeerJoined { peer_id, nickname } => {
            ui_add_peer(&nickname);
        }
        MessagingEvent::FileTransferProgress { file_id, progress } => {
            ui_update_file_progress(&file_id, progress);
        }
        _ => {}
    }
}
```

### Tauri Mobile Usage

```javascript
// In mobile app JavaScript/TypeScript
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// Option 1: Initialize with existing private key
await invoke('messaging_initialize_with_key', {
  privateKey: 'base64-encoded-private-key-here',
  nickname: 'MyMobileNickname'
});

// Option 2: Initialize first, then set private key before creating swarm
await invoke('messaging_initialize', {
  nickname: 'MyMobileNickname'
});

// Set private key from frontend before network operations
await invoke('messaging_set_private_key', {
  privateKey: 'base64-encoded-private-key-here'
});

// Option 3: Generate new keypair and use it
const keypair = await invoke('messaging_generate_keypair');
console.log('Generated keypair:', keypair);

await invoke('messaging_initialize_with_key', {
  privateKey: keypair.privateKey,
  nickname: 'MyMobileNickname'
});

// Send a direct message
await invoke('messaging_send_message', {
  to: 'peer-id-here',
  message: 'Hello from mobile!'
});

// Get connected peers
const peers = await invoke('messaging_get_peers');
console.log('Connected peers:', peers);

// Set nickname
await invoke('messaging_set_nickname', {
  nickname: 'MyMobileNickname'
});

// Join a group
await invoke('messaging_join_group', {
  groupName: 'general'
});

// Send group message
await invoke('messaging_send_group_message', {
  group: 'general',
  message: 'Hello everyone!'
});

// Share a file
const shareCode = await invoke('messaging_share_file', {
  filePath: '/path/to/file.txt'
});

// Get shared files
const sharedFiles = await invoke('messaging_get_shared_files');

// Get current peer ID
const peerId = await invoke('messaging_get_peer_id');
console.log('My peer ID:', peerId);

// Listen for real-time events
const unlistenPeerJoined = await listen('peer-joined', (event) => {
  console.log('New peer joined:', event.payload);
});

const unlistenMessageReceived = await listen('message-received', (event) => {
  console.log('Message received:', event.payload);
});

const unlistenGroupMessage = await listen('group-message-received', (event) => {
  console.log('Group message received:', event.payload);
});

const unlistenFileProgress = await listen('file-progress', (event) => {
  console.log('File transfer progress:', event.payload);
});

// Clean up event listeners when component unmounts
// unlistenPeerJoined();
// unlistenMessageReceived();
// unlistenGroupMessage();
// unlistenFileProgress();
```

### Advanced Usage: Custom Error Types

```rust
// Define custom error type with thiserror
#[derive(Debug, thiserror::Error)]
pub enum MessagingError {
    #[error("P2P error: {0}")]
    P2pError(#[from] gigi_p2p::P2pError),
    
    #[error("Client not initialized")]
    NotInitialized,
    
    #[error("Invalid peer ID: {0}")]
    InvalidPeerId(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
}

// Implement serde::Serialize for error handling
impl serde::Serialize for MessagingError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum ErrorKind {
    P2pError(String),
    NotInitialized,
    InvalidPeerId(String),
    FileNotFound(String),
    NetworkError(String),
}

impl serde::Serialize for MessagingError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let error_kind = match self {
            Self::P2pError(e) => ErrorKind::P2pError(e.to_string()),
            Self::NotInitialized => ErrorKind::NotInitialized,
            Self::InvalidPeerId(id) => ErrorKind::InvalidPeerId(id.clone()),
            Self::FileNotFound(path) => ErrorKind::FileNotFound(path.clone()),
            Self::NetworkError(msg) => ErrorKind::NetworkError(msg.clone()),
        };
        error_kind.serialize(serializer)
    }
}

// Command with proper error handling
#[tauri::command]
async fn messaging_send_message_with_errors(
    state: tauri::State<'_, MessagingState>,
    to: String,
    message: String,
) -> Result<(), MessagingError> {
    let mut state = state.lock().unwrap();
    if let Some(client) = state.client.as_mut() {
        client.send_message(to, message).await
            .map_err(MessagingError::P2pError)
    } else {
        Err(MessagingError::NotInitialized)
    }
}
```

### TypeScript Frontend with Error Handling

```typescript
// Define TypeScript types matching Rust types
type Peer = {
  id: string;
  nickname: string;
  address?: string;
  last_seen?: number;
};

type SharedFileInfo = {
  id: string;
  name: string;
  size: number;
  shareCode: string;
  createdAt: number;
};

type ErrorKind = {
  kind: 'p2pError' | 'notInitialized' | 'invalidPeerId' | 'fileNotFound' | 'networkError';
  message?: string;
};

type MessageReceivedEvent = {
  from: string;
  content: string;
};

type GroupMessageReceivedEvent = {
  from: string;
  group: string;
  content: string;
};

type FileProgressEvent = {
  fileId: string;
  progress: number;
};

class MessagingService {
  async initialize(nickname: string): Promise<void> {
    await invoke('messaging_initialize', { nickname });
  }

  async sendMessage(to: string, message: string): Promise<void> {
    try {
      await invoke('messaging_send_message', { to, message });
    } catch (error) {
      const typedError = error as ErrorKind;
      if (typedError.kind === 'notInitialized') {
        console.error('Please initialize the messaging service first');
      } else if (typedError.kind === 'invalidPeerId') {
        console.error('Invalid peer ID:', typedError.message);
      } else {
        console.error('Failed to send message:', typedError);
      }
      throw error;
    }
  }

  async getPeers(): Promise<Peer[]> {
    return await invoke<Peer[]>('messaging_get_peers');
  }

  onMessageReceived(callback: (event: MessageReceivedEvent) => void): () => void {
    return this.setupEventListener('message-received', callback);
  }

  onGroupMessageReceived(callback: (event: GroupMessageReceivedEvent) => void): () => void {
    return this.setupEventListener('group-message-received', callback);
  }

  onFileProgress(callback: (event: FileProgressEvent) => void): () => void {
    return this.setupEventListener('file-progress', callback);
  }

  private async setupEventListener<T>(
    eventName: string,
    callback: (event: T) => void
  ): Promise<() => void> {
    const unlisten = await listen<T>(eventName, (event) => {
      callback(event.payload);
    });
    return unlisten;
  }
}

// Usage example
const messagingService = new MessagingService();

try {
  await messagingService.initialize('MyNickname');
  
  const peers = await messagingService.getPeers();
  console.log('Connected peers:', peers);

  // Set up event listeners
  const unlistenMessages = messagingService.onMessageReceived((event) => {
    console.log('New message:', event);
  });

  // Send a message
  await messagingService.sendMessage('peer-id', 'Hello!');

} catch (error) {
  console.error('Messaging error:', error);
}
```

### React Component Example with Enhanced Features

```tsx
import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface Peer {
  id: string;
  nickname: string;
  address?: string;
  last_seen?: number;
}

interface DownloadInfo {
  fileId: string;
  filename: string;
  totalSize: number;
  downloadedSize: number;
  speed: number;
}

interface MessagingConfig {
  shareJsonPath: string;
  downloadsDir: string;
  tempDir: string;
  maxFileSize: number;
  chunkSize: number;
}

interface KeyPair {
  privateKey: string;
  publicKey: string;
}

function MessagingApp() {
  const [peers, setPeers] = useState<Peer[]>([]);
  const [message, setMessage] = useState('');
  const [messages, setMessages] = useState<string[]>([]);
  const [isInitialized, setIsInitialized] = useState(false);
  const [nickname, setNickname] = useState('MyNickname');
  const [privateKey, setPrivateKey] = useState<string>('');
  const [publicKey, setPublicKey] = useState<string>('');
  const [downloads, setDownloads] = useState<DownloadInfo[]>([]);
  const [config, setConfig] = useState<MessagingConfig | null>(null);
  const [shareCode, setShareCode] = useState('');
  const [downloadId, setDownloadId] = useState('');
  
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Event listener cleanup functions
  const cleanupFunctions = useRef<(() => void)[]>([]);

  // Initialize the messaging client on component mount
  useEffect(() => {
    const initialize = async () => {
      try {
        // Check if we have a stored private key
        const storedKey = localStorage.getItem('gigi_private_key');
        
        if (storedKey) {
          // Initialize with existing private key
          await invoke('messaging_initialize_with_key', { 
            privateKey: storedKey, 
            nickname 
          });
          setPrivateKey(storedKey);
        } else {
          // Generate new keypair and initialize
          const keypair = await invoke<KeyPair>('messaging_generate_keypair');
          await invoke('messaging_initialize_with_key', { 
            privateKey: keypair.privateKey, 
            nickname 
          });
          setPrivateKey(keypair.privateKey);
          setPublicKey(keypair.publicKey);
          localStorage.setItem('gigi_private_key', keypair.privateKey);
        }
        
        setIsInitialized(true);
        loadPeers();
        loadConfig();
        loadActiveDownloads();
        setupEventListeners();
      } catch (error) {
        console.error('Failed to initialize messaging:', error);
      }
    };
    
    initialize();
    
    // Cleanup event listeners on unmount
    return () => {
      cleanupFunctions.current.forEach(cleanup => cleanup());
    };
  }, []);

  const setupEventListeners = async () => {
    const unlistenPeerJoined = await listen('peer-joined', (event) => {
      console.log('New peer joined:', event.payload);
      loadPeers(); // Refresh peers list
    });

    const unlistenMessageReceived = await listen('message-received', (event) => {
      const { from, content } = event.payload as { from: string; content: string };
      setMessages(prev => [...prev, `${from}: ${content}`]);
    });

    const unlistenGroupMessage = await listen('group-message-received', (event) => {
      const { from, group, content } = event.payload as { from: string; group: string; content: string };
      setMessages(prev => [...prev, `${from} in ${group}: ${content}`]);
    });

    const unlistenFileTransferStarted = await listen('file-transfer-started', (event) => {
      const { fileId, filename, totalSize } = event.payload as { 
        fileId: string; 
        filename: string; 
        totalSize: number; 
      };
      setMessages(prev => [...prev, `Download started: ${filename} (${(totalSize / 1024 / 1024).toFixed(2)} MB)`]);
      loadActiveDownloads();
    });

    const unlistenFileTransferProgress = await listen('file-transfer-progress', (event) => {
      const { fileId, downloadedSize, totalSize, speed } = event.payload as { 
        fileId: string; 
        downloadedSize: number; 
        totalSize: number; 
        speed: number; 
      };
      setDownloads(prev => 
        prev.map(d => 
          d.fileId === fileId 
            ? { ...d, downloadedSize, speed }
            : d
        )
      );
    });

    const unlistenFileTransferCompleted = await listen('file-transfer-completed', (event) => {
      const { fileId, filename, finalPath } = event.payload as { 
        fileId: string; 
        filename: string; 
        finalPath: string; 
      };
      setMessages(prev => [...prev, `Download completed: ${filename} -> ${finalPath}`]);
      setDownloads(prev => prev.filter(d => d.fileId !== fileId));
    });

    const unlistenFileTransferFailed = await listen('file-transfer-failed', (event) => {
      const { fileId, error } = event.payload as { fileId: string; error: string };
      setMessages(prev => [...prev, `Download failed: ${error}`]);
      setDownloads(prev => prev.filter(d => d.fileId !== fileId));
    });

    const unlistenConfigUpdated = await listen('config-updated', (event) => {
      const { field, value } = event.payload as { field: string; value: any };
      setMessages(prev => [...prev, `Config updated: ${field} = ${value}`]);
      loadConfig();
    });

    const unlistenMessagingError = await listen('messaging-error', (event) => {
      const { message } = event.payload as { message: string };
      setMessages(prev => [...prev, `Error: ${message}`]);
    });

    // Store cleanup functions
    cleanupFunctions.current = [
      unlistenPeerJoined,
      unlistenMessageReceived,
      unlistenGroupMessage,
      unlistenFileTransferStarted,
      unlistenFileTransferProgress,
      unlistenFileTransferCompleted,
      unlistenFileTransferFailed,
      unlistenConfigUpdated,
      unlistenMessagingError,
    ];
  };

  const loadPeers = async () => {
    try {
      const peerList = await invoke<Peer[]>('messaging_get_peers');
      setPeers(peerList);
    } catch (error) {
      console.error('Failed to load peers:', error);
    }
  };

  const loadConfig = async () => {
    try {
      const currentConfig = await invoke<MessagingConfig>('messaging_get_config');
      setConfig(currentConfig);
    } catch (error) {
      console.error('Failed to load config:', error);
    }
  };

  const loadActiveDownloads = async () => {
    try {
      const activeDownloads = await invoke<DownloadInfo[]>('messaging_get_active_downloads');
      setDownloads(activeDownloads);
    } catch (error) {
      console.error('Failed to load active downloads:', error);
    }
  };

  const sendMessage = async (toPeer: string, content: string) => {
    try {
      await invoke('messaging_send_message', {
        to: toPeer,
        message: content
      });
      setMessages(prev => [...prev, `Me to ${toPeer}: ${content}`]);
    } catch (error) {
      console.error('Failed to send message:', error);
    }
  };

  const shareFile = async () => {
    const fileInput = fileInputRef.current;
    if (!fileInput?.files?.length) return;

    const file = fileInput.files[0];
    try {
      const fileShareCode = await invoke<string>('messaging_share_file', {
        filePath: file.name // Note: In real implementation, you'd need to handle file selection properly
      });
      setShareCode(fileShareCode);
      setMessages(prev => [...prev, `File shared: ${file.name} (Code: ${fileShareCode})`]);
    } catch (error) {
      console.error('Failed to share file:', error);
    }
  };

  const requestDownload = async () => {
    if (!shareCode.trim()) return;
    
    try {
      const downloadId = await invoke<string>('messaging_request_file', {
        shareCode: shareCode.trim()
      });
      setDownloadId(downloadId);
      setMessages(prev => [...prev, `Download requested: ${shareCode}`]);
      setShareCode(''); // Clear input
    } catch (error) {
      console.error('Failed to request download:', error);
    }
  };

  const cancelDownload = async (fileId: string) => {
    try {
      await invoke('messaging_cancel_download', { downloadId: fileId });
      setMessages(prev => [...prev, `Download cancelled: ${fileId}`]);
    } catch (error) {
      console.error('Failed to cancel download:', error);
    }
  };

  const updateConfig = async (updates: Partial<MessagingConfig>) => {
    try {
      await invoke('messaging_update_config', { configUpdates: updates });
      setMessages(prev => [...prev, 'Configuration updated']);
    } catch (error) {
      console.error('Failed to update config:', error);
    }
  };

  if (!isInitialized) {
    return <div>Initializing messaging...</div>;
  }

  return (
    <div>
      <h1>Gigi Messaging</h1>
      
      <div>
        <h2>Settings</h2>
        <p>Current nickname: {nickname}</p>
        <input
          type="text"
          placeholder="New nickname"
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              setNickname(e.currentTarget.value);
              e.currentTarget.value = '';
            }
          }}
        />
        
        <div>
          <p>Private Key: {privateKey.substring(0, 20)}...</p>
          <p>Public Key: {publicKey.substring(0, 20)}...</p>
        </div>
      </div>

      <div>
        <h2>Connected Peers</h2>
        <ul>
          {peers.map(peer => (
            <li key={peer.id}>
              {peer.nickname} ({peer.id})
              <button onClick={() => sendMessage(peer.id, message)}>
                Send Message
              </button>
            </li>
          ))}
        </ul>
        <button onClick={() => loadPeers()}>Refresh Peers</button>
      </div>

      <div>
        <h2>File Sharing</h2>
        <input 
          type="file" 
          ref={fileInputRef}
          onChange={shareFile}
        />
        
        <input
          type="text"
          placeholder="Share code"
          value={shareCode}
          onChange={(e) => setShareCode(e.target.value)}
        />
        <button onClick={requestDownload}>Download</button>
      </div>

      <div>
        <h2>Active Downloads</h2>
        {downloads.map(download => (
          <div key={download.fileId}>
            <p>{download.filename}</p>
            <progress 
              value={download.downloadedSize} 
              max={download.totalSize}
            />
            <p>{((download.downloadedSize / download.totalSize) * 100).toFixed(1)}% - {download.speed.toFixed(2)} KB/s</p>
            <button onClick={() => cancelDownload(download.fileId)}>Cancel</button>
          </div>
        ))}
      </div>

      <div>
        <h2>Configuration</h2>
        {config && (
          <div>
            <p>Downloads Dir: {config.downloadsDir}</p>
            <p>Share JSON: {config.shareJsonPath}</p>
            <p>Max File Size: {(config.maxFileSize / 1024 / 1024).toFixed(2)} MB</p>
          </div>
        )}
        <button onClick={() => updateConfig({ maxFileSize: 200 * 1024 * 1024 })}>
          Increase max file size to 200MB
        </button>
      </div>

      <div>
        <h2>Messages</h2>
        <ul>
          {messages.map((msg, idx) => (
            <li key={idx}>{msg}</li>
          ))}
        </ul>
      </div>
    </div>
  );
}
```

## Error Handling Strategy

1. **Graceful Degradation**: Network failures should not crash the mobile app
2. **User-Friendly Messages**: Translate technical errors to user-friendly messages
3. **Retry Logic**: Implement automatic retry for transient network failures
4. **Offline Support**: Cache messages and retry when connection is restored

## Performance Considerations

1. **Async Operations**: All I/O operations should be non-blocking
2. **Memory Management**: Efficient handling of large file transfers and image data
3. **Battery Usage**: Optimize discovery and background operations
4. **Data Compression**: Use compression for large messages and file transfers

## Security Considerations

1. **Data Validation**: Validate all incoming data from peers
2. **File Access**: Implement proper file access controls and sandboxing
3. **Privacy**: Handle user nicknames and personal information appropriately
4. **Network Security**: Use secure protocols and encryption where applicable

## Testing Strategy

1. **Unit Tests**: Test command mapping and event translation
2. **Integration Tests**: Test end-to-end scenarios with multiple clients
3. **Mobile Testing**: Test on actual mobile devices and emulators
4. **Stress Testing**: Test with large files and many concurrent connections

## Migration Path

The existing `gigi-messaging` codebase should be refactored:

1. **Keep existing Tauri plugin structure** for backward compatibility
2. **Add new `MessagingClient`** alongside existing code
3. **Gradually migrate** from placeholder implementations to real gigi-p2p integration
4. **Maintain API compatibility** where possible during transition

## Future Enhancements

1. **Push Notifications**: Mobile push notifications for new messages
2. **Offline Sync**: Synchronization when devices come back online
3. **Group Management**: Advanced group creation and management features
4. **Media Support**: Audio/video messaging capabilities
5. **End-to-End Encryption**: Additional security layer for sensitive communications