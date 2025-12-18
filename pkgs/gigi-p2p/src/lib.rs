//! Gigi P2P - A comprehensive peer-to-peer networking library
//!
//! This library provides unified P2P functionality including:
//! - Auto Discovery via mDNS
//! - Nickname Exchange via request-response
//! - Direct Messaging via request-response  
//! - Group Messaging via Gossipsub
//! - File Transfer via request-response
//! - Unified event system

pub mod behaviour;
pub mod client;
pub mod error;
pub mod events;
pub mod file_transfer;

// Re-export public API
pub use client::P2pClient;
pub use error::P2pError;
pub use events::{ChunkInfo, FileInfo, GroupInfo, GroupMessage, P2pEvent, PeerInfo, SharedFile};
pub use file_transfer::CHUNK_SIZE;

/// Re-export commonly used libp2p types for convenience
pub use libp2p::{identity::Keypair, Multiaddr, PeerId};
