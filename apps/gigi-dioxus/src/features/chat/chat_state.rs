use crate::services::auth_service::AuthService;
use crate::services::p2p_service::P2pService;
use chrono::Local;
use dioxus::prelude::*;
use gigi_p2p::PeerId;
use gigi_store::StoredMessage;

// Types for chat data
#[derive(Debug, Clone, PartialEq)]
pub struct Peer {
    pub id: String,
    pub peer_id: PeerId,
    pub nickname: String,
    pub is_online: bool,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub role: String,
    pub member_count: u32,
    pub joined: bool,
}

impl From<&gigi_auth::GroupInfo> for Group {
    fn from(info: &gigi_auth::GroupInfo) -> Self {
        Self {
            id: info.group_id.clone(),
            name: info.name.clone(),
            role: if info.joined {
                "Member".to_string()
            } else {
                "Not joined".to_string()
            },
            member_count: 0,
            joined: info.joined,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Conversation {
    pub id: String,
    pub peer_id: Option<String>,
    pub group_id: Option<String>,
    pub last_message: Option<String>,
    pub last_message_time: Option<String>,
    pub unread_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupShareNotification {
    pub id: String,
    pub group_id: String,
    pub group_name: String,
    pub sender_id: String,
    pub sender_nickname: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveDownload {
    pub download_id: String,
    pub filename: String,
    pub share_code: String,
    pub from_peer_id: PeerId,
    pub from_nickname: String,
    pub downloaded_chunks: usize,
    pub total_chunks: usize,
    pub completed: bool,
    pub failed: bool,
    pub error_message: Option<String>,
    pub final_path: Option<std::path::PathBuf>,
}

// Chat state
#[derive(Debug, Clone, PartialEq)]
pub struct ChatState {
    pub peers: Vec<Peer>,
    pub groups: Vec<Group>,
    pub conversations: Vec<Conversation>,
    pub group_share_notifications: Vec<GroupShareNotification>,
    pub active_downloads: Vec<ActiveDownload>,
    pub loading: bool,
    pub error: Option<String>,
}

// Chat room state
#[derive(Debug, Clone, PartialEq)]
pub struct ChatRoomState {
    pub chat_id: Option<String>,
    pub chat_name: Option<String>,
    pub is_group_chat: bool,
    pub peer: Option<Peer>,
    pub group: Option<Group>,
    pub messages: Vec<Message>,
    pub new_message: String,
    pub sending: bool,
    pub is_loading: bool,
    pub unread_reset_done: bool,
}

// Message type (reused from chat_room.rs)
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub id: String,
    pub content: String,
    pub sender: String,
    pub timestamp: String,
    pub is_own: bool,
    pub message_type: MessageType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    Text,
    Image,
    File,
}

// Initial state
impl Default for ChatState {
    fn default() -> Self {
        Self {
            peers: vec![],
            groups: vec![],
            conversations: vec![],
            group_share_notifications: vec![],
            active_downloads: vec![],
            loading: false,
            error: None,
        }
    }
}

impl Default for ChatRoomState {
    fn default() -> Self {
        Self {
            chat_id: None,
            chat_name: None,
            is_group_chat: false,
            peer: None,
            group: None,
            messages: vec![],
            new_message: "".to_string(),
            sending: false,
            is_loading: false,
            unread_reset_done: false,
        }
    }
}

impl From<&StoredMessage> for Message {
    fn from(msg: &StoredMessage) -> Self {
        let content = match &msg.content {
            gigi_store::MessageContent::Text { text } => text.clone(),
            gigi_store::MessageContent::FileShare { filename, .. } => filename.clone(),
            gigi_store::MessageContent::FileShareWithThumbnail { filename, .. } => filename.clone(),
            gigi_store::MessageContent::ShareGroup { group_name, .. } => {
                format!("Join group: {}", group_name)
            }
        };

        let message_type = match &msg.content {
            gigi_store::MessageContent::Text { .. } => MessageType::Text,
            gigi_store::MessageContent::FileShareWithThumbnail { .. }
            | gigi_store::MessageContent::FileShare { .. } => MessageType::File,
            gigi_store::MessageContent::ShareGroup { .. } => MessageType::Text,
        };

        let is_own = matches!(msg.direction, gigi_store::MessageDirection::Sent);

        Self {
            id: msg.id.clone(),
            content,
            sender: msg.sender_nickname.clone(),
            timestamp: msg
                .timestamp
                .with_timezone(&Local)
                .format("%H:%M %p")
                .to_string(),
            is_own,
            message_type,
        }
    }
}

// State providers
pub fn use_chat_state() -> Signal<ChatState> {
    use_signal(ChatState::default)
}

pub fn use_chat_room_state() -> Signal<ChatRoomState> {
    use_signal(ChatRoomState::default)
}

// Helper functions for chat operations
pub async fn send_message(to_nickname: &str, message: &str) {
    if let Err(err) = P2pService::send_message(to_nickname, message).await {
        println!("Failed to send message: {:?}", err);
    }
}

pub async fn send_group_message(group_name: &str, message: &str) {
    if let Err(err) = P2pService::send_group_message(group_name, message).await {
        println!("Failed to send group message: {:?}", err);
    }
}

pub async fn join_group(group_name: &str) {
    if let Err(err) = P2pService::join_group(group_name).await {
        println!("Failed to join group: {:?}", err);
    }
}

pub async fn leave_group(group_name: &str) {
    if let Err(err) = P2pService::leave_group(group_name).await {
        println!("Failed to leave group: {:?}", err);
    }
}

pub async fn list_peers() -> Vec<Peer> {
    match P2pService::list_peers().await {
        Ok(peers) => peers
            .into_iter()
            .map(|p| Peer {
                id: p.peer_id.to_string(),
                peer_id: p.peer_id,
                nickname: p.nickname,
                is_online: p.connected,
                capabilities: vec!["chat".to_string(), "file_sharing".to_string()],
            })
            .collect(),
        Err(err) => {
            println!("Failed to list peers: {:?}", err);
            vec![]
        }
    }
}
