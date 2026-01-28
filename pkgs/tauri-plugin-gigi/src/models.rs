//! Data models and state management for the Gigi Tauri plugin.
//!
//! This module defines all the data structures used throughout the plugin,
//! including file information, peer information, messages, configuration,
//! and the global plugin state.

use gigi_p2p::FileInfo as P2pFileInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::{Mutex, RwLock};

/// File information structure.
///
/// This struct represents metadata about a shared file, including its unique
/// identifier, name, size, MIME type, and the peer ID of the sharing peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// Unique identifier for the file
    pub id: String,
    /// Name of the file
    pub name: String,
    /// Size of the file in bytes
    pub size: u64,
    /// MIME type of the file
    pub mime_type: String,
    /// Peer ID of the peer sharing this file
    pub peer_id: String,
}

/// Conversion from P2P FileInfo to Plugin FileInfo.
///
/// This implementation converts the internal P2P file info to the plugin's
/// public API file info, automatically detecting the MIME type from the
/// filename.
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

/// Download progress tracking.
///
/// This struct tracks the progress of an active file download, including the
/// unique download ID, completion percentage, and download speed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// Unique identifier for this download
    pub download_id: String,
    /// Progress percentage (0.0 to 100.0)
    pub progress: f32,
    /// Download speed in bytes per second
    pub speed: u64,
}

/// Application configuration.
///
/// This struct contains all user-configurable settings for the plugin,
/// including the local nickname, file handling preferences, and network settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Local peer nickname
    pub nickname: String,
    /// Whether to automatically accept incoming file transfers
    pub auto_accept_files: bool,
    /// Directory path for downloaded files
    pub download_folder: String,
    /// Maximum number of concurrent downloads allowed
    pub max_concurrent_downloads: usize,
    /// Network port for P2P communication (0 for auto-selection)
    pub port: u16,
}

/// Default configuration values.
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

/// Peer information.
///
/// This struct represents a discovered peer on the P2P network, including
/// its unique identifier, nickname, and supported capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    /// Unique peer identifier
    pub id: String,
    /// Peer's display nickname
    pub nickname: String,
    /// List of capabilities the peer supports (e.g., messaging, file_transfer)
    pub capabilities: Vec<String>,
}

/// Conversion from P2P PeerInfo to Plugin Peer.
impl From<gigi_p2p::PeerInfo> for Peer {
    fn from(p2p_peer: gigi_p2p::PeerInfo) -> Self {
        Self {
            id: p2p_peer.peer_id.to_string(),
            nickname: p2p_peer.nickname,
            capabilities: vec!["messaging".to_string(), "file_transfer".to_string()],
        }
    }
}

/// Direct message structure.
///
/// This struct represents a direct (peer-to-peer) message that was sent
/// or received.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message identifier
    pub id: String,
    /// Peer ID of the message sender
    pub from_peer_id: String,
    /// Nickname of the message sender
    pub from_nickname: String,
    /// Message content (text)
    pub content: String,
    /// Unix timestamp when the message was sent
    pub timestamp: u64,
}

/// Group message structure.
///
/// This struct represents a message sent to a group chat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    /// Unique message identifier
    pub id: String,
    /// Group ID this message belongs to
    pub group_id: String,
    /// Peer ID of the message sender
    pub from_peer_id: String,
    /// Nickname of the message sender
    pub from_nickname: String,
    /// Message content (text)
    pub content: String,
    /// Unix timestamp when the message was sent
    pub timestamp: u64,
}

/// Global plugin state.
///
/// This struct contains all the shared state used throughout the plugin,
/// including the P2P client, database managers, configuration, and runtime
/// state tracking.
///
/// The state is thread-safe and can be safely accessed from multiple threads
/// using Arc, Mutex, and RwLock for synchronization.
#[derive(Clone)]
pub struct PluginState {
    /// The P2P client for peer-to-peer communication
    pub p2p_client: std::sync::Arc<Mutex<Option<gigi_p2p::P2pClient>>>,
    /// Event receiver for P2P events
    pub event_receiver: std::sync::Arc<
        Mutex<Option<futures::channel::mpsc::UnboundedReceiver<gigi_p2p::P2pEvent>>>,
    >,
    /// Application configuration
    pub config: std::sync::Arc<RwLock<Config>>,
    /// Map of active downloads and their progress
    pub active_downloads: std::sync::Arc<Mutex<HashMap<String, DownloadProgress>>>,
    /// Message store for persisting messages
    pub message_store: std::sync::Arc<RwLock<Option<gigi_store::MessageStore>>>,
    /// File sharing store for tracking shared files
    pub file_sharing_store: std::sync::Arc<RwLock<Option<gigi_store::FileSharingStore>>>,
    /// Thumbnail store for image thumbnails
    pub thumbnail_store: std::sync::Arc<RwLock<Option<gigi_store::ThumbnailStore>>>,
    /// Conversation store for managing chat conversations
    pub conversation_store: std::sync::Arc<RwLock<Option<gigi_store::ConversationStore>>>,
    /// Authentication manager for user accounts
    pub auth_manager: std::sync::Arc<Mutex<Option<gigi_store::AuthManager>>>,
    /// Group manager for group chat management
    pub group_manager: std::sync::Arc<Mutex<Option<gigi_store::GroupManager>>>,
    /// Contact manager for managing user contacts
    pub contact_manager: std::sync::Arc<Mutex<Option<gigi_store::ContactManager>>>,
    /// Database connection for persistence
    pub db_connection: std::sync::Arc<RwLock<Option<sea_orm::DatabaseConnection>>>,
    /// Notification flag for initialization completion
    pub initialized: std::sync::Arc<tokio::sync::Notify>,
}

impl PluginState {
    /// Creates a new, empty PluginState with all fields initialized to None.
    ///
    /// This is typically called during plugin initialization. The fields are
    /// populated later during the setup process.
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

/// Default implementation for PluginState.
impl Default for PluginState {
    fn default() -> Self {
        Self::new()
    }
}

/// File send target enum.
///
/// This enum represents the possible targets for sending a file, either to a
/// specific peer directly or to a group chat.
pub enum FileSendTarget<'a> {
    /// Send file directly to a peer (peer ID or nickname)
    Direct(&'a str),
    /// Send file to a group (group ID)
    Group(&'a str),
}
