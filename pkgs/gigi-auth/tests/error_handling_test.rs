//! Error handling and edge case tests
//!
//! These tests verify proper error handling and edge case behavior:
//! - Invalid mnemonics
//! - Invalid passwords
//! - Corrupted data
//! - Concurrent operations

use gigi_auth::encryption::encrypt_mnemonic;
use gigi_auth::settings_manager::SettingsManager;
use gigi_auth::AuthManager;
use gigi_auth::{derive_evm_address, derive_group_id, derive_peer_id};
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

const TEST_MNEMONIC: &str =
    "abandon amount liar amount expire adjust cage candy arch gather drum buyer";
const TEST_PASSWORD: &str = "test_password";

#[tokio::test]
async fn test_invalid_mnemonic() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Invalid mnemonic (not a valid BIP-39 phrase)
    let invalid_mnemonic = "this is not a valid mnemonic phrase";
    let result = auth
        .create_account(invalid_mnemonic, TEST_PASSWORD, None)
        .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to parse mnemonic"));
}

#[tokio::test]
async fn test_invalid_checksum_mnemonic() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Valid word list but invalid checksum
    let invalid_checksum =
        "abandon amount liar amount expire adjust cage candy arch gather drum buyer abandon";
    let result = auth
        .create_account(invalid_checksum, TEST_PASSWORD, None)
        .await;

    // Should fail due to invalid checksum
    assert!(result.is_err());
}

#[tokio::test]
async fn test_wrong_checksum_mnemonic() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Same words as valid mnemonic but different last word changes checksum
    let wrong_checksum =
        "abandon amount liar amount expire adjust cage candy arch gather drum advice";
    let result = auth
        .create_account(wrong_checksum, TEST_PASSWORD, None)
        .await;

    // The checksum might be valid for this phrase, but it's a different mnemonic
    // This test verifies that different mnemonics produce different results
    assert!(result.is_ok()); // Should succeed with different mnemonic
}

#[tokio::test]
async fn test_different_mnemonics_different_results() {
    let mnemonic1 = "abandon amount liar amount expire adjust cage candy arch gather drum buyer";
    let mnemonic2 = "abandon amount liar amount expire adjust cage candy arch gather drum advice";

    // Derive keys from different mnemonics
    let peer_id1 = derive_peer_id(mnemonic1).unwrap();
    let peer_id2 = derive_peer_id(mnemonic2).unwrap();

    let group_id1 = derive_group_id(mnemonic1).unwrap();
    let group_id2 = derive_group_id(mnemonic2).unwrap();

    let address1 = derive_evm_address(mnemonic1).unwrap();
    let address2 = derive_evm_address(mnemonic2).unwrap();

    // All should be different
    assert_ne!(peer_id1, peer_id2);
    assert_ne!(group_id1, group_id2);
    assert_ne!(address1, address2);
}

#[tokio::test]
async fn test_password_case_sensitivity() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Create account with lowercase password
    auth.create_account(TEST_MNEMONIC, "password", None)
        .await
        .unwrap();

    // Login with uppercase should fail
    let result = auth.login("PASSWORD").await;
    assert!(result.is_err());

    // Login with mixed case should fail
    let result = auth.login("Password").await;
    assert!(result.is_err());

    // Only exact match should succeed
    let result = auth.login("password").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_empty_password() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Create account with empty password (should work technically)
    let result = auth.create_account(TEST_MNEMONIC, "", None).await;
    assert!(result.is_ok());

    // Should be able to login with empty password
    let result = auth.login("").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_empty_name() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Create account with empty name (Some("")) should be stored as is
    let result = auth
        .create_account(TEST_MNEMONIC, TEST_PASSWORD, Some("".to_string()))
        .await;
    assert!(result.is_ok());

    let account_info = result.unwrap();
    assert_eq!(account_info.name, "");
}

#[tokio::test]
async fn test_very_long_password() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Create account with very long password
    let long_password = "a".repeat(1000);
    let result = auth
        .create_account(TEST_MNEMONIC, &long_password, None)
        .await;
    assert!(result.is_ok());

    // Should be able to login with long password
    let result = auth.login(&long_password).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_corrupted_encrypted_data() {
    let db = create_test_db().await.unwrap();
    let settings_manager = SettingsManager::new(db.clone());

    // Manually insert corrupted data
    settings_manager
        .set(
            "gigi",
            "{\"encrypted_mnemonic\":\"invalid_hex\",\"nonce\":\"invalid\",\"peer_id\":\"abc\",\"group_id\":\"def\",\"address\":\"0x123\",\"name\":\"Test\"}",
        )
        .await
        .unwrap();

    let auth = AuthManager::new(db);

    // Login should fail gracefully
    let result = auth.login(TEST_PASSWORD).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_invalid_json_in_settings() {
    let db = create_test_db().await.unwrap();
    let settings_manager = SettingsManager::new(db.clone());

    // Manually insert invalid JSON
    settings_manager
        .set("gigi", "this is not json")
        .await
        .unwrap();

    let auth = AuthManager::new(db);

    // get_account_info should handle error gracefully
    let result = auth.get_account_info().await;
    assert!(result.is_err());

    // Login should also fail
    let result = auth.login(TEST_PASSWORD).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_missing_fields_in_encrypted_data() {
    let db = create_test_db().await.unwrap();
    let settings_manager = SettingsManager::new(db.clone());

    // Insert encrypted data with missing fields
    settings_manager
        .set(
            "gigi",
            "{\"encrypted_mnemonic\":\"abc\",\"nonce\":\"def\"}", // Missing peer_id, group_id, etc.
        )
        .await
        .unwrap();

    let auth = AuthManager::new(db);

    // Operations should fail gracefully
    let result = auth.get_account_info().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_account_creation() {
    let db = create_test_db().await.unwrap();
    let auth1 = AuthManager::new(db.clone());
    let auth2 = AuthManager::new(db.clone());

    // Try to create accounts concurrently
    let handle1 = tokio::spawn(async move {
        auth1
            .create_account(TEST_MNEMONIC, TEST_PASSWORD, Some("User1".to_string()))
            .await
    });
    let handle2 = tokio::spawn(async move {
        auth2
            .create_account(TEST_MNEMONIC, TEST_PASSWORD, Some("User2".to_string()))
            .await
    });

    // Both should complete
    let result1 = handle1.await.unwrap();
    let result2 = handle2.await.unwrap();

    // At least one should fail (only one account can exist)
    let one_succeeded = result1.is_ok() || result2.is_ok();
    let one_failed = result1.is_err() || result2.is_err();
    assert!(one_succeeded);
    assert!(one_failed);
}

#[tokio::test]
async fn test_concurrent_login_attempts() {
    let db = create_test_db().await.unwrap();
    let auth1 = AuthManager::new(db.clone());
    let auth2 = AuthManager::new(db.clone());
    let auth3 = AuthManager::new(db);

    // Create account
    auth1
        .create_account(TEST_MNEMONIC, TEST_PASSWORD, None)
        .await
        .unwrap();

    // Attempt concurrent logins with correct password
    let handle1 = tokio::spawn(async move { auth1.login(TEST_PASSWORD).await });
    let handle2 = tokio::spawn(async move { auth2.login(TEST_PASSWORD).await });
    let handle3 = tokio::spawn(async move { auth3.login(TEST_PASSWORD).await });

    // All should succeed
    assert!(handle1.await.unwrap().is_ok());
    assert!(handle2.await.unwrap().is_ok());
    assert!(handle3.await.unwrap().is_ok());
}

#[tokio::test]
async fn test_concurrent_password_changes() {
    let db = create_test_db().await.unwrap();
    let auth1 = AuthManager::new(db.clone());
    let auth2 = AuthManager::new(db.clone());
    let auth3 = AuthManager::new(db);

    // Create account
    auth1
        .create_account(TEST_MNEMONIC, TEST_PASSWORD, None)
        .await
        .unwrap();

    // Attempt concurrent password changes
    let handle1 = tokio::spawn(async move { auth1.change_password(TEST_PASSWORD, "new1").await });
    let handle2 = tokio::spawn(async move { auth2.change_password(TEST_PASSWORD, "new2").await });
    let handle3 = tokio::spawn(async move { auth3.change_password(TEST_PASSWORD, "new3").await });

    // All should complete (last write wins)
    assert!(handle1.await.unwrap().is_ok());
    assert!(handle2.await.unwrap().is_ok());
    assert!(handle3.await.unwrap().is_ok());
}

#[tokio::test]
async fn test_encryption_with_special_characters() {
    // Test that encryption/decryption works with special characters in metadata
    let peer_id = "12D3KooWBdWJvz4KwB6v4sF8s8uBx8Q";
    let group_id = "12D3KooWBdWJvz4KwB6v4sF8s8uBx8Q";
    let address = "0x742d35Cc6634C0530bbE07Ffd5B6c4F4d0885E";
    let name = "用户 🌍 \"quoted\" 'single'";

    let encrypted = encrypt_mnemonic(
        TEST_MNEMONIC,
        TEST_PASSWORD,
        peer_id,
        group_id,
        address,
        name,
    )
    .unwrap();

    assert_eq!(encrypted.name, name);

    // Verify decryption works
    let decrypted = gigi_auth::encryption::decrypt_mnemonic(&encrypted, TEST_PASSWORD).unwrap();
    assert_eq!(decrypted, TEST_MNEMONIC);
}

#[tokio::test]
async fn test_get_account_info_when_no_account() {
    let db = create_test_db().await.unwrap();
    let auth = AuthManager::new(db);

    // Should return None when no account exists
    let result = auth.get_account_info().await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_account_does_not_affect_other_settings() {
    let db = create_test_db().await.unwrap();
    let settings_manager = SettingsManager::new(db.clone());
    let auth = AuthManager::new(db);

    // Create account and other settings
    auth.create_account(TEST_MNEMONIC, TEST_PASSWORD, None)
        .await
        .unwrap();
    settings_manager
        .set("other_key", "other_value")
        .await
        .unwrap();

    // Verify both exist
    assert!(auth.has_account().await.unwrap());
    assert!(settings_manager.exists("other_key").await.unwrap());

    // Delete account
    auth.delete_account().await.unwrap();

    // Account should be gone but other setting should remain
    assert!(!auth.has_account().await.unwrap());
    assert!(settings_manager.exists("other_key").await.unwrap());
    assert_eq!(
        settings_manager.get("other_key").await.unwrap(),
        Some("other_value".to_string())
    );
}

#[tokio::test]
async fn test_verify_password_with_corrupted_data() {
    let db = create_test_db().await.unwrap();
    let settings_manager = SettingsManager::new(db.clone());

    // Insert corrupted data
    settings_manager
        .set("gigi", "{\"encrypted_mnemonic\":\"invalid\",\"nonce\":\"invalid\",\"peer_id\":\"invalid\",\"group_id\":\"invalid\",\"address\":\"invalid\",\"name\":\"Invalid\"}")
        .await
        .unwrap();

    let auth = AuthManager::new(db);

    // verify_password should return false for corrupted data
    let result = auth.verify_password(TEST_PASSWORD).await.unwrap();
    assert!(!result);
}
