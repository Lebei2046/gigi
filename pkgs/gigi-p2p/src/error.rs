//! Error types for P2P operations

use libp2p::PeerId;
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur in P2P operations
#[derive(Error, Debug)]
pub enum P2pError {
    #[error("Peer not found: {0}")]
    PeerNotFound(PeerId),
    #[error("Nickname not found: {0}")]
    NicknameNotFound(String),
    #[error("Group not found: {0}")]
    GroupNotFound(String),
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    #[error("Share code invalid: {0}")]
    InvalidShareCode(String),
    #[error("Invalid URI: {0}")]
    InvalidUri(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("Message send error: {0}")]
    MessageSendError(String),
    #[error("Message persistence is not enabled")]
    PersistenceNotEnabled,
}
