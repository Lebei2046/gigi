/// Simple message models for gigi-messaging

/// Message structure for group messaging
#[derive(Debug, Clone)]
pub struct GroupMessage {
    pub topic: String,
    pub sender: String,
    pub content: String,
    pub timestamp: u64,
}

/// Direct message between peers
#[derive(Debug, Clone)]
pub struct DirectMessage {
    pub to: String,
    pub from: String,
    pub content: String,
    pub timestamp: u64,
}
