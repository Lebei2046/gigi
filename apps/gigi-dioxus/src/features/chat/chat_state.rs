use dioxus::prelude::*;

// Types for chat data
#[derive(Debug, Clone, PartialEq)]
pub struct Peer {
    pub id: String,
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

// Chat state
#[derive(Debug, Clone, PartialEq)]
pub struct ChatState {
    pub peers: Vec<Peer>,
    pub groups: Vec<Group>,
    pub conversations: Vec<Conversation>,
    pub group_share_notifications: Vec<GroupShareNotification>,
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
            peers: vec![
                Peer {
                    id: "peer123".to_string(),
                    nickname: "Alice".to_string(),
                    is_online: true,
                    capabilities: vec!["chat".to_string(), "file_sharing".to_string()],
                },
                Peer {
                    id: "peer456".to_string(),
                    nickname: "Bob".to_string(),
                    is_online: true,
                    capabilities: vec!["chat".to_string()],
                },
            ],
            groups: vec![
                Group {
                    id: "group1".to_string(),
                    name: "Development Team".to_string(),
                    role: "Member".to_string(),
                    member_count: 5,
                },
                Group {
                    id: "group2".to_string(),
                    name: "Friends".to_string(),
                    role: "Owner".to_string(),
                    member_count: 3,
                },
            ],
            conversations: vec![
                Conversation {
                    id: "conv1".to_string(),
                    peer_id: Some("peer123".to_string()),
                    group_id: None,
                    last_message: Some("Hello, how are you?".to_string()),
                    last_message_time: Some("2024-01-01 12:00:00".to_string()),
                    unread_count: 2,
                },
                Conversation {
                    id: "conv2".to_string(),
                    peer_id: Some("peer456".to_string()),
                    group_id: None,
                    last_message: None,
                    last_message_time: None,
                    unread_count: 0,
                },
                Conversation {
                    id: "conv3".to_string(),
                    peer_id: None,
                    group_id: Some("group1".to_string()),
                    last_message: Some("💬 Meeting at 3 PM".to_string()),
                    last_message_time: Some("2024-01-01 11:30:00".to_string()),
                    unread_count: 1,
                },
                Conversation {
                    id: "conv4".to_string(),
                    peer_id: None,
                    group_id: Some("group2".to_string()),
                    last_message: None,
                    last_message_time: None,
                    unread_count: 0,
                },
            ],
            group_share_notifications: vec![] as Vec<GroupShareNotification>,
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
            messages: vec![
                Message {
                    id: "1".to_string(),
                    content: "Hello, how are you?".to_string(),
                    sender: "Alice".to_string(),
                    timestamp: "12:00 PM".to_string(),
                    is_own: false,
                    message_type: MessageType::Text,
                },
                Message {
                    id: "2".to_string(),
                    content: "I'm doing well, thanks! How about you?".to_string(),
                    sender: "You".to_string(),
                    timestamp: "12:01 PM".to_string(),
                    is_own: true,
                    message_type: MessageType::Text,
                },
                Message {
                    id: "3".to_string(),
                    content: "I'm good too! Let's meet up later.".to_string(),
                    sender: "Alice".to_string(),
                    timestamp: "12:02 PM".to_string(),
                    is_own: false,
                    message_type: MessageType::Text,
                },
            ],
            new_message: "".to_string(),
            sending: false,
            is_loading: false,
            unread_reset_done: false,
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
