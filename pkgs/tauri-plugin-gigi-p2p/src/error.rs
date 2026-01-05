use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Serde JSON error: {0}")]
    SerdeJson(String),

    #[error("Tauri error: {0}")]
    Tauri(String),

    #[error("P2P client not initialized")]
    P2pNotInitialized,

    #[error("P2P error: {0}")]
    P2p(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerdeJson(e.to_string())
    }
}

impl From<tauri::Error> for Error {
    fn from(e: tauri::Error) -> Self {
        Error::Tauri(e.to_string())
    }
}

impl From<Error> for tauri::ipc::InvokeError {
    fn from(error: Error) -> Self {
        tauri::ipc::InvokeError::from(error.to_string())
    }
}
