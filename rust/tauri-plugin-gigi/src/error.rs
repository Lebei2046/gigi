//! Error types and handling for the Gigi Tauri plugin.
//!
//! This module defines custom error types that can occur throughout the plugin's
//! operation, including I/O errors, serialization errors, Tauri errors, and P2P
//! related errors.
//!
//! # Example
//!
//! ```rust,ignore
//! use tauri_plugin_gigi::{Error, Result};
//!
//! fn do_something() -> Result<()> {
//!     // If something goes wrong:
//!     Err(Error::P2pNotInitialized)
//! }
//! ```

use thiserror::Error;

/// A type alias for `Result<T, Error>`.
///
/// This is the return type for most functions in the plugin that can fail.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types that can occur in the Gigi plugin.
///
/// Each variant represents a different category of error that can occur during
/// plugin operation.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// I/O related errors (file operations, network I/O, etc.)
    ///
    /// This error typically wraps `std::io::Error` and includes the underlying
    /// error message for debugging purposes.
    #[error("IO error: {0}")]
    Io(String),

    /// Serde JSON serialization/deserialization errors
    ///
    /// This occurs when JSON data cannot be serialized or deserialized properly,
    /// such as when parsing incoming messages or converting data structures.
    #[error("Serde JSON error: {0}")]
    SerdeJson(String),

    /// Tauri framework errors
    ///
    /// These errors originate from the Tauri framework, such as errors when
    /// emitting events or accessing Tauri APIs.
    #[error("Tauri error: {0}")]
    Tauri(String),

    /// P2P client has not been initialized
    ///
    /// This error occurs when attempting to use P2P functionality before the
    /// P2P client has been properly initialized. Ensure that the plugin setup
    /// has completed before calling P2P commands.
    #[error("P2P client not initialized")]
    P2pNotInitialized,

    /// P2P protocol or communication errors
    ///
    /// These errors occur during P2P operations, such as peer connection
    /// failures, message send failures, or file transfer errors.
    #[error("P2P error: {0}")]
    P2p(String),

    /// General command execution failures
    ///
    /// This is a catch-all error type for command failures that don't fit
    /// into the other categories. It includes a descriptive message explaining
    /// what went wrong.
    #[error("Command failed: {0}")]
    CommandFailed(String),
}

/// Auto-implementation of `From<std::io::Error>` for `Error`.
///
/// This allows automatic conversion of `std::io::Error` to our custom `Error`
/// type, making it convenient to propagate I/O errors.
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

/// Auto-implementation of `From<serde_json::Error>` for `Error`.
///
/// This allows automatic conversion of JSON serialization/deserialization errors.
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerdeJson(e.to_string())
    }
}

/// Auto-implementation of `From<tauri::Error>` for `Error`.
///
/// This allows automatic conversion of Tauri framework errors.
impl From<tauri::Error> for Error {
    fn from(e: tauri::Error) -> Self {
        Error::Tauri(e.to_string())
    }
}

/// Auto-implementation of `From<Error>` for `tauri::ipc::InvokeError`.
///
/// This allows our custom `Error` to be automatically converted to a Tauri
/// invoke error, which is the error type expected by Tauri command handlers.
/// The error message is converted to a string and passed to the frontend.
impl From<Error> for tauri::ipc::InvokeError {
    fn from(error: Error) -> Self {
        tauri::ipc::InvokeError::from(error.to_string())
    }
}
