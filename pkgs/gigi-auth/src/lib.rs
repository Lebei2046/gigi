//! Authentication and account management for Gigi
//!
//! This crate provides functionality for managing user accounts, including:
//! - Account creation with mnemonics
//! - Password-based authentication
//! - Encrypted mnemonic storage
//! - Key derivation for peer IDs, group IDs, and EVM addresses

pub mod auth_manager;
pub mod encryption;
pub mod entities;
pub mod key_derivation;
pub mod settings_manager;

pub use auth_manager::{AccountInfo, AuthManager, LoginResult};
pub use encryption::{EncryptedAccountData, EncryptionError};
pub use key_derivation::{derive_evm_address, derive_group_id, derive_peer_id};
pub use settings_manager::SettingsManager;
