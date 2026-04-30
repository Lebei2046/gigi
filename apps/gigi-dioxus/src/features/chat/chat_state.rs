use crate::services::auth_service::AuthService;
use crate::services::p2p_service::P2pService;
use crate::services::persistence_service::PersistenceService;
use chrono::Local;
use dioxus::prelude::*;
use dirs;
use gigi_p2p::PeerId;
use gigi_store::{Conversation as StoreConversation, StoredMessage};

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
    pub created: bool,
}

impl From<&gigi_auth::GroupInfo> for Group {
    fn from(info: &gigi_auth::GroupInfo) -> Self {
        Self {
            id: info.group_id.clone(),
            name: info.name.clone(),
            role: if info.created {
                "Created".to_string()
            } else {
                "Joined".to_string()
            },
            member_count: 0,
            created: info.created,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Conversation {
    pub id: String,
    pub name: String,
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
    pub filename: Option<String>,
    pub file_size: Option<u64>,
    pub file_type: Option<String>,
    pub share_code: Option<String>,
    pub is_downloading: bool,
    pub download_progress: Option<u8>,
    pub download_id: Option<String>,
    pub file_path: Option<String>,
    pub group_id: Option<String>,
    pub download_attempts: u32,
    pub last_download_attempt: Option<chrono::DateTime<chrono::Local>>,
    pub download_failed: bool,
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
        let (content, group_id) = match &msg.content {
            gigi_store::MessageContent::Text { text } => (text.clone(), None),
            gigi_store::MessageContent::FileShare { filename, .. } => (filename.clone(), None),
            gigi_store::MessageContent::FileShareWithThumbnail { filename, .. } => {
                (filename.clone(), None)
            }
            gigi_store::MessageContent::ShareGroup {
                group_id,
                group_name,
                ..
            } => (
                format!("Join group: {}", group_name),
                Some(group_id.clone()),
            ),
        };

        let message_type = match &msg.content {
            gigi_store::MessageContent::Text { .. } => MessageType::Text,
            gigi_store::MessageContent::FileShareWithThumbnail { file_type, .. }
            | gigi_store::MessageContent::FileShare { file_type, .. } => {
                // Check if it's an image file (handle both MIME types and extensions)
                let lower_file_type = file_type.to_lowercase();
                let is_image = lower_file_type.starts_with("image/")
                    || ["png", "jpg", "jpeg", "gif", "bmp", "webp"]
                        .contains(&lower_file_type.as_str());
                if is_image {
                    MessageType::Image
                } else {
                    MessageType::File
                }
            }
            gigi_store::MessageContent::ShareGroup { .. } => MessageType::Text,
        };

        let is_own = matches!(msg.direction, gigi_store::MessageDirection::Sent);

        let (filename, file_size, file_type, share_code) = match &msg.content {
            gigi_store::MessageContent::FileShare {
                filename,
                file_size,
                file_type,
                share_code,
                ..
            } => (
                Some(filename.clone()),
                Some(*file_size),
                Some(file_type.clone()),
                Some(share_code.clone()),
            ),
            gigi_store::MessageContent::FileShareWithThumbnail {
                filename,
                file_size,
                file_type,
                share_code,
                ..
            } => (
                Some(filename.clone()),
                Some(*file_size),
                Some(file_type.clone()),
                Some(share_code.clone()),
            ),
            _ => (None, None, None, None),
        };

        // Calculate file path for the message
        let file_path = match &msg.content {
            gigi_store::MessageContent::FileShareWithThumbnail {
                filename,
                file_type,
                ..
            }
            | gigi_store::MessageContent::FileShare {
                filename,
                file_type,
                ..
            } => {
                // Get data directory
                let data_dir = std::env::var("GIGI_DATA_DIR").unwrap_or_else(|_| {
                    dirs::data_local_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join("gigi-dioxus")
                        .to_string_lossy()
                        .to_string()
                });

                // Expand ~ to home directory
                let data_dir_expanded = if data_dir.starts_with('~') {
                    if let Some(home) = dirs::home_dir() {
                        home.join(data_dir.strip_prefix('~').unwrap_or(""))
                    } else {
                        std::path::PathBuf::from(data_dir)
                    }
                } else {
                    std::path::PathBuf::from(data_dir)
                };

                let base_dir = if is_own {
                    data_dir_expanded.join("uploads")
                } else {
                    data_dir_expanded.join("downloads")
                };

                // Check if it's an image and use thumbnail path
                let lower_file_type = file_type.to_lowercase();
                let is_image = lower_file_type.starts_with("image/")
                    || ["png", "jpg", "jpeg", "gif", "bmp", "webp"]
                        .contains(&lower_file_type.as_str());

                if is_image {
                    Some(
                        base_dir
                            .join(format!("{}.thumbnail.jpg", filename))
                            .to_string_lossy()
                            .to_string(),
                    )
                } else {
                    Some(base_dir.join(filename).to_string_lossy().to_string())
                }
            }
            _ => None,
        };

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
            filename,
            file_size,
            file_type,
            share_code,
            is_downloading: false,
            download_progress: None,
            download_id: None,
            file_path,
            group_id,
            download_attempts: 0,
            last_download_attempt: None,
            download_failed: false,
        }
    }
}

use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;

pub static GLOBAL_CHAT_STATE: Lazy<Arc<Mutex<ChatState>>> =
    Lazy::new(|| Arc::new(Mutex::new(ChatState::default())));
static CHAT_INITIALIZED: Lazy<Arc<Mutex<bool>>> = Lazy::new(|| Arc::new(Mutex::new(false)));

pub async fn is_chat_initialized() -> bool {
    let state = GLOBAL_CHAT_STATE.lock().await;
    state.peers.len() > 0 || state.groups.len() > 0 || state.conversations.len() > 0
}

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
        Ok(peers) => {
            let peers: Vec<gigi_p2p::PeerInfo> = peers;
            peers
                .into_iter()
                .map(|p| Peer {
                    id: p.peer_id.to_string(),
                    peer_id: p.peer_id,
                    nickname: p.nickname,
                    is_online: p.connected,
                    capabilities: vec!["chat".to_string(), "file_sharing".to_string()],
                })
                .collect()
        }
        Err(err) => {
            println!("Failed to list peers: {:?}", err);
            vec![]
        }
    }
}

// Delete a message by ID
pub fn delete_message(messages: &mut Vec<Message>, message_id: &str) {
    if let Some(index) = messages.iter().position(|msg| msg.id == message_id) {
        messages.remove(index);
    }
}

// Load conversations from persistence
pub async fn load_conversations() -> Vec<Conversation> {
    match PersistenceService::load_conversations().await {
        Ok(stored_conversations) => stored_conversations
            .into_iter()
            .map(|sc| Conversation {
                id: sc.id,
                name: sc.name,
                peer_id: if sc.is_group {
                    None
                } else {
                    Some(sc.peer_id.clone())
                },
                group_id: if sc.is_group {
                    Some(sc.peer_id.clone())
                } else {
                    None
                },
                last_message: sc.last_message,
                last_message_time: sc
                    .last_message_timestamp
                    .map(|t| t.with_timezone(&Local).format("%H:%M %p").to_string()),
                unread_count: sc.unread_count as u32,
            })
            .collect(),
        Err(err) => {
            println!("Failed to load conversations: {:?}", err);
            vec![]
        }
    }
}
