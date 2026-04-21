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
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, NotSet,
    QueryFilter, Set, Statement,
};
use gigi_logging::{debug, info};

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

    /// Create the groups table if it doesn't exist
    ///
    /// The groups table stores group information with the following schema:
    /// - group_id: Primary key (String)
    /// - name: Group display name (String)
    /// - joined: Whether the user has joined this group (bool)
    /// - created_at: Timestamp in milliseconds (i64)
    ///
    /// This should be called during account creation to ensure the table exists.
    pub async fn create_groups_table(&self) -> Result<(), DbErr> {
        debug!("Creating groups table if it doesn't exist");

        let create_table_sql = r#"
            CREATE TABLE IF NOT EXISTS groups (
                group_id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                joined INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL
            )
        "#;

        self.db
            .execute(Statement::from_string(
                self.db.get_database_backend(),
                create_table_sql.to_string(),
            ))
            .await?;

        info!("Groups table ready");
        Ok(())
    }

    /// Insert or update a group in the groups table
    ///
    /// # Arguments
    ///
    /// * `group_id` - The unique group identifier
    /// * `name` - The group display name
    /// * `joined` - Whether the user has joined this group
    pub async fn upsert_group(
        &self,
        group_id: &str,
        name: &str,
        joined: bool,
    ) -> Result<(), DbErr> {
        debug!("Upserting group: {}", group_id);

        let now = chrono::Utc::now().timestamp_millis();
        let joined_int = if joined { 1 } else { 0 };

        let insert_sql = format!(
            r#"INSERT INTO groups (group_id, name, joined, created_at) VALUES ('{}', '{}', {}, {})"#,
            group_id, name, joined_int, now
        );

        self.db
            .execute(Statement::from_string(
                self.db.get_database_backend(),
                insert_sql,
            ))
            .await?;

        info!("Group '{}' upserted successfully", group_id);
        Ok(())
    }

    /// Clear all groups from the groups table
    ///
    /// This method deletes all records from the groups table. It's called
    /// during account deletion to ensure clean state.
    pub async fn clear_groups(&self) -> Result<(), DbErr> {
        debug!("Clearing all groups");

        let delete_sql = "DELETE FROM groups";

        self.db
            .execute(Statement::from_string(
                self.db.get_database_backend(),
                delete_sql.to_string(),
            ))
            .await?;

        info!("Groups table cleared");
        Ok(())
    }

    /// Get a group by group_id
    pub async fn get_group(&self, group_id: &str) -> Result<Option<GroupInfo>, DbErr> {
        debug!("Getting group: {}", group_id);

        let query_sql = format!(
            "SELECT group_id, name, joined, created_at FROM groups WHERE group_id = '{}'",
            group_id
        );

        let result = self
            .db
            .query_one(Statement::from_string(
                self.db.get_database_backend(),
                query_sql,
            ))
            .await?;

        Ok(result.map(|row| GroupInfo {
            group_id: row.try_get("", "group_id").unwrap_or_default(),
            name: row.try_get("", "name").unwrap_or_default(),
            joined: row.try_get::<i64>("", "joined").unwrap_or(0) == 1,
            created_at: row.try_get("", "created_at").unwrap_or(0),
        }))
    }

    /// Get all groups
    pub async fn get_all_groups(&self) -> Result<Vec<GroupInfo>, DbErr> {
        debug!("Getting all groups");

        let query_sql =
            "SELECT group_id, name, joined, created_at FROM groups ORDER BY created_at DESC";

        let result = self
            .db
            .query_all(Statement::from_string(
                self.db.get_database_backend(),
                query_sql.to_string(),
            ))
            .await?;

        let groups = result
            .into_iter()
            .filter_map(|row| {
                Some(GroupInfo {
                    group_id: row.try_get("", "group_id").ok()?,
                    name: row.try_get("", "name").ok()?,
                    joined: row.try_get::<i64>("", "joined").ok()? == 1,
                    created_at: row.try_get("", "created_at").ok()?,
                })
            })
            .collect();

        Ok(groups)
    }

    /// Get all joined groups
    pub async fn get_joined_groups(&self) -> Result<Vec<GroupInfo>, DbErr> {
        debug!("Getting joined groups");

        let query_sql = "SELECT group_id, name, joined, created_at FROM groups WHERE joined = 1 ORDER BY created_at DESC";

        let result = self
            .db
            .query_all(Statement::from_string(
                self.db.get_database_backend(),
                query_sql.to_string(),
            ))
            .await?;

        let groups = result
            .into_iter()
            .filter_map(|row| {
                Some(GroupInfo {
                    group_id: row.try_get("", "group_id").ok()?,
                    name: row.try_get("", "name").ok()?,
                    joined: true,
                    created_at: row.try_get("", "created_at").ok()?,
                })
            })
            .collect();

        Ok(groups)
    }

    /// Update group join status
    pub async fn update_group_join_status(
        &self,
        group_id: &str,
        joined: bool,
    ) -> Result<bool, DbErr> {
        debug!("Updating join status for group: {} -> {}", group_id, joined);

        let joined_int = if joined { 1 } else { 0 };
        let update_sql = format!(
            "UPDATE groups SET joined = {} WHERE group_id = '{}'",
            joined_int, group_id
        );

        let result = self
            .db
            .execute(Statement::from_string(
                self.db.get_database_backend(),
                update_sql,
            ))
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Update group name
    pub async fn update_group_name(&self, group_id: &str, name: &str) -> Result<bool, DbErr> {
        debug!("Updating name for group: {}", group_id);

        let update_sql = format!(
            "UPDATE groups SET name = '{}' WHERE group_id = '{}'",
            name, group_id
        );

        let result = self
            .db
            .execute(Statement::from_string(
                self.db.get_database_backend(),
                update_sql,
            ))
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete a group
    pub async fn delete_group(&self, group_id: &str) -> Result<bool, DbErr> {
        debug!("Deleting group: {}", group_id);

        let delete_sql = format!("DELETE FROM groups WHERE group_id = '{}'", group_id);

        let result = self
            .db
            .execute(Statement::from_string(
                self.db.get_database_backend(),
                delete_sql,
            ))
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

/// Group information stored in the database
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GroupInfo {
    pub group_id: String,
    pub name: String,
    pub joined: bool,
    pub created_at: i64,
}
