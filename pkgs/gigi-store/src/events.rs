//! Event types for message persistence

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Message type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Direct,
    Group,
}

/// Message direction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageDirection {
    Sent,
    Received,
}

/// Sync status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    Pending,
    Synced,
    Delivered,
    Acknowledged,
}

/// Text message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    pub text: String,
}

/// File share message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileShareContent {
    pub share_code: String,
    pub filename: String,
    pub file_size: u64,
    pub file_type: String,
}

/// Group share message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupShareContent {
    pub group_id: String,
    pub group_name: String,
    pub inviter_nickname: String,
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
    /// File share with optional thumbnail path (used after download)
    FileShareWithThumbnail {
        share_code: String,
        filename: String,
        file_size: u64,
        file_type: String,
        thumbnail_path: Option<String>,
    },
    ShareGroup {
        group_id: String,
        group_name: String,
        inviter_nickname: String,
    },
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
