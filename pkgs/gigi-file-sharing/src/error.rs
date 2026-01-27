//! Error types for file sharing operations
//!
//! This module defines all error types that can occur during file sharing operations,
//! using `thiserror` for automatic error display and conversion.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur in file sharing operations
///
/// # Error Types
///
/// ## FileNotFound
/// Returned when attempting to share a file that doesn't exist on the filesystem.
///
/// ## InvalidShareCode
/// Returned when trying to unshare or access a file with a non-existent share code.
///
/// ## InvalidUri
/// Returned when parsing a malformed content URI string.
///
/// ## IoError
/// Propagated from underlying filesystem I/O operations (reading files, etc.).
///
/// ## SerializationError
/// Propagated from JSON serialization/deserialization operations.
///
/// # Example
///
/// ```rust,no_run
/// use gigi_file_sharing::FileSharingError;
/// use std::path::PathBuf;
///
/// # let mut manager = FileSharingManager::new();
/// match manager.share_file(&PathBuf::from("file.txt")).await {
///     Ok(code) => println!("Share code: {}", code),
///     Err(FileSharingError::FileNotFound(p)) => {
///         eprintln!("File not found: {:?}", p);
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// # Ok(())
/// ```
#[derive(Error, Debug)]
pub enum FileSharingError {
    /// File not found at specified path
    ///
    /// Occurs when:
    /// - File was deleted before sharing
    /// - Path is incorrect
    /// - Permissions prevent access
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// Invalid share code provided
    ///
    /// Occurs when:
    /// - Share code doesn't exist in registry
    /// - Share code is malformed
    #[error("Share code invalid: {0}")]
    InvalidShareCode(String),

    /// Invalid URI format
    ///
    /// Occurs when:
    /// - URI string is malformed
    /// - Scheme is not supported
    /// - URI parsing fails
    #[error("Invalid URI: {0}")]
    InvalidUri(String),

    /// I/O error from filesystem operations
    ///
    /// Propagated from:
    /// - File reading operations
    /// - Metadata queries
    /// - Hash calculation
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization/deserialization error
    ///
    /// Propagated from:
    /// - Converting metadata to/from JSON
    /// - Serializing for network transfer
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
