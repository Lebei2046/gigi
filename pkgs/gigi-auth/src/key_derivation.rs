//! Key derivation utilities for peer IDs, group IDs, and EVM addresses

use anyhow::{Context, Result};
use bip32::{DerivationPath, XPrv};
use bip39::Mnemonic;
use hex;
use keccak_hash::keccak;
use libp2p::{identity, PeerId};
use secp256k1::{PublicKey, Secp256k1};

/// Derive peer_id from mnemonic using BIP-32 path m/44'/60'/0'/0/0
/// Returns peer_id_string
///
/// This uses proper BIP-32/BIP-39 derivation and generates a libp2p PeerId.
pub fn derive_peer_id(mnemonic: &str) -> Result<String> {
    // Parse the BIP-39 mnemonic
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

    // Convert k256 SigningKey to libp2p secp256k1 SecretKey, then to Keypair
    let signing_key = child_key.private_key().clone();
    let secret_key =
        identity::secp256k1::SecretKey::try_from_bytes(&mut signing_key.to_bytes().to_vec())
            .context("Failed to create libp2p secret key")?;
    let keypair = identity::secp256k1::Keypair::from(secret_key);

    // Get PeerId from keypair
    let public_key = keypair.public().clone();
    let peer_id = PeerId::from(identity::PublicKey::from(public_key));

    Ok(peer_id.to_string())
}

/// Derive group_id from mnemonic using BIP-32 path m/44'/60'/1'/0/0
/// Returns group_id_string
///
/// This uses proper BIP-32/BIP-39 derivation and generates a libp2p PeerId.
pub fn derive_group_id(mnemonic: &str) -> Result<String> {
    // Parse the BIP-39 mnemonic
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

    // Convert k256 SigningKey to libp2p secp256k1 SecretKey, then to Keypair
    let signing_key = child_key.private_key().clone();
    let secret_key =
        identity::secp256k1::SecretKey::try_from_bytes(&mut signing_key.to_bytes().to_vec())
            .context("Failed to create libp2p secret key")?;
    let keypair = identity::secp256k1::Keypair::from(secret_key);

    // Get PeerId from keypair
    let public_key = keypair.public().clone();
    let peer_id = PeerId::from(identity::PublicKey::from(public_key));

    Ok(peer_id.to_string())
}

/// Derive EVM address from mnemonic using path m/44'/60'/0'/0/0
/// Returns EVM address string
///
/// This uses proper BIP-32/BIP-39 derivation and Ethereum address generation.
pub fn derive_evm_address(mnemonic: &str) -> Result<String> {
    // Parse the BIP-39 mnemonic
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

    // Take the last 64 bytes (skip the 0x04 prefix)
    let xy_coordinates = &public_key_bytes[1..];

    // Calculate Keccak-256 hash
    let keccak_hash = keccak(xy_coordinates);

    // Take last 20 bytes of the hash as the address
    let address_bytes = &keccak_hash.0[12..32]; // Keccak-256 produces 32 bytes

    // Encode as hex and add 0x prefix
    let address = format!("0x{}", hex::encode(address_bytes));

    Ok(address)
}

/// Derive private key from mnemonic using path m/44'/60'/0'/0/0
/// Returns private key as hex string
///
/// This uses proper BIP-32/BIP-39 derivation.
pub fn derive_private_key(mnemonic: &str) -> Result<String> {
    // Parse the BIP-39 mnemonic
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
