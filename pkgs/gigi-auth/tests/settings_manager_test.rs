//! Tests for SettingsManager database operations
//!
//! These tests cover all SettingsManager operations:
//! - Getting, setting, deleting settings
//! - Checking existence
//! - Updating existing settings
//! - Handling edge cases

use gigi_auth::settings_manager::SettingsManager;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr, Statement};

/// Helper function to create an in-memory database for testing
async fn create_test_db() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect("sqlite::memory:").await?;

    // Create the settings table manually
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE TABLE IF NOT EXISTS settings (id INTEGER PRIMARY KEY AUTOINCREMENT, key TEXT UNIQUE NOT NULL, value TEXT NOT NULL, updated_at BIGINT NOT NULL)".to_string(),
    )).await?;

    Ok(db)
}

#[tokio::test]
async fn test_set_and_get_setting() {
    let db = create_test_db().await.unwrap();
    let manager = SettingsManager::new(db);

    // Set a setting
    manager.set("test_key", "test_value").await.unwrap();

    // Get the setting
    let value = manager.get("test_key").await.unwrap();

    assert_eq!(value, Some("test_value".to_string()));
}

#[tokio::test]
async fn test_get_nonexistent_setting() {
    let db = create_test_db().await.unwrap();
    let manager = SettingsManager::new(db);

    // Get a setting that doesn't exist
    let value = manager.get("nonexistent_key").await.unwrap();

    assert_eq!(value, None);
}

#[tokio::test]
async fn test_update_existing_setting() {
    let db = create_test_db().await.unwrap();
    let manager = SettingsManager::new(db);

    // Set initial value
    manager.set("config", "initial").await.unwrap();
    let value = manager.get("config").await.unwrap();
    assert_eq!(value, Some("initial".to_string()));

    // Update the setting
    manager.set("config", "updated").await.unwrap();
    let value = manager.get("config").await.unwrap();
    assert_eq!(value, Some("updated".to_string()));
}

#[tokio::test]
async fn test_delete_existing_setting() {
    let db = create_test_db().await.unwrap();
    let manager = SettingsManager::new(db);

    // Set a setting
    manager.set("temp", "data").await.unwrap();
    assert!(manager.exists("temp").await.unwrap());

    // Delete the setting
    let deleted = manager.delete("temp").await.unwrap();

    assert!(deleted);
    assert!(!manager.exists("temp").await.unwrap());
}

#[tokio::test]
async fn test_delete_nonexistent_setting() {
    let db = create_test_db().await.unwrap();
    let manager = SettingsManager::new(db);

    // Delete a setting that doesn't exist
    let deleted = manager.delete("nonexistent").await.unwrap();

    assert!(!deleted);
}

#[tokio::test]
async fn test_exists_true() {
    let db = create_test_db().await.unwrap();
    let manager = SettingsManager::new(db);

    // Set a setting
    manager.set("theme", "dark").await.unwrap();

    // Check if it exists
    let exists = manager.exists("theme").await.unwrap();

    assert!(exists);
}

#[tokio::test]
async fn test_exists_false() {
    let db = create_test_db().await.unwrap();
    let manager = SettingsManager::new(db);

    // Check if a nonexistent setting exists
    let exists = manager.exists("nonexistent").await.unwrap();

    assert!(!exists);
}

#[tokio::test]
async fn test_multiple_settings() {
    let db = create_test_db().await.unwrap();
    let manager = SettingsManager::new(db);

    // Set multiple settings
    manager.set("theme", "dark").await.unwrap();
    manager.set("language", "en").await.unwrap();
    manager.set("notifications", "enabled").await.unwrap();

    // Verify all exist
    assert!(manager.exists("theme").await.unwrap());
    assert!(manager.exists("language").await.unwrap());
    assert!(manager.exists("notifications").await.unwrap());

    // Verify values
    assert_eq!(
        manager.get("theme").await.unwrap(),
        Some("dark".to_string())
    );
    assert_eq!(
        manager.get("language").await.unwrap(),
        Some("en".to_string())
    );
    assert_eq!(
        manager.get("notifications").await.unwrap(),
        Some("enabled".to_string())
    );
}

#[tokio::test]
async fn test_empty_string_value() {
    let db = create_test_db().await.unwrap();
    let manager = SettingsManager::new(db);

    // Set a setting with empty string value
    manager.set("empty", "").await.unwrap();

    // Should be retrievable
    let value = manager.get("empty").await.unwrap();
    assert_eq!(value, Some("".to_string()));

    // Should still exist
    assert!(manager.exists("empty").await.unwrap());
}

#[tokio::test]
async fn test_long_value() {
    let db = create_test_db().await.unwrap();
    let manager = SettingsManager::new(db);

    // Set a setting with a long value (simulating large JSON)
    let long_value = "x".repeat(10000);
    manager.set("long_data", &long_value).await.unwrap();

    // Should be retrievable
    let value = manager.get("long_data").await.unwrap();
    assert_eq!(value, Some(long_value));
}

#[tokio::test]
async fn test_special_characters_in_value() {
    let db = create_test_db().await.unwrap();
    let manager = SettingsManager::new(db);

    // Set settings with special characters
    manager.set("json", "{\"key\":\"value\"}").await.unwrap();
    manager.set("unicode", "你好🌍").await.unwrap();
    manager
        .set("newlines", "line1\nline2\nline3")
        .await
        .unwrap();

    // Verify all are retrievable
    assert_eq!(
        manager.get("json").await.unwrap(),
        Some("{\"key\":\"value\"}".to_string())
    );
    assert_eq!(
        manager.get("unicode").await.unwrap(),
        Some("你好🌍".to_string())
    );
    assert_eq!(
        manager.get("newlines").await.unwrap(),
        Some("line1\nline2\nline3".to_string())
    );
}

#[tokio::test]
async fn test_delete_and_recreate() {
    let db = create_test_db().await.unwrap();
    let manager = SettingsManager::new(db);

    // Set a setting
    manager.set("test", "value1").await.unwrap();
    assert_eq!(
        manager.get("test").await.unwrap(),
        Some("value1".to_string())
    );

    // Delete it
    manager.delete("test").await.unwrap();
    assert_eq!(manager.get("test").await.unwrap(), None);

    // Recreate with different value
    manager.set("test", "value2").await.unwrap();
    assert_eq!(
        manager.get("test").await.unwrap(),
        Some("value2".to_string())
    );
}

#[tokio::test]
async fn test_persistence_across_managers() {
    let db = create_test_db().await.unwrap();

    // Create first manager and set a value
    let manager1 = SettingsManager::new(db.clone());
    manager1.set("shared", "data").await.unwrap();

    // Create second manager with same database
    let manager2 = SettingsManager::new(db);

    // Second manager should see the data
    let value = manager2.get("shared").await.unwrap();
    assert_eq!(value, Some("data".to_string()));
}
