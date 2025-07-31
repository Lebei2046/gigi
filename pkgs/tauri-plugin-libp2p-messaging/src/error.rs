use serde::{Serialize, Serializer};
use tokio::sync::{mpsc::error::SendError, oneshot};

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error("Gossipsub config error: {0}")]
  GossipsubConfigError(String),
  #[error("Gossipsub error: {0}")]
  GossipsubError(String),
  #[error("mDNS error: {0}")]
  MdnsError(String),
  #[error("Swarm error: {0}")]
  SwarmError(String),
  #[error("Behaviour error: {0}")]
  BehaviourError(String),
  #[error("Subscription error: {0}")]
  SubscriptionError(String),
  #[error("Publish error: {0}")]
  PublishError(String),
  #[error("Lock timeout: {0}")]
  LockTimeout(String),
  #[error("Channel send error: {0}")]
  ChannelSend(String),
  #[error("Channel receive error: {0}")]
  ChannelReceive(String),
  #[error("Channel closed")]
  ChannelClosed,
}

impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}

impl<T> From<SendError<T>> for Error {
  fn from(err: SendError<T>) -> Self {
    Error::ChannelSend(format!("Failed to send command: {}", err))
  }
}

impl From<oneshot::error::RecvError> for Error {
  fn from(err: oneshot::error::RecvError) -> Self {
    Error::ChannelReceive(format!("Failed to receive response: {}", err))
  }
}
