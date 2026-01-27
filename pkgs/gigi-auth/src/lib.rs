//! Authentication and account management for Gigi
//!
//! This crate provides comprehensive functionality for managing user accounts in the Gigi ecosystem.
//! It implements secure password-based authentication, encrypted mnemonic storage, and hierarchical
//! deterministic key derivation for multiple cryptographic identities.
//!
//! # Features
//!
//! - **Account Creation**: Create accounts using BIP-39 mnemonics
//! - **Password Authentication**: Secure password-based login with encrypted storage
//! - **Key Derivation**: Derive multiple cryptographic identities from a single mnemonic:
//!   - EVM addresses (Secp256k1) for blockchain interactions
//!   - Peer IDs (Ed25519) for libp2p identity
//!   - Group IDs (Ed25519) for P2P group management
//! - **Encrypted Storage**: Secure mnemonic encryption using ChaCha20-Poly1305
//!
//! # Architecture
//!
//! The crate is organized into several modules:
//!
//! - [`auth_manager`] - High-level API for account operations
//! - [`encryption`] - ChaCha20-Poly1305 encryption for mnemonics
//! - [`key_derivation`] - BIP-32/BIP-39 key derivation
//! - [`settings_manager`] - Database abstraction for encrypted data storage
//! - [`entities`] - Sea-ORM entity definitions
//!
//! # Security
//!
//! - **ChaCha20-Poly1305**: NIST-approved authenticated encryption
//! - **HKDF-HMAC-SHA256**: RFC 5869 key derivation
//! - **BIP-32/BIP-39**: Industry-standard HD wallet key derivation
//! - **Decrypt-and-Verify**: Password verification through successful decryption
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
//! let account = auth.create_account(mnemonic, password, Some("Alice".to_string())).await?;
//!
//! // Login
//! let result = auth.login(password).await?;
//! println!("Welcome, {}!", result.account_info.name);
//!
//! // Change password
//! auth.change_password(password, "new_password").await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Key Derivation Paths
//!
//! | Purpose | Key Type | Path | Function |
//! |---------|----------|------|----------|
//! | EVM Address | Secp256k1 | `m/44'/60'/0'/0/0` | [`derive_evm_address()`] |
//! | Peer ID | Ed25519 | `m/44'/60'/2'/0/0` | [`derive_peer_id()`] |
//! | Group ID | Ed25519 | `m/44'/60'/1'/0/0` | [`derive_group_id()`] |
//!
//! [`derive_evm_address()`]: crate::key_derivation::derive_evm_address
//! [`derive_peer_id()`]: crate::key_derivation::derive_peer_id
//! [`derive_group_id()`]: crate::key_derivation::derive_group_id

pub mod auth_manager;
pub mod encryption;
pub mod entities;
pub mod key_derivation;
pub mod settings_manager;

pub use auth_manager::{AccountInfo, AuthManager, LoginResult};
pub use encryption::{EncryptedAccountData, EncryptionError};
pub use key_derivation::{derive_evm_address, derive_group_id, derive_peer_id};
pub use settings_manager::SettingsManager;
