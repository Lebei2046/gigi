use libp2p::BehaviourBuilderError;
use serde::{Serialize, Serializer};
use tokio::sync::mpsc::error::SendError;

/// Defines error types in the libp2p messaging plugin.
///
/// This enum covers various errors that may occur in the plugin, including I/O errors, configuration errors, network errors, etc.
/// Each error variant provides detailed error information for debugging and handling.

#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// I/O operation error.
  #[error(transparent)]
  Io(#[from] std::io::Error),

  /// Gossipsub configuration error.
  #[error("Gossipsub config error: {0}")]
  GossipsubConfigError(String),

  /// Gossipsub protocol error.
  #[error("Gossipsub error: {0}")]
  GossipsubError(String),

  /// mDNS protocol error.
  #[error("mDNS error: {0}")]
  MdnsError(String),

  /// Swarm network error.
  #[error("Swarm error: {0}")]
  SwarmError(String),

  /// Behaviour error.
  #[error("Behaviour error: {0}")]
  BehaviourError(String),

  #[error("Noise protocol error: {0}")]
  NoiseError(String),

  /// Subscription error.
  #[error("Subscription error: {0}")]
  SubscriptionError(String),

  /// Publish error.
  #[error("Publish error: {0}")]
  PublishError(String),

  /// Lock timeout error.
  #[error("Lock timeout: {0}")]
  LockTimeout(String),

  /// Channel send error.
  #[error("Channel send error: {0}")]
  ChannelSend(String),

  /// Channel receive error.
  #[error("Channel receive error: {0}")]
  ChannelReceive(String),

  /// Channel closed error.
  #[error("Channel closed")]
  ChannelClosed,
}

/// Implement `Serialize` trait for `Error`, allowing it to be serialized to a string.
///
/// Mainly used to convert error information to JSON or other formats.
impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
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

impl From<libp2p::noise::Error> for Error {
  fn from(err: libp2p::noise::Error) -> Self {
    Error::NoiseError(err.to_string())
  }
}

impl From<BehaviourBuilderError> for Error {
  fn from(err: BehaviourBuilderError) -> Self {
    Error::BehaviourError(err.to_string())
  }
}
