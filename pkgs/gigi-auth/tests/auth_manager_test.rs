//! Integration tests for AuthManager
//!
//! These tests cover the complete authentication workflow including:
//! - Account creation
//! - Login and password verification
//! - Password changes
//! - Account deletion
//! - Error handling and edge cases

use gigi_auth::AuthManager;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr, Statement};

/// Test mnemonic for all tests
const TEST_MNEMONIC: &str =
    "abandon amount liar amount expire adjust cage candy arch gather drum buyer";

/// Test password
const TEST_PASSWORD: &str = "test_secure_password_123";

/// Helper function to create an in-memory database for testing
async fn create_test_db() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect("sqlite::memory:").await?;

    // Create the settings table manually
    use sea_orm::{ConnectionTrait, Statement};
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE TABLE IF NOT EXISTS settings (id INTEGER PRIMARY KEY AUTOINCREMENT, key TEXT UNIQUE NOT NULL, value TEXT NOT NULL, updated_at BIGINT NOT NULL)".to_string(),
    )).await?;

    Ok(db)
}

#[tokio::test]
async fn test_create_account() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Initially, no account should exist
    assert!(!auth.has_account().await.unwrap());

    // Create an account
    let account_info = auth
        .create_account(TEST_MNEMONIC, TEST_PASSWORD, Some("TestUser".to_string()))
        .await
        .unwrap();

    // Verify account now exists
    assert!(auth.has_account().await.unwrap());

    // Verify account details
    assert_eq!(account_info.name, "TestUser");
    assert!(account_info.address.starts_with("0x"));
    assert!(account_info.peer_id.len() > 20);
    assert!(account_info.group_id.len() > 20);
}

#[tokio::test]
async fn test_create_account_with_default_name() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Create account without providing a name
    let account_info = auth
        .create_account(TEST_MNEMONIC, TEST_PASSWORD, None)
        .await
        .unwrap();

    // Should use default name "User"
    assert_eq!(account_info.name, "User");
}

#[tokio::test]
async fn test_create_duplicate_account() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Create first account
    auth.create_account(TEST_MNEMONIC, TEST_PASSWORD, Some("User1".to_string()))
        .await
        .unwrap();

    // Attempt to create a second account should fail
    let result = auth
        .create_account(
            TEST_MNEMONIC,
            "different_password",
            Some("User2".to_string()),
        )
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[tokio::test]
async fn test_login_success() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Create account
    let created_info = auth
        .create_account(TEST_MNEMONIC, TEST_PASSWORD, Some("Alice".to_string()))
        .await
        .unwrap();

    // Login with correct password
    let login_result = auth.login(TEST_PASSWORD).await.unwrap();

    // Verify account info matches
    assert_eq!(login_result.account_info.address, created_info.address);
    assert_eq!(login_result.account_info.peer_id, created_info.peer_id);
    assert_eq!(login_result.account_info.group_id, created_info.group_id);
    assert_eq!(login_result.account_info.name, "Alice");

    // Verify private key is returned
    assert!(!login_result.private_key.is_empty());
    assert_eq!(login_result.private_key.len(), 64); // 32 bytes = 64 hex chars
}

#[tokio::test]
async fn test_login_wrong_password() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Create account
    auth.create_account(TEST_MNEMONIC, TEST_PASSWORD, None)
        .await
        .unwrap();

    // Attempt to login with wrong password should fail
    let result = auth.login("wrong_password").await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid password"));
}

#[tokio::test]
async fn test_login_no_account() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Attempt to login when no account exists should fail
    let result = auth.login(TEST_PASSWORD).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_verify_password_correct() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Create account
    auth.create_account(TEST_MNEMONIC, TEST_PASSWORD, None)
        .await
        .unwrap();

    // Verify correct password
    let is_valid = auth.verify_password(TEST_PASSWORD).await.unwrap();
    assert!(is_valid);
}

#[tokio::test]
async fn test_verify_password_incorrect() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Create account
    auth.create_account(TEST_MNEMONIC, TEST_PASSWORD, None)
        .await
        .unwrap();

    // Verify incorrect password
    let is_valid = auth.verify_password("wrong_password").await.unwrap();
    assert!(!is_valid);
}

#[tokio::test]
async fn test_verify_password_no_account() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Verify password when no account exists
    let is_valid = auth.verify_password(TEST_PASSWORD).await.unwrap();
    assert!(!is_valid);
}

#[tokio::test]
async fn test_change_password() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Create account
    auth.create_account(TEST_MNEMONIC, TEST_PASSWORD, Some("User".to_string()))
        .await
        .unwrap();

    // Change password
    let new_password = "new_secure_password_456";
    auth.change_password(TEST_PASSWORD, new_password)
        .await
        .unwrap();

    // Should be able to login with new password
    let result = auth.login(new_password).await;
    assert!(result.is_ok());

    // Should NOT be able to login with old password
    let result = auth.login(TEST_PASSWORD).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_change_password_wrong_old_password() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Create account
    auth.create_account(TEST_MNEMONIC, TEST_PASSWORD, None)
        .await
        .unwrap();

    // Attempt to change password with wrong old password should fail
    let result = auth.change_password("wrong_password", "new_password").await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid old password"));
}

#[tokio::test]
async fn test_get_account_info() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Initially, should return None
    assert!(auth.get_account_info().await.unwrap().is_none());

    // Create account
    let created_info = auth
        .create_account(TEST_MNEMONIC, TEST_PASSWORD, Some("Bob".to_string()))
        .await
        .unwrap();

    // Get account info
    let retrieved_info = auth.get_account_info().await.unwrap().unwrap();

    // Verify it matches created info
    assert_eq!(retrieved_info.address, created_info.address);
    assert_eq!(retrieved_info.peer_id, created_info.peer_id);
    assert_eq!(retrieved_info.group_id, created_info.group_id);
    assert_eq!(retrieved_info.name, "Bob");
}

#[tokio::test]
async fn test_delete_account() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Create account
    auth.create_account(TEST_MNEMONIC, TEST_PASSWORD, None)
        .await
        .unwrap();

    // Verify account exists
    assert!(auth.has_account().await.unwrap());

    // Delete account
    auth.delete_account().await.unwrap();

    // Verify account no longer exists
    assert!(!auth.has_account().await.unwrap());

    // Should not be able to login
    let result = auth.login(TEST_PASSWORD).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_account_no_account() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Attempt to delete when no account exists should succeed (no-op)
    let result = auth.delete_account().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_complete_workflow() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // 1. Check no account exists
    assert!(!auth.has_account().await.unwrap());

    // 2. Create account
    let account = auth
        .create_account(TEST_MNEMONIC, TEST_PASSWORD, Some("Charlie".to_string()))
        .await
        .unwrap();
    assert_eq!(account.name, "Charlie");
    assert!(auth.has_account().await.unwrap());

    // 3. Verify password
    assert!(auth.verify_password(TEST_PASSWORD).await.unwrap());

    // 4. Login
    let login_result = auth.login(TEST_PASSWORD).await.unwrap();
    assert_eq!(login_result.account_info.name, "Charlie");
    assert!(!login_result.private_key.is_empty());

    // 5. Change password
    auth.change_password(TEST_PASSWORD, "new_password")
        .await
        .unwrap();

    // 6. Verify new password works
    assert!(auth.verify_password("new_password").await.unwrap());
    assert!(!auth.verify_password(TEST_PASSWORD).await.unwrap());

    // 7. Get account info
    let info = auth.get_account_info().await.unwrap().unwrap();
    assert_eq!(info.name, "Charlie");

    // 8. Delete account
    auth.delete_account().await.unwrap();
    assert!(!auth.has_account().await.unwrap());
}

#[tokio::test]
async fn test_account_persistence() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db.clone());

    // Create account
    let original_info = auth
        .create_account(TEST_MNEMONIC, TEST_PASSWORD, Some("Diana".to_string()))
        .await
        .unwrap();

    // Create new auth manager with same database
    let auth2 = AuthManager::new(db);

    // Account should still exist
    assert!(auth2.has_account().await.unwrap());

    // Login should work with new auth manager
    let login_result = auth2.login(TEST_PASSWORD).await.unwrap();
    assert_eq!(login_result.account_info.name, "Diana");

    // Account info should match
    assert_eq!(login_result.account_info.address, original_info.address);
    assert_eq!(login_result.account_info.peer_id, original_info.peer_id);
}
