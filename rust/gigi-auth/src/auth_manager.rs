//! Authentication manager for handling user accounts and authentication
//!
//! This module provides the central `AuthManager` struct that orchestrates all account operations
//! including creation, login, password changes, and account deletion. It uses encrypted mnemonic
//! storage with ChaCha20-Poly1305 encryption and derives multiple cryptographic identities from
//! a single BIP-39 mnemonic using BIP-32 hierarchical derivation.
//!
//! # Authentication Flow
//!
//! The authentication flow follows a secure "decrypt-and-verify" approach:
//!
//! 1. **Account Creation**:
//!    - User provides a BIP-39 mnemonic and password
//!    - Derive peer_id, group_id, and EVM address from the mnemonic
//!    - Encrypt the mnemonic using ChaCha20-Poly1305 with the password
//!    - Store encrypted data with metadata in the database
//!
//! 2. **Login**:
//!    - Retrieve encrypted mnemonic from database
//!    - Attempt to decrypt with provided password
//!    - Derive peer_id from decrypted mnemonic
//!    - Compare derived peer_id with stored peer_id (prevents data corruption)
//!    - Return account info and private key if successful
//!
//! # Security Considerations
//!
//! - **No Password Hashing**: Passwords are used directly for encryption/decryption.
//!   This means we never store a password hash that could be attacked.
//! - **Decrypt-and-Verify**: Password verification happens through successful decryption,
//!   not through password comparison.
//! - **Peer ID Verification**: The derived peer_id must match the stored peer_id,
//!   ensuring data integrity and preventing corruption.
//! - **Single Source of Truth**: The mnemonic is the master key from which all other
//!   keys are derived deterministically.
//!
//! # Key Derivation Paths
//!
//! All keys are derived from a single BIP-39 mnemonic using BIP-32 paths:
//!
//! | Purpose | Key Type | Derivation Path |
//! |---------|----------|----------------|
//! | Peer ID (libp2p) | Ed25519 | `m/44'/60'/2'/0/0` |
//! | Group ID | Ed25519 | `m/44'/60'/1'/0/0` |
//! | EVM Address | Secp256k1 | `m/44'/60'/0'/0/0` |
//! | Peer Private Key | Ed25519 | `m/44'/60'/2'/0/0` |
//!
//! # Example Usage
//!
//! ```no_run
//! use gigi_auth::AuthManager;
//! use sea_orm::Database;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create database connection
//! let db = Database::connect("sqlite::memory:").await?;
//!
//! // Create auth manager
//! let auth = AuthManager::new(db);
//!
//! // Create a new account
//! let mnemonic = "abandon amount liar amount expire adjust cage candy arch gather drum buyer";
//! let password = "my_secure_password";
//! let account_info = auth.create_account(mnemonic, password, Some("Alice".to_string())).await?;
//!
//! // Login
//! let login_result = auth.login(password).await?;
//! println!("Logged in as: {}", login_result.account_info.name);
//!
//! // Change password
//! auth.change_password(password, "new_secure_password").await?;
//!
//! // Delete account
//! auth.delete_account().await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::encryption::EncryptedAccountData;
use crate::key_derivation;

/// Account information (public - doesn't contain sensitive mnemonic)
///
/// This struct contains all publicly-visible account information. It can be safely
/// shared or displayed without exposing sensitive cryptographic material like the
/// mnemonic or private keys.
///
/// # Fields
///
/// - `address`: EVM-compatible wallet address (0x-prefixed, 42 characters)
/// - `peer_id`: libp2p peer identifier for P2P network operations
/// - `group_id`: Unique identifier for P2P group membership
/// - `name`: User-displayable account name
///
/// # Example
///
/// ```no_run
/// # use gigi_auth::AccountInfo;
/// let info = AccountInfo {
///     address: "0x742d35Cc6634C0530bbE07Ffd5B6c4F4d0885E".to_string(),
///     peer_id: "12D3KooWBdWJvz4KwB6v4sF8s8uBx8Q".to_string(),
///     group_id: "12D3KooWBdWJvz4KwB6v4sF8s8uBx8Q".to_string(),
///     name: "Alice".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub address: String,
    pub peer_id: String,
    pub group_id: String,
    pub name: String,
}

/// Login result containing account info and private key
///
/// This struct is returned upon successful authentication and contains both
/// public account information and the sensitive Ed25519 private key required
/// for libp2p P2P operations.
///
/// # Security Warning
///
/// The `private_key` field contains sensitive cryptographic material. It should
/// be stored securely in memory and never logged, written to insecure storage,
/// or transmitted over unencrypted channels.
///
/// # Fields
///
/// - `account_info`: Public account information (see [`AccountInfo`])
/// - `private_key`: Ed25519 private key (64 hex characters = 32 bytes) for libp2p
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResult {
    pub account_info: AccountInfo,
    pub private_key: String,
}

/// Authentication manager
///
/// The `AuthManager` is the central component for account management. It handles
/// all account operations including creation, login, password changes, and deletion.
/// It uses encrypted mnemonic storage and derives multiple cryptographic identities
/// from a single BIP-39 mnemonic.
///
/// # Thread Safety
///
/// `AuthManager` can be safely shared across threads if the underlying database
/// connection supports concurrent access (e.g., via connection pooling).
///
/// # Example
///
/// ```no_run
/// use gigi_auth::AuthManager;
/// use sea_orm::Database;
///
/// # async fn example() -> anyhow::Result<()> {
/// let db = Database::connect("sqlite::memory:").await?;
/// let auth = AuthManager::new(db);
/// # Ok(())
/// # }
/// ```
pub struct AuthManager {
    #[allow(dead_code)]
    db: DatabaseConnection,
    settings_manager: crate::settings_manager::SettingsManager,
}

impl AuthManager {
    /// Create a new auth manager
    ///
    /// Creates a new `AuthManager` instance with the provided database connection.
    /// The database connection is cloned for use by the settings manager.
    ///
    /// # Arguments
    ///
    /// * `db` - A Sea-ORM database connection (e.g., SQLite, PostgreSQL, MySQL)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gigi_auth::AuthManager;
    /// use sea_orm::Database;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let db = Database::connect("sqlite::memory:").await?;
    /// let auth = AuthManager::new(db);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(db: DatabaseConnection) -> Self {
        let settings_manager = crate::settings_manager::SettingsManager::new(db.clone());
        Self {
            db,
            settings_manager,
        }
    }

    /// Check if an account exists
    ///
    /// Returns `true` if an account has been created and stored in the database,
    /// `false` otherwise. This is useful for checking whether the user needs to
    /// create an account or can log in with an existing one.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Account exists
    /// * `Ok(false)` - No account exists
    /// * `Err(...)` - Database query failed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gigi_auth::AuthManager;
    /// # async fn example(auth: AuthManager) -> anyhow::Result<()> {
    /// if auth.has_account().await? {
    ///     println!("Account exists, showing login screen");
    /// } else {
    ///     println!("No account found, showing registration screen");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn has_account(&self) -> Result<bool> {
        debug!("Checking if account exists");

        let has_mnemonic = self
            .settings_manager
            .exists(crate::settings_manager::GIGI_KEY)
            .await
            .context("Failed to check for mnemonic")?;

        Ok(has_mnemonic)
    }

    /// Create a new account with mnemonic
    ///
    /// Creates a new user account using a BIP-39 mnemonic and password. The mnemonic
    /// is encrypted with the password using ChaCha20-Poly1305 and stored in the database.
    /// All cryptographic identities (peer_id, group_id, EVM address) are derived from
    /// the mnemonic using BIP-32 paths.
    ///
    /// # Arguments
    ///
    /// * `mnemonic` - A valid BIP-39 mnemonic phrase (12 or 24 words)
    /// * `password` - Password to encrypt the mnemonic with (stored in encrypted form only)
    /// * `name` - Optional display name for the account (defaults to "User" if not provided)
    ///
    /// # Returns
    ///
    /// Returns `Ok(AccountInfo)` containing the public account information upon success.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - An account already exists
    /// - The mnemonic is invalid or malformed
    /// - Key derivation fails
    /// - Encryption fails
    /// - Database storage fails
    ///
    /// # Security Note
    ///
    /// The mnemonic is encrypted using ChaCha20-Poly1305 with the provided password.
    /// The password is never stored in plaintext - it's only used to derive the
    /// encryption key. Without the correct password, the mnemonic cannot be recovered.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gigi_auth::AuthManager;
    /// # async fn example(auth: AuthManager) -> anyhow::Result<()> {
    /// let mnemonic = "abandon amount liar amount expire adjust cage candy arch gather drum buyer";
    /// let password = "my_secure_password";
    /// let account = auth.create_account(mnemonic, password, Some("Alice".to_string())).await?;
    /// println!("Created account for: {}", account.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_account(
        &self,
        mnemonic: &str,
        password: &str,
        name: Option<String>,
    ) -> Result<AccountInfo> {
        info!("Creating new account");

        // Check if account already exists
        if self.has_account().await? {
            return Err(anyhow::anyhow!("Account already exists"));
        }

        // Derive keys from mnemonic
        let peer_id = key_derivation::derive_peer_id(mnemonic)?;
        let group_id = key_derivation::derive_group_id(mnemonic)?;
        let address = key_derivation::derive_evm_address(mnemonic)?;

        // Store name if provided, otherwise use default
        let name = name.unwrap_or_else(|| "User".to_string());

        // Encrypt mnemonic with password
        let encrypted_data = crate::encryption::encrypt_mnemonic(
            mnemonic, password, &peer_id, &group_id, &address, &name,
        )?;

        // Store encrypted account data (contains all account info)
        self.settings_manager
            .set(
                crate::settings_manager::GIGI_KEY,
                &serde_json::to_string(&encrypted_data)
                    .context("Failed to serialize encrypted data")?,
            )
            .await
            .context("Failed to store encrypted mnemonic")?;

        info!("Account created successfully for peer_id: {}", peer_id);

        Ok(AccountInfo {
            address,
            peer_id,
            group_id,
            name,
        })
    }

    /// Login with password
    ///
    /// Authenticates a user by attempting to decrypt the stored encrypted mnemonic
    /// with the provided password. Uses a "decrypt-and-verify" approach: the password
    /// is correct only if decryption succeeds AND the derived peer_id matches the
    /// stored peer_id (preventing data corruption).
    ///
    /// # Arguments
    ///
    /// * `password` - The password used during account creation
    ///
    /// # Returns
    ///
    /// Returns `Ok(LoginResult)` containing:
    /// - `account_info`: Public account information
    /// - `private_key`: Ed25519 private key for libp2p P2P operations
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No account exists
    /// - The password is incorrect
    /// - The stored data is corrupted or tampered with
    /// - Key derivation fails
    ///
    /// # Security Details
    ///
    /// The login process:
    /// 1. Retrieve encrypted mnemonic from database
    /// 2. Attempt to decrypt with the password (ChaCha20-Poly1305)
    /// 3. Derive peer_id from decrypted mnemonic
    /// 4. Compare derived peer_id with stored peer_id
    /// 5. If they match, password is correct and data is intact
    ///
    /// The peer_id verification step ensures that:
    /// - The password is correct (decryption succeeded)
    /// - The data hasn't been corrupted (derived peer_id matches)
    /// - The data hasn't been tampered with (Poly1305 authentication tag)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gigi_auth::AuthManager;
    /// # async fn example(auth: AuthManager) -> anyhow::Result<()> {
    /// let password = "my_secure_password";
    /// let result = auth.login(password).await?;
    /// println!("Welcome, {}!", result.account_info.name);
    /// println!("Your peer ID: {}", result.account_info.peer_id);
    /// // Use result.private_key for libp2p operations
    /// # Ok(())
    /// # }
    /// ```
    pub async fn login(&self, password: &str) -> Result<LoginResult> {
        info!("Attempting login");

        // Get encrypted data
        let encrypted_data_str = self
            .settings_manager
            .get(crate::settings_manager::GIGI_KEY)
            .await?
            .context("Encrypted mnemonic not found")?;

        let encrypted_data: EncryptedAccountData = serde_json::from_str(&encrypted_data_str)
            .context("Failed to deserialize encrypted data")?;

        // Try to decrypt mnemonic with password
        let mnemonic = match crate::encryption::decrypt_mnemonic(&encrypted_data, password) {
            Ok(mnemonic) => mnemonic,
            Err(_) => {
                warn!("Password decryption failed");
                return Err(anyhow::anyhow!("Invalid password"));
            }
        };

        // Derive peer_id from decrypted mnemonic
        let derived_peer_id = key_derivation::derive_peer_id(&mnemonic)?;

        // Verify derived peer_id matches stored peer_id
        // This ensures data integrity and prevents corruption
        if derived_peer_id != encrypted_data.peer_id {
            warn!(
                "Peer ID mismatch: derived={} != stored={}",
                derived_peer_id, encrypted_data.peer_id
            );
            return Err(anyhow::anyhow!("Invalid password"));
        }

        // Derive group_id and private keys
        let group_id = key_derivation::derive_group_id(&mnemonic)?;
        let address = encrypted_data.address.clone();
        let private_key = key_derivation::derive_peer_private_key(&mnemonic)?;

        info!("Login successful for peer_id: {}", derived_peer_id);

        Ok(LoginResult {
            account_info: AccountInfo {
                address,
                peer_id: derived_peer_id,
                group_id,
                name: encrypted_data.name,
            },
            private_key,
        })
    }

    /// Get account info without exposing mnemonic
    ///
    /// Retrieves public account information without requiring authentication.
    /// This is useful for displaying user profile information, checking if an
    /// account exists, or getting basic account details.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(AccountInfo))` - Account exists with the provided information
    /// * `Ok(None)` - No account exists
    /// * `Err(...)` - Database query or deserialization failed
    ///
    /// # Security Note
    ///
    /// This method does NOT expose sensitive data like the mnemonic or private keys.
    /// It only returns public information (address, peer_id, group_id, name) that
    /// is safe to display or share.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gigi_auth::AuthManager;
    /// # async fn example(auth: AuthManager) -> anyhow::Result<()> {
    /// if let Some(info) = auth.get_account_info().await? {
    ///     println!("Account: {}", info.name);
    ///     println!("Address: {}", info.address);
    ///     println!("Peer ID: {}", info.peer_id);
    /// } else {
    ///     println!("No account found");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_account_info(&self) -> Result<Option<AccountInfo>> {
        debug!("Getting account info");

        if !self.has_account().await? {
            return Ok(None);
        }

        let encrypted_data_str = self
            .settings_manager
            .get(crate::settings_manager::GIGI_KEY)
            .await?
            .context("Encrypted mnemonic not found")?;

        let encrypted_data: EncryptedAccountData = serde_json::from_str(&encrypted_data_str)
            .context("Failed to deserialize encrypted data")?;

        Ok(Some(AccountInfo {
            address: encrypted_data.address,
            peer_id: encrypted_data.peer_id,
            group_id: encrypted_data.group_id,
            name: encrypted_data.name,
        }))
    }

    /// Change password
    ///
    /// Changes the account password by re-encrypting the stored mnemonic with
    /// a new password. The old password must be correct to proceed, and all
    /// derived keys (peer_id, group_id, address) remain unchanged.
    ///
    /// # Arguments
    ///
    /// * `old_password` - The current password for verification
    /// * `new_password` - The new password to use for encryption
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the password was successfully changed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No account exists
    /// - The old password is incorrect
    /// - Re-encryption with the new password fails
    /// - Database update fails
    ///
    /// # Important Notes
    ///
    /// - Changing the password does NOT change any derived keys (peer_id, group_id, address)
    /// - The mnemonic itself remains unchanged; only its encryption changes
    /// - The user can log in with the new password immediately after a successful change
    /// - All account data (addresses, IDs) remain valid after password change
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gigi_auth::AuthManager;
    /// # async fn example(auth: AuthManager) -> anyhow::Result<()> {
    /// let old_password = "my_secure_password";
    /// let new_password = "even_more_secure_password";
    /// auth.change_password(old_password, new_password).await?;
    /// println!("Password changed successfully");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn change_password(&self, old_password: &str, new_password: &str) -> Result<()> {
        info!("Changing password");

        // Get encrypted data
        let encrypted_data_str = self
            .settings_manager
            .get(crate::settings_manager::GIGI_KEY)
            .await?
            .context("Encrypted mnemonic not found")?;

        let encrypted_data: EncryptedAccountData = serde_json::from_str(&encrypted_data_str)
            .context("Failed to deserialize encrypted data")?;

        // Try to decrypt with old password
        let mnemonic = match crate::encryption::decrypt_mnemonic(&encrypted_data, old_password) {
            Ok(mnemonic) => mnemonic,
            Err(_) => {
                warn!("Old password decryption failed");
                return Err(anyhow::anyhow!("Invalid old password"));
            }
        };

        // Verify derived peer_id matches stored peer_id
        let derived_peer_id = key_derivation::derive_peer_id(&mnemonic)?;
        if derived_peer_id != encrypted_data.peer_id {
            return Err(anyhow::anyhow!("Invalid old password"));
        }

        // Re-encrypt with new password
        let new_encrypted_data = crate::encryption::encrypt_mnemonic(
            &mnemonic,
            new_password,
            &encrypted_data.peer_id,
            &encrypted_data.group_id,
            &encrypted_data.address,
            &encrypted_data.name,
        )?;

        // Store new encrypted data
        self.settings_manager
            .set(
                crate::settings_manager::GIGI_KEY,
                &serde_json::to_string(&new_encrypted_data)
                    .context("Failed to serialize encrypted data")?,
            )
            .await
            .context("Failed to update encrypted mnemonic")?;

        info!("Password changed successfully");
        Ok(())
    }

    /// Delete account and all related data
    ///
    /// Permanently deletes the account by removing the encrypted mnemonic from
    /// the database. This operation is irreversible - all account data including
    /// the mnemonic is lost and cannot be recovered.
    ///
    /// # Warning
    ///
    /// **This operation is irreversible!** Once deleted, the account cannot be
    /// recovered without the original mnemonic. Make sure the user has backed up
    /// their mnemonic before proceeding.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the account was successfully deleted.
    ///
    /// # Errors
    ///
    /// Returns an error if the database delete operation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gigi_auth::AuthManager;
    /// # async fn example(auth: AuthManager) -> anyhow::Result<()> {
    /// // Make sure to warn user and confirm before deletion
    /// auth.delete_account().await?;
    /// println!("Account deleted permanently");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_account(&self) -> Result<()> {
        info!("Deleting account");

        self.settings_manager
            .delete(crate::settings_manager::GIGI_KEY)
            .await
            .context("Failed to delete encrypted mnemonic")?;

        info!("Account deleted successfully");
        Ok(())
    }

    /// Verify password without exposing account data
    ///
    /// Verifies if the provided password is correct without returning any
    /// sensitive account information. This is useful for pre-login validation,
    /// password confirmation dialogs, or security checks.
    ///
    /// # Arguments
    ///
    /// * `password` - The password to verify
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Password is correct
    /// * `Ok(false)` - Password is incorrect or account doesn't exist
    /// * `Err(...)` - Database or deserialization error
    ///
    /// # Security Note
    ///
    /// This method uses the same "decrypt-and-verify" approach as `login()` but
    /// returns only a boolean result, ensuring no sensitive data is exposed even
    /// to the calling code.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gigi_auth::AuthManager;
    /// # async fn example(auth: AuthManager) -> anyhow::Result<()> {
    /// let password = "my_password";
    /// if auth.verify_password(password).await? {
    ///     println!("Password is correct");
    /// } else {
    ///     println!("Password is incorrect");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn verify_password(&self, password: &str) -> Result<bool> {
        debug!("Verifying password");

        if !self.has_account().await? {
            return Ok(false);
        }

        // Get encrypted data
        let encrypted_data_str = match self
            .settings_manager
            .get(crate::settings_manager::GIGI_KEY)
            .await
        {
            Ok(Some(data)) => data,
            Ok(None) => return Ok(false),
            Err(_) => return Ok(false),
        };

        let encrypted_data: EncryptedAccountData = match serde_json::from_str(&encrypted_data_str) {
            Ok(data) => data,
            Err(_) => return Ok(false),
        };

        // Try to decrypt mnemonic with password
        let mnemonic = match crate::encryption::decrypt_mnemonic(&encrypted_data, password) {
            Ok(mnemonic) => mnemonic,
            Err(_) => return Ok(false),
        };

        // Derive peer_id and verify it matches
        let derived_peer_id = match key_derivation::derive_peer_id(&mnemonic) {
            Ok(data) => data,
            Err(_) => return Ok(false),
        };

        Ok(derived_peer_id == encrypted_data.peer_id)
    }
}
