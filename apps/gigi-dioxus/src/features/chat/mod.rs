pub mod chat_room;
pub mod chat_state;
pub mod components;
pub mod hooks;

use dioxus::prelude::*;
use dioxus_router::use_navigator;

use crate::features::chat::components::{GroupChatCard, PeerChatCard};
use crate::features::chat::hooks::{
    use_chat_data_refresh, use_chat_event_listeners, use_chat_initialization, use_group_actions,
    use_peer_actions,
};

// Main Chat Component
#[component]
pub fn Chat() -> Element {
    let navigator = use_navigator();
    let chat_state = use_chat_initialization();

    // Set up event listeners and data refresh
    use_chat_event_listeners(chat_state);
    use_chat_data_refresh();

    // Get action handlers
    let (
        handle_share_group,
        handle_accept_group_share,
        handle_ignore_group_share,
        handle_clear_messages,
    ) = use_group_actions();

    let handle_group_click = move |group_id: String| {
        navigator.push(format!("/chat/{}", group_id));
    };

    let handle_peer_click = move |peer_id: String| {
        navigator.push(format!("/chat/{}", peer_id));
    };

    // Create group chat cards
    let group_cards = chat_state
        .read()
        .groups
        .clone()
        .into_iter()
        .map(|group| {
            let share_group = handle_share_group.clone();
            let clear_messages = handle_clear_messages.clone();
            let group_id = group.id.clone();
            let group_click = handle_group_click.clone();
            rsx! {
                GroupChatCard {
                    key: "{group.id}",
                    group: group.clone(),
                    conversation: chat_state
                        .read()
                        .conversations
                        .clone()
                        .into_iter()
                        .find(|c| c.group_id == Some(group.id.clone())),
                    on_click: move |_| group_click(group_id.clone()),
                    on_share: move |id| share_group(id),
                    on_clear: move |id| clear_messages(id),
                }
            }
        })
        .collect::<Vec<_>>();

    // Create peer chat cards
    let peer_cards = chat_state
        .read()
        .peers
        .clone()
        .into_iter()
        .map(|peer| {
            let peer_id = peer.id.clone();
            let clear_messages = handle_clear_messages.clone();
            let peer_click = handle_peer_click.clone();
            rsx! {
                PeerChatCard {
                    key: "{peer.id}",
                    peer: peer.clone(),
                    conversation: chat_state
                        .read()
                        .conversations
                        .clone()
                        .into_iter()
                        .find(|c| c.peer_id == Some(peer.id.clone())),
                    on_click: move |_| peer_click(peer_id.clone()),
                    on_clear: move |id| clear_messages(id),
                }
            }
        })
        .collect::<Vec<_>>();

    rsx! {
        div { class: "flex flex-col h-full bg-gray-50",
            // Header
            div { class: "bg-white border-b border-gray-200 px-4 py-4",
                h2 { class: "text-2xl font-bold text-gray-900", "Messages" }
            }

            div { class: "flex-1 overflow-y-auto px-4 py-4",
                // Groups Section
                div { class: "mb-6",
                    div { class: "flex items-center gap-2 mb-3",
                        span { class: "text-lg", "👥" }
                        h3 { class: "text-lg font-semibold text-gray-900", "Groups" }
                        span { class: "bg-blue-100 text-blue-600 text-xs font-medium px-2 py-1 rounded-full",
                            "{chat_state.read().groups.len()}"
                        }
                        span { class: "bg-red-500 text-white text-xs font-bold px-2 py-1 rounded-full",
                            "{chat_state.read().conversations.clone().into_iter().filter(|c| c.group_id.is_some() && c.unread_count > 0).count()}"
                        }
                    }
                    div { class: "space-y-3",
                        for card in group_cards {
                            {card}
                        }
                    }
                }

                // Direct Chats Section
                div {
                    div { class: "flex items-center gap-2 mb-3",
                        span { class: "text-lg", "💬" }
                        h3 { class: "text-lg font-semibold text-gray-900", "Direct Chats" }
                        span { class: "bg-green-100 text-green-600 text-xs font-medium px-2 py-1 rounded-full",
                            "{chat_state.read().peers.len()}"
                        }
                    }
                    div { class: "space-y-3",
                        for card in peer_cards {
                            {card}
                        }
                    }
                }
            }
        }
    }
}
