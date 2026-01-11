//! Gigi P2P - A comprehensive peer-to-peer networking library
//!
//! This library provides unified P2P functionality including:
//! - Auto Discovery via gigi-dns (with nicknames, capabilities, metadata)
//! - Direct Messaging via request-response
//! - Group Messaging via Gossipsub
//! - File Transfer via request-response
//! - Unified event system

pub mod behaviour;
pub mod client;
pub mod error;
pub mod events;

/// Initialize tracing subscriber for the library
///
/// This is a convenience function for consumers who want to use the default
/// logging configuration. Advanced users should set up their own tracing subscriber.
///
/// # Example
/// ```rust
/// gigi_p2p::init_tracing();
/// ```
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .compact()
        .init();
}

/// Initialize tracing with custom level
///
/// # Example
/// ```rust
/// gigi_p2p::init_tracing_with_level(tracing::Level::DEBUG);
/// ```
pub fn init_tracing_with_level(level: tracing::Level) {
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .compact()
        .init();
}

// Re-export public API
pub use client::P2pClient;
pub use client::CHUNK_SIZE;
pub use error::P2pError;
pub use events::{
    ActiveDownload, ChunkInfo, FileInfo, GroupInfo, GroupMessage, P2pEvent, PeerInfo, SharedFile,
};

/// Re-export commonly used libp2p types for convenience
pub use libp2p::{identity::Keypair, Multiaddr, PeerId};
