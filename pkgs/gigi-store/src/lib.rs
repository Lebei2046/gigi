//! Gigi Store - Persistent storage for P2P applications
//!
//! This crate provides SQLite-based persistent storage for messages using Sea-ORM,
//! including offline queuing, message history, and delivery tracking.
//!
//! # Architecture
//!
//! The storage layer is organized into several specialized managers:
//!
//! - **MessageStore**: Core message persistence, offline queuing, and sync status
//! - **ConversationStore**: Chat/conversation metadata and last message tracking
//! - **ContactManager**: Contact book management (add, update, remove contacts)
//! - **GroupManager**: Group creation and membership tracking
//! - **FileSharingStore**: Shared file metadata and transfer tracking
//! - **ThumbnailStore**: Mapping between original files and generated thumbnails
//! - **SettingsManager**: Application-wide settings (mnemonic, peer_id, etc.)
//! - **SyncManager**: Message synchronization and acknowledgment tracking
//!
//! # Database Schema
//!
//! The database uses Sea-ORM with SQLite and includes these tables:
//!
//! - `messages`: Message content, timestamps, delivery status, sync status
//! - `offline_queue`: Queued messages for offline peers with retry logic
//! - `conversations`: Chat/conversation metadata and unread counts
//! - `contacts`: Contact book entries
//! - `groups`: Group definitions and member lists
//! - `shared_files`: File share metadata (hash, chunks, transfer status)
//! - `thumbnails`: File-to-thumbnail path mappings
//! - `settings`: Key-value settings storage
//! - `message_acknowledgments`: Read receipts and delivery confirmations
//!
//! # Key Features
//!
//! - **Offline Queue**: Messages are queued when recipients are offline
//! - **Retry Logic**: Exponential backoff for failed message delivery
//! - **Sync Tracking**: Track message sync status across devices
//! - **Expiration**: Automatic cleanup of old messages and queue items
//! - **Pagination**: Efficient pagination for large conversation histories
//! - **Indexes**: Optimized queries for common access patterns
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use gigi_store::{
//!     MessageContent, MessageDirection, MessageType, SyncStatus, StoredMessage,
//!     MessageStore, PersistenceConfig,
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = PersistenceConfig {
//!     db_path: "gigi.db".into(),
//!     ..Default::default()
//! };
//!
//! let store = MessageStore::with_config(config).await?;
//!
//! // Store a message
//! let msg = StoredMessage {
//!     id: uuid::Uuid::new_v4().to_string(),
//!     msg_type: MessageType::Direct,
//!     direction: MessageDirection::Sent,
//!     content: MessageContent::Text { text: "Hello!".to_string() },
//!     sender_nickname: "Alice".to_string(),
//!     recipient_nickname: Some("Bob".to_string()),
//!     group_name: None,
//!     peer_id: "peer123".to_string(),
//!     timestamp: chrono::Utc::now(),
//!     created_at: chrono::Utc::now(),
//!     delivered: false,
//!     delivered_at: None,
//!     read: false,
//!     read_at: None,
//!     sync_status: SyncStatus::Pending,
//!     sync_attempts: 0,
//!     last_sync_attempt: None,
//!     expires_at: chrono::Utc::now() + chrono::Duration::days(7),
//! };
//! store.store_message(msg).await?;
//! # Ok(())
//! # }
//! ```

pub mod contact_manager;
pub mod conversation_store;
pub mod entities;
pub mod file_sharing_store;
pub mod group_manager;
pub mod message_store;
pub mod migration;
pub mod settings_manager;
pub mod sync_manager;
pub mod thumbnail;
pub mod thumbnail_store;

// Re-export from gigi-auth
pub use gigi_auth::{AccountInfo, AuthManager, LoginResult};

pub use contact_manager::{ContactInfo, ContactManager};
pub use conversation_store::{Conversation, ConversationStore};
pub use file_sharing_store::{FileSharingStore, SharedFileInfo};
pub use group_manager::{GroupInfo, GroupManager};
pub use message_store::MessageStore;
pub use settings_manager::SettingsManager;
pub use sync_manager::{AckType, SyncAction, SyncManager, SyncMessage, SyncMessageHandler};
pub use thumbnail_store::ThumbnailStore;

mod events;

pub use events::{
    AckType as EventAckType, FileShareContent, GroupShareContent, MessageAcknowledgment,
    MessageContent, MessageDirection, MessageType, OfflineQueueItem, QueueStatus, StoredMessage,
    SyncStatus, TextContent,
};

/// Configuration for persistence layer
#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    /// Path to the SQLite database file
    pub db_path: std::path::PathBuf,

    /// Interval in seconds for periodic sync (default: 30s)
    pub sync_interval_seconds: u64,

    /// Interval in seconds for retrying failed messages (default: 300s)
    pub retry_interval_seconds: u64,

    /// Interval in seconds for cleanup tasks (default: 3600s)
    pub cleanup_interval_seconds: u64,

    /// Time-to-live for messages in seconds (default: 7 days)
    pub message_ttl_seconds: u64,

    /// Maximum retry attempts for failed messages (default: 10)
    pub max_retry_attempts: u32,

    /// Maximum batch size for sync operations (default: 50)
    pub max_batch_size: usize,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            db_path: std::path::PathBuf::from("gigi-store.db"),
            sync_interval_seconds: 30,
            retry_interval_seconds: 300,
            cleanup_interval_seconds: 3600,
            message_ttl_seconds: 7 * 24 * 3600, // 7 days
            max_retry_attempts: 10,
            max_batch_size: 50,
        }
    }
}
