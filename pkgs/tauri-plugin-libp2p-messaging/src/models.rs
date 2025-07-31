use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct MessageReceivedEvent {
  pub(crate) topic: String,
  pub(crate) data: String,
  pub(crate) sender: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PeerDiscoveredEvent {
  pub(crate) id: String,
  pub(crate) addresses: Vec<String>,
}
