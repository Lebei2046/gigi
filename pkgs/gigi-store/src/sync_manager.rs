//! Sync manager - coordinates message synchronization between peers

use crate::events::StoredMessage;
use crate::{MessageStore, PersistenceConfig};
use anyhow::Result;
use chrono::{DateTime, Utc};
use libp2p::PeerId;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

/// Sync state for a peer
#[derive(Debug, Clone)]
struct SyncState {
    #[allow(dead_code)] // Kept for future use in sync logic
    last_sync: DateTime<Utc>,
    in_progress: bool,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            last_sync: Utc::now() - chrono::Duration::days(1),
            in_progress: false,
        }
    }
}

/// Sync manager - handles synchronization of offline messages
pub struct SyncManager {
    message_store: Arc<MessageStore>,
    sync_states: Arc<Mutex<HashMap<String, SyncState>>>,
    config: PersistenceConfig,
    #[allow(dead_code)]
    sync_state_path: std::path::PathBuf,
}

impl SyncManager {
    /// Create a new sync manager
    pub fn new(
        message_store: Arc<MessageStore>,
        _local_nickname: String,
        sync_state_path: std::path::PathBuf,
    ) -> Self {
        Self {
            message_store: message_store.clone(),
            sync_states: Arc::new(Mutex::new(HashMap::new())),
            config: message_store.config.clone(),
            sync_state_path,
        }
    }

    /// Handle peer coming online
    pub async fn on_peer_online(
        &self,
        nickname: &str,
        peer_id: PeerId,
    ) -> Result<Vec<StoredMessage>> {
        info!("Peer {} ({}) came online", nickname, peer_id);

        // Get pending messages for this peer
        let messages: Vec<StoredMessage> = self
            .message_store
            .get_pending_messages(nickname, self.config.max_batch_size)
            .await?;

        if messages.is_empty() {
            debug!("No pending messages for {}", nickname);
            return Ok(messages);
        }

        info!("Found {} pending messages for {}", messages.len(), nickname);

        // Update sync state
        let mut syncs = self.sync_states.lock().await;
        syncs.insert(
            nickname.to_string(),
            SyncState {
                last_sync: Utc::now(),
                in_progress: true,
            },
        );

        Ok(messages)
    }

    /// Handle peer going offline
    pub async fn on_peer_offline(&self, nickname: &str, peer_id: PeerId) {
        info!("Peer {} ({}) went offline", nickname, peer_id);

        // Mark sync as not in progress
        let mut syncs = self.sync_states.lock().await;
        if let Some(state) = syncs.get_mut(nickname) {
            state.in_progress = false;
        }
    }

    /// Handle acknowledgment of message delivery
    pub async fn on_message_acknowledged(
        &self,
        message_id: &str,
        nickname: &str,
        ack_type: AckType,
    ) -> Result<()> {
        info!(
            "Message {} acknowledged by {} as {:?}",
            message_id, nickname, ack_type
        );

        match ack_type {
            AckType::Delivered => {
                self.message_store.mark_delivered(message_id).await?;
            }
            AckType::Read => {
                self.message_store.mark_read(message_id).await?;
            }
        }

        Ok(())
    }

    /// Handle message send failure (for retry)
    pub async fn on_message_send_failure(
        &self,
        message_id: &str,
        nickname: &str,
        error: String,
    ) -> Result<()> {
        error!(
            "Failed to send message {} to {}: {}",
            message_id, nickname, error
        );

        // Update retry info
        self.message_store.update_retry(message_id, false).await?;

        Ok(())
    }

    /// Get unread message count for a peer
    pub async fn get_unread_count(&self, peer_nickname: &str) -> Result<u64> {
        self.message_store.get_unread_count(peer_nickname).await
    }

    /// Start periodic retry task
    #[allow(dead_code)]
    pub async fn start_retry_task<F, Fut>(&self, on_retry: F)
    where
        F: Fn(String, String, StoredMessage) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        let mut interval =
            tokio::time::interval(Duration::from_secs(self.config.retry_interval_seconds));

        loop {
            interval.tick().await;

            // Get messages that need retry
            let retry_items: Vec<(String, String)> = self
                .message_store
                .get_retry_messages(self.config.max_batch_size)
                .await
                .unwrap_or_default();

            for (message_id, target_nickname) in retry_items {
                // Get the message
                match self.message_store.get_message(&message_id).await {
                    Ok(Some(msg)) => {
                        // Call the retry callback
                        let result: Result<()> =
                            on_retry(message_id.clone(), target_nickname.clone(), msg).await;
                        if let Err(e) = result {
                            error!("Retry callback failed for message {}: {}", message_id, e);
                        }
                    }
                    Ok(None) => {
                        error!("Message {} not found for retry", message_id);
                    }
                    Err(e) => {
                        error!("Failed to get message {} for retry: {}", message_id, e);
                    }
                }
            }
        }
    }

    /// Start periodic cleanup task
    #[allow(dead_code)]
    pub async fn start_cleanup_task(&self) {
        let mut interval =
            tokio::time::interval(Duration::from_secs(self.config.cleanup_interval_seconds));

        loop {
            interval.tick().await;

            if let Err(e) = self.message_store.cleanup_expired().await {
                error!("Cleanup task failed: {}", e);
            }
        }
    }
}

/// Sync message handler - handles sync protocol messages
pub struct SyncMessageHandler {
    local_nickname: String,
}

impl SyncMessageHandler {
    pub fn new(local_nickname: String) -> Self {
        Self { local_nickname }
    }

    /// Create sync request
    #[allow(dead_code)]
    pub fn create_sync_request(&self, message_ids: Vec<String>) -> SyncMessage {
        SyncMessage {
            from_nickname: self.local_nickname.clone(),
            action: SyncAction::SyncRequest { message_ids },
        }
    }

    /// Create sync response
    #[allow(dead_code)]
    pub fn create_sync_response(&self, messages: Vec<StoredMessage>) -> SyncMessage {
        SyncMessage {
            from_nickname: self.local_nickname.clone(),
            action: SyncAction::SyncResponse { messages },
        }
    }

    /// Create acknowledgment
    #[allow(dead_code)]
    pub fn create_acknowledgment(&self, message_id: String, ack_type: AckType) -> SyncMessage {
        SyncMessage {
            from_nickname: self.local_nickname.clone(),
            action: SyncAction::Acknowledgment {
                message_id,
                ack_type,
            },
        }
    }
}

/// Sync message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncMessage {
    pub from_nickname: String,
    pub action: SyncAction,
}

/// Sync action
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SyncAction {
    SyncRequest {
        message_ids: Vec<String>,
    },
    SyncResponse {
        messages: Vec<StoredMessage>,
    },
    HistoryRequest {
        limit: usize,
        offset: usize,
    },
    HistoryResponse {
        messages: Vec<StoredMessage>,
    },
    Acknowledgment {
        message_id: String,
        ack_type: AckType,
    },
}

/// Acknowledgment type
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AckType {
    Delivered,
    Read,
}
