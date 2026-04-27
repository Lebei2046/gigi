pub mod chat_room;
pub mod chat_state;
pub mod components;
pub mod hooks;

use dioxus::prelude::*;
use dioxus_router::use_navigator;
use std::str::FromStr;

use crate::features::chat::chat_state::use_chat_room_state;
use crate::features::chat::components::{
    ConfirmationDialog, GroupChatCard, GroupShareModal, PeerChatCard,
};
use crate::features::chat::hooks::{
    use_chat_data_refresh, use_chat_event_listeners, use_chat_initialization, use_group_actions,
    use_peer_actions,
};

// Main Chat Component
#[component]
pub fn Chat() -> Element {
    let navigator = use_navigator();
    let chat_state = use_chat_initialization();

    // Set up event listeners
    let chat_room_state = use_chat_room_state();
    use_chat_event_listeners(chat_state.clone(), chat_room_state);

    // Set up data refresh
    use_chat_data_refresh();

    // Get action handlers
    let (
        handle_share_group,
        handle_accept_group_share,
        handle_ignore_group_share,
        handle_clear_messages,
    ) = use_group_actions();

    // State for confirmation dialog
    let mut show_confirm_dialog = use_signal(|| false);
    let mut selected_peer_nickname = use_signal(|| String::new());

    // State for group share modal
    let mut show_group_share_modal = use_signal(|| false);
    let mut selected_group_id = use_signal(|| String::new());
    let mut selected_group_name = use_signal(|| String::new());
    let mut selected_peers = use_signal(|| vec![]);

    let handle_group_click = move |group_id: String| {
        navigator.push(format!("/chat/{}", group_id));
    };

    // Handle share group
    let handle_share_group = move |group_id: String| {
        // Find the group by ID
        if let Some(group) = chat_state.read().groups.iter().find(|g| g.id == group_id) {
            selected_group_id.set(group.id.clone());
            selected_group_name.set(group.name.clone());
            selected_peers.set(vec![]); // Reset selected peers
            show_group_share_modal.set(true);
        }
    };

    let handle_peer_click = move |peer_id: String| {
        navigator.push(format!("/chat/{}", peer_id));
    };

    // Handle clear messages with confirmation
    let handle_clear_messages_with_confirm = move |peer_nickname: String| {
        selected_peer_nickname.set(peer_nickname);
        show_confirm_dialog.set(true);
    };

    // Handle confirm action
    let handle_confirm = move || {
        let peer_nickname = selected_peer_nickname.read().clone();
        handle_clear_messages(peer_nickname);
        show_confirm_dialog.set(false);
    };

    // Handle cancel action
    let handle_cancel = move || {
        show_confirm_dialog.set(false);
    };

    // Handle group share submit
    let handle_group_share_submit = move || {
        let group_id = selected_group_id();
        let group_name = selected_group_name();
        let peers = selected_peers();

        // Send group share messages
        if !peers.is_empty() {
            // Convert peer IDs to nicknames
            let peer_nicknames: Vec<String> = peers
                .iter()
                .filter_map(|peer_id| {
                    chat_state
                        .read()
                        .peers
                        .iter()
                        .find(|p| p.id == *peer_id)
                        .map(|p| p.nickname.clone())
                })
                .collect();

            if !peer_nicknames.is_empty() {
                tokio::spawn(async move {
                    // Convert to &str for the function call
                    let peer_nickname_refs: Vec<&str> =
                        peer_nicknames.iter().map(|s| s.as_str()).collect();

                    if let Err(err) =
                        crate::services::p2p_service::P2pService::send_group_share_message(
                            &peer_nickname_refs,
                            &group_id,
                            &group_name,
                        )
                        .await
                    {
                        println!("Failed to send group share messages: {:?}", err);
                    }
                });
            }
        }

        show_group_share_modal.set(false);
    };

    // Combine conversations, groups, and peers into a single list
    let mut combined_list: Vec<Element> = vec![];

    // Add conversations first (from the conversation table)
    let conversations = chat_state.read().conversations.clone();
    for conversation in &conversations {
        if let Some(group_id) = &conversation.group_id {
            // Group conversation
            // Try to find the group in chat_state.groups
            let group = chat_state
                .read()
                .groups
                .iter()
                .find(|g| g.id == *group_id)
                .cloned();

            // If group not found, create a virtual group from conversation data
            let group = group.unwrap_or_else(|| chat_state::Group {
                id: group_id.clone(),
                name: conversation.name.clone(),
                role: "Member".to_string(),
                member_count: 0,
                joined: true,
            });

            let mut share_group = handle_share_group.clone();
            let mut clear_messages = handle_clear_messages_with_confirm.clone();
            let group_id = group.id.clone();
            let group_click = handle_group_click.clone();
            let group_name = group.name.clone();
            combined_list.push(rsx! {
                GroupChatCard {
                    key: "group-{group.id}",
                    group: group.clone(),
                    conversation: Some(conversation.clone()),
                    on_click: move |_| group_click(group_id.clone()),
                    on_share: move |id| share_group(id),
                    on_clear: move |id| clear_messages(group_name.clone()),
                }
            });
        } else if let Some(peer_id) = &conversation.peer_id {
            // Direct conversation - try to find peer in chat_state.peers
            let peer = chat_state
                .read()
                .peers
                .iter()
                .find(|p| p.id == *peer_id)
                .cloned();

            // If peer not found in peers list (offline), create a virtual peer from conversation data
            let peer = peer.unwrap_or_else(|| {
                // Extract nickname from conversation name
                let nickname = conversation.name.clone();
                // Parse PeerId from the peer_id string
                let peer_id_obj = peer_id.parse::<gigi_p2p::PeerId>().unwrap_or_else(|_| {
                    // If parsing fails, create a dummy PeerId (shouldn't happen with valid data)
                    gigi_p2p::PeerId::from_bytes(&[0u8; 32]).expect("Failed to create dummy PeerId")
                });

                chat_state::Peer {
                    id: peer_id.clone(),
                    peer_id: peer_id_obj,
                    nickname,
                    is_online: false, // Mark as offline since not in peers list
                    capabilities: vec![],
                }
            });

            let peer_id_str = peer.id.clone();
            let mut clear_messages = handle_clear_messages_with_confirm.clone();
            let peer_nickname = peer.nickname.clone();
            let peer_click = handle_peer_click.clone();
            combined_list.push(rsx! {
                PeerChatCard {
                    key: "peer-{peer.id}",
                    peer: peer.clone(),
                    conversation: Some(conversation.clone()),
                    on_click: move |_| peer_click(peer_id_str.clone()),
                    on_clear: move |_| clear_messages(peer_nickname.clone()),
                }
            });
        }
    }

    // Add groups not in conversations
    let existing_group_ids: std::collections::HashSet<String> = conversations
        .iter()
        .filter_map(|c| c.group_id.clone())
        .collect();

    for group in &chat_state.read().groups {
        if !existing_group_ids.contains(&group.id) {
            let mut share_group = handle_share_group.clone();
            let mut clear_messages = handle_clear_messages_with_confirm.clone();
            let group_id = group.id.clone();
            let group_click = handle_group_click.clone();
            let group_name = group.name.clone();
            combined_list.push(rsx! {
                GroupChatCard {
                    key: "group-{group.id}",
                    group: group.clone(),
                    conversation: None,
                    on_click: move |_| group_click(group_id.clone()),
                    on_share: move |id| share_group(id),
                    on_clear: move |id| clear_messages(group_name.clone()),
                }
            });
        }
    }

    // Add peers not in conversations
    let existing_peer_ids: std::collections::HashSet<String> = conversations
        .iter()
        .filter_map(|c| c.peer_id.clone())
        .collect();

    for peer in &chat_state.read().peers {
        if !existing_peer_ids.contains(&peer.id) {
            let peer_id = peer.id.clone();
            let mut clear_messages = handle_clear_messages_with_confirm.clone();
            let peer_nickname = peer.nickname.clone();
            let peer_click = handle_peer_click.clone();
            combined_list.push(rsx! {
                PeerChatCard {
                    key: "peer-{peer.id}",
                    peer: peer.clone(),
                    conversation: None,
                    on_click: move |_| peer_click(peer_id.clone()),
                    on_clear: move |_| clear_messages(peer_nickname.clone()),
                }
            });
        }
    }

    rsx! {
        div { class: "flex flex-col h-full bg-gray-50",
            // Header
            div { class: "bg-white border-b border-gray-200 px-4 py-4",
                h2 { class: "text-2xl font-bold text-gray-900", "Messages" }
            }

            div { class: "flex-1 overflow-y-auto px-4 py-4",
                // Combined Conversation List
                div { class: "space-y-3",
                    for card in combined_list {
                        {card}
                    }
                }
            }

            // Confirmation Dialog
            ConfirmationDialog {
                is_open: show_confirm_dialog(),
                title: "Clear Messages".to_string(),
                message: "Are you sure you want to clear all messages for this conversation? This action cannot be undone."
                    .to_string(),
                on_confirm: handle_confirm,
                on_cancel: handle_cancel,
            }

            // Group Share Modal
            GroupShareModal {
                is_open: show_group_share_modal,
                group_id: selected_group_id,
                group_name: selected_group_name,
                peers: chat_state.read().peers.clone(),
                selected_peers,
                on_close: move |_| show_group_share_modal.set(false),
                on_share: handle_group_share_submit,
            }
        }
    }
}
