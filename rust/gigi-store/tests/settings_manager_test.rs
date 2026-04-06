// Copyright 2024 Gigi Team.
//
// Comprehensive tests for SettingsManager

use gigi_store::settings_manager::{
    SettingsManager, ENCRYPTED_MNEMONIC_KEY, PASSWORD_HASH_KEY, PEER_ID_KEY,
};
use tempfile::NamedTempFile;

async fn create_test_db(path: &tempfile::NamedTempFile) -> sea_orm::DatabaseConnection {
    let db = sea_orm::Database::connect(&format!(
        "sqlite:{}?mode=rwc",
        path.path().to_str().unwrap().replace("\\", "/")
    ))
    .await
    .expect("Failed to connect to database");

    // Run migrations
    <gigi_store::migration::Migrator as gigi_store::migration::MigratorTrait>::up(&db, None)
        .await
        .expect("Failed to run migrations");

    db
}

#[tokio::test]
async fn test_set_and_get_setting() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = create_test_db(&temp_file).await;

    let manager = SettingsManager::new(db);

    // Set a setting
    manager
        .set("test_key", "test_value")
        .await
        .expect("Failed to set setting");

    // Get the setting
    let value = manager
        .get("test_key")
        .await
        .expect("Failed to get setting");

    assert!(value.is_some());
    assert_eq!(value.unwrap(), "test_value");
}

#[tokio::test]
async fn test_update_existing_setting() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = create_test_db(&temp_file).await;

    let manager = SettingsManager::new(db);

    // Set initial value
    manager
        .set("test_key", "initial_value")
        .await
        .expect("Failed to set setting");

    // Update the setting
    manager
        .set("test_key", "updated_value")
        .await
        .expect("Failed to update setting");

    // Get updated value
    let value = manager.get("test_key").await.unwrap().unwrap();
    assert_eq!(value, "updated_value");
}

#[tokio::test]
async fn test_get_nonexistent_setting() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = create_test_db(&temp_file).await;

    let manager = SettingsManager::new(db);

    // Get non-existent setting
    let value = manager
        .get("nonexistent_key")
        .await
        .expect("Failed to get setting");

    assert!(value.is_none());
}

#[tokio::test]
async fn test_delete_setting() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = create_test_db(&temp_file).await;

    let manager = SettingsManager::new(db);

    // Set a setting
    manager
        .set("test_key", "test_value")
        .await
        .expect("Failed to set setting");

    // Delete the setting
    let deleted = manager
        .delete("test_key")
        .await
        .expect("Failed to delete setting");

    assert!(deleted, "Setting should be deleted");

    // Verify it's gone
    let value = manager.get("test_key").await.unwrap();
    assert!(value.is_none());
}

#[tokio::test]
async fn test_delete_nonexistent_setting() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = create_test_db(&temp_file).await;

    let manager = SettingsManager::new(db);

    // Delete non-existent setting
    let deleted = manager
        .delete("nonexistent_key")
        .await
        .expect("Failed to delete setting");

    assert!(!deleted, "Non-existent setting should not be deleted");
}

#[tokio::test]
async fn test_get_all_settings() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = create_test_db(&temp_file).await;

    let manager = SettingsManager::new(db);

    // Set multiple settings
    manager
        .set("key1", "value1")
        .await
        .expect("Failed to set key1");
    manager
        .set("key2", "value2")
        .await
        .expect("Failed to set key2");
    manager
        .set("key3", "value3")
        .await
        .expect("Failed to set key3");

    // Get all settings
    let settings = manager.get_all().await.expect("Failed to get all settings");

    assert_eq!(settings.len(), 3);
    assert!(settings.contains(&("key1".to_string(), "value1".to_string())));
    assert!(settings.contains(&("key2".to_string(), "value2".to_string())));
    assert!(settings.contains(&("key3".to_string(), "value3".to_string())));
}

#[tokio::test]
async fn test_set_many_in_transaction() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = create_test_db(&temp_file).await;

    let manager = SettingsManager::new(db);

    // Set multiple settings in a transaction
    let items = vec![
        ("key1".to_string(), "value1".to_string()),
        ("key2".to_string(), "value2".to_string()),
        ("key3".to_string(), "value3".to_string()),
    ];

    manager
        .set_many(&items)
        .await
        .expect("Failed to set many settings");

    // Verify all settings were set
    assert_eq!(manager.get("key1").await.unwrap().unwrap(), "value1");
    assert_eq!(manager.get("key2").await.unwrap().unwrap(), "value2");
    assert_eq!(manager.get("key3").await.unwrap().unwrap(), "value3");
}

#[tokio::test]
async fn test_clear_all_settings() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = create_test_db(&temp_file).await;

    let manager = SettingsManager::new(db);

    // Set some settings
    manager.set("key1", "value1").await.unwrap();
    manager.set("key2", "value2").await.unwrap();

    // Clear all settings
    let count = manager.clear_all().await.expect("Failed to clear settings");

    assert_eq!(count, 2, "Should have deleted 2 settings");

    // Verify all are gone
    let settings = manager.get_all().await.unwrap();
    assert_eq!(settings.len(), 0);
}

#[tokio::test]
async fn test_setting_exists() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = create_test_db(&temp_file).await;

    let manager = SettingsManager::new(db);

    // Check non-existent setting
    assert!(!manager.exists("test_key").await.unwrap());

    // Set the setting
    manager.set("test_key", "test_value").await.unwrap();

    // Check if exists now
    assert!(manager.exists("test_key").await.unwrap());
}

#[tokio::test]
async fn test_predefined_keys() {
    assert_eq!(ENCRYPTED_MNEMONIC_KEY, "encrypted_mnemonic");
    assert_eq!(PEER_ID_KEY, "peer_id");
    assert_eq!(PASSWORD_HASH_KEY, "password_hash");
}
