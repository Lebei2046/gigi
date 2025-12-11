use thiserror::Error;

#[derive(Debug, Error)]
pub enum MessagingError {
    #[error("P2P error: {0}")]
    P2pError(#[from] gigi_p2p::P2pError),
    
    #[error("Client not initialized")]
    NotInitialized,
    
    #[error("Invalid peer ID: {0}")]
    InvalidPeerId(String),
    
    #[error("Invalid group name: {0}")]
    InvalidGroupName(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Event channel closed")]
    EventChannelClosed,
    
    #[error("Invalid private key format")]
    InvalidPrivateKey,
    
    #[error("Key generation failed")]
    KeyGenerationFailed,
    
    #[error("Key update failed")]
    KeyUpdateFailed,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
}