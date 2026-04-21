use dioxus::prelude::*;

use crate::features::chat::chat_state::{Conversation, Group, Peer};

// Chat Card for Direct Chats
#[component]
pub fn PeerChatCard(
    peer: Peer,
    conversation: Option<Conversation>,
    on_click: EventHandler<String>,
    on_clear: EventHandler<String>,
) -> Element {
    let unread_count = conversation.as_ref().map(|c| c.unread_count).unwrap_or(0);
    let last_message = conversation.as_ref().and_then(|c| c.last_message.clone());
    let last_message_time = conversation
        .as_ref()
        .and_then(|c| c.last_message_time.clone());
    let first_letter = peer
        .nickname
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();
    let peer_id_click = peer.id.clone();
    let peer_id_clear = peer.id.clone();

    rsx! {
        div {
            class: "bg-white border border-gray-200 rounded-xl shadow-sm hover:shadow-md transition-all cursor-pointer hover:border-green-300",
            onclick: move |_| on_click.call(peer_id_click.clone()),
            div { class: "p-4",
                div { class: "flex justify-between items-start",
                    div { class: "flex items-start gap-3 flex-1 cursor-pointer",
                        div { class: "w-12 h-12 bg-gradient-to-br from-green-400 to-green-600 rounded-full flex items-center justify-center flex-shrink-0",
                            span { class: "text-white font-bold text-lg", "{first_letter}" }
                        }
                        div { class: "flex-1 min-w-0",
                            div { class: "flex items-center gap-2 mb-1",
                                span { class: "font-semibold text-gray-900", "{peer.nickname}" }
                                if unread_count > 0 {
                                    span { class: "bg-red-500 text-white text-xs font-bold px-2 py-0.5 rounded-full min-w-[20px] text-center",
                                        "{unread_count}"
                                    }
                                }
                            }
                            div { class: "text-xs text-gray-500 font-mono truncate",
                                "{peer.id}"
                            }
                            if let Some(message) = last_message {
                                div { class: "text-sm text-gray-600 truncate mt-1",
                                    "{message}"
                                }
                            }
                            div { class: "flex flex-wrap gap-1 mt-1",
                                for capability in peer.capabilities.clone() {
                                    span { class: "bg-gray-100 text-gray-600 text-xs px-2 py-0.5 rounded",
                                        "{capability}"
                                    }
                                }
                            }
                        }
                    }
                    div { class: "text-right ml-3 flex-shrink-0 flex flex-col items-end",
                        if let Some(time) = last_message_time {
                            div { class: "text-xs text-gray-400 mb-1", "{time}" }
                        }
                        button {
                            class: "p-2 text-red-600 hover:bg-red-50 rounded-lg transition-colors",
                            title: "Clear messages",
                            onclick: move |e| {
                                e.stop_propagation();
                                on_clear.call(peer_id_clear.clone());
                            },
                            svg {
                                class: "w-4 h-4",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16",
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// Chat Card for Groups
#[component]
pub fn GroupChatCard(
    group: Group,
    conversation: Option<Conversation>,
    on_click: EventHandler<String>,
    on_share: EventHandler<String>,
    on_clear: EventHandler<String>,
) -> Element {
    let unread_count = conversation.as_ref().map(|c| c.unread_count).unwrap_or(0);
    let last_message = conversation.as_ref().and_then(|c| c.last_message.clone());
    let group_click = group.clone();
    let group_share = group.clone();
    let group_clear = group.clone();

    rsx! {
        div {
            class: "bg-white border border-gray-200 rounded-xl shadow-sm hover:shadow-md transition-shadow",
            onclick: move |_| on_click.call(group_click.id.clone()),
            div { class: "flex justify-between items-start p-4 cursor-pointer",
                div { class: "flex-1",
                    div { class: "flex items-center gap-2 mb-1",
                        div { class: "w-10 h-10 bg-blue-100 rounded-full flex items-center justify-center",
                            span { class: "text-blue-600 font-semibold", "G" }
                        }
                        div { class: "flex-1",
                            div { class: "flex items-center gap-2",
                                span { class: "font-semibold text-gray-900", "{group.name}" }
                                if unread_count > 0 {
                                    span { class: "bg-red-500 text-white text-xs font-bold px-2 py-0.5 rounded-full min-w-[20px] text-center",
                                        "{unread_count}"
                                    }
                                }
                            }
                            div { class: "flex items-center gap-2 text-xs text-gray-500",
                                span { class: "bg-gray-100 px-2 py-0.5 rounded", "{group.role}" }
                            }
                        }
                    }
                    if let Some(message) = last_message {
                        div { class: "text-sm text-gray-600 mt-2 truncate ml-12", "{message}" }
                    }
                }
                button {
                    class: "p-2 text-blue-600 hover:bg-blue-50 rounded-lg transition-colors",
                    onclick: move |e| {
                        e.stop_propagation();
                        on_share.call(group_share.id.clone());
                    },
                    svg {
                        class: "w-5 h-5",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m9.032 4.026a3 3 0 10-4.732 2.684m4.732-2.684a3 3 0 00-4.732-2.684",
                        }
                    }
                }
                button {
                    class: "p-2 text-red-600 hover:bg-red-50 rounded-lg transition-colors",
                    title: "Clear messages",
                    onclick: move |e| {
                        e.stop_propagation();
                        on_clear.call(group_clear.id.clone());
                    },
                    svg {
                        class: "w-5 h-5",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16",
                        }
                    }
                }
            }
        }
    }
}
