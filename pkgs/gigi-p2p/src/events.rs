//! P2P events and public data structures

use chrono::{DateTime, Utc};
use libp2p::gossipsub::IdentTopic;
use libp2p::{multiaddr::Multiaddr, PeerId};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

/// File path representation - supports both filesystem paths and URIs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FilePath {
    Url(Url),      // Android content:// or iOS file:// URIs
    Path(PathBuf), // Regular filesystem paths
}

/// Unified P2P event
#[derive(Debug, Clone)]
pub enum P2pEvent {
    // Discovery events
    PeerDiscovered {
        peer_id: PeerId,
        nickname: String,
        address: Multiaddr,
    },
    PeerExpired {
        peer_id: PeerId,
        nickname: String,
    },
    NicknameUpdated {
        peer_id: PeerId,
        nickname: String,
    },

    // Direct messaging events
    DirectMessage {
        from: PeerId,
        from_nickname: String,
        message: String,
    },
    DirectFileShareMessage {
        from: PeerId,
        from_nickname: String,
        share_code: String,
        filename: String,
        file_size: u64,
        file_type: String,
    },
    DirectGroupShareMessage {
        from: PeerId,
        from_nickname: String,
        group_id: String,
        group_name: String,
    },

    // Group messaging events
    GroupMessage {
        from: PeerId,
        from_nickname: String,
        group: String,
        message: String,
    },
    GroupFileShareMessage {
        from: PeerId,
        from_nickname: String,
        group: String,
        share_code: String,
        filename: String,
        file_size: u64,
        file_type: String,
        message: String,
    },
    GroupJoined {
        group: String,
    },
    GroupLeft {
        group: String,
    },

    // File transfer events
    FileShareRequest {
        from: PeerId,
        from_nickname: String,
        share_code: String,
        filename: String,
        size: u64,
    },
    FileShared {
        file_id: String,
        info: FileInfo,
    },
    FileRevoked {
        file_id: String,
    },
    FileInfoReceived {
        from: PeerId,
        info: FileInfo,
    },
    ChunkReceived {
        from: PeerId,
        file_id: String,
        chunk_index: usize,
        chunk: ChunkInfo,
    },
    FileListReceived {
        from: PeerId,
        files: Vec<FileInfo>,
    },
    FileDownloadStarted {
        from: PeerId,
        from_nickname: String,
        filename: String,
        download_id: String,
        share_code: String,
    },
    FileDownloadProgress {
        download_id: String,
        filename: String,
        share_code: String,
        from_peer_id: libp2p::PeerId,
        from_nickname: String,
        downloaded_chunks: usize,
        total_chunks: usize,
    },
    FileDownloadCompleted {
        download_id: String,
        filename: String,
        share_code: String,
        from_peer_id: libp2p::PeerId,
        from_nickname: String,
        path: PathBuf,
    },
    FileDownloadFailed {
        download_id: String,
        filename: String,
        share_code: String,
        from_peer_id: libp2p::PeerId,
        from_nickname: String,
        error: String,
    },

    // System events
    ListeningOn {
        address: Multiaddr,
    },
    Connected {
        peer_id: PeerId,
        nickname: String,
    },
    Disconnected {
        peer_id: PeerId,
        nickname: String,
    },
    Error(String),

    // Persistence events
    PendingMessagesAvailable {
        peer: PeerId,
        nickname: String,
    },
}

/// File information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub hash: String,
    pub chunk_count: usize,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    pub file_id: String,
    pub chunk_index: usize,
    pub data: Vec<u8>,
    pub hash: String,
}

/// File sharing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedFile {
    pub info: FileInfo,
    pub path: FilePath,
    pub share_code: String,
    pub revoked: bool,
}

/// Peer information
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub nickname: String,
    pub addresses: Vec<Multiaddr>,
    pub last_seen: std::time::Instant,
    pub connected: bool,
}

/// Group information
#[derive(Debug, Clone)]
pub struct GroupInfo {
    pub name: String,
    pub topic: IdentTopic,
    pub joined_at: DateTime<Utc>,
}

/// Group message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    pub sender_nickname: String,
    pub content: String,
    pub timestamp: u64,
    pub has_file_share: bool,
    pub share_code: Option<String>,
    pub filename: Option<String>,
    pub file_size: Option<u64>,
    pub file_type: Option<String>,
}

/// Active download tracking for mobile UI applications
#[derive(Debug, Clone)]
pub struct ActiveDownload {
    pub download_id: String,
    pub filename: String,
    pub share_code: String,
    pub from_peer_id: libp2p::PeerId,
    pub from_nickname: String,
    pub total_chunks: usize,
    pub downloaded_chunks: usize,
    pub started_at: std::time::Instant,
    pub completed: bool,
    pub failed: bool,
    pub error_message: Option<String>,
    pub final_path: Option<PathBuf>,
}

// ============================================================================
// Message Persistence Types
// ============================================================================

/// Message type (direct or group)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Direct,
    Group,
}

/// Message direction (sent or received)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageDirection {
    Sent,
    Received,
}

/// Message content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContent {
    Text {
        text: String,
    },
    FileShare {
        share_code: String,
        filename: String,
        file_size: u64,
        file_type: String,
    },
    ShareGroup {
        group_id: String,
        group_name: String,
        inviter_nickname: String,
    },
}

impl MessageContent {
    pub fn get_text(&self) -> std::result::Result<String, String> {
        match self {
            MessageContent::Text { text } => Ok(text.clone()),
            _ => Err("Not a text message".to_string()),
        }
    }
}

/// Sync status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    Pending,
    Synced,
    Delivered,
    Acknowledged,
}

/// Stored message (persistent)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: String,
    pub msg_type: MessageType,
    pub direction: MessageDirection,
    pub content: MessageContent,
    pub sender_nickname: String,
    pub recipient_nickname: Option<String>,
    pub group_name: Option<String>,
    pub peer_id: String,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,

    pub delivered: bool,
    pub delivered_at: Option<DateTime<Utc>>,

    pub read: bool,
    pub read_at: Option<DateTime<Utc>>,

    pub sync_status: SyncStatus,
    pub sync_attempts: u32,
    pub last_sync_attempt: Option<DateTime<Utc>>,

    pub expires_at: DateTime<Utc>,
}

/// Queue status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueueStatus {
    Pending,
    InProgress,
    Delivered,
    Expired,
}

/// Offline queue item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineQueueItem {
    pub message_id: String,
    pub target_nickname: String,
    pub target_peer_id: Option<String>,
    pub queued_at: DateTime<Utc>,

    pub retry_count: u32,
    pub max_retries: u32,
    pub last_retry_at: Option<DateTime<Utc>>,
    pub next_retry_at: DateTime<Utc>,

    pub expires_at: DateTime<Utc>,
    pub status: QueueStatus,
}

/// Acknowledgment type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AckType {
    Delivered,
    Read,
}

/// Message acknowledgment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAcknowledgment {
    pub id: String,
    pub message_id: String,
    pub acknowledged_by_nickname: String,
    pub acknowledged_by_peer_id: String,
    pub acknowledged_at: DateTime<Utc>,
    pub ack_type: AckType,
}
