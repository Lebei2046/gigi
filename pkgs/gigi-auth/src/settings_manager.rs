//! Settings manager for storing and retrieving application settings
//!
//! This module provides a database abstraction layer for persisting key-value
//! settings using Sea-ORM. The settings are stored in a simple key-value table
//! with timestamps for tracking updates.
//!
//! # Database Schema
//!
//! The settings table has the following structure:
//!
//! ```sql
//! CREATE TABLE settings (
//!     id INTEGER PRIMARY KEY AUTOINCREMENT,
//!     key TEXT UNIQUE NOT NULL,
//!     value TEXT NOT NULL,
//!     updated_at BIGINT NOT NULL
//! );
//! ```
//!
//! # Primary Usage
//!
//! The primary use case in `gigi-auth` is storing encrypted account data under
//! the `GIGI_KEY` constant. However, the manager is generic enough to store any
//! application settings.
//!
//! # Example
//!
//! ```no_run
//! use gigi_auth::settings_manager::SettingsManager;
//! use sea_orm::Database;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let db = Database::connect("sqlite::memory:").await?;
//! let manager = SettingsManager::new(db);
//!
//! // Set a value
//! manager.set("user_preference", "dark_mode").await?;
//!
//! // Get a value
//! let value = manager.get("user_preference").await?;
//! assert_eq!(value, Some("dark_mode".to_string()));
//!
//! // Delete a value
//! manager.delete("user_preference").await?;
//! # Ok(())
//! # }
//! ```

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, NotSet, QueryFilter, Set,
};
use tracing::{debug, info};

use crate::entities::settings;

/// Key for storing encrypted account data
///
/// This constant is used throughout the auth module to identify the encrypted
/// mnemonic data in the settings table. Changing this constant would break
/// compatibility with existing accounts.
pub const GIGI_KEY: &str = "gigi";

/// Settings manager for storing and retrieving application settings
///
/// The `SettingsManager` provides a simple key-value store interface backed by
/// a database table. It handles both insertion and updates seamlessly, tracking
/// the timestamp of each modification.
///
/// # Thread Safety
///
/// `SettingsManager` can be safely shared across threads if the underlying
/// database connection supports concurrent access (e.g., via connection pooling).
///
/// # Example
///
/// ```no_run
/// use gigi_auth::settings_manager::SettingsManager;
/// use sea_orm::Database;
///
/// # async fn example() -> anyhow::Result<()> {
/// let db = Database::connect("sqlite::memory:").await?;
/// let manager = SettingsManager::new(db);
///
/// manager.set("theme", "dark").await?;
/// let theme = manager.get("theme").await?;
/// println!("Theme: {:?}", theme);
/// # Ok(())
/// # }
/// ```
pub struct SettingsManager {
    db: DatabaseConnection,
}

impl SettingsManager {
    /// Create a new settings manager
    ///
    /// Creates a new `SettingsManager` instance with the provided database
    /// connection. The connection is used for all database operations.
    ///
    /// # Arguments
    ///
    /// * `db` - A Sea-ORM database connection
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gigi_auth::settings_manager::SettingsManager;
    /// use sea_orm::Database;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let db = Database::connect("sqlite::memory:").await?;
    /// let manager = SettingsManager::new(db);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Get a setting value by key
    ///
    /// Retrieves the value associated with the given key from the database.
    /// Returns `None` if the key doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `key` - The setting key to look up
    ///
    /// # Returns
    ///
    /// * `Ok(Some(value))` - Setting found with the given value
    /// * `Ok(None)` - Setting not found
    /// * `Err(...)` - Database query failed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gigi_auth::settings_manager::SettingsManager;
    /// # async fn example(manager: SettingsManager) -> anyhow::Result<(), sea_orm::DbErr> {
    /// if let Some(value) = manager.get("theme").await? {
    ///     println!("Theme: {}", value);
    /// } else {
    ///     println!("Theme not set");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, key: &str) -> Result<Option<String>, DbErr> {
        debug!("Getting setting: {}", key);

        let result = settings::Entity::find()
            .filter(settings::Column::Key.eq(key))
            .one(&self.db)
            .await?;

        Ok(result.map(|model| model.value))
    }

    /// Set a setting value
    ///
    /// Sets or updates a setting with the given key and value. If the key
    /// already exists, the value is updated. If it doesn't exist, a new
    /// row is inserted. The `updated_at` timestamp is always refreshed.
    ///
    /// # Arguments
    ///
    /// * `key` - The setting key to set
    /// * `value` - The value to store
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the operation was successful.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gigi_auth::settings_manager::SettingsManager;
    /// # async fn example(manager: SettingsManager) -> anyhow::Result<(), sea_orm::DbErr> {
    /// manager.set("theme", "dark").await?;
    /// manager.set("theme", "light").await?; // Updates existing
    /// # Ok(())
    /// # }
    /// ```
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
            // id is auto-generated by the database
            let new_setting = settings::ActiveModel {
                id: NotSet,
                key: Set(key.to_string()),
                value: Set(value.to_string()),
                updated_at: Set(now),
            };
            new_setting.insert(&self.db).await?;
        }

        info!("Setting '{}' updated successfully", key);
        Ok(())
    }

    /// Delete a setting by key
    ///
    /// Removes the setting with the given key from the database.
    ///
    /// # Arguments
    ///
    /// * `key` - The setting key to delete
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Setting was found and deleted
    /// * `Ok(false)` - Setting was not found (nothing to delete)
    /// * `Err(...)` - Database operation failed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gigi_auth::settings_manager::SettingsManager;
    /// # async fn example(manager: SettingsManager) -> anyhow::Result<(), sea_orm::DbErr> {
    /// let deleted = manager.delete("old_setting").await?;
    /// if deleted {
    ///     println!("Setting deleted");
    /// } else {
    ///     println!("Setting didn't exist");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, key: &str) -> Result<bool, DbErr> {
        debug!("Deleting setting: {}", key);

        let result = settings::Entity::delete_many()
            .filter(settings::Column::Key.eq(key))
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected > 0)
    }

    /// Check if a setting exists
    ///
    /// Checks whether a setting with the given key exists in the database.
    ///
    /// # Arguments
    ///
    /// * `key` - The setting key to check
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Setting exists
    /// * `Ok(false)` - Setting doesn't exist
    /// * `Err(...)` - Database query failed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gigi_auth::settings_manager::SettingsManager;
    /// # async fn example(manager: SettingsManager) -> anyhow::Result<(), sea_orm::DbErr> {
    /// if manager.exists("theme").await? {
    ///     println!("Theme is set");
    /// } else {
    ///     println!("Theme is not set");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn exists(&self, key: &str) -> Result<bool, DbErr> {
        let result = settings::Entity::find()
            .filter(settings::Column::Key.eq(key))
            .one(&self.db)
            .await?;

        Ok(result.is_some())
    }
}
