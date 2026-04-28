use dioxus::prelude::*;
use futures_util::StreamExt;
use gigi_p2p::PeerId;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;

use crate::features::chat::chat_state::{
    load_conversations, use_chat_room_state, use_chat_state, ChatRoomState, ChatState, Group,
    Message, Peer, GLOBAL_CHAT_STATE,
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
                if !global_state.peers.is_empty()
                    || !global_state.groups.is_empty()
                    || !global_state.conversations.is_empty()
                {
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
                        let converted: Vec<crate::features::chat::chat_state::Group> =
                            groups.iter().map(|g| g.into()).collect();
                        let current_state = chat_state_clone.read();
                        let peers = current_state.peers.clone();
                        let conversations = current_state.conversations.clone();
                        let group_share_notifications =
                            current_state.group_share_notifications.clone();
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

                        // Subscribe to all groups
                        for group in &groups {
                            if group.joined {
                                println!("Subscribing to group: {}", group.name);
                                if let Err(err) = P2pService::join_group(&group.name).await {
                                    println!("Failed to join group {}: {:?}", group.name, err);
                                } else {
                                    println!("Successfully joined group: {}", group.name);
                                }
                            }
                        }
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
                    let mut groups = current_state.groups.clone();
                    let group_share_notifications = current_state.group_share_notifications.clone();
                    let active_downloads = current_state.active_downloads.clone();
                    let loading = current_state.loading;
                    let error = current_state.error.clone();
                    drop(current_state);
                    let new_state = ChatState {
                        peers,
                        groups: groups.clone(),
                        conversations: conversations.clone(),
                        group_share_notifications,
                        active_downloads,
                        loading,
                        error,
                    };
                    *chat_state_clone.write() = new_state.clone();
                    *GLOBAL_CHAT_STATE.lock().await = new_state;
                    println!(
                        "Loaded {} conversations from persistence",
                        conversation_count
                    );

                    // Subscribe to all group conversations
                    for conversation in &conversations {
                        if let Some(group_id) = &conversation.group_id {
                            // Check if we already have this group in our groups list
                            let group_exists = groups
                                .iter()
                                .any(|g| g.id == *group_id || g.name == *group_id);

                            if !group_exists {
                                // Subscribe to the group using the conversation name (which is the group name)
                                println!(
                                    "Subscribing to group from conversation: {}",
                                    conversation.name
                                );
                                if let Err(err) = P2pService::join_group(&conversation.name).await {
                                    println!(
                                        "Failed to join group {} from conversation: {:?}",
                                        conversation.name, err
                                    );
                                } else {
                                    println!(
                                        "Successfully joined group from conversation: {}",
                                        conversation.name
                                    );
                                    // Add the group to our groups list
                                    groups.push(crate::features::chat::chat_state::Group {
                                        id: group_id.clone(),
                                        name: conversation.name.clone(),
                                        role: "Member".to_string(),
                                        member_count: 0,
                                        joined: true,
                                    });
                                }
                            }
                        }
                    }

                    // Update state with newly joined groups
                    if !groups.is_empty() {
                        let groups_clone = groups.clone();
                        let mut state = chat_state_clone.write();
                        state.groups = groups;
                        let mut global_state = GLOBAL_CHAT_STATE.lock().await;
                        global_state.groups = groups_clone;
                    }
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
pub fn use_chat_room_initialization(
    chat_id: String,
    chat_state: Signal<ChatState>,
) -> Signal<ChatRoomState> {
    let mut chat_room_state = use_chat_room_state();
    let history_loaded = use_signal(|| false);

    // Initialize chat room data based on the chat_id
    use_effect(move || {
        // Only load once
        if !*history_loaded.read() {
            // Set initial state
            chat_room_state.write().is_loading = true;
            chat_room_state.write().chat_id = Some(chat_id.clone());

            // Determine if it's a group chat and set chat name
            let mut is_group_chat = false;
            let mut chat_name = None;
            let mut found_group_name = None;

            // First check if it's a group by ID or name
            if let Some(group) = chat_state
                .read()
                .groups
                .iter()
                .find(|g| g.id == chat_id || g.name == chat_id)
            {
                is_group_chat = true;
                chat_name = Some(group.name.clone());
                found_group_name = Some(group.name.clone());
                chat_room_state.write().chat_name = Some(group.name.clone());
                chat_room_state.write().group = Some(group.clone());
            }
            // Then check if it's a peer by ID
            else if let Some(peer) = chat_state.read().peers.iter().find(|p| p.id == chat_id) {
                is_group_chat = false;
                chat_name = Some(peer.nickname.clone());
                chat_room_state.write().chat_name = Some(peer.nickname.clone());
                chat_room_state.write().peer = Some(peer.clone());
            }
            // Check if it's a group conversation
            else if let Some(conversation) = chat_state
                .read()
                .conversations
                .iter()
                .find(|c| c.group_id.as_ref() == Some(&chat_id))
            {
                is_group_chat = true;
                chat_name = Some(conversation.name.clone());
                found_group_name = Some(conversation.name.clone());
                chat_room_state.write().chat_name = Some(conversation.name.clone());

                // Try to find the group info
                if let Some(group) = chat_state
                    .read()
                    .groups
                    .iter()
                    .find(|g| g.id == chat_id || g.name == conversation.name)
                {
                    chat_room_state.write().group = Some(group.clone());
                }
            }
            // Check if it's a group conversation by conversation id (group-{name} format)
            else if let Some(conversation) = chat_state
                .read()
                .conversations
                .iter()
                .find(|c| c.id == chat_id && c.group_id.is_some())
            {
                is_group_chat = true;
                chat_name = Some(conversation.name.clone());
                found_group_name = Some(conversation.name.clone());
                chat_room_state.write().chat_name = Some(conversation.name.clone());

                // Try to find the group info
                if let Some(group) = chat_state.read().groups.iter().find(|g| {
                    g.id == conversation.group_id.clone().unwrap_or_default()
                        || g.name == conversation.name
                }) {
                    chat_room_state.write().group = Some(group.clone());
                }
            }
            // Check if it's an offline peer with existing conversation
            else if let Some(conversation) = chat_state
                .read()
                .conversations
                .iter()
                .find(|c| c.peer_id.as_ref() == Some(&chat_id))
            {
                is_group_chat = false;
                chat_name = Some(conversation.name.clone());
                chat_room_state.write().chat_name = Some(conversation.name.clone());

                // Create virtual offline peer
                let peer_id_obj = chat_id.parse::<gigi_p2p::PeerId>().unwrap_or_else(|_| {
                    gigi_p2p::PeerId::from_bytes(&[0u8; 32]).expect("Failed to create dummy PeerId")
                });

                let frontend_peer = Peer {
                    id: chat_id.clone(),
                    peer_id: peer_id_obj,
                    nickname: conversation.name.clone(),
                    is_online: false,
                    capabilities: vec![],
                };
                chat_room_state.write().peer = Some(frontend_peer);
            }
            // Try to get from P2P service
            else {
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
                                    capabilities: vec!["chat", "file_sharing"]
                                        .iter()
                                        .map(|s| s.to_string())
                                        .collect(),
                                };
                                chat_room_state_clone.write().chat_name = Some(nickname.clone());
                                chat_room_state_clone.write().peer = Some(frontend_peer);
                                chat_room_state_clone.write().is_group_chat = false;

                                // Load messages
                                if let Ok(stored_messages) =
                                    PersistenceService::load_messages(&nickname, 50, 0).await
                                {
                                    let mut sorted_messages = stored_messages;
                                    sorted_messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                                    let messages: Vec<Message> =
                                        sorted_messages.iter().map(|m| m.into()).collect();
                                    chat_room_state_clone.write().messages = messages;
                                    println!("Loaded messages for chat: {}", chat_id_clone);
                                }

                                chat_room_state_clone.write().is_loading = false;
                                *history_loaded_clone.write() = true;
                                break;
                            }
                        }
                    } else {
                        chat_room_state_clone.write().is_loading = false;
                        *history_loaded_clone.write() = true;
                    }
                });
            }

            // Set the is_group_chat flag
            chat_room_state.write().is_group_chat = is_group_chat;

            // Join group if it's a group chat
            if is_group_chat && found_group_name.is_some() {
                let group_name = found_group_name.clone().unwrap();
                let mut chat_room_state_clone = chat_room_state.clone();
                spawn(async move {
                    println!("Joining group when entering chat room: {}", group_name);
                    if let Err(err) = P2pService::join_group(&group_name).await {
                        println!(
                            "Failed to join group {} when entering chat room: {:?}",
                            group_name, err
                        );
                    } else {
                        println!("Successfully joined group: {}", group_name);
                    }
                });
            }

            // Load message history if chat name is already available
            if let Some(chat_name_value) = chat_name {
                let chat_id_clone = chat_id.clone();
                let mut chat_room_state_clone = chat_room_state.clone();
                let mut history_loaded_clone = history_loaded.clone();
                let is_group_chat_clone = is_group_chat;
                spawn(async move {
                    let stored_messages = if is_group_chat_clone {
                        PersistenceService::load_group_messages(&chat_name_value, 50, 0).await
                    } else {
                        PersistenceService::load_messages(&chat_name_value, 50, 0).await
                    };

                    if let Ok(stored_messages) = stored_messages {
                        // Sort messages by timestamp (oldest first)
                        let mut sorted_messages = stored_messages;
                        sorted_messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

                        let messages: Vec<Message> =
                            sorted_messages.iter().map(|m| m.into()).collect();
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
pub fn use_chat_event_listeners(
    chat_state: Signal<ChatState>,
    chat_room_state: Signal<ChatRoomState>,
) {
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
                                    if let Some(peer) =
                                        state.peers.iter_mut().find(|p| p.id == peer_id.to_string())
                                    {
                                        // Peer is already in the list, set to online
                                        peer.is_online = true;
                                        println!(
                                            "Updated peer status: {} is now online",
                                            peer.nickname
                                        );
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

                                    if let Some(peer) =
                                        state.peers.iter_mut().find(|p| p.peer_id == peer_id)
                                    {
                                        peer.is_online = true;
                                        println!(
                                            "Updated peer status: {} is now online",
                                            peer.nickname
                                        );
                                    } else {
                                        // Add the peer to the list if they're not already there
                                        println!(
                                            "Adding new peer from Connected event: {} ({})",
                                            nickname, peer_id
                                        );
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
                                            println!(
                                                "Updated chat room peer status: {} is now online",
                                                nickname
                                            );
                                        }
                                    }

                                    let nickname_clone = nickname.clone();
                                    spawn(async move {
                                        if let Err(e) =
                                            P2pService::deliver_pending_messages(&nickname_clone)
                                                .await
                                        {
                                            println!(
                                                "Failed to deliver pending messages to {}: {:?}",
                                                nickname_clone, e
                                            );
                                        } else {
                                            println!(
                                                "Delivered pending messages to {}",
                                                nickname_clone
                                            );
                                        }
                                    });
                                }
                                gigi_p2p::P2pEvent::Disconnected { peer_id, .. } => {
                                    println!("Peer disconnected: {}", peer_id);
                                    let mut state = chat_state_clone.write();
                                    let peer_id_str = peer_id.to_string();

                                    if let Some(peer) =
                                        state.peers.iter_mut().find(|p| p.peer_id == peer_id)
                                    {
                                        peer.is_online = false;
                                        println!(
                                            "Updated peer status: {} is now offline",
                                            peer.nickname
                                        );
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
                                            if let Some(peer) = chat_state_clone
                                                .read()
                                                .peers
                                                .iter()
                                                .find(|p| p.peer_id == peer_id)
                                            {
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
                                            filename: None,
                                            file_size: None,
                                            file_type: None,
                                            share_code: None,
                                            is_downloading: false,
                                            download_progress: None,
                                            download_id: None,
                                            file_path: None,
                                        });
                                        println!("Added message to chat room");
                                    } else {
                                        println!(
                                            "Message from {} not added - current chat is: {:?}",
                                            from_nickname, state.chat_name
                                        );
                                    }
                                }
                                gigi_p2p::P2pEvent::GroupMessage {
                                    from_nickname,
                                    group,
                                    message,
                                    ..
                                } => {
                                    println!(
                                        "Group message from {} in {}: {}",
                                        from_nickname, group, message
                                    );
                                    let state = chat_room_state_clone.read();
                                    // Check if this message is for the current chat
                                    // For group messages, the chat name should match the group name
                                    if state.chat_name == Some(group.clone()) && state.is_group_chat
                                    {
                                        drop(state);
                                        let mut state = chat_room_state_clone.write();
                                        state.messages.push(Message {
                                            id: uuid::Uuid::new_v4().to_string(),
                                            content: message,
                                            sender: from_nickname,
                                            timestamp: chrono::Local::now().format("%H:%M %p").to_string(),
                                            is_own: false,
                                            message_type: crate::features::chat::chat_state::MessageType::Text,
                                            filename: None,
                                            file_size: None,
                                            file_type: None,
                                            share_code: None,
                                            is_downloading: false,
                                            download_progress: None,
                                            download_id: None,
                                            file_path: None,
                                        });
                                        println!("Added group message to chat room");
                                    } else {
                                        println!(
                                            "Group message from {} in {} not added - current chat is: {:?} (group: {})",
                                            from_nickname, group, state.chat_name, state.is_group_chat
                                        );
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
                            // chat_id could be either nickname or peer_id, so check both chat_id and chat_name
                            let state = chat_room_state_clone.read();
                            let should_refresh = state.chat_id == Some(chat_id.clone())
                                || state.chat_name == Some(chat_id.clone());
                            let is_group_chat = state.is_group_chat;
                            drop(state);

                            if should_refresh {
                                let chat_id_clone = chat_id.clone();
                                let chat_name = chat_room_state_clone.read().chat_name.clone();
                                let mut chat_room_state_refresh = chat_room_state_clone.clone();
                                spawn(async move {
                                    if let Some(chat_name_value) = chat_name {
                                        let stored_messages = if is_group_chat {
                                            PersistenceService::load_group_messages(
                                                &chat_name_value,
                                                50,
                                                0,
                                            )
                                            .await
                                        } else {
                                            PersistenceService::load_messages(
                                                &chat_name_value,
                                                50,
                                                0,
                                            )
                                            .await
                                        };

                                        if let Ok(stored_messages) = stored_messages {
                                            // Sort messages by timestamp (oldest first)
                                            let mut sorted_messages = stored_messages;
                                            sorted_messages
                                                .sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

                                            // Get current messages before replacing
                                            let current_messages =
                                                chat_room_state_refresh.read().messages.clone();

                                            let mut new_messages: Vec<Message> =
                                                sorted_messages.iter().map(|m| m.into()).collect();

                                            // Merge file_path, is_downloading, download_progress, download_id from existing messages
                                            for new_msg in new_messages.iter_mut() {
                                                if let Some(existing_msg) = current_messages
                                                    .iter()
                                                    .find(|m| m.id == new_msg.id)
                                                {
                                                    if new_msg.file_path.is_none()
                                                        && existing_msg.file_path.is_some()
                                                    {
                                                        new_msg.file_path =
                                                            existing_msg.file_path.clone();
                                                    }
                                                    if !new_msg.is_downloading
                                                        && existing_msg.is_downloading
                                                    {
                                                        new_msg.is_downloading =
                                                            existing_msg.is_downloading;
                                                    }
                                                    if new_msg.download_progress.is_none()
                                                        && existing_msg.download_progress.is_some()
                                                    {
                                                        new_msg.download_progress =
                                                            existing_msg.download_progress;
                                                    }
                                                    if new_msg.download_id.is_none()
                                                        && existing_msg.download_id.is_some()
                                                    {
                                                        new_msg.download_id =
                                                            existing_msg.download_id.clone();
                                                    }
                                                }
                                            }

                                            chat_room_state_refresh.write().messages = new_messages;
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
                        AppEvent::FileDownloadProgress {
                            download_id,
                            progress,
                        } => {
                            println!("File download progress: {} - {}%", download_id, progress);
                            // Update the message with the download progress
                            let mut chat_room = chat_room_state_clone.write();
                            if let Some(message) = chat_room
                                .messages
                                .iter_mut()
                                .find(|m| m.download_id == Some(download_id.clone()))
                            {
                                message.download_progress = Some(progress);
                                println!("Updated message download progress: {}%", progress);
                            }
                        }
                        AppEvent::FileDownloadCompleted { download_id, path } => {
                            println!("File download completed: {} at {:?}", download_id, path);
                            // Update the message with the download completion
                            let mut chat_room = chat_room_state_clone.write();
                            if let Some(message) = chat_room
                                .messages
                                .iter_mut()
                                .find(|m| m.download_id == Some(download_id.clone()))
                            {
                                message.is_downloading = false;
                                message.download_progress = Some(100);

                                // Check if it's an image and update file_path to use the thumbnail
                                if let Some(file_type) = &message.file_type {
                                    let lower_file_type = file_type.to_lowercase();
                                    let is_image = lower_file_type.starts_with("image/")
                                        || ["png", "jpg", "jpeg", "gif", "bmp", "webp"]
                                            .contains(&lower_file_type.as_str());
                                    if is_image {
                                        // Use the original filename from the message, not the one from the path (which may have a suffix)
                                        if let Some(original_filename) = &message.filename {
                                            // Get data directory
                                            let data_dir = std::env::var("GIGI_DATA_DIR")
                                                .unwrap_or_else(|_| {
                                                    dirs::data_local_dir()
                                                        .unwrap_or_else(|| {
                                                            std::path::PathBuf::from(".")
                                                        })
                                                        .join("gigi-dioxus")
                                                        .to_string_lossy()
                                                        .to_string()
                                                });

                                            // Expand ~ to home directory
                                            let data_dir_expanded = if data_dir.starts_with('~') {
                                                if let Some(home) = dirs::home_dir() {
                                                    home.join(
                                                        data_dir.strip_prefix('~').unwrap_or(""),
                                                    )
                                                } else {
                                                    std::path::PathBuf::from(data_dir)
                                                }
                                            } else {
                                                std::path::PathBuf::from(data_dir)
                                            };

                                            let downloads_dir = data_dir_expanded.join("downloads");
                                            let thumbnail_path = downloads_dir.join(format!(
                                                "{}.thumbnail.jpg",
                                                original_filename
                                            ));
                                            message.file_path =
                                                Some(thumbnail_path.to_string_lossy().to_string());
                                            println!(
                                                "Updated message file_path to thumbnail: {:?}",
                                                thumbnail_path
                                            );
                                        } else {
                                            message.file_path =
                                                Some(path.to_string_lossy().to_string());
                                            println!("Updated message download status to completed with path: {:?}", path);
                                        }
                                    } else {
                                        message.file_path =
                                            Some(path.to_string_lossy().to_string());
                                        println!("Updated message download status to completed with path: {:?}", path);
                                    }
                                } else {
                                    message.file_path = Some(path.to_string_lossy().to_string());
                                    println!("Updated message download status to completed with path: {:?}", path);
                                }
                            }
                        }
                        AppEvent::FileDownloadFailed { download_id, error } => {
                            println!("File download failed: {} - {}", download_id, error);
                            // Update the message with the download failure
                            let mut chat_room = chat_room_state_clone.write();
                            if let Some(message) = chat_room
                                .messages
                                .iter_mut()
                                .find(|m| m.download_id == Some(download_id.clone()))
                            {
                                message.is_downloading = false;
                                message.download_progress = None;
                                println!("Updated message download status to failed");
                            }
                        }
                        AppEvent::ContactUpdated => {
                            println!("Contact updated event");
                        }
                        AppEvent::GroupUpdated => {
                            println!("Group updated event");
                        }
                        AppEvent::FileShareReceived {
                            from_peer_id,
                            from_nickname,
                            share_code,
                            filename,
                            file_size,
                            file_type,
                            conv_id: _,
                        } => {
                            println!(
                                "File share received from {}: {} (code: {})",
                                from_nickname, filename, share_code
                            );
                            let state = chat_room_state_clone.read();
                            // Check if this message is for the current chat by comparing from_nickname with chat_name
                            // chat_id is set to the peer's nickname when opening a conversation
                            // from_nickname is the sender's nickname
                            if state.chat_name == Some(from_nickname.clone()) {
                                drop(state);
                                let mut chat_room = chat_room_state_clone.write();

                                // Check if it's an image file (handle both MIME types and extensions)
                                println!(
                                    "Checking file type: {}, lowercase: {}",
                                    file_type,
                                    file_type.to_lowercase()
                                );
                                let lower_file_type = file_type.to_lowercase();
                                let is_image = lower_file_type.starts_with("image/")
                                    || ["png", "jpg", "jpeg", "gif", "bmp", "webp"]
                                        .contains(&lower_file_type.as_str());

                                // Create a new message for the file share
                                let new_message = Message {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    content: format!("Shared file: {}", filename),
                                    sender: from_nickname.clone(),
                                    timestamp: chrono::Local::now().format("%H:%M %p").to_string(),
                                    is_own: false,
                                    message_type: if is_image {
                                        crate::features::chat::chat_state::MessageType::Image
                                    } else {
                                        crate::features::chat::chat_state::MessageType::File
                                    },
                                    filename: Some(filename),
                                    file_size: Some(file_size),
                                    file_type: Some(file_type),
                                    share_code: Some(share_code),
                                    is_downloading: false,
                                    download_progress: None,
                                    download_id: None,
                                    file_path: None,
                                };

                                chat_room.messages.push(new_message);
                                println!("Added file share message to chat room");
                            }
                        }
                    }
                }
            });
        }
    });
}

// Hook for message actions like sending, image selection, etc.
pub fn use_message_actions(
    chat_room_state: Signal<ChatRoomState>,
) -> (
    impl Fn(),
    impl Fn(),
    impl Fn(),
    impl Fn(String, String, String),
    impl Fn(String, String, Option<String>),
    impl Fn(String),
) {
    let mut chat_room_state_clone = chat_room_state.clone();

    let handle_send_message = move || {
        println!("handle_send_message called");
        let mut chat_room_state = chat_room_state_clone.clone();
        let message_content = chat_room_state.read().new_message.clone();
        println!("Message content: {}", message_content);

        if message_content.is_empty() {
            println!("Message content is empty, returning early");
            return;
        }

        // Check if chat name is available
        let chat_name = chat_room_state.read().chat_name.clone();
        if let Some(chat_name) = chat_name {
            println!("Chat name: {}", chat_name);

            // Create a new message
            let new_msg = Message {
                id: uuid::Uuid::new_v4().to_string(),
                content: message_content.clone(),
                sender: "You".to_string(),
                timestamp: chrono::Local::now().format("%H:%M %p").to_string(),
                is_own: true,
                message_type: crate::features::chat::chat_state::MessageType::Text,
                filename: None,
                file_size: None,
                file_type: None,
                share_code: None,
                is_downloading: false,
                download_progress: None,
                download_id: None,
                file_path: None,
            };

            println!("Created message: {:?}", new_msg);

            // Add the message to the chat room state
            chat_room_state.write().sending = true;
            chat_room_state.write().messages.push(new_msg.clone());
            println!(
                "Messages count after push: {}",
                chat_room_state.read().messages.len()
            );

            // Clear the input field
            chat_room_state.write().new_message = "".to_string();
            println!(
                "New message after clear: '{}'",
                chat_room_state.read().new_message
            );

            // Send the message via P2P service
            let is_group_chat = chat_room_state.read().is_group_chat;
            let chat_id = chat_room_state.read().chat_id.clone();

            spawn(async move {
                if is_group_chat {
                    // Ensure group is joined before sending message
                    println!("Ensuring group is joined before sending: {}", chat_name);
                    if let Err(err) =
                        crate::services::p2p_service::P2pService::join_group(&chat_name).await
                    {
                        println!(
                            "Warning: Failed to join group {} (may already be joined): {:?}",
                            chat_name, err
                        );
                    }

                    // Send group message
                    if let Err(err) = crate::services::p2p_service::P2pService::send_group_message(
                        &chat_name,
                        &message_content,
                    )
                    .await
                    {
                        println!("Error sending group message: {:?}", err);
                    } else {
                        println!("Group message sent successfully");
                    }
                } else {
                    // Send direct message
                    if let Err(err) = crate::services::p2p_service::P2pService::send_message(
                        &chat_name,
                        &message_content,
                    )
                    .await
                    {
                        println!("Error sending message: {:?}", err);
                    } else {
                        println!("Message sent successfully");
                    }
                }

                // Set sending to false
                chat_room_state.write().sending = false;

                // Save message to persistence and send event
                if let Some(chat_id) = chat_id {
                    println!("Saving message to persistence for chat: {}", chat_id);
                    let new_msg_clone = new_msg.clone();
                    let chat_name = chat_room_state.read().chat_name.clone();
                    let is_group_chat = chat_room_state.read().is_group_chat;
                    let message_id = new_msg.id.clone(); // Preserve the message ID
                    spawn(async move {
                        // Get local nickname from P2P service
                        let local_nickname =
                            crate::services::p2p_service::P2pService::get_local_nickname()
                                .await
                                .unwrap_or("You".to_string());

                        if is_group_chat {
                            // Save group message with the original message ID
                            if let Err(err) = crate::services::persistence_service::PersistenceService::store_group_message_with_id(
                                message_id,
                                local_nickname.clone(),
                                chat_name.clone().unwrap_or_default(),
                                new_msg_clone.content.clone(),
                                true,
                            )
                            .await
                            {
                                println!("Error saving group message: {:?}", err);
                            }

                            // Update conversation for group chat
                            if let Some(ref group_name) = chat_name {
                                if let Err(err) = crate::services::persistence_service::PersistenceService::upsert_conversation(
                                    format!("group-{}", group_name),
                                    group_name.clone(),
                                    true, // is group
                                    group_name.clone(),
                                    Some(new_msg_clone.content),
                                    Some(chrono::Utc::now()),
                                )
                                .await
                                {
                                    println!("Error upserting group conversation: {:?}", err);
                                }
                            }
                        } else {
                            // Save direct message
                            if let Err(err) = crate::services::persistence_service::PersistenceService::store_direct_message(
                                local_nickname.clone(),
                                chat_name.clone().unwrap_or_default(),
                                new_msg_clone.content.clone(),
                                true,
                            )
                            .await
                            {
                                println!("Error saving message: {:?}", err);
                            }

                            // Create or update conversation for direct messages
                            if let Some(ref peer_nickname) = chat_name {
                                if let Err(err) = crate::services::persistence_service::PersistenceService::upsert_conversation(
                                    chat_id.clone(),
                                    peer_nickname.clone(),
                                    false, // not a group
                                    chat_id.clone(),
                                    Some(new_msg_clone.content),
                                    Some(chrono::Utc::now()),
                                )
                                .await
                                {
                                    println!("Error upserting conversation: {:?}", err);
                                }
                            }
                        }

                        if let Err(err) = crate::services::event_bus::EventBus::send(
                            crate::services::event_bus::AppEvent::MessageSaved(chat_id),
                        ) {
                            println!("Error sending MessageSaved event: {:?}", err);
                        }
                    });
                } else {
                    println!("No chat ID available for saving message");
                }
            });
        } else {
            println!("No chat name available for sending message");
        }
    };

    let handle_image_select = move || {
        println!("handle_image_select called");
        // Implementation for image selection
    };

    let handle_file_select = move || {
        println!("handle_file_select called");
        // Implementation for file selection
    };

    let handle_file_download_request =
        move |share_code: String, filename: String, file_type: String| {
            println!(
                "handle_file_download_request called with share code: {}",
                share_code
            );
            // Implementation for file download request
        };

    let handle_share_file = move |peer_id: String, filename: String, file_path: Option<String>| {
        println!("handle_share_file called for peer: {}", peer_id);
        // Implementation for file sharing
    };

    let handle_delete_message = move |message_id: String| {
        println!("handle_delete_message called for message: {}", message_id);
        // Implementation for message deletion
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
