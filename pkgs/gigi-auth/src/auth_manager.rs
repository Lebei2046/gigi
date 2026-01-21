//! Authentication manager for handling user accounts and authentication

use anyhow::{Context, Result};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::encryption::EncryptedAccountData;
use crate::key_derivation;

/// Account information (public - doesn't contain sensitive mnemonic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub address: String,
    pub peer_id: String,
    pub group_id: String,
    pub name: String,
}

/// Login result containing account info and private key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResult {
    pub account_info: AccountInfo,
    pub private_key: String,
}

/// Authentication manager
pub struct AuthManager {
    #[allow(dead_code)]
    db: DatabaseConnection,
    settings_manager: crate::settings_manager::SettingsManager,
}

impl AuthManager {
    /// Create a new auth manager
    pub fn new(db: DatabaseConnection) -> Self {
        let settings_manager = crate::settings_manager::SettingsManager::new(db.clone());
        Self {
            db,
            settings_manager,
        }
    }

    /// Check if an account exists
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
