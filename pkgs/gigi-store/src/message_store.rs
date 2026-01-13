//! Message store - persistent storage for messages using SQLite

use crate::events::StoredMessage;
use crate::PersistenceConfig;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension, Row};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info};

/// Message store - manages persistent message storage
pub struct MessageStore {
    pub(crate) db: Arc<Mutex<rusqlite::Connection>>,
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
        // Open database
        let db_path = config.db_path.clone();
        let conn =
            rusqlite::Connection::open(&db_path).context("Failed to open message database")?;

        // Create tables
        Self::create_tables(&conn)?;
        Self::create_indexes(&conn)?;

        info!("Message store initialized at {}", config.db_path.display());

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            config,
        })
    }

    /// Initialize database tables
    fn create_tables(conn: &rusqlite::Connection) -> Result<()> {
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                msg_type TEXT NOT NULL,
                direction TEXT NOT NULL,
                content_type TEXT NOT NULL,
                content_json TEXT NOT NULL,
                sender_nickname TEXT NOT NULL,
                recipient_nickname TEXT,
                group_name TEXT,
                peer_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                delivered BOOLEAN DEFAULT FALSE,
                delivered_at INTEGER,
                read BOOLEAN DEFAULT FALSE,
                read_at INTEGER,
                sync_status TEXT DEFAULT 'Pending',
                sync_attempts INTEGER DEFAULT 0,
                last_sync_attempt INTEGER,
                expires_at INTEGER NOT NULL
            )
            "#,
            [],
        )
        .context("Failed to create messages table")?;

        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS offline_queue (
                message_id TEXT PRIMARY KEY,
                target_nickname TEXT NOT NULL,
                target_peer_id TEXT,
                queued_at INTEGER NOT NULL,
                retry_count INTEGER DEFAULT 0,
                max_retries INTEGER DEFAULT 10,
                last_retry_at INTEGER,
                next_retry_at INTEGER,
                expires_at INTEGER NOT NULL,
                status TEXT DEFAULT 'Pending',
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
            )
            "#,
            [],
        )
        .context("Failed to create offline_queue table")?;

        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS message_acknowledgments (
                id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                acknowledged_by_nickname TEXT NOT NULL,
                acknowledged_by_peer_id TEXT NOT NULL,
                acknowledged_at INTEGER NOT NULL,
                ack_type TEXT NOT NULL,
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
            )
            "#,
            [],
        )
        .context("Failed to create message_acknowledgments table")?;

        Ok(())
    }

    /// Create database indexes
    fn create_indexes(conn: &rusqlite::Connection) -> Result<()> {
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
            "CREATE INDEX IF NOT EXISTS idx_acknowledgments_message ON message_acknowledgments(message_id)",
            "CREATE INDEX IF NOT EXISTS idx_acknowledgments_by_peer ON message_acknowledgments(acknowledged_by_nickname)",
        ];

        for idx in indexes {
            conn.execute(idx, [])
                .context(format!("Failed to create index: {}", idx))?;
        }

        Ok(())
    }

    /// Store a message
    pub async fn store_message(&self, msg: StoredMessage) -> Result<()> {
        let db = self.db.lock().unwrap();

        db.execute(
            r#"
            INSERT OR REPLACE INTO messages (
                id, msg_type, direction, content_type, content_json,
                sender_nickname, recipient_nickname, group_name, peer_id,
                timestamp, created_at, delivered, delivered_at,
                read, read_at, sync_status, sync_attempts,
                last_sync_attempt, expires_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                    ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
            "#,
            params![
                msg.id,
                serde_json::to_string(&msg.msg_type)?,
                serde_json::to_string(&msg.direction)?,
                serde_json::to_string(&msg.content)?,
                serde_json::to_string(&msg.content)?,
                msg.sender_nickname,
                msg.recipient_nickname,
                msg.group_name,
                msg.peer_id,
                msg.timestamp.timestamp_millis(),
                msg.created_at.timestamp_millis(),
                msg.delivered,
                msg.delivered_at.map(|t| t.timestamp_millis()),
                msg.read,
                msg.read_at.map(|t| t.timestamp_millis()),
                serde_json::to_string(&msg.sync_status)?,
                msg.sync_attempts,
                msg.last_sync_attempt.map(|t| t.timestamp_millis()),
                msg.expires_at.timestamp_millis(),
            ],
        )
        .context("Failed to store message")?;

        debug!("Stored message: {}", msg.id);
        Ok(())
    }

    /// Add message to offline queue
    pub async fn enqueue_offline(&self, message_id: String, target_nickname: String) -> Result<()> {
        let db = self.db.lock().unwrap();

        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(self.config.message_ttl_seconds as i64);
        let next_retry_at = now + chrono::Duration::seconds(300); // 5 minutes

        db.execute(
            r#"
            INSERT OR REPLACE INTO offline_queue (
                message_id, target_nickname, queued_at,
                retry_count, max_retries, last_retry_at,
                next_retry_at, expires_at, status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                message_id,
                target_nickname,
                now.timestamp_millis(),
                0u32,
                self.config.max_retry_attempts,
                Option::<i64>::None,
                next_retry_at.timestamp_millis(),
                expires_at.timestamp_millis(),
                "Pending",
            ],
        )
        .context("Failed to enqueue message")?;

        info!(
            "Enqueued message {} for offline peer {}",
            message_id, target_nickname
        );
        Ok(())
    }

    /// Get pending messages for a peer
    pub async fn get_pending_messages(
        &self,
        target_nickname: &str,
        limit: usize,
    ) -> Result<Vec<StoredMessage>> {
        let db = self.db.lock().unwrap();

        let mut stmt = db
            .prepare(
                r#"
                SELECT m.* FROM messages m
                INNER JOIN offline_queue q ON m.id = q.message_id
                WHERE q.target_nickname = ?1 AND q.status = 'Pending'
                ORDER BY m.timestamp ASC
                LIMIT ?2
                "#,
            )
            .context("Failed to prepare pending messages query")?;

        let messages: Vec<StoredMessage> = stmt
            .query_map(params![target_nickname, limit], |row| {
                Self::row_to_stored_message(row)
            })
            .map_err(|e| anyhow::anyhow!("Failed to map rows: {}", e))?
            .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()
            .map_err(|e| anyhow::anyhow!("Failed to collect messages: {}", e))
            .context("Failed to fetch pending messages")?;

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
        let db = self.db.lock().unwrap();

        let mut stmt = db
            .prepare(
                r#"
                SELECT * FROM messages
                WHERE (sender_nickname = ?1 OR recipient_nickname = ?1)
                ORDER BY timestamp DESC
                LIMIT ?2 OFFSET ?3
                "#,
            )
            .context("Failed to prepare conversation query")?;

        let messages: Vec<StoredMessage> = stmt
            .query_map(params![peer_nickname, limit, offset], |row| {
                Self::row_to_stored_message(row)
            })
            .map_err(|e| anyhow::anyhow!("Failed to map rows: {}", e))?
            .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()
            .map_err(|e| anyhow::anyhow!("Failed to collect messages: {}", e))
            .context("Failed to fetch conversation")?;

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
        let db = self.db.lock().unwrap();

        let mut stmt = db
            .prepare(
                r#"
                SELECT * FROM messages
                WHERE group_name = ?1
                ORDER BY timestamp DESC
                LIMIT ?2 OFFSET ?3
                "#,
            )
            .context("Failed to prepare group messages query")?;

        let messages: Vec<StoredMessage> = stmt
            .query_map(params![group_name, limit, offset], |row| {
                Self::row_to_stored_message(row)
            })
            .map_err(|e| anyhow::anyhow!("Failed to map rows: {}", e))?
            .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()
            .map_err(|e| anyhow::anyhow!("Failed to collect messages: {}", e))
            .context("Failed to fetch group messages")?;

        debug!(
            "Retrieved {} messages from group {}",
            messages.len(),
            group_name
        );
        Ok(messages)
    }

    /// Mark message as delivered
    pub async fn mark_delivered(&self, message_id: &str) -> Result<()> {
        let db = self.db.lock().unwrap();

        let now = Utc::now().timestamp_millis();

        // Update message status
        db.execute(
            "UPDATE messages SET delivered = TRUE, delivered_at = ?1, sync_status = 'Delivered' WHERE id = ?2",
            params![now, message_id],
        )
        .context("Failed to mark message as delivered")?;

        // Update queue status
        db.execute(
            "UPDATE offline_queue SET status = 'Delivered' WHERE message_id = ?1",
            params![message_id],
        )
        .context("Failed to update queue status")?;

        debug!("Marked message {} as delivered", message_id);
        Ok(())
    }

    /// Mark message as read
    pub async fn mark_read(&self, message_id: &str) -> Result<()> {
        let db = self.db.lock().unwrap();

        let now = Utc::now().timestamp_millis();

        db.execute(
            "UPDATE messages SET read = TRUE, read_at = ?1 WHERE id = ?2",
            params![now, message_id],
        )
        .context("Failed to mark message as read")?;

        debug!("Marked message {} as read", message_id);
        Ok(())
    }

    /// Update retry information for a message
    pub async fn update_retry(&self, message_id: &str, success: bool) -> Result<()> {
        let db = self.db.lock().unwrap();

        let now = Utc::now();

        if success {
            db.execute(
                "UPDATE offline_queue SET status = 'Delivered' WHERE message_id = ?1",
                params![message_id],
            )
            .context("Failed to update queue status on success")?;
        } else {
            // Get current retry count
            let retry_count: u32 = db
                .query_row(
                    "SELECT retry_count FROM offline_queue WHERE message_id = ?1",
                    params![message_id],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            if retry_count < self.config.max_retry_attempts {
                // Exponential backoff: 5, 10, 20, 40, 80, 160, 320, 640, 1280, 2560 minutes
                let backoff_minutes = 5 * 2u32.pow(retry_count);
                let next_retry = now + chrono::Duration::minutes(backoff_minutes as i64);

                db.execute(
                    r#"
                    UPDATE offline_queue
                    SET retry_count = retry_count + 1,
                        last_retry_at = ?1,
                        next_retry_at = ?2
                    WHERE message_id = ?3
                    "#,
                    params![
                        now.timestamp_millis(),
                        next_retry.timestamp_millis(),
                        message_id,
                    ],
                )
                .context("Failed to update retry info")?;

                info!(
                    "Scheduled retry for message {} (attempt {}, next retry in {} minutes)",
                    message_id,
                    retry_count + 1,
                    backoff_minutes
                );
            } else {
                // Max retries reached, mark as expired
                db.execute(
                    "UPDATE offline_queue SET status = 'Expired' WHERE message_id = ?1",
                    params![message_id],
                )
                .context("Failed to mark queue item as expired")?;

                error!(
                    "Message {} reached max retry attempts, marked as expired",
                    message_id
                );
            }
        }

        Ok(())
    }

    /// Get messages that need retry
    pub async fn get_retry_messages(&self, limit: usize) -> Result<Vec<(String, String)>> {
        let db = self.db.lock().unwrap();

        let mut stmt = db
            .prepare(
                r#"
                SELECT message_id, target_nickname FROM offline_queue
                WHERE status = 'Pending' AND next_retry_at <= ?1
                AND retry_count < max_retries
                ORDER BY next_retry_at ASC
                LIMIT ?2
                "#,
            )
            .context("Failed to prepare retry messages query")?;

        let items = stmt
            .query_map(params![Utc::now().timestamp_millis(), limit], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, rusqlite::Error>>()
            .context("Failed to fetch retry messages")?;

        debug!("Found {} messages ready for retry", items.len());
        Ok(items)
    }

    /// Clean up expired messages
    pub async fn cleanup_expired(&self) -> Result<u64> {
        let db = self.db.lock().unwrap();

        let now = Utc::now().timestamp_millis();

        // Clean up expired queue items
        let queue_count = db
            .execute(
                "DELETE FROM offline_queue WHERE expires_at < ?1 OR (retry_count >= max_retries AND next_retry_at < ?1)",
                params![now],
            )
            .context("Failed to cleanup expired queue items")?;

        // Clean up expired delivered messages
        let msg_count = db
            .execute(
                "DELETE FROM messages WHERE expires_at < ?1 AND delivered = TRUE",
                params![now],
            )
            .context("Failed to cleanup expired messages")?;

        let total = (queue_count + msg_count) as u64;
        if total > 0 {
            info!("Cleaned up {} expired messages", total);
        }

        Ok(total)
    }

    /// Get unread message count for a peer
    pub async fn get_unread_count(&self, peer_nickname: &str) -> Result<u64> {
        let db = self.db.lock().unwrap();

        let count: u64 = db
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE sender_nickname = ?1 AND read = FALSE",
                params![peer_nickname],
                |row| row.get(0),
            )
            .context("Failed to get unread count")?;

        debug!("Unread messages from {}: {}", peer_nickname, count);
        Ok(count)
    }

    /// Get message by ID
    pub async fn get_message(&self, message_id: &str) -> Result<Option<StoredMessage>> {
        let db = self.db.lock().unwrap();

        let mut stmt = db
            .prepare("SELECT * FROM messages WHERE id = ?1")
            .context("Failed to prepare get message query")?;

        let result: Option<StoredMessage> = stmt
            .query_row(params![message_id], |row| Self::row_to_stored_message(row))
            .optional()
            .map_err(|e| anyhow::anyhow!("Failed to fetch message: {}", e))
            .context("Failed to fetch message")?;

        Ok(result)
    }

    /// Convert database row to StoredMessage
    fn row_to_stored_message(row: &Row) -> std::result::Result<StoredMessage, rusqlite::Error> {
        Ok(StoredMessage {
            id: row.get(0)?,
            msg_type: serde_json::from_str(&row.get::<_, String>(1)?)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            direction: serde_json::from_str(&row.get::<_, String>(2)?)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            content: serde_json::from_str(&row.get::<_, String>(4)?)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            sender_nickname: row.get(5)?,
            recipient_nickname: row.get(6)?,
            group_name: row.get(7)?,
            peer_id: row.get(8)?,
            timestamp: DateTime::from_timestamp_millis(row.get(9)?)
                .ok_or(rusqlite::Error::InvalidQuery)?
                .with_timezone(&Utc),
            created_at: DateTime::from_timestamp_millis(row.get(10)?)
                .ok_or(rusqlite::Error::InvalidQuery)?
                .with_timezone(&Utc),
            delivered: row.get(11)?,
            delivered_at: row
                .get::<_, Option<i64>>(12)?
                .map(|t| DateTime::from_timestamp_millis(t).ok_or(rusqlite::Error::InvalidQuery))
                .transpose()?
                .map(|dt| dt.with_timezone(&Utc)),
            read: row.get(13)?,
            read_at: row
                .get::<_, Option<i64>>(14)?
                .map(|t| DateTime::from_timestamp_millis(t).ok_or(rusqlite::Error::InvalidQuery))
                .transpose()?
                .map(|dt| dt.with_timezone(&Utc)),
            sync_status: serde_json::from_str(&row.get::<_, String>(15)?)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            sync_attempts: row.get(16)?,
            last_sync_attempt: row
                .get::<_, Option<i64>>(17)?
                .map(|t| DateTime::from_timestamp_millis(t).ok_or(rusqlite::Error::InvalidQuery))
                .transpose()?
                .map(|dt| dt.with_timezone(&Utc)),
            expires_at: DateTime::from_timestamp_millis(row.get(18)?)
                .ok_or(rusqlite::Error::InvalidQuery)?
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
