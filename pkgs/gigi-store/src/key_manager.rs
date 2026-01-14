//! Key manager - Store and retrieve application-wide data like peer IDs and nicknames

use anyhow::{Context, Result};
use libp2p::{identity::Keypair, PeerId};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Application data stored in gigi-store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppData {
    pub peer_id: String,
    pub nickname: Option<String>,
    pub created_at: i64,
}

impl AppData {
    /// Create new app data from a keypair and nickname
    pub fn from_keypair(keypair: &Keypair, nickname: Option<String>) -> Self {
        let peer_id = PeerId::from_public_key(&keypair.public());
        Self {
            peer_id: peer_id.to_string(),
            nickname,
            created_at: chrono::Utc::now().timestamp_millis(),
        }
    }

    /// Get the stored peer_id as PeerId
    pub fn to_peer_id(&self) -> Result<PeerId> {
        PeerId::from_bytes(self.peer_id.as_bytes())
            .context("Failed to parse PeerId from stored string")
    }
}

/// Key manager - handles storage and retrieval of application keys
pub struct KeyManager {
    db: DatabaseConnection,
}

impl KeyManager {
    /// Create a new key manager
    pub async fn new(db: DatabaseConnection) -> Result<Self> {
        Ok(Self { db })
    }

    /// Store or update key for a nickname
    pub async fn store_key(&self, key_data: &AppData) -> Result<()> {
        use crate::entities::app_data;
        use sea_orm::DbErr;

        // Check if key already exists
        let existing = app_data::Entity::find()
            .filter(app_data::Column::Key.eq(&key_data.peer_id))
            .one(&self.db)
            .await
            .context("Failed to query existing key")?;

        if let Some(existing) = existing {
            // Update existing
            let mut active_model: app_data::ActiveModel = existing.into();
            active_model.nickname = Set(key_data.nickname.clone());
            active_model
                .update(&self.db)
                .await
                .context("Failed to update key")?;
        } else {
            // Insert new
            let new_key = app_data::ActiveModel {
                key: Set(key_data.peer_id.clone()),
                nickname: Set(key_data.nickname.clone()),
                created_at: Set(key_data.created_at),
            };
            // Ignore RecordNotFound error - insert likely succeeded
            match new_key.insert(&self.db).await {
                Ok(_) | Err(DbErr::RecordNotFound(_)) => {}
                Err(e) => return Err(e).context("Failed to insert key")?,
            }
        }

        info!("Stored key for nickname: {:?}", key_data.nickname);
        Ok(())
    }

    /// Retrieve key by nickname
    pub async fn get_key(&self, nickname: &str) -> Result<Option<AppData>> {
        use crate::entities::app_data;

        let result = app_data::Entity::find()
            .filter(app_data::Column::Nickname.eq(nickname))
            .one(&self.db)
            .await
            .context("Failed to query key")?;

        if let Some(data) = result {
            let app_data = AppData {
                peer_id: data.key,
                nickname: data.nickname,
                created_at: data.created_at,
            };
            debug!("Retrieved key for nickname: {}", nickname);
            Ok(Some(app_data))
        } else {
            info!("No key found for nickname: {}", nickname);
            Ok(None)
        }
    }

    /// Retrieve key by peer_id
    pub async fn get_key_by_peer_id(&self, peer_id: &str) -> Result<Option<AppData>> {
        use crate::entities::app_data;

        let result = app_data::Entity::find()
            .filter(app_data::Column::Key.eq(peer_id))
            .one(&self.db)
            .await
            .context("Failed to query key")?;

        Ok(result.map(|data| AppData {
            peer_id: data.key,
            nickname: data.nickname,
            created_at: data.created_at,
        }))
    }

    /// Get all nicknames with stored keys
    pub async fn list_nicknames(&self) -> Result<Vec<String>> {
        use crate::entities::app_data;

        let results = app_data::Entity::find()
            .all(&self.db)
            .await
            .context("Failed to list keys")?;

        let nicknames: Vec<String> = results
            .into_iter()
            .filter_map(|data| data.nickname)
            .collect();

        Ok(nicknames)
    }

    /// Delete key for a nickname
    pub async fn delete_key(&self, nickname: &str) -> Result<bool> {
        use crate::entities::app_data;

        let result = app_data::Entity::delete_many()
            .filter(app_data::Column::Nickname.eq(nickname))
            .exec(&self.db)
            .await
            .context("Failed to delete key")?;

        info!("Deleted key for nickname: {}", nickname);
        Ok(result.rows_affected > 0)
    }
}
