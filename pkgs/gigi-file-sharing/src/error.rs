//! Error types for file sharing operations

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur in file sharing operations
#[derive(Error, Debug)]
pub enum FileSharingError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    #[error("Share code invalid: {0}")]
    InvalidShareCode(String),
    #[error("Invalid URI: {0}")]
    InvalidUri(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
