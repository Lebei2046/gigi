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

#[derive(Debug, Serialize, Deserialize)]
pub struct MessagingConfigUpdates {
    pub share_json_path: Option<PathBuf>,
    pub downloads_dir: Option<PathBuf>,
    pub temp_dir: Option<PathBuf>,
    pub max_file_size: Option<u64>,
    pub chunk_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    #[serde(with = "base64_serde")]
    pub private_key: Vec<u8>,
    #[serde(with = "base64_serde")]
    pub public_key: Vec<u8>,
}

impl KeyPair {
    pub fn new(private_key: Vec<u8>, public_key: Vec<u8>) -> Self {
        Self {
            private_key,
            public_key,
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub filename: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSharedInfo {
    pub filename: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub file_id: String,
    pub filename: String,
    pub total_size: u64,
    pub downloaded_size: u64,
    pub temp_path: PathBuf,
    pub final_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub struct MessagingClient {
    pub p2p_client: std::sync::Arc<std::sync::Mutex<gigi_p2p::P2pClient>>,
    pub event_sender: tokio::sync::mpsc::UnboundedSender<MessagingEvent>,
    pub event_receiver: Option<tokio::sync::mpsc::UnboundedReceiver<MessagingEvent>>,
    pub config: MessagingConfig,
    pub active_downloads: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, DownloadInfo>>>,
}