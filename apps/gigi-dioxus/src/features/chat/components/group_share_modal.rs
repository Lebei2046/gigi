use dioxus::prelude::*;

use crate::features::chat::chat_state::Peer;

#[component]
pub fn GroupShareModal(
    is_open: Signal<bool>,
    group_id: Signal<String>,
    group_name: Signal<String>,
    peers: Vec<Peer>,
    selected_peers: Signal<Vec<String>>,
    on_close: EventHandler<()>,
    on_share: EventHandler<()>,
) -> Element {
    if !is_open() {
        return Ok(rsx! {
            {}
        }?);
    }

    let mut toggle_peer = move |peer_id: String| {
        selected_peers.with_mut(|selected| {
            if selected.contains(&peer_id) {
                selected.retain(|id| id != &peer_id);
            } else {
                selected.push(peer_id);
            }
        });
    };

    // Create peer elements outside the rsx! macro
    let peer_elements = peers.iter().filter(|p| p.is_online).map(|peer| {
        let peer_id = peer.id.clone();
        let is_selected = selected_peers().contains(&peer.id);
        rsx! {
            div {
                key: "{peer.id}",
                class: "flex items-center p-3 hover:bg-gray-100 rounded-lg cursor-pointer",
                onclick: move |_| toggle_peer(peer_id.clone()),
                div { class: "w-10 h-10 bg-gradient-to-br from-green-400 to-green-600 rounded-full flex items-center justify-center flex-shrink-0",
                    span { class: "text-white font-bold",
                        "{peer.nickname.chars().next().unwrap_or('?').to_uppercase()}"
                    }
                }
                div { class: "ml-3 flex-1",
                    div { class: "font-medium text-gray-900", "{peer.nickname}" }
                    div { class: "text-xs text-gray-500", "{peer.id}" }
                }
                div { class: "flex-shrink-0",
                    if is_selected {
                        svg {
                            class: "w-5 h-5 text-blue-600",
                            fill: "currentColor",
                            view_box: "0 0 20 20",
                            path {
                                fill_rule: "evenodd",
                                d: "M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z",
                                clip_rule: "evenodd",
                            }
                        }
                    } else {
                        svg {
                            class: "w-5 h-5 text-gray-400",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z",
                            }
                        }
                    }
                }
            }
        }
    }).collect::<Vec<_>>();

    rsx! {
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            onclick: move |_| on_close.call(()),
            div {
                class: "bg-white rounded-xl shadow-lg w-full max-w-md p-6",
                onclick: move |e| e.stop_propagation(),
                div { class: "flex justify-between items-center mb-4",
                    h3 { class: "text-lg font-semibold text-gray-900", "Share Group: {group_name}" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close.call(()),
                        svg {
                            class: "w-5 h-5",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M6 18L18 6M6 6l12 12",
                            }
                        }
                    }
                }
                div { class: "mb-4",
                    p { class: "text-sm text-gray-600", "Select peers to share this group with" }
                }
                div { class: "max-h-64 overflow-y-auto mb-6", {peer_elements.into_iter()} }
                div { class: "flex space-x-3",
                    button {
                        class: "flex-1 py-2 px-4 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 hover:bg-gray-50",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "flex-1 py-2 px-4 bg-blue-600 border border-transparent rounded-lg text-sm font-medium text-white hover:bg-blue-700",
                        onclick: move |_| on_share.call(()),
                        "Share"
                    }
                }
            }
        }
    }
}
