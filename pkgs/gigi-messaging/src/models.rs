use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct MessageReceivedEvent {
  pub topic: String,
  pub data: String,
  pub sender: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PeerDiscoveredEvent {
  pub id: String,
  pub addresses: Vec<String>,
}
