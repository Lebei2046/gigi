//! Peer-related Tauri commands.
//!
//! This module provides commands for retrieving peer information such as
//! the local peer ID and deriving peer IDs from private keys.

use tauri::State;

use crate::{models::PluginState, Error, Result};

/// Gets the local peer ID.
///
/// This command retrieves the unique identifier of the local peer from the
/// initialized P2P client. The peer ID is derived from the cryptographic
/// keypair used for P2P communication.
///
/// # Arguments
///
/// * `state` - The plugin state containing the P2P client
///
/// # Returns
///
/// A `Result` containing the local peer ID as a string, or an error if the
/// P2P client is not initialized
///
/// # Example
///
/// ```typescript,ignore
/// const peerId = await invoke('get_peer_id');
/// console.log('My peer ID:', peerId);
/// ```
#[tauri::command]
pub(crate) async fn get_peer_id(state: State<'_, PluginState>) -> Result<String> {
    let p2p_client = state.p2p_client.lock().await;
    match p2p_client.as_ref() {
        Some(client) => Ok(client.local_peer_id().to_string()),
        None => Err(Error::P2pNotInitialized),
    }
}

/// Tries to derive a peer ID from private key bytes.
///
/// This command attempts to create an Ed25519 keypair from the provided
/// private key bytes and derives the corresponding peer ID. This is useful
/// for previewing what peer ID would be generated from a particular keypair.
///
/// # Arguments
///
/// * `priv_key` - Raw bytes of an Ed25519 private key
///
/// # Returns
///
/// A `Result` containing the derived peer ID as a string, or an error if the
/// keypair cannot be created from the provided bytes
///
/// # Errors
///
/// Returns an error if the provided bytes do not represent a valid Ed25519
/// private key.
///
/// # Example
///
/// ```typescript,ignore
/// const privateKey = new Uint8Array([...]); // 32 bytes for Ed25519
/// const peerId = await invoke('try_get_peer_id', { privKey: privateKey });
/// console.log('Derived peer ID:', peerId);
/// ```
#[tauri::command]
pub(crate) fn try_get_peer_id(priv_key: Vec<u8>) -> Result<String> {
    use libp2p::identity;
    match identity::Keypair::ed25519_from_bytes(priv_key) {
        Ok(id_keys) => {
            let peer_id = id_keys.public().to_peer_id();
            Ok(peer_id.to_string())
        }
        Err(e) => Err(Error::CommandFailed(format!(
            "Failed to create keypair: {}",
            e
        ))),
    }
}
