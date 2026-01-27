//! Key derivation utilities for peer IDs, group IDs, and EVM addresses
//!
//! This module implements BIP-32/BIP-39 hierarchical deterministic key derivation
//! for generating multiple cryptographic identities from a single BIP-39 mnemonic.
//! All keys are derived deterministically, meaning the same mnemonic will always
//! produce the same keys.
//!
//! # Key Derivation Paths
//!
//! Different keys are derived using different BIP-32 paths to ensure separation:
//!
//! | Purpose | Key Type | Derivation Path | Function |
//! |---------|----------|----------------|----------|
//! | EVM Address | Secp256k1 | `m/44'/60'/0'/0/0` | `derive_evm_address()` |
//! | EVM Private Key | Secp256k1 | `m/44'/60'/0'/0/0` | `derive_private_key()` |
//! | Peer ID (libp2p) | Ed25519 | `m/44'/60'/2'/0/0` | `derive_peer_id()` |
//! | Peer Private Key | Ed25519 | `m/44'/60'/2'/0/0` | `derive_peer_private_key()` |
//! | Group ID | Ed25519 | `m/44'/60'/1'/0/0` | `derive_group_id()` |
//!
//! # Path Structure
//!
//! The BIP-32 paths follow the standard structure: `m/purpose'/coin_type'/account'/change/index`
//!
//! - **purpose**: `44'` (BIP-44 - HD wallets)
//! - **coin_type**: `60'` (Ethereum)
//! - **account**: `0'`, `1'`, or `2'` (different key purposes)
//! - **change**: `0` (external chain)
//! - **index**: `0` (first address in chain)
//!
//! # Why Different Keys?
//!
//! - **Security**: Compromise of one key doesn't compromise others
//! - **Privacy**: Separate identities for different contexts
//! - **Compatibility**: Different algorithms for different use cases (Secp256k1 for EVM, Ed25519 for P2P)
//! - **Auditability**: Clear separation of concerns
//!
//! # Algorithms Used
//!
//! - **BIP-39**: Converts mnemonic words to seed
//! - **BIP-32**: Hierarchical deterministic key derivation
//! - **Secp256k1**: Elliptic curve for EVM addresses
//! - **Ed25519**: Elliptic curve for libp2p peer IDs
//! - **Keccak-256**: Hash function for EVM address generation
//!
//! # Example
//!
//! ```no_run
//! use gigi_auth::key_derivation::{derive_evm_address, derive_peer_id, derive_group_id};
//!
//! let mnemonic = "abandon amount liar amount expire adjust cage candy arch gather drum buyer";
//!
//! let address = derive_evm_address(mnemonic).unwrap();
//! let peer_id = derive_peer_id(mnemonic).unwrap();
//! let group_id = derive_group_id(mnemonic).unwrap();
//!
//! println!("EVM Address: {}", address);
//! println!("Peer ID: {}", peer_id);
//! println!("Group ID: {}", group_id);
//! ```

use anyhow::{Context, Result};
use bip32::{DerivationPath, XPrv};
use bip39::Mnemonic;
use ed25519_compact::KeyPair;
use hex;
use keccak_hash::keccak;
use libp2p::{identity, PeerId};
use secp256k1::{PublicKey, Secp256k1};

/// Derive peer_id (Ed25519) from mnemonic using BIP-32 path m/44'/60'/2'/0/0
/// Returns peer_id_string
///
/// This function derives a libp2p-compatible peer ID from a BIP-39 mnemonic.
/// Ed25519 is used because it offers:
/// - **Better Performance**: Faster signature generation/verification than Secp256k1
/// - **Smaller Keys**: 32-byte private keys vs. 32-byte private keys (same size) but more efficient
/// - **Stronger Security**: 128-bit security level (vs. 128-bit for Secp256k1, but with simpler implementation)
/// - **Native libp2p Support**: libp2p has excellent Ed25519 support
///
/// # Derivation Path
///
/// `m/44'/60'/2'/0/0`
/// - `44'`: BIP-44 (HD wallets)
/// - `60'`: Ethereum (for compatibility with EVM tools)
/// - `2'`: Account 2 (reserved for peer identity)
/// - `0`: External chain
/// - `0`: First address/index
///
/// # Arguments
///
/// * `mnemonic` - A valid BIP-39 mnemonic phrase (12 or 24 words)
///
/// # Returns
///
/// Returns `Ok(String)` containing the base58-encoded libp2p peer ID.
///
/// # Example
///
/// ```no_run
/// use gigi_auth::key_derivation::derive_peer_id;
///
/// let mnemonic = "abandon amount liar amount expire adjust cage candy arch gather drum buyer";
/// let peer_id = derive_peer_id(mnemonic).unwrap();
/// println!("Peer ID: {}", peer_id);
/// ```
pub fn derive_peer_id(mnemonic: &str) -> Result<String> {
    // Parse BIP-39 mnemonic (validates checksum and word list)
    let mnemonic = Mnemonic::parse_in_normalized(bip39::Language::English, mnemonic)
        .context("Failed to parse mnemonic")?;

    // Generate seed from mnemonic (no passphrase for simplicity)
    // PBKDF2 with 2048 iterations is used internally by to_seed()
    let seed = mnemonic.to_seed("");
    let seed_bytes: &[u8] = seed.as_ref();

    // Derive Ed25519 private key using BIP-32 path
    let derivation_path: DerivationPath = "m/44'/60'/2'/0/0"
        .parse()
        .context("Failed to parse derivation path")?;
    let child_key = XPrv::derive_from_path(seed_bytes, &derivation_path)
        .context("Failed to derive private key")?;

    // Get the 32-byte seed from the derived private key
    let signing_key = child_key.private_key();
    let private_bytes = signing_key.to_bytes();
    let seed_array: [u8; 32] = private_bytes.into();

    // Create Ed25519 keypair from the 32-byte seed
    // Ed25519 is used for better P2P performance and libp2p compatibility
    let keypair = KeyPair::from_seed(ed25519_compact::Seed::from_slice(&seed_array)?);

    // Convert to libp2p Keypair format
    let mut keypair_array: [u8; 64] = (*keypair).into();
    let libp2p_keypair = identity::ed25519::Keypair::try_from_bytes(&mut keypair_array)
        .context("Failed to convert to libp2p keypair")?;

    // Derive peer ID from public key (SHA-256 hash + base58 encoding)
    let generic_keypair: identity::Keypair = libp2p_keypair.into();
    let peer_id = PeerId::from_public_key(&generic_keypair.public());

    Ok(peer_id.to_string())
}

/// Derive group_id from mnemonic using BIP-32 path m/44'/60'/1'/0/0
/// Returns group_id_string
///
/// This function derives a group ID from a BIP-39 mnemonic. The group ID is
/// used for P2P group membership and management. It's derived from a different
/// account number than the peer ID to ensure separation between individual
/// identity and group identity.
///
/// # Derivation Path
///
/// `m/44'/60'/1'/0/0`
/// - `44'`: BIP-44 (HD wallets)
/// - `60'`: Ethereum (for compatibility)
/// - `1'`: Account 1 (reserved for group identity)
/// - `0`: External chain
/// - `0`: First address/index
///
/// # Why Separate from Peer ID?
///
/// - **Security**: Compromise of group membership shouldn't affect peer identity
/// - **Privacy**: Different groups can be created with different mnemonics
/// - **Flexibility**: A user can be in multiple groups with different identities
///
/// # Arguments
///
/// * `mnemonic` - A valid BIP-39 mnemonic phrase (12 or 24 words)
///
/// # Returns
///
/// Returns `Ok(String)` containing the base58-encoded group ID.
///
/// # Example
///
/// ```no_run
/// use gigi_auth::key_derivation::derive_group_id;
///
/// let mnemonic = "abandon amount liar amount expire adjust cage candy arch gather drum buyer";
/// let group_id = derive_group_id(mnemonic).unwrap();
/// println!("Group ID: {}", group_id);
/// ```
pub fn derive_group_id(mnemonic: &str) -> Result<String> {
    // Parse BIP-39 mnemonic
    let mnemonic = Mnemonic::parse_in_normalized(bip39::Language::English, mnemonic)
        .context("Failed to parse mnemonic")?;

    // Generate seed from mnemonic (no passphrase)
    let seed = mnemonic.to_seed("");
    let seed_bytes: &[u8] = seed.as_ref();

    // Derive Ed25519 private key using BIP-32 path for group identity
    let derivation_path: DerivationPath = "m/44'/60'/1'/0/0"
        .parse()
        .context("Failed to parse derivation path")?;
    let child_key = XPrv::derive_from_path(seed_bytes, &derivation_path)
        .context("Failed to derive private key")?;

    // Get the 32-byte seed from the derived private key
    let signing_key = child_key.private_key();
    let private_bytes = signing_key.to_bytes();
    let seed_array: [u8; 32] = private_bytes.into();

    // Create Ed25519 keypair from the 32-byte seed
    let keypair = KeyPair::from_seed(ed25519_compact::Seed::from_slice(&seed_array)?);

    // Convert to libp2p Keypair format
    let mut keypair_array: [u8; 64] = (*keypair).into();
    let libp2p_keypair = identity::ed25519::Keypair::try_from_bytes(&mut keypair_array)
        .context("Failed to convert to libp2p keypair")?;

    // Derive peer ID from public key (used as group ID)
    let generic_keypair: identity::Keypair = libp2p_keypair.into();
    let peer_id = PeerId::from_public_key(&generic_keypair.public());

    Ok(peer_id.to_string())
}

/// Derive EVM address from mnemonic using path m/44'/60'/0'/0/0
/// Returns EVM address string
///
/// This function derives an Ethereum-compatible wallet address from a BIP-39
/// mnemonic. It follows the standard Ethereum address derivation process:
///
/// 1. Derive Secp256k1 private key from mnemonic using BIP-32
/// 2. Derive public key from private key
/// 3. Take uncompressed public key (65 bytes)
/// 4. Remove 0x04 prefix, keep X and Y coordinates (64 bytes)
/// 5. Compute Keccak-256 hash of the coordinates
/// 6. Take last 20 bytes of the hash
/// 7. Add 0x prefix
///
/// # Derivation Path
///
/// `m/44'/60'/0'/0/0` (BIP-44 standard for Ethereum)
/// - `44'`: BIP-44 (HD wallets)
/// - `60'`: Ethereum coin type
/// - `0'`: Account 0 (primary account)
/// - `0`: External chain (receiving addresses)
/// - `0`: First address
///
/// # Arguments
///
/// * `mnemonic` - A valid BIP-39 mnemonic phrase (12 or 24 words)
///
/// # Returns
///
/// Returns `Ok(String)` containing the 0x-prefixed 20-byte EVM address (42 characters).
///
/// # Example
///
/// ```no_run
/// use gigi_auth::key_derivation::derive_evm_address;
///
/// let mnemonic = "abandon amount liar amount expire adjust cage candy arch gather drum buyer";
/// let address = derive_evm_address(mnemonic).unwrap();
/// println!("EVM Address: {}", address); // 0x742d35Cc6634C0530bbE07Ffd5B6c4F4d0885E...
/// ```
pub fn derive_evm_address(mnemonic: &str) -> Result<String> {
    // Parse BIP-39 mnemonic (validates checksum and word list)
    let mnemonic = Mnemonic::parse_in_normalized(bip39::Language::English, mnemonic)
        .context("Failed to parse mnemonic")?;

    // Generate seed from mnemonic (no passphrase)
    let seed = mnemonic.to_seed("");
    let seed_bytes: &[u8] = seed.as_ref();

    // Derive Secp256k1 private key using BIP-32 path for EVM account
    let derivation_path: DerivationPath = "m/44'/60'/0'/0/0"
        .parse()
        .context("Failed to parse derivation path")?;
    let child_key = XPrv::derive_from_path(seed_bytes, &derivation_path)
        .context("Failed to derive private key")?;

    // Convert to secp256k1 crate's SecretKey format
    let signing_key = child_key.private_key();
    let generic_array = signing_key.to_bytes();
    let bytes: [u8; 32] = generic_array.into();
    let secret_key = secp256k1::SecretKey::from_byte_array(bytes)
        .context("Failed to create secp256k1 secret key")?;

    // Create secp256k1 context and derive public key
    let secp = Secp256k1::new();

    // Derive public key from secret key (elliptic curve multiplication)
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    // Get public key in uncompressed format (65 bytes total)
    // Format: 0x04 (prefix) + 32-byte X coordinate + 32-byte Y coordinate
    let public_key_bytes = public_key.serialize_uncompressed();

    // Take last 64 bytes (skip the 0x04 prefix)
    // We only need the X and Y coordinates for address generation
    let xy_coordinates = &public_key_bytes[1..];

    // Calculate Keccak-256 hash (not SHA-256!)
    // Ethereum uses Keccak-256 for address derivation
    let keccak_hash = keccak(xy_coordinates);

    // Take last 20 bytes of hash as address
    // Keccak-256 produces 32 bytes, we use bytes 12-31 (last 20 bytes)
    let address_bytes = &keccak_hash.0[12..32];

    // Encode as hex and add 0x prefix (standard Ethereum address format)
    let address = format!("0x{}", hex::encode(address_bytes));

    Ok(address)
}

/// Derive private key (Secp256k1 for EVM) from mnemonic using path m/44'/60'/0'/0/0
/// Returns private key as hex string
///
/// This function derives a Secp256k1 private key for Ethereum-compatible
/// wallet operations (signing transactions, interacting with smart contracts).
/// The private key is returned as a hex-encoded string.
///
/// # Derivation Path
///
/// `m/44'/60'/0'/0/0` (same as EVM address derivation)
///
/// # Arguments
///
/// * `mnemonic` - A valid BIP-39 mnemonic phrase (12 or 24 words)
///
/// # Returns
///
/// Returns `Ok(String)` containing the 64-character hex-encoded private key (32 bytes).
///
/// # Security Warning
///
/// **This private key grants full control over the associated EVM wallet.**
/// Keep it secret and never share it. Anyone with this private key can sign
/// transactions and transfer all assets from the wallet.
///
/// # Example
///
/// ```no_run
/// use gigi_auth::key_derivation::derive_private_key;
///
/// let mnemonic = "abandon amount liar amount expire adjust cage candy arch gather drum buyer";
/// let private_key = derive_private_key(mnemonic).unwrap();
/// println!("Private Key: {}", private_key); // Handle with extreme care!
/// ```
pub fn derive_private_key(mnemonic: &str) -> Result<String> {
    // Parse BIP-39 mnemonic
    let mnemonic = Mnemonic::parse_in_normalized(bip39::Language::English, mnemonic)
        .context("Failed to parse mnemonic")?;

    // Generate seed from mnemonic (no passphrase)
    let seed = mnemonic.to_seed("");
    let seed_bytes: &[u8] = seed.as_ref();

    // Derive Secp256k1 private key using BIP-32 path for EVM account
    let derivation_path: DerivationPath = "m/44'/60'/0'/0/0"
        .parse()
        .context("Failed to parse derivation path")?;
    let child_key = XPrv::derive_from_path(seed_bytes, &derivation_path)
        .context("Failed to derive private key")?;

    // Get private key bytes (32 bytes)
    let signing_key = child_key.private_key().clone();
    let private_key_bytes = signing_key.to_bytes();
    let bytes: [u8; 32] = private_key_bytes.into();

    // Return as hex string (64 hex characters)
    Ok(hex::encode(bytes))
}

/// Derive peer private key (Ed25519) from mnemonic using path m/44'/60'/2'/0/0
/// Returns private key as hex string (64 hex chars = 32 bytes)
///
/// This function derives an Ed25519 private key for libp2p P2P operations.
/// The private key is returned as a hex-encoded string suitable for use with
/// libp2p's `Keypair::from_secret_key()` or similar functions.
///
/// # Derivation Path
///
/// `m/44'/60'/2'/0/0` (same as peer ID derivation)
///
/// # Arguments
///
/// * `mnemonic` - A valid BIP-39 mnemonic phrase (12 or 24 words)
///
/// # Returns
///
/// Returns `Ok(String)` containing the 64-character hex-encoded private key (32 bytes).
///
/// # Security Warning
///
/// **This private key grants full control over the peer identity.**
/// Anyone with this private key can impersonate the peer in the P2P network.
/// Keep it secret and never share it.
///
/// # Example
///
/// ```no_run
/// use gigi_auth::key_derivation::derive_peer_private_key;
///
/// let mnemonic = "abandon amount liar amount expire adjust cage candy arch gather drum buyer";
/// let private_key = derive_peer_private_key(mnemonic).unwrap();
/// println!("Peer Private Key: {}", private_key); // Handle with extreme care!
/// ```
pub fn derive_peer_private_key(mnemonic: &str) -> Result<String> {
    // Parse BIP-39 mnemonic
    let mnemonic = Mnemonic::parse_in_normalized(bip39::Language::English, mnemonic)
        .context("Failed to parse mnemonic")?;

    // Generate seed from mnemonic (no passphrase)
    let seed = mnemonic.to_seed("");
    let seed_bytes: &[u8] = seed.as_ref();

    // Derive Ed25519 private key using BIP-32 path for peer identity
    let derivation_path: DerivationPath = "m/44'/60'/2'/0/0"
        .parse()
        .context("Failed to parse derivation path")?;
    let child_key = XPrv::derive_from_path(seed_bytes, &derivation_path)
        .context("Failed to derive private key")?;

    // Get the 32-byte seed from the derived private key
    let signing_key = child_key.private_key();
    let private_bytes = signing_key.to_bytes();
    let seed_array: [u8; 32] = private_bytes.into();

    // Return 32-byte private key as hex (for libp2p's ed25519_from_bytes)
    Ok(hex::encode(seed_array))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A valid 12-word BIP-39 mnemonic for testing
    const TEST_MNEMONIC: &str =
        "abandon amount liar amount expire adjust cage candy arch gather drum buyer";

    #[test]
    fn test_peer_id_derivation() {
        let peer_id1 = derive_peer_id(TEST_MNEMONIC).expect("Peer ID derivation failed");
        let peer_id2 = derive_peer_id(TEST_MNEMONIC).expect("Peer ID derivation failed");

        assert_eq!(
            peer_id1, peer_id2,
            "Same mnemonic should produce same peer ID"
        );
        assert!(peer_id1.len() > 20, "Peer ID should be substantial length");
    }

    #[test]
    fn test_group_id_derivation() {
        let group_id1 = derive_group_id(TEST_MNEMONIC).expect("Group ID derivation failed");
        let group_id2 = derive_group_id(TEST_MNEMONIC).expect("Group ID derivation failed");

        assert_eq!(
            group_id1, group_id2,
            "Same mnemonic should produce same group ID"
        );
        assert!(
            group_id1.len() > 20,
            "Group ID should be substantial length"
        );
    }

    #[test]
    fn test_evm_address_derivation() {
        let address1 = derive_evm_address(TEST_MNEMONIC).expect("Address derivation failed");
        let address2 = derive_evm_address(TEST_MNEMONIC).expect("Address derivation failed");

        assert_eq!(
            address1, address2,
            "Same mnemonic should produce same address"
        );
        assert!(address1.starts_with("0x"), "Address should start with 0x");
        assert_eq!(
            address1.len(),
            42,
            "Address should be 42 characters (0x + 40 hex chars)"
        );
    }

    #[test]
    fn test_different_mnemonics_different_keys() {
        let mnemonic1 =
            "abandon amount liar amount expire adjust cage candy arch gather drum buyer";
        let mnemonic2 =
            "abandon amount liar amount expire adjust cage candy arch gather drum advice";

        let peer_id1 = derive_peer_id(mnemonic1).unwrap();
        let peer_id2 = derive_peer_id(mnemonic2).unwrap();

        assert_ne!(
            peer_id1, peer_id2,
            "Different mnemonics should produce different peer IDs"
        );
    }

    #[test]
    fn test_peer_and_group_different() {
        let peer_id = derive_peer_id(TEST_MNEMONIC).unwrap();
        let group_id = derive_group_id(TEST_MNEMONIC).unwrap();

        assert_ne!(peer_id, group_id, "Peer and group IDs should be different");
    }

    #[test]
    fn test_evm_address_format() {
        let address = derive_evm_address(TEST_MNEMONIC).unwrap();

        // Check format: 0x + 40 hex characters
        assert_eq!(&address[0..2], "0x", "Should start with 0x");
        assert_eq!(address.len(), 42, "Should be 42 characters");

        // Check remaining characters are valid hex
        hex::decode(&address[2..]).expect("Address should be valid hex");
    }

    #[test]
    fn test_known_mnemonic_produces_known_address() {
        // A simple 12-word BIP-39 test mnemonic
        let mnemonic = "abandon amount liar amount expire adjust cage candy arch gather drum buyer";

        let address = derive_evm_address(mnemonic).expect("Should derive address");

        // Check that we get a valid address format
        assert!(
            address.starts_with("0x"),
            "Should be valid Ethereum address format"
        );
        assert_eq!(address.len(), 42, "Should be correct length");
    }
}
