use gigi_p2p::FileInfo as P2pFileInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::{Mutex, RwLock};

/// File information structure
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
            peer_id: String::new(),
        }
    }
}

/// Download progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub download_id: String,
    pub progress: f32,
    pub speed: u64,
}

/// Application configuration
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
            download_folder: String::new(),
            max_concurrent_downloads: 3,
            port: 0,
        }
    }
}

/// Peer information
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

/// Direct message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from_peer_id: String,
    pub from_nickname: String,
    pub content: String,
    pub timestamp: u64,
}

/// Group message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    pub id: String,
    pub group_id: String,
    pub from_peer_id: String,
    pub from_nickname: String,
    pub content: String,
    pub timestamp: u64,
}

/// Plugin state
#[derive(Clone)]
pub struct PluginState {
    pub p2p_client: std::sync::Arc<Mutex<Option<gigi_p2p::P2pClient>>>,
    pub event_receiver: std::sync::Arc<
        Mutex<Option<futures::channel::mpsc::UnboundedReceiver<gigi_p2p::P2pEvent>>>,
    >,
    pub config: std::sync::Arc<RwLock<Config>>,
    pub active_downloads: std::sync::Arc<Mutex<HashMap<String, DownloadProgress>>>,
    pub message_store: std::sync::Arc<RwLock<Option<gigi_store::MessageStore>>>,
    pub file_sharing_store: std::sync::Arc<RwLock<Option<gigi_store::FileSharingStore>>>,
    pub thumbnail_store: std::sync::Arc<RwLock<Option<gigi_store::ThumbnailStore>>>,
    pub conversation_store: std::sync::Arc<RwLock<Option<gigi_store::ConversationStore>>>,
    pub auth_manager: std::sync::Arc<Mutex<Option<gigi_store::AuthManager>>>,
    pub group_manager: std::sync::Arc<Mutex<Option<gigi_store::GroupManager>>>,
    pub contact_manager: std::sync::Arc<Mutex<Option<gigi_store::ContactManager>>>,
    pub db_connection: std::sync::Arc<RwLock<Option<sea_orm::DatabaseConnection>>>,
    pub initialized: std::sync::Arc<tokio::sync::Notify>,
}

impl PluginState {
    pub fn new() -> Self {
        Self {
            p2p_client: std::sync::Arc::new(Mutex::new(None)),
            event_receiver: std::sync::Arc::new(Mutex::new(None)),
            config: std::sync::Arc::new(RwLock::new(Config::default())),
            active_downloads: std::sync::Arc::new(Mutex::new(HashMap::new())),
            message_store: std::sync::Arc::new(RwLock::new(None)),
            file_sharing_store: std::sync::Arc::new(RwLock::new(None)),
            thumbnail_store: std::sync::Arc::new(RwLock::new(None)),
            conversation_store: std::sync::Arc::new(RwLock::new(None)),
            auth_manager: std::sync::Arc::new(Mutex::new(None)),
            group_manager: std::sync::Arc::new(Mutex::new(None)),
            contact_manager: std::sync::Arc::new(Mutex::new(None)),
            db_connection: std::sync::Arc::new(RwLock::new(None)),
            initialized: std::sync::Arc::new(tokio::sync::Notify::new()),
        }
    }
}

impl Default for PluginState {
    fn default() -> Self {
        Self::new()
    }
}

/// File send target enum
pub enum FileSendTarget<'a> {
    Direct(&'a str),
    Group(&'a str),
}
