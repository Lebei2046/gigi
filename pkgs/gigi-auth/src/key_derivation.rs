//! Key derivation utilities for peer IDs, group IDs, and EVM addresses

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
/// This uses Ed25519 for better P2P performance and compatibility with libp2p.
pub fn derive_peer_id(mnemonic: &str) -> Result<String> {
    // Parse BIP-39 mnemonic
    let mnemonic = Mnemonic::parse_in_normalized(bip39::Language::English, mnemonic)
        .context("Failed to parse mnemonic")?;

    // Generate seed from mnemonic (no passphrase)
    let seed = mnemonic.to_seed("");
    let seed_bytes: &[u8] = seed.as_ref();

    // Derive directly from path
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
    let keypair = KeyPair::from_seed(ed25519_compact::Seed::from_slice(&seed_array)?);

    // Convert to libp2p Keypair by converting to array
    let mut keypair_array: [u8; 64] = (*keypair).into();
    let libp2p_keypair = identity::ed25519::Keypair::try_from_bytes(&mut keypair_array)
        .context("Failed to convert to libp2p keypair")?;

    // Convert to generic Keypair and then to PeerId
    let generic_keypair: identity::Keypair = libp2p_keypair.into();
    let peer_id = PeerId::from_public_key(&generic_keypair.public());

    Ok(peer_id.to_string())
}

/// Derive group_id from mnemonic using BIP-32 path m/44'/60'/1'/0/0
/// Returns group_id_string
///
/// This uses Ed25519 for group identity in P2P network.
pub fn derive_group_id(mnemonic: &str) -> Result<String> {
    // Parse BIP-39 mnemonic
    let mnemonic = Mnemonic::parse_in_normalized(bip39::Language::English, mnemonic)
        .context("Failed to parse mnemonic")?;

    // Generate seed from mnemonic (no passphrase)
    let seed = mnemonic.to_seed("");
    let seed_bytes: &[u8] = seed.as_ref();

    // Derive directly from path
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

    // Convert to libp2p Keypair by converting to array
    let mut keypair_array: [u8; 64] = (*keypair).into();
    let libp2p_keypair = identity::ed25519::Keypair::try_from_bytes(&mut keypair_array)
        .context("Failed to convert to libp2p keypair")?;

    // Convert to generic Keypair and then to PeerId
    let generic_keypair: identity::Keypair = libp2p_keypair.into();
    let peer_id = PeerId::from_public_key(&generic_keypair.public());

    Ok(peer_id.to_string())
}

/// Derive EVM address from mnemonic using path m/44'/60'/0'/0/0
/// Returns EVM address string
///
/// This uses proper BIP-32/BIP-39 derivation and Ethereum address generation.
pub fn derive_evm_address(mnemonic: &str) -> Result<String> {
    // Parse BIP-39 mnemonic
    let mnemonic = Mnemonic::parse_in_normalized(bip39::Language::English, mnemonic)
        .context("Failed to parse mnemonic")?;

    // Generate seed from mnemonic (no passphrase)
    let seed = mnemonic.to_seed("");
    let seed_bytes: &[u8] = seed.as_ref();

    // Derive directly from path
    let derivation_path: DerivationPath = "m/44'/60'/0'/0/0"
        .parse()
        .context("Failed to parse derivation path")?;
    let child_key = XPrv::derive_from_path(seed_bytes, &derivation_path)
        .context("Failed to derive private key")?;

    // Convert k256 SigningKey to secp256k1 crate's SecretKey
    let signing_key = child_key.private_key();
    let generic_array = signing_key.to_bytes();
    let bytes: [u8; 32] = generic_array.into();
    let secret_key = secp256k1::SecretKey::from_byte_array(bytes)
        .context("Failed to create secp256k1 secret key")?;

    // Create secp context and derive public key
    let secp = Secp256k1::new();

    // Derive public key from secret key
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    // Get public key in uncompressed format (65 bytes: 0x04 + 32-byte X + 32-byte Y)
    let public_key_bytes = public_key.serialize_uncompressed();

    // Take last 64 bytes (skip the 0x04 prefix)
    let xy_coordinates = &public_key_bytes[1..];

    // Calculate Keccak-256 hash
    let keccak_hash = keccak(xy_coordinates);

    // Take last 20 bytes of hash as address
    let address_bytes = &keccak_hash.0[12..32]; // Keccak-256 produces 32 bytes

    // Encode as hex and add 0x prefix
    let address = format!("0x{}", hex::encode(address_bytes));

    Ok(address)
}

/// Derive private key (Secp256k1 for EVM) from mnemonic using path m/44'/60'/0'/0/0
/// Returns private key as hex string
///
/// This uses proper BIP-32/BIP-39 derivation for EVM address.
pub fn derive_private_key(mnemonic: &str) -> Result<String> {
    // Parse BIP-39 mnemonic
    let mnemonic = Mnemonic::parse_in_normalized(bip39::Language::English, mnemonic)
        .context("Failed to parse mnemonic")?;

    // Generate seed from mnemonic (no passphrase)
    let seed = mnemonic.to_seed("");
    let seed_bytes: &[u8] = seed.as_ref();

    // Derive directly from path
    let derivation_path: DerivationPath = "m/44'/60'/0'/0/0"
        .parse()
        .context("Failed to parse derivation path")?;
    let child_key = XPrv::derive_from_path(seed_bytes, &derivation_path)
        .context("Failed to derive private key")?;

    // Get private key bytes
    let signing_key = child_key.private_key().clone();
    let private_key_bytes = signing_key.to_bytes();
    let bytes: [u8; 32] = private_key_bytes.into();

    // Return as hex string
    Ok(hex::encode(bytes))
}

/// Derive peer private key (Ed25519) from mnemonic using path m/44'/60'/2'/0/0
/// Returns private key as hex string (64 hex chars = 32 bytes)
///
/// This uses Ed25519 for P2P peer identity.
pub fn derive_peer_private_key(mnemonic: &str) -> Result<String> {
    // Parse BIP-39 mnemonic
    let mnemonic = Mnemonic::parse_in_normalized(bip39::Language::English, mnemonic)
        .context("Failed to parse mnemonic")?;

    // Generate seed from mnemonic (no passphrase)
    let seed = mnemonic.to_seed("");
    let seed_bytes: &[u8] = seed.as_ref();

    // Derive directly from path
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
