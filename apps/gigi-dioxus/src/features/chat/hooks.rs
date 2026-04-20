use dioxus::prelude::*;
use std::sync::Arc;

use crate::features::chat::chat_state::{use_chat_state, use_chat_room_state, ChatState, ChatRoomState};

// Hook for chat initialization
pub fn use_chat_initialization() -> Signal<ChatState> {
    let chat_state = use_chat_state();

    // Initialize chat data on mount
    use_effect(|| {
        // In a real app, this would load data from the backend
        // For now, we'll use the default data from the ChatState
        println!("Chat initialized");
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
        spawn(async move {
            let message_service = crate::services::message_service::MessageService::new();
            let messages = message_service.load_message_history(&chat_id_clone, 50, 0).await;
            
            chat_room_state.write().messages = messages;
            chat_room_state.write().is_loading = false;
        });
    });

    chat_room_state
}

// Hook for chat event listeners
pub fn use_chat_event_listeners() {
    // In a real app, this would set up event listeners for incoming messages, peer status changes, etc.
    use_effect(|| {
        println!("Chat event listeners initialized");

        // Cleanup function
        println!("Chat event listeners cleaned up");
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
pub fn use_peer_actions() -> (Arc<dyn Fn(String) + Send + Sync>,) {
    let chat_state = use_chat_state();

    let handle_peer_click = Arc::new(move |peer_id: String| {
        // In a real app, this would navigate to the chat room for the peer
        println!("Peer clicked: {}", peer_id);
    });

    (handle_peer_click,)
}

// Hook for group actions
pub fn use_group_actions() -> (
    Arc<dyn Fn(String) + Send + Sync>,
    Arc<dyn Fn(String) + Send + Sync>,
    Arc<dyn Fn(String) + Send + Sync>,
    Arc<dyn Fn(String) + Send + Sync>
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

    (handle_share_group, handle_accept_group_share, handle_ignore_group_share, handle_clear_messages)
}

// Hook for message actions
pub fn use_message_actions() -> (
    impl FnMut(),
    impl FnMut(),
    impl FnMut(),
    impl Fn(&str)
) {
    let mut chat_room_state = use_chat_room_state();

    let handle_send_message = move || {
        if !chat_room_state.read().new_message.is_empty() {
            let new_msg = crate::features::chat::chat_state::Message {
                id: (chat_room_state.read().messages.len() + 1).to_string(),
                content: chat_room_state.read().new_message.clone(),
                sender: "You".to_string(),
                timestamp: "12:03 PM".to_string(), // In a real app, this would be the current time
                is_own: true,
                message_type: crate::features::chat::chat_state::MessageType::Text,
            };
            chat_room_state.write().messages.push(new_msg.clone());
            chat_room_state.write().new_message = "".to_string();
            
            // Save message to persistence
            if let Some(chat_id) = chat_room_state.read().chat_id.clone() {
                let new_msg_clone = new_msg.clone();
                spawn(async move {
                    let message_service = crate::services::message_service::MessageService::new();
                    let _ = message_service.save_message(&new_msg_clone, &chat_id).await;
                });
            }
        }
    };

    let handle_image_select = move || {
        // In a real app, this would open a file picker for images
        println!("Select image");
        // For demo purposes, we'll add a sample image message
        let new_msg = crate::features::chat::chat_state::Message {
            id: (chat_room_state.read().messages.len() + 1).to_string(),
            content: "image.jpg".to_string(),
            sender: "You".to_string(),
            timestamp: "12:03 PM".to_string(), // In a real app, this would be the current time
            is_own: true,
            message_type: crate::features::chat::chat_state::MessageType::Image,
        };
        chat_room_state.write().messages.push(new_msg.clone());
        
        // Save message to persistence
        if let Some(chat_id) = chat_room_state.read().chat_id.clone() {
            let new_msg_clone = new_msg.clone();
            spawn(async move {
                let message_service = crate::services::message_service::MessageService::new();
                let _ = message_service.save_message(&new_msg_clone, &chat_id).await;
            });
        }
    };

    let handle_file_select = move || {
        // In a real app, this would open a file picker for files
        println!("Select file");
        // For demo purposes, we'll add a sample file message
        let new_msg = crate::features::chat::chat_state::Message {
            id: (chat_room_state.read().messages.len() + 1).to_string(),
            content: "document.pdf".to_string(),
            sender: "You".to_string(),
            timestamp: "12:03 PM".to_string(), // In a real app, this would be the current time
            is_own: true,
            message_type: crate::features::chat::chat_state::MessageType::File,
        };
        chat_room_state.write().messages.push(new_msg.clone());
        
        // Save message to persistence
        if let Some(chat_id) = chat_room_state.read().chat_id.clone() {
            let new_msg_clone = new_msg.clone();
            spawn(async move {
                let message_service = crate::services::message_service::MessageService::new();
                let _ = message_service.save_message(&new_msg_clone, &chat_id).await;
            });
        }
    };

    let handle_file_download_request = move |file_id: &str| {
        // In a real app, this would initiate a file download
        println!("Download file: {}", file_id);
    };

    (handle_send_message, handle_image_select, handle_file_select, handle_file_download_request)
}
