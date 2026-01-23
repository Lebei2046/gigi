//! Gigi Store - Persistent storage for P2P applications
//!
//! This crate provides SQLite-based persistent storage for messages using Sea-ORM,
//! including offline queuing, message history, and delivery tracking.
//!
//! It also manages application-wide data such as private keys and nicknames,
//! and shared file information.

pub mod contact_manager;
pub mod conversation_store;
pub mod entities;
pub mod file_sharing_store;
pub mod group_manager;
pub mod key_manager;
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
pub use key_manager::{AppData, KeyManager};
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
