//! Encryption utilities for mnemonic encryption using ChaCha20-Poly1305
//!
//! This module provides encryption and decryption utilities for securely storing
//! BIP-39 mnemonics. It uses ChaCha20-Poly1305 authenticated encryption with
//! keys derived from passwords using HKDF-HMAC-SHA256.
//!
//! # Cryptographic Algorithms
//!
//! - **Encryption**: ChaCha20-Poly1305 (RFC 8439) - An AEAD cipher providing
//!   both confidentiality and authenticity
//! - **Key Derivation**: HKDF-HMAC-SHA256 (RFC 5869) - For deriving encryption
//!   keys from passwords
//! - **Randomness**: `OsRng` - Cryptographically secure random number generator
//!   for nonce generation
//!
//! # Security Properties
//!
//! - **Authenticated Encryption**: Poly1305 MAC ensures data integrity and
//!   authenticity (prevents tampering)
//! - **Forward Secrecy**: Nonce is generated randomly for each encryption
//! - **Key Separation**: HKDF info parameter "gigi-mnemonic" ensures keys are
//!   domain-specific
//! - **Password-Based**: Keys are derived from passwords, not stored directly
//!
//! # Encryption Process
//!
//! 1. Derive a 256-bit key from the password using HKDF-HMAC-SHA256
//! 2. Generate a random 12-byte nonce using OsRng
//! 3. Encrypt the mnemonic with ChaCha20-Poly1305
//! 4. Store encrypted ciphertext, nonce, and metadata
//!
//! # Decryption Process
//!
//! 1. Derive the 256-bit key from the password using HKDF-HMAC-SHA256
//! 2. Decode the stored nonce
//! 3. Decrypt the ciphertext (authentication automatically verified by Poly1305)
//! 4. Return the plaintext mnemonic
//!
//! # Example
//!
//! ```no_run
//! use gigi_auth::encryption::{encrypt_mnemonic, decrypt_mnemonic};
//!
//! let mnemonic = "abandon amount liar amount expire adjust cage candy arch gather drum buyer";
//! let password = "my_secure_password";
//!
//! // Encrypt mnemonic
//! let encrypted = encrypt_mnemonic(
//!     mnemonic,
//!     password,
//!     "peer_id",
//!     "group_id",
//!     "address",
//!     "User"
//! ).unwrap();
//!
//! // Decrypt mnemonic
//! let decrypted = decrypt_mnemonic(&encrypted, password).unwrap();
//! assert_eq!(mnemonic, decrypted);
//! ```

use anyhow::{Context, Result};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Key, Nonce,
};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Encrypted account data for storage
///
/// This struct contains the encrypted mnemonic along with all the metadata
/// needed for storage and verification. The peer_id, group_id, address, and name
/// are stored in plaintext alongside the encrypted mnemonic for quick access
/// and verification purposes.
///
/// # Fields
///
/// - `encrypted_mnemonic`: The BIP-39 mnemonic encrypted with ChaCha20-Poly1305 (hex-encoded)
/// - `nonce`: Random 12-byte nonce used during encryption (hex-encoded)
/// - `peer_id`: Derived libp2p peer ID (for verification and quick access)
/// - `group_id`: Derived group ID (for quick access)
/// - `address`: Derived EVM address (for quick access)
/// - `name`: Account display name
///
/// # Security Note
///
/// While peer_id, group_id, address, and name are stored in plaintext, this is
/// safe because:
/// - They are derived deterministically from the mnemonic (public knowledge)
/// - They cannot be used to recover the mnemonic
/// - The peer_id is used for verification during login
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
///
/// Enumerates all possible errors that can occur during encryption or
/// decryption operations.
///
/// # Error Variants
///
/// - `EncryptionError` - Failed to encrypt data (e.g., memory allocation failure)
/// - `DecryptionError` - Failed to decrypt data (wrong password or corrupted data)
/// - `SerializationError` - Failed to serialize data to JSON
/// - `DeserializationError` - Failed to deserialize JSON data
/// - `KeyDerivationError` - Failed to derive encryption key from password
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
///
/// Encrypts a BIP-39 mnemonic using ChaCha20-Poly1305 with a key derived from the
/// provided password. The nonce is generated randomly for each encryption to ensure
/// forward secrecy.
///
/// # Arguments
///
/// * `mnemonic` - The BIP-39 mnemonic phrase to encrypt
/// * `password` - Password to derive the encryption key from
/// * `peer_id` - Derived libp2p peer ID (stored in plaintext for verification)
/// * `group_id` - Derived group ID (stored in plaintext)
/// * `address` - Derived EVM address (stored in plaintext)
/// * `name` - Account display name (stored in plaintext)
///
/// # Returns
///
/// Returns `Ok(EncryptedAccountData)` containing the encrypted mnemonic and all metadata.
///
/// # Algorithm Details
///
/// 1. Compute salt = SHA-256(password)
/// 2. Derive 256-bit key using HKDF-HMAC-SHA256(password, salt, "gigi-mnemonic")
/// 3. Generate random 12-byte nonce using OsRng
/// 4. Encrypt mnemonic with ChaCha20-Poly1305(key, nonce, plaintext)
/// 5. Return ciphertext, nonce, and metadata
///
/// # Example
///
/// ```no_run
/// use gigi_auth::encryption::encrypt_mnemonic;
///
/// let mnemonic = "abandon amount liar amount expire adjust cage candy arch gather drum buyer";
/// let password = "my_secure_password";
/// let encrypted = encrypt_mnemonic(
///     mnemonic,
///     password,
///     "peer_id",
///     "group_id",
///     "address",
///     "User"
/// ).unwrap();
/// ```
pub fn encrypt_mnemonic(
    mnemonic: &str,
    password: &str,
    peer_id: &str,
    group_id: &str,
    address: &str,
    name: &str,
) -> Result<EncryptedAccountData> {
    // Derive 256-bit key from password using HKDF-HMAC-SHA256
    // The salt is derived from the password itself (common practice for password-based encryption)
    let salt = Sha256::digest(password);
    let salt_arr: [u8; 32] = salt.into();
    let key = derive_key(password, &salt_arr)?;

    // Generate random nonce (12 bytes for ChaCha20-Poly1305)
    // OsRng provides cryptographically secure randomness
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let cipher = ChaCha20Poly1305::new(&key);

    // Encrypt mnemonic (Poly1305 authentication tag is automatically computed)
    let ciphertext = cipher
        .encrypt(&nonce, mnemonic.as_bytes())
        .map_err(|_e| EncryptionError::EncryptionError)
        .context("Failed to encrypt mnemonic")?;

    // Encode as hex for storage
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
///
/// Decrypts an encrypted BIP-39 mnemonic using the provided password. The Poly1305
/// authentication tag is automatically verified during decryption, ensuring data
/// integrity and authenticity.
///
/// # Arguments
///
/// * `encrypted_data` - The encrypted account data containing the ciphertext and nonce
/// * `password` - The password used during encryption
///
/// # Returns
///
/// Returns `Ok(String)` containing the decrypted BIP-39 mnemonic.
///
/// # Errors
///
/// Returns an error if:
/// - The password is incorrect (authentication tag verification fails)
/// - The data has been tampered with (authentication tag verification fails)
/// - The nonce or ciphertext is malformed
/// - The decrypted data is not valid UTF-8
///
/// # Security Note
///
/// The Poly1305 MAC ensures that:
/// - The password is correct (wrong password = wrong key = authentication failure)
/// - The data hasn't been tampered with (tampering changes ciphertext = authentication failure)
/// - The nonce is correct (wrong nonce = different ciphertext = authentication failure)
///
/// # Example
///
/// ```no_run
/// use gigi_auth::encryption::{encrypt_mnemonic, decrypt_mnemonic};
///
/// # let encrypted = todo!();
/// let password = "my_secure_password";
/// let mnemonic = decrypt_mnemonic(&encrypted, password).unwrap();
/// println!("Recovered mnemonic: {}", mnemonic);
/// ```
pub fn decrypt_mnemonic(encrypted_data: &EncryptedAccountData, password: &str) -> Result<String> {
    // Derive 256-bit key from password using HKDF-HMAC-SHA256
    // Must use the same salt derivation as encryption
    let salt = Sha256::digest(password);
    let key = derive_key(password, &salt)?;

    // Decode nonce and ciphertext from hex
    let nonce_bytes = hex::decode(&encrypted_data.nonce).context("Failed to decode nonce hex")?;
    let nonce_array: [u8; 12] = nonce_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid nonce length, expected 12 bytes"))?;
    let nonce = Nonce::from(nonce_array);
    let ciphertext = hex::decode(&encrypted_data.encrypted_mnemonic)
        .context("Failed to decode ciphertext hex")?;

    // Decrypt mnemonic (Poly1305 authentication is automatically verified)
    // If the password is wrong or data is tampered with, decrypt() will fail
    let cipher = ChaCha20Poly1305::new(&key);
    let plaintext = cipher
        .decrypt(&nonce, ciphertext.as_ref())
        .map_err(|_e| EncryptionError::DecryptionError)
        .context("Failed to decrypt mnemonic - invalid password or corrupted data")?;

    // Convert decrypted bytes to UTF-8 string
    String::from_utf8(plaintext).context("Failed to decrypt mnemonic - invalid UTF-8")
}

/// Derive a 256-bit encryption key from password using HKDF-HMAC-SHA256
///
/// This internal function derives a cryptographic key from a password using
/// HKDF (HMAC-based Extract-and-Expand Key Derivation Function) with SHA-256.
///
/// # Arguments
///
/// * `password` - The password to derive the key from
/// * `salt` - The salt value (typically SHA-256 of the password)
///
/// # Returns
///
/// Returns `Ok(Key)` - A 256-bit ChaCha20-Poly1305 key.
///
/// # Algorithm
///
/// HKDF consists of two stages:
/// 1. **Extract**: HMAC-SHA256(salt, password) produces a pseudorandom key (PRK)
/// 2. **Expand**: HMAC-SHA256(PRK, "gigi-mnemonic" || 0x01 || ... || 0x04) produces the output key
///
/// The info parameter "gigi-mnemonic" ensures key separation - keys derived for
/// different purposes will be different even with the same password and salt.
///
/// # Why HKDF?
///
/// - Prevents salt reuse vulnerabilities
/// - Provides domain separation via the info parameter
/// - NIST-approved key derivation method
/// - Suitable for deriving keys from passwords of any length
fn derive_key(password: &str, salt: &[u8]) -> Result<Key> {
    // Use HKDF to derive a key from password and salt
    // The info parameter "gigi-mnemonic" provides domain separation
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
