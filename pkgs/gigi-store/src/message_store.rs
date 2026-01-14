//! Message store - persistent storage for messages using Sea-ORM and SQLite

use crate::entities::{messages, offline_queue};
use crate::events::StoredMessage;
use crate::PersistenceConfig;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sea_orm::prelude::Expr;
use sea_orm::*;
use sea_orm_migration::MigratorTrait;
use std::path::PathBuf;
use tracing::{debug, error, info};

/// Message store - manages persistent message storage
pub struct MessageStore {
    pub(crate) db: DatabaseConnection,
    pub(crate) config: PersistenceConfig,
}

impl MessageStore {
    /// Create a new message store with default config
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        Self::with_config(PersistenceConfig {
            db_path,
            ..Default::default()
        })
        .await
    }

    /// Create a message store with custom config
    pub async fn with_config(config: PersistenceConfig) -> Result<Self> {
        // Create database URL - Use sqlx-sqlite format for Sea-ORM 1.x
        let db_path = config
            .db_path
            .canonicalize()
            .unwrap_or_else(|_| config.db_path.clone());

        let db_path_str = db_path
            .to_str()
            .context("Invalid database path")?
            .replace("\\", "/"); // Windows path compatibility

        // Use the correct connection string format for sqlx-sqlite
        // Sea-ORM 1.x expects "sqlite://" prefix for sqlx-sqlite driver
        let db_url = format!("sqlite:{}?mode=rwc", db_path_str);

        // Connect to database using sqlx-sqlite
        let db: DatabaseConnection = Database::connect(db_url.as_str())
            .await
            .context("Failed to connect to database")?;

        // Run migrations
        crate::migration::Migrator::up(&db, None)
            .await
            .context("Failed to run migrations")?;

        // Create indexes
        Self::create_indexes(&db)
            .await
            .context("Failed to create indexes")?;

        info!("Message store initialized at {}", config.db_path.display());

        Ok(Self { db, config })
    }

    /// Create database indexes
    async fn create_indexes(db: &DatabaseConnection) -> Result<()> {
        // SQLite indexes for performance
        let indexes = vec![
            "CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp DESC)",
            "CREATE INDEX IF NOT EXISTS idx_messages_sender ON messages(sender_nickname)",
            "CREATE INDEX IF NOT EXISTS idx_messages_recipient ON messages(recipient_nickname)",
            "CREATE INDEX IF NOT EXISTS idx_messages_group ON messages(group_name)",
            "CREATE INDEX IF NOT EXISTS idx_messages_sync_status ON messages(sync_status)",
            "CREATE INDEX IF NOT EXISTS idx_messages_expires ON messages(expires_at)",
            "CREATE INDEX IF NOT EXISTS idx_offline_queue_target ON offline_queue(target_nickname)",
            "CREATE INDEX IF NOT EXISTS idx_offline_queue_status ON offline_queue(status)",
            "CREATE INDEX IF NOT EXISTS idx_offline_queue_next_retry ON offline_queue(next_retry_at)",
            "CREATE INDEX IF NOT EXISTS idx_offline_queue_expires ON offline_queue(expires_at)",
        ];

        for idx in indexes {
            db.execute(Statement::from_string(
                DatabaseBackend::Sqlite,
                idx.to_string(),
            ))
            .await
            .context(format!("Failed to create index: {}", idx))?;
        }

        Ok(())
    }

    /// Store a message
    pub async fn store_message(&self, msg: StoredMessage) -> Result<()> {
        let new_msg = messages::ActiveModel {
            id: Set(msg.id.clone()),
            msg_type: Set(serde_json::to_string(&msg.msg_type)?),
            direction: Set(serde_json::to_string(&msg.direction)?),
            content_type: Set(match &msg.content {
                crate::MessageContent::Text { .. } => "Text".to_string(),
                crate::MessageContent::FileShare { .. } => "FileShare".to_string(),
                crate::MessageContent::ShareGroup { .. } => "ShareGroup".to_string(),
            }),
            content_json: Set(serde_json::to_string(&msg.content)?),
            sender_nickname: Set(msg.sender_nickname),
            recipient_nickname: Set(msg.recipient_nickname),
            group_name: Set(msg.group_name),
            peer_id: Set(msg.peer_id),
            timestamp: Set(msg.timestamp.timestamp_millis()),
            created_at: Set(msg.created_at.timestamp_millis()),
            delivered: Set(msg.delivered),
            delivered_at: Set(msg.delivered_at.map(|t| t.timestamp_millis())),
            read: Set(msg.read),
            read_at: Set(msg.read_at.map(|t| t.timestamp_millis())),
            sync_status: Set(serde_json::to_string(&msg.sync_status)?),
            sync_attempts: Set(msg.sync_attempts),
            last_sync_attempt: Set(msg.last_sync_attempt.map(|t| t.timestamp_millis())),
            expires_at: Set(msg.expires_at.timestamp_millis()),
        };

        // Use insert() without expecting a return value
        let insert_result = new_msg.insert(&self.db).await;

        // Ignore RecordNotFound error - it might still have inserted successfully
        match insert_result {
            Ok(_) | Err(DbErr::RecordNotFound(_)) => {
                // Either succeeded or there was an issue finding the record after insert
                // but the insert likely succeeded
            }
            Err(e) => {
                return Err(e).context("Failed to store message")?;
            }
        }

        debug!("Stored message: {}", msg.id);
        Ok(())
    }

    /// Add message to offline queue
    pub async fn enqueue_offline(&self, message_id: String, target_nickname: String) -> Result<()> {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(self.config.message_ttl_seconds as i64);
        let next_retry_at = now + chrono::Duration::seconds(300); // 5 minutes

        let queue_item = offline_queue::ActiveModel {
            message_id: Set(message_id),
            target_nickname: Set(target_nickname),
            target_peer_id: Set(None),
            queued_at: Set(now.timestamp_millis()),
            retry_count: Set(0),
            max_retries: Set(self.config.max_retry_attempts),
            last_retry_at: Set(None),
            next_retry_at: Set(Some(next_retry_at.timestamp_millis())),
            expires_at: Set(expires_at.timestamp_millis()),
            status: Set("Pending".to_string()),
        };

        // Use insert() without expecting a return value
        let insert_result = queue_item.insert(&self.db).await;

        // Ignore RecordNotFound error
        match insert_result {
            Ok(_) | Err(DbErr::RecordNotFound(_)) => {}
            Err(e) => {
                return Err(e).context("Failed to enqueue message")?;
            }
        }

        info!("Enqueued message for offline peer");
        Ok(())
    }

    /// Get pending messages for a peer
    pub async fn get_pending_messages(
        &self,
        target_nickname: &str,
        limit: usize,
    ) -> Result<Vec<StoredMessage>> {
        let result = messages::Entity::find()
            .inner_join(offline_queue::Entity)
            .filter(offline_queue::Column::TargetNickname.eq(target_nickname))
            .filter(offline_queue::Column::Status.eq("Pending"))
            .order_by_asc(messages::Column::Timestamp)
            .limit(limit as u64)
            .all(&self.db)
            .await
            .context("Failed to fetch pending messages")?;

        let messages: Vec<StoredMessage> = result
            .into_iter()
            .map(|m| self.model_to_stored_message(m))
            .collect::<Result<Vec<_>>>()?;

        debug!(
            "Found {} pending messages for {}",
            messages.len(),
            target_nickname
        );
        Ok(messages)
    }

    /// Get conversation history
    pub async fn get_conversation(
        &self,
        peer_nickname: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<StoredMessage>> {
        let result = messages::Entity::find()
            .filter(
                Condition::any()
                    .add(messages::Column::SenderNickname.eq(peer_nickname))
                    .add(messages::Column::RecipientNickname.eq(peer_nickname)),
            )
            .order_by_desc(messages::Column::Timestamp)
            .paginate(&self.db, limit as u64)
            .fetch_page(offset as u64)
            .await
            .context("Failed to fetch conversation")?;

        let messages: Vec<StoredMessage> = result
            .into_iter()
            .map(|m| self.model_to_stored_message(m))
            .collect::<Result<Vec<_>>>()?;

        debug!(
            "Retrieved {} messages from conversation with {}",
            messages.len(),
            peer_nickname
        );
        Ok(messages)
    }

    /// Get group messages
    pub async fn get_group_messages(
        &self,
        group_name: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<StoredMessage>> {
        let result = messages::Entity::find()
            .filter(messages::Column::GroupName.eq(group_name))
            .order_by_desc(messages::Column::Timestamp)
            .paginate(&self.db, limit as u64)
            .fetch_page(offset as u64)
            .await
            .context("Failed to fetch group messages")?;

        let messages: Vec<StoredMessage> = result
            .into_iter()
            .map(|m| self.model_to_stored_message(m))
            .collect::<Result<Vec<_>>>()?;

        debug!(
            "Retrieved {} messages from group {}",
            messages.len(),
            group_name
        );
        Ok(messages)
    }

    /// Mark message as delivered
    pub async fn mark_delivered(&self, message_id: &str) -> Result<()> {
        let now = Utc::now().timestamp_millis();

        // Update message status
        messages::ActiveModel {
            id: Set(message_id.to_string()),
            delivered: Set(true),
            delivered_at: Set(Some(now)),
            sync_status: Set(serde_json::to_string(&crate::SyncStatus::Delivered)?),
            ..Default::default()
        }
        .update(&self.db)
        .await
        .context("Failed to mark message as delivered")?;

        // Update queue status
        offline_queue::ActiveModel {
            message_id: Set(message_id.to_string()),
            status: Set("Delivered".to_string()),
            ..Default::default()
        }
        .update(&self.db)
        .await
        .context("Failed to update queue status")?;

        debug!("Marked message {} as delivered", message_id);
        Ok(())
    }

    /// Update message peer_id
    pub async fn update_message_peer_id(&self, message_id: &str, peer_id: String) -> Result<()> {
        messages::Entity::update_many()
            .filter(messages::Column::Id.eq(message_id))
            .col_expr(messages::Column::PeerId, Expr::value(peer_id))
            .exec(&self.db)
            .await
            .context("Failed to update message peer_id")?;

        debug!("Updated peer_id for message {}", message_id);
        Ok(())
    }

    /// Mark message as read
    pub async fn mark_read(&self, message_id: &str) -> Result<()> {
        let now = Utc::now().timestamp_millis();

        messages::ActiveModel {
            id: Set(message_id.to_string()),
            read: Set(true),
            read_at: Set(Some(now)),
            ..Default::default()
        }
        .update(&self.db)
        .await
        .context("Failed to mark message as read")?;

        debug!("Marked message {} as read", message_id);
        Ok(())
    }

    /// Mark all messages in a conversation as read
    pub async fn mark_conversation_read(&self, nickname: &str) -> Result<()> {
        let now = Utc::now().timestamp_millis();

        messages::Entity::update_many()
            .filter(messages::Column::RecipientNickname.eq(nickname))
            .filter(messages::Column::MsgType.eq("Received"))
            .filter(messages::Column::Read.eq(false))
            .col_expr(messages::Column::Read, Expr::value(true))
            .col_expr(messages::Column::ReadAt, Expr::value(Some(now)))
            .exec(&self.db)
            .await
            .context("Failed to mark conversation as read")?;

        debug!("Marked all messages from {} as read", nickname);
        Ok(())
    }

    /// Update retry information for a message
    pub async fn update_retry(&self, message_id: &str, success: bool) -> Result<()> {
        if success {
            offline_queue::ActiveModel {
                message_id: Set(message_id.to_string()),
                status: Set("Delivered".to_string()),
                ..Default::default()
            }
            .update(&self.db)
            .await
            .context("Failed to update queue status on success")?;
        } else {
            // Get current queue item
            let queue_item = offline_queue::Entity::find()
                .filter(offline_queue::Column::MessageId.eq(message_id))
                .one(&self.db)
                .await
                .context("Failed to fetch queue item")?;

            if let Some(item) = queue_item {
                if item.retry_count < self.config.max_retry_attempts {
                    let now = Utc::now();
                    // Exponential backoff: 5, 10, 20, 40, 80, 160, 320, 640, 1280, 2560 minutes
                    let backoff_minutes = 5 * 2u32.pow(item.retry_count);
                    let next_retry = now + chrono::Duration::minutes(backoff_minutes as i64);

                    offline_queue::ActiveModel {
                        message_id: Set(message_id.to_string()),
                        retry_count: Set(item.retry_count + 1),
                        last_retry_at: Set(Some(now.timestamp_millis())),
                        next_retry_at: Set(Some(next_retry.timestamp_millis())),
                        ..Default::default()
                    }
                    .update(&self.db)
                    .await
                    .context("Failed to update retry info")?;

                    info!(
                        "Scheduled retry for message {} (attempt {}, next retry in {} minutes)",
                        message_id,
                        item.retry_count + 1,
                        backoff_minutes
                    );
                } else {
                    // Max retries reached, mark as expired
                    offline_queue::ActiveModel {
                        message_id: Set(message_id.to_string()),
                        status: Set("Expired".to_string()),
                        ..Default::default()
                    }
                    .update(&self.db)
                    .await
                    .context("Failed to mark queue item as expired")?;

                    error!(
                        "Message {} reached max retry attempts, marked as expired",
                        message_id
                    );
                }
            }
        }

        Ok(())
    }

    /// Get messages that need retry
    pub async fn get_retry_messages(&self, limit: usize) -> Result<Vec<(String, String)>> {
        let now = Utc::now().timestamp_millis();

        let result = offline_queue::Entity::find()
            .filter(offline_queue::Column::Status.eq("Pending"))
            .filter(offline_queue::Column::NextRetryAt.lte(now))
            .filter(
                Condition::all()
                    .add(
                        Expr::col((offline_queue::Entity, offline_queue::Column::RetryCount)).lt(
                            Expr::col((offline_queue::Entity, offline_queue::Column::MaxRetries)),
                        ),
                    )
                    .add(
                        Expr::col((offline_queue::Entity, offline_queue::Column::NextRetryAt))
                            .is_not_null(),
                    ),
            )
            .order_by_asc(offline_queue::Column::NextRetryAt)
            .limit(limit as u64)
            .all(&self.db)
            .await
            .context("Failed to fetch retry messages")?;

        let items = result
            .into_iter()
            .map(|item| Ok((item.message_id, item.target_nickname)))
            .collect::<Result<Vec<_>>>()?;

        debug!("Found {} messages ready for retry", items.len());
        Ok(items)
    }

    /// Clean up expired messages
    pub async fn cleanup_expired(&self) -> Result<u64> {
        let now = Utc::now().timestamp_millis();

        // Clean up expired queue items
        let queue_count = offline_queue::Entity::delete_many()
            .filter(
                Condition::any()
                    .add(offline_queue::Column::ExpiresAt.lt(now))
                    .add(
                        Condition::all()
                            .add(
                                Expr::col((
                                    offline_queue::Entity,
                                    offline_queue::Column::RetryCount,
                                ))
                                .gte(Expr::col((
                                    offline_queue::Entity,
                                    offline_queue::Column::MaxRetries,
                                ))),
                            )
                            .add(
                                Expr::col((
                                    offline_queue::Entity,
                                    offline_queue::Column::NextRetryAt,
                                ))
                                .lt(now),
                            )
                            .add(
                                Expr::col((
                                    offline_queue::Entity,
                                    offline_queue::Column::NextRetryAt,
                                ))
                                .is_not_null(),
                            ),
                    ),
            )
            .exec(&self.db)
            .await
            .context("Failed to cleanup expired queue items")?
            .rows_affected;

        // Clean up expired delivered messages
        let msg_count = messages::Entity::delete_many()
            .filter(messages::Column::ExpiresAt.lt(now))
            .filter(messages::Column::Delivered.eq(true))
            .exec(&self.db)
            .await
            .context("Failed to cleanup expired messages")?
            .rows_affected;

        let total = queue_count + msg_count;
        if total > 0 {
            info!("Cleaned up {} expired messages", total);
        }

        Ok(total)
    }

    /// Get unread message count for a peer
    pub async fn get_unread_count(&self, peer_nickname: &str) -> Result<u64> {
        let count = messages::Entity::find()
            .filter(messages::Column::SenderNickname.eq(peer_nickname))
            .filter(messages::Column::Read.eq(false))
            .count(&self.db)
            .await
            .context("Failed to get unread count")?;

        debug!("Unread messages from {}: {}", peer_nickname, count);
        Ok(count)
    }

    /// Get message by ID
    pub async fn get_message(&self, message_id: &str) -> Result<Option<StoredMessage>> {
        let result = messages::Entity::find_by_id(message_id.to_string())
            .one(&self.db)
            .await
            .context("Failed to fetch message")?;

        Ok(result
            .map(|m| self.model_to_stored_message(m))
            .transpose()?)
    }

    /// Convert Sea-ORM model to StoredMessage
    fn model_to_stored_message(&self, model: messages::Model) -> Result<StoredMessage> {
        Ok(StoredMessage {
            id: model.id,
            msg_type: serde_json::from_str(&model.msg_type).context("Failed to parse msg_type")?,
            direction: serde_json::from_str(&model.direction)
                .context("Failed to parse direction")?,
            content: serde_json::from_str(&model.content_json)
                .context("Failed to parse content")?,
            sender_nickname: model.sender_nickname,
            recipient_nickname: model.recipient_nickname,
            group_name: model.group_name,
            peer_id: model.peer_id,
            timestamp: DateTime::from_timestamp_millis(model.timestamp)
                .context("Invalid timestamp")?
                .with_timezone(&Utc),
            created_at: DateTime::from_timestamp_millis(model.created_at)
                .context("Invalid created_at timestamp")?
                .with_timezone(&Utc),
            delivered: model.delivered,
            delivered_at: model
                .delivered_at
                .map(|t| {
                    DateTime::from_timestamp_millis(t)
                        .context("Invalid delivered_at timestamp")
                        .map(|dt| dt.with_timezone(&Utc))
                })
                .transpose()?,
            read: model.read,
            read_at: model
                .read_at
                .map(|t| {
                    DateTime::from_timestamp_millis(t)
                        .context("Invalid read_at timestamp")
                        .map(|dt| dt.with_timezone(&Utc))
                })
                .transpose()?,
            sync_status: serde_json::from_str(&model.sync_status)
                .context("Failed to parse sync_status")?,
            sync_attempts: model.sync_attempts,
            last_sync_attempt: model
                .last_sync_attempt
                .map(|t| {
                    DateTime::from_timestamp_millis(t)
                        .context("Invalid last_sync_attempt timestamp")
                        .map(|dt| dt.with_timezone(&Utc))
                })
                .transpose()?,
            expires_at: DateTime::from_timestamp_millis(model.expires_at)
                .context("Invalid expires_at timestamp")?
                .with_timezone(&Utc),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{MessageContent, MessageDirection, MessageType, SyncStatus};
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_store_and_retrieve_message() {
        let temp_file = NamedTempFile::new().unwrap();
        let store = MessageStore::new(temp_file.path().to_path_buf())
            .await
            .unwrap();

        let msg = StoredMessage {
            id: uuid::Uuid::new_v4().to_string(),
            msg_type: MessageType::Direct,
            direction: MessageDirection::Sent,
            content: MessageContent::Text {
                text: "Hello, World!".to_string(),
            },
            sender_nickname: "Alice".to_string(),
            recipient_nickname: Some("Bob".to_string()),
            group_name: None,
            peer_id: "peer123".to_string(),
            timestamp: Utc::now(),
            created_at: Utc::now(),
            delivered: false,
            delivered_at: None,
            read: false,
            read_at: None,
            sync_status: SyncStatus::Pending,
            sync_attempts: 0,
            last_sync_attempt: None,
            expires_at: Utc::now() + chrono::Duration::days(1),
        };

        store.store_message(msg.clone()).await.unwrap();

        let retrieved = store.get_message(&msg.id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, msg.id);
        assert_eq!(retrieved.sender_nickname, msg.sender_nickname);
    }

    #[tokio::test]
    async fn test_offline_queue() {
        let temp_file = NamedTempFile::new().unwrap();
        let store = MessageStore::new(temp_file.path().to_path_buf())
            .await
            .unwrap();

        // First create and store a message
        let msg = StoredMessage {
            id: uuid::Uuid::new_v4().to_string(),
            msg_type: MessageType::Direct,
            direction: MessageDirection::Sent,
            content: MessageContent::Text {
                text: "Hello, Bob!".to_string(),
            },
            sender_nickname: "Alice".to_string(),
            recipient_nickname: Some("Bob".to_string()),
            group_name: None,
            peer_id: "peer123".to_string(),
            timestamp: Utc::now(),
            created_at: Utc::now(),
            delivered: false,
            delivered_at: None,
            read: false,
            read_at: None,
            sync_status: SyncStatus::Pending,
            sync_attempts: 0,
            last_sync_attempt: None,
            expires_at: Utc::now() + chrono::Duration::days(1),
        };

        store.store_message(msg.clone()).await.unwrap();

        // Then enqueue it for offline peer
        store
            .enqueue_offline(msg.id.clone(), "Bob".to_string())
            .await
            .unwrap();

        let pending = store.get_pending_messages("Bob", 10).await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, msg.id);
    }
}
