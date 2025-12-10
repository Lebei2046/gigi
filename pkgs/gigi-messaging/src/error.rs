use tokio::sync::mpsc::error::SendError;

/// Defines error types in the Gigi messaging plugin.
///
/// This enum covers various errors that may occur in the plugin, including I/O errors, configuration errors, network errors, etc.
/// Each error variant provides detailed error information for debugging and handling.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O operation error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// P2P service error.
    #[error("P2P service error: {0}")]
    P2pError(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Channel send error.
    #[error("Channel send error: {0}")]
    ChannelSend(String),

    /// Channel receive error.
    #[error("Channel receive error: {0}")]
    ChannelReceive(String),

    /// Channel closed error.
    #[error("Channel closed")]
    ChannelClosed,

    /// Not implemented error.
    #[error("Feature not implemented: {0}")]
    NotImplemented(String),
}

// Implement conversion to Tauri's InvokeError
impl From<Error> for tauri::ipc::InvokeError {
    fn from(err: Error) -> Self {
        tauri::ipc::InvokeError::from(err.to_string())
    }
}

/// Convert from `SendError<T>` to `Error`.
///
/// Used for error conversion when channel sending fails.
impl<T> From<SendError<T>> for Error {
    fn from(err: SendError<T>) -> Self {
        Error::ChannelSend(format!("Failed to send command: {}", err))
    }
}
