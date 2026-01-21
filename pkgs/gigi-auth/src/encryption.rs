//! Encryption utilities for mnemonic encryption using ChaCha20-Poly1305

use anyhow::{Context, Result};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Key, Nonce,
};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Encrypted account data for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedAccountData {
    pub encrypted_mnemonic: String,
    pub nonce: String,
    pub peer_id: String,
    pub group_id: String,
    pub address: String,
    pub name: String,
}

/// Encryption error types
#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Failed to encrypt data")]
    EncryptionError,

    #[error("Failed to decrypt data")]
    DecryptionError,

    #[error("Failed to serialize data")]
    SerializationError,

    #[error("Failed to deserialize data")]
    DeserializationError,

    #[error("Failed to derive encryption key")]
    KeyDerivationError,
}

/// Encrypt mnemonic with password using ChaCha20-Poly1305 authenticated encryption
pub fn encrypt_mnemonic(
    mnemonic: &str,
    password: &str,
    peer_id: &str,
    group_id: &str,
    address: &str,
    name: &str,
) -> Result<EncryptedAccountData> {
    // Derive 256-bit key from password using HKDF-HMAC-SHA256
    let salt = Sha256::digest(password);
    let salt_arr: [u8; 32] = salt.into();
    let key = derive_key(password, &salt_arr)?;

    // Generate random nonce and create cipher
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let cipher = ChaCha20Poly1305::new(&key);

    // Encrypt mnemonic
    let ciphertext = cipher
        .encrypt(&nonce, mnemonic.as_bytes())
        .map_err(|_e| EncryptionError::EncryptionError)
        .context("Failed to encrypt mnemonic")?;

    let encrypted_mnemonic = hex::encode(&ciphertext);
    let nonce_hex = hex::encode(&nonce);

    Ok(EncryptedAccountData {
        encrypted_mnemonic,
        nonce: nonce_hex,
        peer_id: peer_id.to_string(),
        group_id: group_id.to_string(),
        address: address.to_string(),
        name: name.to_string(),
    })
}

/// Decrypt mnemonic with password using ChaCha20-Poly1305 authenticated decryption
pub fn decrypt_mnemonic(encrypted_data: &EncryptedAccountData, password: &str) -> Result<String> {
    // Derive 256-bit key from password using HKDF-HMAC-SHA256
    let salt = Sha256::digest(password);
    let key = derive_key(password, &salt)?;

    // Decode nonce and ciphertext
    let nonce_bytes = hex::decode(&encrypted_data.nonce).context("Failed to decode nonce hex")?;
    let nonce_array: [u8; 12] = nonce_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid nonce length, expected 12 bytes"))?;
    let nonce = Nonce::from(nonce_array);
    let ciphertext = hex::decode(&encrypted_data.encrypted_mnemonic)
        .context("Failed to decode ciphertext hex")?;

    // Decrypt mnemonic (authentication is automatically verified by Poly1305)
    let cipher = ChaCha20Poly1305::new(&key);
    let plaintext = cipher
        .decrypt(&nonce, ciphertext.as_ref())
        .map_err(|_e| EncryptionError::DecryptionError)
        .context("Failed to decrypt mnemonic - invalid password or corrupted data")?;

    String::from_utf8(plaintext).context("Failed to decrypt mnemonic - invalid UTF-8")
}

/// Derive a 256-bit encryption key from password using HKDF-HMAC-SHA256
fn derive_key(password: &str, salt: &[u8]) -> Result<Key> {
    // Use HKDF to derive a key from password and salt
    let hk = Hkdf::<Sha256>::new(Some(salt), password.as_bytes());
    let mut key_bytes = [0u8; 32];
    hk.expand(b"gigi-mnemonic", &mut key_bytes)
        .map_err(|_| EncryptionError::KeyDerivationError)
        .context("Failed to derive encryption key")?;

    Ok(Key::from(key_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let mnemonic = "abandon amount liar amount expire adjust cage candy arch gather drum buyer";
        let password = "secure_password_123";
        let peer_id = "12D3KooWBdWJvz4KwB6v4sF8s8uBx8Q";
        let group_id = "12D3KooWBdWJvz4KwB6v4sF8s8uBx8Q";
        let address = "0x742d35Cc6634C0530bbE07Ffd5B6c4F4d0885E";
        let name = "TestUser";

        let encrypted = encrypt_mnemonic(mnemonic, password, peer_id, group_id, address, name)
            .expect("Encryption failed");

        assert_eq!(encrypted.name, name);

        let decrypted = decrypt_mnemonic(&encrypted, password).expect("Decryption failed");

        assert_eq!(mnemonic, decrypted);
    }

    #[test]
    fn test_encryption_wrong_password() {
        let mnemonic = "abandon amount liar amount expire adjust cage candy arch gather drum buyer";
        let password = "secure_password_123";
        let wrong_password = "wrong_password";
        let peer_id = "12D3KooWBdWJvz4KwB6v4sF8s8uBx8Q";
        let group_id = "12D3KooWBdWJvz4KwB6v4sF8s8uBx8Q";
        let address = "0x742d35Cc6634C0530bbE07Ffd5B6c4F4d0885E";
        let name = "TestUser";

        let encrypted = encrypt_mnemonic(mnemonic, password, peer_id, group_id, address, name)
            .expect("Encryption failed");

        let result = decrypt_mnemonic(&encrypted, wrong_password);

        assert!(result.is_err(), "Wrong password should fail");
    }

    #[test]
    fn test_serialization() {
        let data = EncryptedAccountData {
            encrypted_mnemonic: "encrypted_data".to_string(),
            nonce: hex::encode(&[0u8; 12]),
            peer_id: "12D3KooWBdWJvz4KwB6v4sF8s8uBx8Q".to_string(),
            group_id: "12D3KooWBdWJvz4KwB6v4sF8s8uBx8Q".to_string(),
            address: "0x742d35Cc6634C0530bbE07Ffd5B6c4F4d0885E".to_string(),
            name: "TestUser".to_string(),
        };

        let serialized = serde_json::to_string(&data).expect("Serialization failed");
        let deserialized: EncryptedAccountData =
            serde_json::from_str(&serialized).expect("Deserialization failed");

        assert_eq!(data.encrypted_mnemonic, deserialized.encrypted_mnemonic);
        assert_eq!(data.nonce, deserialized.nonce);
        assert_eq!(data.peer_id, deserialized.peer_id);
        assert_eq!(data.group_id, deserialized.group_id);
        assert_eq!(data.address, deserialized.address);
        assert_eq!(data.name, deserialized.name);
    }
}
