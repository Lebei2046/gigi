// Copyright 2024 Gigi Team.
//
// Comprehensive tests for ContactManager

use gigi_store::ContactManager;
use sea_orm::DatabaseConnection;
use tempfile::NamedTempFile;

async fn create_test_db(path: &tempfile::NamedTempFile) -> DatabaseConnection {
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
async fn test_add_and_get_contact() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = create_test_db(&temp_file).await;
    let manager = ContactManager::new(db);

    // Add a contact
    manager
        .add("12D3KooW...", "Alice")
        .await
        .expect("Failed to add contact");

    // Retrieve the contact
    let contact = manager
        .get("12D3KooW...")
        .await
        .expect("Failed to get contact");

    assert!(contact.is_some());
    let contact = contact.unwrap();
    assert_eq!(contact.name, "Alice");
    assert_eq!(contact.peer_id, "12D3KooW...");
}

#[tokio::test]
async fn test_update_contact_name() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = create_test_db(&temp_file).await;

    let manager = ContactManager::new(db);

    // Add contact
    manager
        .add("12D3KooW...", "Alice")
        .await
        .expect("Failed to add contact");

    // Update name
    manager
        .update_name("12D3KooW...", "Alice Smith")
        .await
        .expect("Failed to update contact name");

    // Verify update
    let contact = manager
        .get("12D3KooW...")
        .await
        .expect("Failed to get contact")
        .unwrap();

    assert_eq!(contact.name, "Alice Smith");
}

#[tokio::test]
async fn test_remove_contact() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = create_test_db(&temp_file).await;

    let manager = ContactManager::new(db);

    // Add contact
    manager
        .add("12D3KooW...", "Alice")
        .await
        .expect("Failed to add contact");

    // Remove contact
    manager
        .remove("12D3KooW...")
        .await
        .expect("Failed to remove contact");

    // Verify removal
    let contact = manager
        .get("12D3KooW...")
        .await
        .expect("Failed to get contact");

    assert!(contact.is_none(), "Contact should be removed");
}

#[tokio::test]
async fn test_get_all_contacts() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = create_test_db(&temp_file).await;

    let manager = ContactManager::new(db);

    // Add multiple contacts
    manager
        .add("peer1", "Alice")
        .await
        .expect("Failed to add contact");
    manager
        .add("peer2", "Bob")
        .await
        .expect("Failed to add contact");
    manager
        .add("peer3", "Charlie")
        .await
        .expect("Failed to add contact");

    // Get all contacts
    let contacts = manager.get_all().await.expect("Failed to get all contacts");

    assert_eq!(contacts.len(), 3);
    assert_eq!(contacts[0].name, "Alice");
    assert_eq!(contacts[1].name, "Bob");
    assert_eq!(contacts[2].name, "Charlie");
}

#[tokio::test]
async fn test_contact_exists() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = create_test_db(&temp_file).await;

    let manager = ContactManager::new(db);

    // Add contact
    manager
        .add("12D3KooW...", "Alice")
        .await
        .expect("Failed to add contact");

    // Check if exists
    let exists = manager
        .exists("12D3KooW...")
        .await
        .expect("Failed to check if contact exists");

    assert!(exists, "Contact should exist");

    // Check non-existent contact
    let not_exists = manager
        .exists("nonexistent")
        .await
        .expect("Failed to check if contact exists");

    assert!(!not_exists, "Non-existent contact should not exist");
}

#[tokio::test]
async fn test_duplicate_contact_handling() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = create_test_db(&temp_file).await;

    let manager = ContactManager::new(db);

    // Add contact twice
    manager
        .add("12D3KooW...", "Alice")
        .await
        .expect("Failed to add contact");

    // Second add should fail (unique constraint)
    let result = manager.add("12D3KooW...", "Alice").await;

    assert!(result.is_err(), "Adding duplicate contact should fail");
}
