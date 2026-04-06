//! Conversation store - manages conversation/chats metadata

use crate::entities::conversations;
use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use sea_orm::*;
use sea_orm_migration::MigratorTrait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

/// Conversation store
pub struct ConversationStore {
    pub(crate) db: DatabaseConnection,
}

impl ConversationStore {
    /// Create a new conversation store
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        let db_path_str = db_path
            .to_str()
            .context("Invalid database path")?
            .replace("\\", "/");

        let db_url = format!("sqlite:{}?mode=rwc", db_path_str);

        let db: DatabaseConnection = Database::connect(db_url.as_str())
            .await
            .context("Failed to connect to database")?;

        // Run migrations
        crate::migration::Migrator::up(&db, None)
            .await
            .context("Failed to run migrations")?;

        info!("Conversation store initialized at {}", db_path.display());

        Ok(Self { db })
    }

    /// Create a conversation store with an existing database connection
    pub async fn with_connection(db: DatabaseConnection) -> Result<Self> {
        info!("Conversation store initialized with existing connection");

        Ok(Self { db })
    }

    /// Create or update a conversation
    pub async fn upsert_conversation(
        &self,
        id: String,
        name: String,
        is_group: bool,
        peer_id: String,
        last_message: Option<String>,
        last_message_timestamp: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let now = Utc::now().timestamp_millis();

        let last_message_time = last_message_timestamp.map(|dt| dt.timestamp());
        let last_message_ts = last_message_timestamp.map(|dt| dt.timestamp_millis());

        let existing_conv = conversations::Entity::find()
            .filter(conversations::Column::Id.eq(&id))
            .one(&self.db)
            .await?;

        if let Some(model) = existing_conv {
            // Update existing conversation
            let mut active: conversations::ActiveModel = model.into();
            active.name = Set(name.clone());
            if last_message.is_some() {
                active.last_message = Set(last_message);
            }
            if last_message_time.is_some() {
                active.last_message_time = Set(last_message_time);
            }
            if last_message_ts.is_some() {
                active.last_message_timestamp = Set(last_message_ts);
            }
            active.updated_at = Set(now);
            active.update(&self.db).await?;
        } else {
            // Create new conversation
            let new_conv = conversations::ActiveModel {
                id: Set(id.clone()),
                name: Set(name.clone()),
                is_group: Set(is_group),
                peer_id: Set(peer_id),
                last_message: Set(last_message),
                last_message_time: Set(last_message_time),
                last_message_timestamp: Set(last_message_ts),
                unread_count: Set(0),
                created_at: Set(now),
                updated_at: Set(now),
            };

            match new_conv.insert(&self.db).await {
                Ok(_) => (),
                Err(e) => {
                    // Check if it's a RecordNotFound error and the conversation might actually exist
                    let err_str = e.to_string();
                    if err_str.contains("RecordNotFound") {
                        // Try to query the conversation to see if it actually exists
                        if let Ok(Some(_)) = conversations::Entity::find()
                            .filter(conversations::Column::Id.eq(&id))
                            .one(&self.db)
                            .await
                        {
                            info!("Conversation '{}' already exists, treating as success", id);
                            return Ok(());
                        }
                    } else if err_str.contains("UNIQUE constraint failed") {
                        // UNIQUE constraint violation - another process inserted it concurrently
                        // Try to update instead
                        let model = conversations::Entity::find()
                            .filter(conversations::Column::Id.eq(&id))
                            .one(&self.db)
                            .await?
                            .ok_or_else(|| {
                                anyhow::anyhow!("Conversation not found after concurrent insert")
                            })?;

                        let mut active: conversations::ActiveModel = model.into();
                        active.name = Set(name);
                        active.updated_at = Set(now);
                        active.update(&self.db).await?;
                    } else {
                        return Err(e.into());
                    }
                }
            }
        }

        Ok(())
    }

    /// Get all conversations sorted by last message timestamp
    pub async fn get_conversations(&self) -> Result<Vec<Conversation>> {
        let convs = conversations::Entity::find()
            .order_by_desc(conversations::Column::LastMessageTimestamp)
            .all(&self.db)
            .await?;

        Ok(convs
            .into_iter()
            .map(|m| self.model_to_conversation(m))
            .collect())
    }

    /// Get a single conversation by ID
    pub async fn get_conversation(&self, id: &str) -> Result<Option<Conversation>> {
        let conv = conversations::Entity::find()
            .filter(conversations::Column::Id.eq(id))
            .one(&self.db)
            .await?;

        Ok(conv.map(|m| self.model_to_conversation(m)))
    }

    /// Update last message info for a conversation
    pub async fn update_last_message(
        &self,
        id: &str,
        last_message: String,
        last_message_timestamp: DateTime<Utc>,
    ) -> Result<()> {
        let conv = conversations::Entity::find()
            .filter(conversations::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Conversation not found"))?;

        let mut active: conversations::ActiveModel = conv.into();
        active.last_message = Set(Some(last_message));
        active.last_message_time = Set(Some(last_message_timestamp.timestamp()));
        active.last_message_timestamp = Set(Some(last_message_timestamp.timestamp_millis()));
        active.updated_at = Set(Utc::now().timestamp_millis());
        active.update(&self.db).await?;

        Ok(())
    }

    /// Increment unread count for a conversation
    pub async fn increment_unread(&self, id: &str) -> Result<()> {
        let conv = conversations::Entity::find()
            .filter(conversations::Column::Id.eq(id))
            .one(&self.db)
            .await?;

        if let Some(model) = conv {
            let mut active: conversations::ActiveModel = model.into();
            active.unread_count = Set(active.unread_count.unwrap() + 1);
            active.updated_at = Set(Utc::now().timestamp_millis());
            active.update(&self.db).await?;
        }

        Ok(())
    }

    /// Mark all messages as read (reset unread count)
    pub async fn mark_as_read(&self, id: &str) -> Result<()> {
        let conv = conversations::Entity::find()
            .filter(conversations::Column::Id.eq(id))
            .one(&self.db)
            .await?;

        if let Some(model) = conv {
            let mut active: conversations::ActiveModel = model.into();
            active.unread_count = Set(0);
            active.updated_at = Set(Utc::now().timestamp_millis());
            active.update(&self.db).await?;
        }

        Ok(())
    }

    /// Delete a conversation
    pub async fn delete_conversation(&self, id: &str) -> Result<()> {
        conversations::Entity::delete_many()
            .filter(conversations::Column::Id.eq(id))
            .exec(&self.db)
            .await?;

        Ok(())
    }

    /// Convert Sea-ORM model to Conversation
    fn model_to_conversation(&self, model: conversations::Model) -> Conversation {
        Conversation {
            id: model.id,
            name: model.name,
            is_group: model.is_group,
            peer_id: model.peer_id,
            last_message: model.last_message,
            last_message_time: model
                .last_message_time
                .and_then(|t| Utc.timestamp_opt(t, 0).single()),
            last_message_timestamp: model
                .last_message_timestamp
                .and_then(|t| Utc.timestamp_millis_opt(t).single()),
            unread_count: model.unread_count,
        }
    }
}

/// Conversation model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub name: String,
    pub is_group: bool,
    pub peer_id: String,
    pub last_message: Option<String>,
    pub last_message_time: Option<DateTime<Utc>>,
    pub last_message_timestamp: Option<DateTime<Utc>>,
    pub unread_count: i32,
}
