//! Error types for P2P operations
//!
//! This module defines comprehensive error types for all P2P operations including
//! peer management, messaging, file sharing, and download tracking.

use libp2p::PeerId;
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur in P2P operations
///
/// Comprehensive error types for the P2P client, covering all possible
/// failure modes from peer lookup to file transfer.
///
/// # Error Categories
///
/// - **Lookup Errors**: Peer, nickname, or group not found
/// - **File Errors**: File not found, invalid share code, invalid URI
/// - **Network Errors**: Connection failures, timeouts, message send failures
/// - **System Errors**: IO errors, serialization errors, persistence not enabled
///
/// # Example
///
/// ```no_run
/// use gigi_p2p::P2pClient;
/// use gigi_p2p::P2pError;
///
/// # fn example(client: &mut P2pClient) {
/// match client.send_direct_message("Alice", "Hello".to_string()) {
///     Ok(_) => println!("Message sent"),
///     Err(e) => {
///         if e.is::<P2pError>() {
///             let p2p_err = e.downcast_ref::<P2pError>().unwrap();
///             match p2p_err {
///                 P2pError::PeerNotFound(_) => println!("Peer not found"),
///                 P2pError::MessageSendError(msg) => println!("Send failed: {}", msg),
///                 _ => println!("Error: {}", e),
///             }
///         }
///     }
/// }
/// }
/// ```
#[derive(Error, Debug)]
pub enum P2pError {
    /// Peer not found in peer manager
    ///
    /// Occurs when attempting to send a message or file to a peer ID
    /// that hasn't been discovered or is no longer tracked.
    #[error("Peer not found: {0}")]
    PeerNotFound(PeerId),

    /// Nickname not found in peer manager
    ///
    /// Occurs when attempting to lookup a peer by nickname that hasn't
    /// been discovered or has expired.
    #[error("Nickname not found: {0}")]
    NicknameNotFound(String),

    /// Group not found in group manager
    ///
    /// Occurs when attempting to send a message to a group that
    /// hasn't been joined or no longer exists.
    #[error("Group not found: {0}")]
    GroupNotFound(String),

    /// File not found on local filesystem
    ///
    /// Occurs when attempting to share a file that doesn't exist
    /// at the specified path.
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// Invalid share code
    ///
    /// Occurs when attempting to download a file with an invalid or
    /// non-existent share code. Share codes are validated format.
    #[error("Share code invalid: {0}")]
    InvalidShareCode(String),

    /// Invalid URI for content-based file sharing
    ///
    /// Occurs when attempting to share content via URI with an invalid
    /// or unsupported URI scheme (e.g., content://, file://, android://).
    #[error("Invalid URI: {0}")]
    InvalidUri(String),

    /// Network-related error
    ///
    /// Generic network error for connection failures, dialing errors,
    /// or other network-related issues.
    #[error("Network error: {0}")]
    NetworkError(String),

    /// IO error from filesystem operations
    ///
    /// Wraps standard IO errors for file reading, writing, or
    /// directory operations.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization/deserialization error
    ///
    /// Occurs when CBOR or JSON serialization fails, typically when
    /// sending/receiving messages with incompatible data types.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Operation timeout
    ///
    /// Occurs when an operation (e.g., file download) takes longer
    /// than the configured timeout period.
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Message send error
    ///
    /// Occurs when sending a message fails due to connection issues,
    /// serialization errors, or peer unavailability.
    #[error("Message send error: {0}")]
    MessageSendError(String),

    /// Message persistence not enabled
    ///
    /// Occurs when attempting to use persistence features (e.g.,
    /// `get_conversation_history`) without initializing the message store.
    #[error("Message persistence is not enabled")]
    PersistenceNotEnabled,
}
