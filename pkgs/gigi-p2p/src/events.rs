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
    DirectImageMessage {
        from: PeerId,
        from_nickname: String,
        filename: String,
        data: Vec<u8>,
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
    GroupImageMessage {
        from: PeerId,
        from_nickname: String,
        group: String,
        filename: String,
        data: Vec<u8>,
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
        file_id: String,
        downloaded_chunks: usize,
        total_chunks: usize,
    },
    FileDownloadCompleted {
        file_id: String,
        path: PathBuf,
    },
    FileDownloadFailed {
        file_id: String,
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
    pub is_image: bool,
    pub filename: Option<String>,
    pub data: Option<Vec<u8>>,
}
