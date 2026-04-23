use dioxus::prelude::*;
use futures_util::StreamExt;
use gigi_p2p::PeerId;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;



use crate::features::chat::chat_state::{
    use_chat_room_state, use_chat_state, ChatRoomState, ChatState, Message, Peer,
    GLOBAL_CHAT_STATE, load_conversations,
};
use crate::services::auth_service::AuthService;
use crate::services::event_bus::{AppEvent, EventBus};
use crate::services::p2p_service::P2pService;
use crate::services::persistence_service::PersistenceService;

use once_cell::sync::Lazy;
use tokio::sync::Mutex;

static CHAT_INIT_FLAG: Lazy<Arc<Mutex<bool>>> = Lazy::new(|| Arc::new(Mutex::new(false)));

// Hook for chat initialization
pub fn use_chat_initialization() -> Signal<ChatState> {
    let chat_state = use_chat_state();

    use_effect(move || {
        let mut chat_state_clone = chat_state.clone();
        spawn(async move {
            let init_flag = CHAT_INIT_FLAG.lock().await;
            if *init_flag {
                let global_state = GLOBAL_CHAT_STATE.lock().await;
                if !global_state.peers.is_empty() || !global_state.groups.is_empty() || !global_state.conversations.is_empty() {
                    *chat_state_clone.write() = global_state.clone();
                    drop(global_state);
                    println!("Chat state restored from global state");
                    return;
                }
            }
            drop(init_flag);

            println!("Chat initialized - loading groups and peers");

            match AuthService::new().await {
                Ok(auth_service) => match auth_service.get_all_groups().await {
                    Ok(groups) => {
                        let converted: Vec<crate::features::chat::chat_state::Group> = groups.iter().map(|g| g.into()).collect();
                        let current_state = chat_state_clone.read();
                        let peers = current_state.peers.clone();
                        let conversations = current_state.conversations.clone();
                        let group_share_notifications = current_state.group_share_notifications.clone();
                        let active_downloads = current_state.active_downloads.clone();
                        let loading = current_state.loading;
                        let error = current_state.error.clone();
                        drop(current_state);
                        let new_state = ChatState {
                            peers,
                            groups: converted,
                            conversations,
                            group_share_notifications,
                            active_downloads,
                            loading,
                            error,
                        };
                        *chat_state_clone.write() = new_state.clone();
                        *GLOBAL_CHAT_STATE.lock().await = new_state;
                        println!("Loaded {} groups from auth service", groups.len());
                    }
                    Err(e) => {
                        println!("Failed to load groups: {:?}", e);
                    }
                },
                Err(e) => {
                    println!("Failed to create auth service: {:?}", e);
                }
            }

            match crate::features::chat::chat_state::list_peers().await {
                peers if !peers.is_empty() => {
                    let peer_count = peers.len();
                    let current_state = chat_state_clone.read();
                    let groups = current_state.groups.clone();
                    let conversations = current_state.conversations.clone();
                    let group_share_notifications = current_state.group_share_notifications.clone();
                    let active_downloads = current_state.active_downloads.clone();
                    let loading = current_state.loading;
                    let error = current_state.error.clone();
                    drop(current_state);
                    let new_state = ChatState {
                        peers,
                        groups,
                        conversations,
                        group_share_notifications,
                        active_downloads,
                        loading,
                        error,
                    };
                    *chat_state_clone.write() = new_state.clone();
                    *GLOBAL_CHAT_STATE.lock().await = new_state;
                    println!("Loaded {} existing peers from P2P service", peer_count);
                }
                _ => {
                    println!("No existing peers found or failed to load peers");
                }
            }

            match load_conversations().await {
                conversations if !conversations.is_empty() => {
                    let conversation_count = conversations.len();
                    let current_state = chat_state_clone.read();
                    let peers = current_state.peers.clone();
                    let groups = current_state.groups.clone();
                    let group_share_notifications = current_state.group_share_notifications.clone();
                    let active_downloads = current_state.active_downloads.clone();
                    let loading = current_state.loading;
                    let error = current_state.error.clone();
                    drop(current_state);
                    let new_state = ChatState {
                        peers,
                        groups,
                        conversations,
                        group_share_notifications,
                        active_downloads,
                        loading,
                        error,
                    };
                    *chat_state_clone.write() = new_state.clone();
                    *GLOBAL_CHAT_STATE.lock().await = new_state;
                    println!("Loaded {} conversations from persistence", conversation_count);
                }
                _ => {
                    println!("No conversations found or failed to load conversations");
                }
            }

            let mut init_flag = CHAT_INIT_FLAG.lock().await;
            *init_flag = true;
        });
    });

    chat_state
}

// Hook for chat room initialization
pub fn use_chat_room_initialization(chat_id: String, chat_state: Signal<ChatState>) -> Signal<ChatRoomState> {
    let mut chat_room_state = use_chat_room_state();
    let history_loaded = use_signal(|| false);

    // Initialize chat room data based on the chat_id
    use_effect(move || {
        // Only load once
        if !*history_loaded.read() {
            // Set initial state
            chat_room_state.write().is_loading = true;
            chat_room_state.write().chat_id = Some(chat_id.clone());
            chat_room_state.write().is_group_chat = chat_id.starts_with("group");

            let is_group_chat = chat_id.starts_with("group");
            let mut chat_name = None;

            if is_group_chat {
                // Find group by id
                if let Some(group) = chat_state.read().groups.iter().find(|g| g.id == chat_id) {
                    chat_name = Some(group.name.clone());
                    chat_room_state.write().chat_name = Some(group.name.clone());
                    chat_room_state.write().group = Some(group.clone());
                }
            } else {
                // Find peer by id
                let chat_id_clone = chat_id.clone();
                if let Some(peer) = chat_state.read().peers.iter().find(|p| p.id == chat_id_clone) {
                    chat_name = Some(peer.nickname.clone());
                    chat_room_state.write().chat_name = Some(peer.nickname.clone());
                    chat_room_state.write().peer = Some(peer.clone());
                } else {
                    // If peer not found in chat_state, check if it's an offline peer with existing conversation
                    let chat_id_clone = chat_id.clone();
                    if let Some(conversation) = chat_state.read().conversations.iter().find(|c| {
                        if let Some(peer_id) = &c.peer_id {
                            peer_id == &chat_id_clone
                        } else {
                            false
                        }
                    }) {
                        // Create virtual offline peer from conversation data
                        let nickname = conversation.name.clone();
                        let peer_id_obj = chat_id_clone.parse::<gigi_p2p::PeerId>()
                            .unwrap_or_else(|_| {
                                gigi_p2p::PeerId::from_bytes(&[0u8; 32])
                                    .expect("Failed to create dummy PeerId")
                            });
                        
                        let frontend_peer = Peer {
                            id: chat_id_clone.clone(),
                            peer_id: peer_id_obj,
                            nickname: nickname.clone(),
                            is_online: false, // Mark as offline
                            capabilities: vec![],
                        };
                        
                        chat_name = Some(nickname.clone());
                        chat_room_state.write().chat_name = Some(nickname.clone());
                        chat_room_state.write().peer = Some(frontend_peer);
                    } else {
                        // If no conversation found, try to get from P2P service
                        let chat_id_clone = chat_id.clone();
                        let mut chat_room_state_clone = chat_room_state.clone();
                        let mut history_loaded_clone = history_loaded.clone();
                        spawn(async move {
                            if let Ok(peers) = P2pService::list_peers().await {
                                for peer in &peers {
                                    if peer.peer_id.to_string() == chat_id_clone {
                                        let nickname = peer.nickname.clone();
                                        let frontend_peer = Peer {
                                            id: peer.peer_id.to_string(),
                                            peer_id: peer.peer_id,
                                            nickname: nickname.clone(),
                                            is_online: peer.connected,
                                            capabilities: vec!["chat".to_string(), "file_sharing".to_string()],
                                        };
                                        chat_room_state_clone.write().chat_name = Some(nickname.clone());
                                        chat_room_state_clone.write().peer = Some(frontend_peer);
                                        
                                        // Load messages once chat name is available
                                        if let Ok(stored_messages) = PersistenceService::load_messages(&nickname, 50, 0).await {
                                            // Sort messages by timestamp (oldest first)
                                            let mut sorted_messages = stored_messages;
                                            sorted_messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                                            
                                            let messages: Vec<Message> = sorted_messages.iter().map(|m| m.into()).collect();
                                            chat_room_state_clone.write().messages = messages;
                                            println!("Loaded messages for chat: {}", chat_id_clone);
                                        }
                                        
                                        chat_room_state_clone.write().is_loading = false;
                                        *history_loaded_clone.write() = true;
                                        break;
                                    }
                                }
                            } else {
                                // If no peers found, mark as loaded
                                chat_room_state_clone.write().is_loading = false;
                                *history_loaded_clone.write() = true;
                            }
                        });
                    }
                }
            }

            // Load message history if chat name is already available
            if let Some(peer_nickname) = chat_name {
                let chat_id_clone = chat_id.clone();
                let mut chat_room_state_clone = chat_room_state.clone();
                let mut history_loaded_clone = history_loaded.clone();
                spawn(async move {
                    if let Ok(stored_messages) = PersistenceService::load_messages(&peer_nickname, 50, 0).await {
                        // Sort messages by timestamp (oldest first)
                        let mut sorted_messages = stored_messages;
                        sorted_messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                        
                        let messages: Vec<Message> = sorted_messages.iter().map(|m| m.into()).collect();
                        chat_room_state_clone.write().messages = messages;
                        println!("Loaded messages for chat: {}", chat_id_clone);
                    }
                    chat_room_state_clone.write().is_loading = false;
                    *history_loaded_clone.write() = true;
                });
            }
        }
    });

    chat_room_state
}

// Hook for chat event listeners
pub fn use_chat_event_listeners(chat_state: Signal<ChatState>, chat_room_state: Signal<ChatRoomState>) {

    use_effect(move || {
        println!("Chat event listeners initialized");

        // Subscribe to event bus
        if let Some(rx) = EventBus::subscribe() {
            let mut stream = BroadcastStream::new(rx);
            let mut chat_state_clone = chat_state.clone();
            let mut chat_room_state_clone = chat_room_state.clone();

            spawn(async move {
                println!("Chat event listener started");
                while let Some(Ok(event)) = stream.next().await {
                    println!("Received event: {:?}", event);
                    match event {
                        AppEvent::P2P(p2p_event) => {
                            println!("Received P2P event: {:?}", p2p_event);
                            match p2p_event {
                                gigi_p2p::P2pEvent::PeerDiscovered {
                                    peer_id, nickname, ..
                                } => {
                                    println!("Peer discovered: {} ({})", nickname, peer_id);
                                    let mut state = chat_state_clone.write();
                                    println!("Current peers: {:?}", state.peers);
                                    if let Some(peer) = state.peers.iter_mut().find(|p| p.id == peer_id.to_string()) {
                                        // Peer is already in the list, set to online
                                        peer.is_online = true;
                                        println!("Updated peer status: {} is now online", peer.nickname);
                                    } else {
                                        // Peer is not in the list, add it
                                        println!("Adding new peer: {} ({})", nickname, peer_id);
                                        state.peers.push(Peer {
                                            id: peer_id.to_string(),
                                            peer_id,
                                            nickname,
                                            is_online: true,
                                            capabilities: vec![
                                                "chat".to_string(),
                                                "file_sharing".to_string(),
                                            ],
                                        });
                                        println!("Updated peers: {:?}", state.peers);
                                    }
                                    // Update global state
                                    let mut global_state = GLOBAL_CHAT_STATE.lock().await;
                                    *global_state = state.clone();
                                }
                                gigi_p2p::P2pEvent::Connected { peer_id, nickname } => {
                                    println!("Peer connected: {}", peer_id);
                                    let mut state = chat_state_clone.write();
                                    let peer_id_str = peer_id.to_string();
                                    
                                    if let Some(peer) = state.peers.iter_mut().find(|p| p.peer_id == peer_id) {
                                        peer.is_online = true;
                                        println!("Updated peer status: {} is now online", peer.nickname);
                                    } else {
                                        // Add the peer to the list if they're not already there
                                        println!("Adding new peer from Connected event: {} ({})", nickname, peer_id);
                                        state.peers.push(Peer {
                                            id: peer_id_str.clone(),
                                            peer_id,
                                            nickname: nickname.clone(),
                                            is_online: true,
                                            capabilities: vec![
                                                "chat".to_string(),
                                                "file_sharing".to_string(),
                                            ],
                                        });
                                        println!("Updated peers: {:?}", state.peers);
                                    }
                                    // Update global state
                                    let mut global_state = GLOBAL_CHAT_STATE.lock().await;
                                    *global_state = state.clone();
                                    drop(state);
                                    
                                    // Update chat room state if this is the current peer
                                    let mut chat_room = chat_room_state_clone.write();
                                    if let Some(current_peer) = &chat_room.peer {
                                        if current_peer.id == peer_id_str {
                                            // Update the peer in chat room state with online status
                                            chat_room.peer = Some(Peer {
                                                id: peer_id_str.clone(),
                                                peer_id,
                                                nickname: nickname.clone(),
                                                is_online: true,
                                                capabilities: vec![
                                                    "chat".to_string(),
                                                    "file_sharing".to_string(),
                                                ],
                                            });
                                            println!("Updated chat room peer status: {} is now online", nickname);
                                        }
                                    }
                                    
                                    let nickname_clone = nickname.clone();
                                    spawn(async move {
                                        if let Err(e) = P2pService::deliver_pending_messages(&nickname_clone).await {
                                            println!("Failed to deliver pending messages to {}: {:?}", nickname_clone, e);
                                        } else {
                                            println!("Delivered pending messages to {}", nickname_clone);
                                        }
                                    });
                                }
                                gigi_p2p::P2pEvent::Disconnected { peer_id, .. } => {
                                    println!("Peer disconnected: {}", peer_id);
                                    let mut state = chat_state_clone.write();
                                    let peer_id_str = peer_id.to_string();
                                    
                                    if let Some(peer) = state.peers.iter_mut().find(|p| p.peer_id == peer_id) {
                                        peer.is_online = false;
                                        println!("Updated peer status: {} is now offline", peer.nickname);
                                    }
                                    // Update global state
                                    let mut global_state = GLOBAL_CHAT_STATE.lock().await;
                                    *global_state = state.clone();
                                    drop(state);
                                    
                                    // Update chat room state if this is the current peer
                                    let mut chat_room = chat_room_state_clone.write();
                                    if let Some(current_peer) = &chat_room.peer {
                                        if current_peer.id == peer_id_str {
                                            // Update the peer in chat room state with offline status
                                            if let Some(peer) = chat_state_clone.read().peers.iter().find(|p| p.peer_id == peer_id) {
                                                chat_room.peer = Some(Peer {
                                                    id: peer_id_str.clone(),
                                                    peer_id,
                                                    nickname: peer.nickname.clone(),
                                                    is_online: false,
                                                    capabilities: peer.capabilities.clone(),
                                                });
                                                println!("Updated chat room peer status: {} is now offline", peer.nickname);
                                            }
                                        }
                                    }
                                }
                                gigi_p2p::P2pEvent::DirectMessage {
                                    from_nickname,
                                    message,
                                    ..
                                } => {
                                    println!("Direct message from {}: {}", from_nickname, message);
                                    let state = chat_room_state_clone.read();
                                    // Check if this message is for the current chat
                                    // For direct messages, the chat name should match the sender's nickname
                                    if state.chat_name == Some(from_nickname.clone()) {
                                        drop(state);
                                        let mut state = chat_room_state_clone.write();
                                        state.messages.push(Message {
                                            id: uuid::Uuid::new_v4().to_string(),
                                            content: message,
                                            sender: from_nickname,
                                            timestamp: chrono::Local::now().format("%H:%M %p").to_string(),
                                            is_own: false,
                                            message_type: crate::features::chat::chat_state::MessageType::Text,
                                        });
                                        println!("Added message to chat room");
                                    } else {
                                        println!("Message from {} not added - current chat is: {:?}", from_nickname, state.chat_name);
                                    }
                                }
                                gigi_p2p::P2pEvent::FileDownloadStarted {
                                    download_id,
                                    filename,
                                    share_code,
                                    from,
                                    from_nickname,
                                    ..
                                } => {
                                    println!(
                                        "File download started: {} from {}",
                                        filename, from_nickname
                                    );
                                    let mut state = chat_state_clone.write();
                                    state.active_downloads.push(
                                        crate::features::chat::chat_state::ActiveDownload {
                                            download_id,
                                            filename,
                                            share_code,
                                            from_peer_id: from,
                                            from_nickname,
                                            downloaded_chunks: 0,
                                            total_chunks: 0,
                                            completed: false,
                                            failed: false,
                                            error_message: None,
                                            final_path: None,
                                        },
                                    );
                                    // Update global state
                                    let mut global_state = GLOBAL_CHAT_STATE.lock().await;
                                    *global_state = state.clone();
                                    println!("Added active download");
                                }
                                gigi_p2p::P2pEvent::FileDownloadProgress {
                                    download_id,
                                    downloaded_chunks,
                                    total_chunks,
                                    ..
                                } => {
                                    println!(
                                        "File download progress: {} chunks of {}",
                                        downloaded_chunks, total_chunks
                                    );
                                    let mut state = chat_state_clone.write();
                                    if let Some(dl) = state
                                        .active_downloads
                                        .iter_mut()
                                        .find(|d| d.download_id == download_id)
                                    {
                                        dl.downloaded_chunks = downloaded_chunks;
                                        dl.total_chunks = total_chunks;
                                        println!("Updated download progress");
                                    }
                                    // Update global state
                                    let mut global_state = GLOBAL_CHAT_STATE.lock().await;
                                    *global_state = state.clone();
                                }
                                gigi_p2p::P2pEvent::FileDownloadCompleted {
                                    download_id,
                                    path,
                                    ..
                                } => {
                                    println!("File download completed: {}", download_id);
                                    let mut state = chat_state_clone.write();
                                    if let Some(dl) = state
                                        .active_downloads
                                        .iter_mut()
                                        .find(|d| d.download_id == download_id)
                                    {
                                        dl.completed = true;
                                        dl.final_path = Some(path);
                                        println!("Updated download status to completed");
                                    }
                                    // Update global state
                                    let mut global_state = GLOBAL_CHAT_STATE.lock().await;
                                    *global_state = state.clone();
                                }
                                gigi_p2p::P2pEvent::FileDownloadFailed {
                                    download_id,
                                    error,
                                    ..
                                } => {
                                    println!("File download failed: {} - {}", download_id, error);
                                    let mut state = chat_state_clone.write();
                                    if let Some(dl) = state
                                        .active_downloads
                                        .iter_mut()
                                        .find(|d| d.download_id == download_id)
                                    {
                                        dl.failed = true;
                                        dl.error_message = Some(error);
                                        println!("Updated download status to failed");
                                    }
                                    // Update global state
                                    let mut global_state = GLOBAL_CHAT_STATE.lock().await;
                                    *global_state = state.clone();
                                }
                                _ => {
                                    println!("Other P2P event: {:?}", p2p_event);
                                }
                            }
                        }
                        AppEvent::MessageSaved(chat_id) => {
                            println!("Message saved for chat: {}", chat_id);
                            // Refresh current chat room if it matches
                            if chat_room_state_clone.read().chat_id == Some(chat_id.clone()) {
                                let chat_id_clone = chat_id.clone();
                                let chat_name = chat_room_state_clone.read().chat_name.clone();
                                let mut chat_room_state_refresh = chat_room_state_clone.clone();
                                spawn(async move {
                                    if let Some(peer_nickname) = chat_name {
                                        if let Ok(stored_messages) = 
                                                PersistenceService::load_messages(&peer_nickname, 50, 0)
                                                    .await
                                            {
                                                // Sort messages by timestamp (oldest first)
                                                let mut sorted_messages = stored_messages;
                                                sorted_messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                                                
                                                let messages: Vec<Message> = 
                                                    sorted_messages.iter().map(|m| m.into()).collect();
                                                chat_room_state_refresh.write().messages = messages;
                                                println!(
                                                    "Refreshed messages for chat: {}",
                                                    chat_id_clone
                                                );
                                            }
                                    }
                                });
                            }
                            // Always refresh conversations list to show new messages and unread counts
                            let mut chat_state_refresh = chat_state_clone.clone();
                            spawn(async move {
                                let conversations = load_conversations().await;
                                if !conversations.is_empty() {
                                    let mut state = chat_state_refresh.write();
                                    state.conversations = conversations;
                                    // Update global state
                                    let mut global_state = GLOBAL_CHAT_STATE.lock().await;
                                    *global_state = state.clone();
                                    println!("Refreshed conversations list");
                                } else {
                                    println!("No conversations to refresh");
                                }
                            });
                        }
                        AppEvent::ContactUpdated => {
                            println!("Contact updated event");
                        }
                        AppEvent::GroupUpdated => {
                            println!("Group updated event");
                        }
                    }
                }
                println!("Chat event listener stopped");
            });
        }
    });
}

// Hook for chat data refresh
pub fn use_chat_data_refresh() {
    // In a real app, this would periodically refresh chat data
    use_effect(|| {
        let interval = std::time::Duration::from_secs(30);
        let handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                // In a real app, this would refresh data from the backend
                println!("Refreshing chat data");
            }
        });

        // Cleanup function
        handle.abort();
    });
}

// Hook for peer actions
pub fn use_peer_actions() -> (impl Fn(String),) {
    let chat_state = use_chat_state();
    let navigator = dioxus_router::use_navigator();

    let handle_peer_click = move |peer_id: String| {
        // Navigate to the chat room for the peer
        navigator.push(format!("/chat/{}", peer_id));
    };

    (handle_peer_click,)
}

// Hook for group actions
pub fn use_group_actions() -> (
    Arc<dyn Fn(String) + Send + Sync>,
    Arc<dyn Fn(String) + Send + Sync>,
    Arc<dyn Fn(String) + Send + Sync>,
    Arc<dyn Fn(String) + Send + Sync>,
) {
    let chat_state = use_chat_state();

    let handle_share_group = Arc::new(move |group_id: String| {
        // In a real app, this would open a share dialog for the group
        println!("Share group: {}", group_id);
    });

    let handle_accept_group_share = Arc::new(move |notification_id: String| {
        // In a real app, this would accept a group share invitation
        println!("Accept group share: {}", notification_id);
    });

    let handle_ignore_group_share = Arc::new(move |notification_id: String| {
        // In a real app, this would ignore a group share invitation
        println!("Ignore group share: {}", notification_id);
    });

    let handle_clear_messages = Arc::new(move |peer_nickname: String| {
        println!("Clear messages for peer: {}", peer_nickname);
        
        // Direct chat - clear messages using the provided peer nickname
        spawn(async move {
            println!("Clearing messages for peer with nickname: {}", peer_nickname);
            
            // Clear messages using the actual nickname
            match PersistenceService::clear_conversation(&peer_nickname).await {
                Ok(count) => {
                    println!("Cleared {} messages from conversation with {}", count, peer_nickname);
                    
                    // Emit event to update chat room state
                    // We don't have a chat_id, but the event will trigger a refresh
                    if let Err(err) = EventBus::send(AppEvent::MessageSaved(peer_nickname.clone())) {
                        println!("Error sending MessageSaved event: {:?}", err);
                    }
                },
                Err(e) => {
                    println!("Failed to clear messages from persistence: {:?}", e);
                }
            }
        });
    });

    (
        handle_share_group,
        handle_accept_group_share,
        handle_ignore_group_share,
        handle_clear_messages,
    )
}

// Hook for message actions
pub fn use_message_actions(mut chat_room_state: Signal<ChatRoomState>) -> (
    impl FnMut(),
    impl FnMut(),
    impl FnMut(),
    impl Fn(String),
    impl Fn(&str, PathBuf),
    impl FnMut(String),
) {

    let handle_send_message = move || {
        println!("handle_send_message called");
        if !chat_room_state.read().new_message.is_empty() {
            println!("Message content: {}", chat_room_state.read().new_message);
            let new_msg = crate::features::chat::chat_state::Message {
                id: uuid::Uuid::new_v4().to_string(),
                content: chat_room_state.read().new_message.clone(),
                sender: "You".to_string(),
                timestamp: chrono::Local::now().format("%H:%M %p").to_string(),
                is_own: true,
                message_type: crate::features::chat::chat_state::MessageType::Text,
            };
            println!("Created message: {:?}", new_msg);
            chat_room_state.write().messages.push(new_msg.clone());
            println!("Messages count after push: {}", chat_room_state.read().messages.len());
            chat_room_state.write().new_message = "".to_string();
            println!("New message after clear: '{}'", chat_room_state.read().new_message);

            // Send message via P2P
            if let Some(chat_name) = chat_room_state.read().chat_name.clone() {
                println!("Sending message to: {}", chat_name);
                let message_content = new_msg.content.clone();
                let is_group_chat = chat_room_state.read().is_group_chat;
                spawn(async move {
                    println!("Async sending message: {} to {}", message_content, chat_name);
                    if is_group_chat {
                        crate::features::chat::chat_state::send_group_message(
                            &chat_name,
                            &message_content,
                        )
                        .await;
                    } else {
                        crate::features::chat::chat_state::send_message(
                            &chat_name,
                            &message_content,
                        )
                        .await;
                    }
                });
            } else {
                println!("No chat name available for sending message");
            }

            // Save message to persistence and send event
            if let Some(chat_id) = chat_room_state.read().chat_id.clone() {
                println!("Saving message to persistence for chat: {}", chat_id);
                let new_msg_clone = new_msg.clone();
                let chat_name = chat_room_state.read().chat_name.clone();
                let is_group_chat = chat_room_state.read().is_group_chat;
                spawn(async move {
                    // Get local nickname from P2P service
                    let local_nickname = crate::services::p2p_service::P2pService::get_local_nickname().await.unwrap_or("You".to_string());
                    
                    if let Err(err) = PersistenceService::store_direct_message(
                        local_nickname.clone(),
                        chat_name.clone().unwrap_or_default(),
                        new_msg_clone.content.clone(),
                        true,
                    )
                    .await {
                        println!("Error saving message: {:?}", err);
                    }
                    
                    // Create or update conversation for direct messages
                    if !is_group_chat {
                        if let Some(ref peer_nickname) = chat_name {
                            if let Err(err) = PersistenceService::upsert_conversation(
                                chat_id.clone(),
                                peer_nickname.clone(),
                                false, // not a group
                                chat_id.clone(),
                                Some(new_msg_clone.content),
                                Some(chrono::Utc::now()),
                            )
                            .await {
                                println!("Error upserting conversation: {:?}", err);
                            }
                        }
                    }
                    
                    if let Err(err) = EventBus::send(AppEvent::MessageSaved(chat_id)) {
                        println!("Error sending MessageSaved event: {:?}", err);
                    }
                });
            } else {
                println!("No chat ID available for saving message");
            }
        } else {
            println!("Message is empty, not sending");
        }
    };

    let handle_image_select = move || {
        println!("Select image");
    };

    let handle_file_select = move || {
        println!("Select file");
    };

    let handle_file_download_request = move |_share_code: String| {
        // For now, we don't have the nickname from MessageList, so just log
        println!("Download requested for: {}", _share_code);
        // TODO: We need both nickname and share_code to download
    };

    let handle_share_file = move |to_nickname: &str, file_path: PathBuf| {
        let to_nickname = to_nickname.to_string();
        spawn(async move {
            let _ = P2pService::share_file(&to_nickname, &file_path).await;
        });
    };

    let handle_delete_message = move |message_id: String| {
        println!("Deleting message with ID: {}", message_id);
        let mut state = chat_room_state.write();
        crate::features::chat::chat_state::delete_message(&mut state.messages, &message_id);
        println!("Message deleted, new count: {}", state.messages.len());
    };

    (
        handle_send_message,
        handle_image_select,
        handle_file_select,
        handle_file_download_request,
        handle_share_file,
        handle_delete_message,
    )
}
