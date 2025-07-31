use serde::{Serialize, Serializer};

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
}

impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}
