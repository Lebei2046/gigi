use dioxus::prelude::*;
use futures_util::StreamExt;
use gigi_p2p::PeerId;
use std::path::PathBuf;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;

use crate::features::chat::chat_state::{
    use_chat_room_state, use_chat_state, ChatRoomState, ChatState, Message, Peer,
};
use crate::services::auth_service::AuthService;
use crate::services::event_bus::{AppEvent, EventBus};
use crate::services::p2p_service::P2pService;
use crate::services::persistence_service::PersistenceService;

// Hook for chat initialization
pub fn use_chat_initialization() -> Signal<ChatState> {
    let chat_state = use_chat_state();

    use_effect(move || {
        println!("Chat initialized - loading groups and peers");

        let mut chat_state_clone = chat_state.clone();
        spawn(async move {
            // Load groups from auth service
            match AuthService::new().await {
                Ok(auth_service) => match auth_service.get_all_groups().await {
                    Ok(groups) => {
                        let converted: Vec<crate::features::chat::chat_state::Group> =
                            groups.iter().map(|g| g.into()).collect();
                        chat_state_clone.write().groups = converted;
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

            // Load existing peers from P2P service
            match crate::features::chat::chat_state::list_peers().await {
                peers if !peers.is_empty() => {
                    let peer_count = peers.len();
                    chat_state_clone.write().peers = peers;
                    println!("Loaded {} existing peers from P2P service", peer_count);
                }
                _ => {
                    println!("No existing peers found or failed to load peers");
                }
            }
        });
    });

    chat_state
}

// Hook for chat room initialization
pub fn use_chat_room_initialization(chat_id: String) -> Signal<ChatRoomState> {
    let mut chat_room_state = use_chat_room_state();
    let chat_state = use_chat_state();

    // Initialize chat room data based on the chat_id
    use_effect(move || {
        chat_room_state.write().is_loading = true;
        chat_room_state.write().chat_id = Some(chat_id.clone());
        chat_room_state.write().is_group_chat = chat_id.starts_with("group");

        if chat_room_state.read().is_group_chat {
            // Find group by id
            if let Some(group) = chat_state.read().groups.iter().find(|g| g.id == chat_id) {
                chat_room_state.write().chat_name = Some(group.name.clone());
                chat_room_state.write().group = Some(group.clone());
            }
        } else {
            // Find peer by id
            if let Some(peer) = chat_state.read().peers.iter().find(|p| p.id == chat_id) {
                chat_room_state.write().chat_name = Some(peer.nickname.clone());
                chat_room_state.write().peer = Some(peer.clone());
            }
        }

        // Load message history
        let chat_id_clone = chat_id.clone();
        let mut chat_room_state_clone = chat_room_state.clone();
        let chat_name = chat_room_state.read().chat_name.clone();
        spawn(async move {
            if let Some(peer_nickname) = chat_name {
                if let Ok(stored_messages) =
                    PersistenceService::load_messages(&peer_nickname, 50, 0).await
                {
                    let messages: Vec<Message> = stored_messages.iter().map(|m| m.into()).collect();
                    chat_room_state_clone.write().messages = messages;
                }
            }
            chat_room_state_clone.write().is_loading = false;
        });
    });

    chat_room_state
}

// Hook for chat event listeners
pub fn use_chat_event_listeners(chat_state: Signal<ChatState>) {
    let mut chat_room_state = use_chat_room_state();

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
                                    if !state.peers.iter().any(|p| p.id == peer_id.to_string()) {
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
                                    } else {
                                        println!("Peer already exists: {} ({})", nickname, peer_id);
                                    }
                                }
                                gigi_p2p::P2pEvent::Connected { peer_id, nickname } => {
                                    println!("Peer connected: {}", peer_id);
                                    let mut state = chat_state_clone.write();
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
                                }
                                gigi_p2p::P2pEvent::Disconnected { peer_id, .. } => {
                                    println!("Peer disconnected: {}", peer_id);
                                    let mut state = chat_state_clone.write();
                                    if let Some(peer) =
                                        state.peers.iter_mut().find(|p| p.peer_id == peer_id)
                                    {
                                        peer.is_online = false;
                                        println!(
                                            "Updated peer status: {} is now offline",
                                            peer.nickname
                                        );
                                    }
                                }
                                gigi_p2p::P2pEvent::DirectMessage {
                                    from_nickname,
                                    message,
                                    ..
                                } => {
                                    println!("Direct message from {}: {}", from_nickname, message);
                                    let state = chat_room_state_clone.read();
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
                                }
                                _ => {
                                    println!("Other P2P event: {:?}", p2p_event);
                                }
                            }
                        }
                        AppEvent::MessageSaved(chat_id) => {
                            println!("Message saved for chat: {}", chat_id);
                            if chat_room_state_clone.read().chat_id == Some(chat_id.clone()) {
                                let chat_id_clone = chat_id;
                                let chat_name = chat_room_state_clone.read().chat_name.clone();
                                let mut chat_room_state_refresh = chat_room_state_clone.clone();
                                spawn(async move {
                                    if let Some(peer_nickname) = chat_name {
                                        if let Ok(stored_messages) =
                                            PersistenceService::load_messages(&peer_nickname, 50, 0)
                                                .await
                                        {
                                            let messages: Vec<Message> =
                                                stored_messages.iter().map(|m| m.into()).collect();
                                            chat_room_state_refresh.write().messages = messages;
                                            println!(
                                                "Refreshed messages for chat: {}",
                                                chat_id_clone
                                            );
                                        }
                                    }
                                });
                            }
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

    let handle_clear_messages = Arc::new(move |chat_id: String| {
        // In a real app, this would clear messages for the chat
        println!("Clear messages: {}", chat_id);
    });

    (
        handle_share_group,
        handle_accept_group_share,
        handle_ignore_group_share,
        handle_clear_messages,
    )
}

// Hook for message actions
pub fn use_message_actions() -> (
    impl FnMut(),
    impl FnMut(),
    impl FnMut(),
    impl Fn(String),
    impl Fn(&str, PathBuf),
) {
    let mut chat_room_state = use_chat_room_state();

    let handle_send_message = move || {
        if !chat_room_state.read().new_message.is_empty() {
            let new_msg = crate::features::chat::chat_state::Message {
                id: uuid::Uuid::new_v4().to_string(),
                content: chat_room_state.read().new_message.clone(),
                sender: "You".to_string(),
                timestamp: chrono::Local::now().format("%H:%M %p").to_string(),
                is_own: true,
                message_type: crate::features::chat::chat_state::MessageType::Text,
            };
            chat_room_state.write().messages.push(new_msg.clone());
            chat_room_state.write().new_message = "".to_string();

            // Send message via P2P
            if let Some(chat_name) = chat_room_state.read().chat_name.clone() {
                let message_content = new_msg.content.clone();
                let is_group_chat = chat_room_state.read().is_group_chat;
                spawn(async move {
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
            }

            // Save message to persistence and send event
            if let Some(chat_id) = chat_room_state.read().chat_id.clone() {
                let new_msg_clone = new_msg.clone();
                let chat_name = chat_room_state.read().chat_name.clone();
                spawn(async move {
                    let _ = PersistenceService::store_direct_message(
                        "You".to_string(),
                        chat_name.unwrap_or_default(),
                        new_msg_clone.content,
                        true,
                    )
                    .await;
                    let _ = EventBus::send(AppEvent::MessageSaved(chat_id));
                });
            }
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

    (
        handle_send_message,
        handle_image_select,
        handle_file_select,
        handle_file_download_request,
        handle_share_file,
    )
}
