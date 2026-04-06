//! Settings manager for storing and retrieving application settings

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use tracing::{debug, info};

use crate::entities::settings;

/// Key for storing encrypted mnemonic
pub const ENCRYPTED_MNEMONIC_KEY: &str = "encrypted_mnemonic";

/// Key for storing peer_id
pub const PEER_ID_KEY: &str = "peer_id";

/// Key for storing group_id
pub const GROUP_ID_KEY: &str = "group_id";

/// Key for storing encrypted password hash
pub const PASSWORD_HASH_KEY: &str = "password_hash";

/// Settings manager for storing and retrieving application settings
pub struct SettingsManager {
    db: DatabaseConnection,
}

impl SettingsManager {
    /// Create a new settings manager
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Get a setting value by key
    pub async fn get(&self, key: &str) -> Result<Option<String>, DbErr> {
        debug!("Getting setting: {}", key);

        let result = settings::Entity::find()
            .filter(settings::Column::Key.eq(key))
            .one(&self.db)
            .await?;

        Ok(result.map(|model| model.value))
    }

    /// Set a setting value
    pub async fn set(&self, key: &str, value: &str) -> Result<(), DbErr> {
        debug!("Setting key: {}", key);

        let now = chrono::Utc::now().timestamp_millis();

        // Check if setting already exists
        let existing = settings::Entity::find()
            .filter(settings::Column::Key.eq(key))
            .one(&self.db)
            .await?;

        if let Some(model) = existing {
            // Update existing setting
            let mut active_model: settings::ActiveModel = model.into();
            active_model.value = Set(value.to_string());
            active_model.updated_at = Set(now);
            active_model.update(&self.db).await?;
        } else {
            // Insert new setting
            let new_setting = settings::ActiveModel {
                key: Set(key.to_string()),
                value: Set(value.to_string()),
                updated_at: Set(now),
            };
            // Handle RecordNotFound from insert return value
            match new_setting.insert(&self.db).await {
                Ok(_) => {}
                Err(DbErr::RecordNotFound(_)) => {}
                Err(e) => return Err(e),
            }
        }

        info!("Setting '{}' updated successfully", key);
        Ok(())
    }

    /// Delete a setting by key
    pub async fn delete(&self, key: &str) -> Result<bool, DbErr> {
        debug!("Deleting setting: {}", key);

        let result = settings::Entity::delete(settings::ActiveModel {
            key: Set(key.to_string()),
            ..Default::default()
        })
        .exec(&self.db)
        .await?;

        Ok(result.rows_affected > 0)
    }

    /// Get all settings
    pub async fn get_all(&self) -> Result<Vec<(String, String)>, DbErr> {
        let settings = settings::Entity::find().all(&self.db).await?;

        Ok(settings.into_iter().map(|s| (s.key, s.value)).collect())
    }

    /// Clear all settings
    pub async fn clear_all(&self) -> Result<u64, DbErr> {
        info!("Clearing all settings");

        let result = settings::Entity::delete_many().exec(&self.db).await?;

        Ok(result.rows_affected)
    }

    /// Check if a setting exists
    pub async fn exists(&self, key: &str) -> Result<bool, DbErr> {
        let result = settings::Entity::find()
            .filter(settings::Column::Key.eq(key))
            .one(&self.db)
            .await?;

        Ok(result.is_some())
    }

    /// Set multiple settings in a transaction
    pub async fn set_many(&self, items: &[(String, String)]) -> Result<(), DbErr> {
        if items.is_empty() {
            return Ok(());
        }

        let txn = self.db.begin().await?;
        let now = chrono::Utc::now().timestamp_millis();

        for (key, value) in items {
            let existing = settings::Entity::find()
                .filter(settings::Column::Key.eq(key))
                .one(&txn)
                .await?;

            if let Some(model) = existing {
                let mut active_model: settings::ActiveModel = model.into();
                active_model.value = Set(value.clone());
                active_model.updated_at = Set(now);
                active_model.update(&txn).await?;
            } else {
                let new_setting = settings::ActiveModel {
                    key: Set(key.clone()),
                    value: Set(value.clone()),
                    updated_at: Set(now),
                };
                // Handle RecordNotFound from insert return value
                match new_setting.insert(&txn).await {
                    Ok(_) => {}
                    Err(DbErr::RecordNotFound(_)) => {}
                    Err(e) => return Err(e),
                }
            }
        }

        txn.commit().await?;
        info!("Set {} settings successfully", items.len());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_settings_constants() {
        assert_eq!(ENCRYPTED_MNEMONIC_KEY, "encrypted_mnemonic");
        assert_eq!(PEER_ID_KEY, "peer_id");
        assert_eq!(GROUP_ID_KEY, "group_id");
        assert_eq!(PASSWORD_HASH_KEY, "password_hash");
    }
}
