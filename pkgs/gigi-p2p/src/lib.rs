//! Gigi P2P - A comprehensive peer-to-peer networking library
//!
//! This library provides unified P2P functionality for the Gigi ecosystem including:
//! - **Auto Discovery**: Automatic peer discovery via gigi-dns (with nicknames, capabilities, metadata)
//! - **Direct Messaging**: 1-to-1 peer communication via request-response protocol
//! - **Group Messaging**: Publish-subscribe model using GossipSub for group chats
//! - **File Transfer**: Request-response protocol for file sharing with integrity verification
//! - **Unified Event System**: All P2P activities emitted as typed events
//!
//! # Architecture
//!
//! The library is organized into several modules:
//!
//! - [`P2pClient`] - Main API client that coordinates all P2P operations
//! - [`behaviour`] - Network protocol definitions and unified behaviour
//! - [`P2pEvent`] - Event types for all P2P activities
//! - [`P2pError`] - Error types for P2P operations
//!
//! # Protocol Stack
//!
//! The library uses multiple libp2p protocols combined into a unified behaviour:
//!
//! | Protocol | Purpose | Type |
//! |-----------|---------|------|
//! | gigi-dns | Peer discovery + nicknames + metadata | mDNS + custom protocol |
//! | Direct Messaging | 1-to-1 communication | Request-Response (CBOR) |
//! | Group Messaging | Group chat with pub/sub | GossipSub |
//! | File Sharing | Chunked file transfer | Request-Response (CBOR) |
//!
//! # Event-Driven Architecture
//!
//! All P2P activities are emitted as events via an `mpsc::UnboundedSender<P2pEvent>`:
//!
//! ```no_run
//! use gigi_p2p::P2pClient;
//! use gigi_p2p::Keypair;
//! use gigi_p2p::P2pEvent;
//! use std::path::PathBuf;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create client
//! let keypair = Keypair::generate_ed25519();
//! let (mut client, mut event_receiver) = P2pClient::new(
//!     keypair,
//!     "nickname".to_string(),
//!     PathBuf::from("./downloads"),
//! )?;
//!
//! // Handle events
//! use futures::StreamExt;
//! while let Some(event) = event_receiver.next().await {
//!     match event {
//!         P2pEvent::PeerDiscovered { peer_id, nickname, .. } => {
//!             println!("Discovered {} ({})", nickname, peer_id);
//!         }
//!         P2pEvent::DirectMessage { from_nickname, message, .. } => {
//!             println!("Message from {}: {}", from_nickname, message);
//!         }
//!         P2pEvent::FileDownloadProgress { downloaded_chunks, total_chunks, filename, .. } => {
//!             let progress = (downloaded_chunks * 100) / total_chunks;
//!             println!("Downloading {}: {}%", filename, progress);
//!         }
//!         _ => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # File Sharing Architecture
//!
//! Files are shared using a unique **share code** system:
//!
//! 1. **Share**: `share_file()` generates a unique share code
//! 2. **Announce**: Share code sent to peer(s) via direct/group message
//! 3. **Request**: Receiver uses `download_file()` with the share code
//! 4. **Transfer**: File split into 256KB chunks, transferred on-demand
//! 5. **Verify**: Each chunk verified with Blake3 hash, final file verified with SHA256
//!
//! This pull-based approach is efficient for group chats:
//! - No need to broadcast large files
//! - Multiple receivers can download from same source
//! - Parallel chunk requests for better performance
//!
//! # Download Tracking
//!
//! The library tracks each download instance with a unique `download_id`:
//!
//! - `download_id` = Unique per download instance (allows parallel downloads of same file)
//! - `file_id` = Content identifier (same for all downloads of same file)
//!
//! This enables mobile UI features:
//! - Show active downloads with progress
//! - Cancel specific downloads
//! - Show download history per peer
//!
//! # Message Persistence
//!
//! Optional message persistence via `gigi-store`:
//!
//! - Store all messages for offline viewing
//! - Automatic retry of pending messages when peer reconnects
//! - Message delivery tracking (sent, delivered, read)
//!
//! # Security Considerations
//!
//! - **No encryption**: Data is transmitted unencrypted over the transport layer
//! - **Peer verification**: No peer identity verification (relies on transport security)
//! - **Path traversal**: Use validated share codes to prevent directory traversal
//!
//! See [`SECURITY.md`](https://github.com/your-repo/gigi-p2p/docs/SECURITY.md) for detailed security analysis.
//!
//! # Example: Basic Usage
//!
//! ```no_run
//! use gigi_p2p::P2pClient;
//! use gigi_p2p::Keypair;
//! use gigi_p2p::Multiaddr;
//! use std::path::PathBuf;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create keypair (in production, load from auth module)
//! let keypair = Keypair::generate_ed25519();
//!
//! // Create client
//! let (mut client, mut event_receiver) = P2pClient::new(
//!     keypair,
//!     "Alice".to_string(),
//!     PathBuf::from("./downloads"),
//! )?;
//!
//! // Start listening
//! client.start_listening("/ip4/0.0.0.0/tcp/0".parse()?)?;
//!
//! // Handle events
//! use futures::StreamExt;
//! while let Some(event) = event_receiver.next().await {
//!     // Process events...
//! }
//! # Ok(())
//! # }
//! ```

pub mod behaviour;
pub mod client;
pub mod error;
pub mod events;

/// Initialize tracing subscriber for library
///
/// This is a convenience function for consumers who want to use default
/// logging configuration. Advanced users should set up their own tracing subscriber
/// with custom filters and formatters.
///
/// # Example
///
/// ```no_run
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
/// Allows consumers to set their preferred log level for debugging.
///
/// # Arguments
///
/// * `level` - The minimum log level to display
///
/// # Example
///
/// ```no_run
/// use tracing::Level;
///
/// // Enable debug logging
/// gigi_p2p::init_tracing_with_level(Level::DEBUG);
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

// Re-export persistence types from gigi-store
pub use gigi_store::{
    MessageContent, MessageDirection, MessageStore, PersistenceConfig, StoredMessage, SyncManager,
};

// Re-export other event types
pub use events::{
    ActiveDownload, ChunkInfo, FileInfo, GroupInfo, GroupMessage, P2pEvent, PeerInfo, SharedFile,
};

/// Re-export commonly used libp2p types for convenience
pub use libp2p::{identity::Keypair, Multiaddr, PeerId};
