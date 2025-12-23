//! P2P events and public data structures

use chrono::{DateTime, Utc};
use libp2p::gossipsub::IdentTopic;
use libp2p::{multiaddr::Multiaddr, PeerId};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    pub path: PathBuf,
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
