use dioxus::prelude::*;
use dioxus_router::use_navigator;
use std::cell::RefCell;
use std::rc::Rc;

use crate::services::auth_service::AuthService;
use crate::services::event_bus::{AppEvent, EventBus};
use crate::services::persistence_service::PersistenceService;

use crate::features::chat::components::{ChatRoomHeader, GroupShareNotificationModal};
use crate::features::chat::hooks::{
    use_chat_event_listeners, use_chat_initialization, use_chat_room_initialization,
    use_message_actions,
};
use crate::services::p2p_service::P2pService;

fn scroll_to_bottom(element_id: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(container) = document.get_element_by_id(element_id) {
                    let scroll_height = container.scroll_height();
                    container
                        .set_attribute("scrollTop", &scroll_height.to_string())
                        .ok();
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let js = format!(
            r#"
            setTimeout(function() {{
                var el = document.getElementById('{}');
                if (el) {{
                    el.scrollTop = el.scrollHeight;
                }}
            }}, 10);
            "#,
            element_id
        );
        spawn(async move {
            let _ = dioxus::document::eval(&js).await;
        });
    }
}

// Chat Room Component
#[component]
pub fn ChatRoom(id: String) -> Element {
    let navigator = use_navigator();
    let chat_state = use_chat_initialization();
    let mut chat_room_state = use_chat_room_initialization(id, chat_state);

    // State for group share notification modal
    let mut show_group_share_modal = use_signal(|| false);
    let mut group_id = use_signal(|| String::new());
    let mut group_name = use_signal(|| String::new());
    let mut sender_name = use_signal(|| String::new());

    // Set up event listeners for messages
    use_chat_event_listeners(chat_state, chat_room_state);
    let (
        mut handle_send_message,
        mut handle_image_select,
        mut handle_file_select,
        handle_file_download_request,
        _handle_share_file,
        mut handle_delete_message,
    ) = use_message_actions(chat_room_state.clone());

    // Convert to EventHandler for Dioxus component
    let on_download_request = move |args: (String, String, String)| {
        handle_file_download_request(args.0, args.1, args.2);
    };

    // Handle group share messages
    let handle_group_share = move |args: (String, String, String)| {
        let (g_id, g_name, s_name) = args;
        group_id.set(g_id);
        group_name.set(g_name);
        sender_name.set(s_name);
        show_group_share_modal.set(true);
    };

    // Handle join group
    let handle_join_group = move |_| {
        let g_name = group_name();
        let g_id = group_id();
        tokio::spawn(async move {
            // Join the group in the P2P service
            if let Err(err) = P2pService::join_group(&g_name).await {
                println!("Failed to join group: {:?}", err);
                return;
            }

            // Add the group to the conversations list
            let _ = PersistenceService::upsert_conversation(
                format!("group-{}", g_id),
                g_name.clone(),
                true,         // is_group
                g_id.clone(), // peer_id (use group id for proper matching)
                Some("Joined group".to_string()),
                Some(chrono::Utc::now()),
            )
            .await;

            // Save the group to auth service for persistence across restarts
            // created=false because this is a joined group (via invitation), not created by the user
            if let Ok(mut auth_service) = AuthService::new().await {
                if let Err(err) = auth_service.upsert_group(&g_id, &g_name, false).await {
                    println!("Failed to upsert group: {:?}", err);
                }
            } else {
                println!("Failed to create auth service");
            }

            // Send event to refresh the UI
            let _ = EventBus::send(AppEvent::GroupUpdated);
        });
        show_group_share_modal.set(false);
    };

    // Handle ignore group
    let handle_ignore_group = move |_| {
        show_group_share_modal.set(false);
    };

    // Create shared references to the closures
    let send_message = Rc::new(RefCell::new(handle_send_message));
    let send_message_clone = send_message.clone();

    let file_select = Rc::new(RefCell::new(handle_file_select));
    let file_select_clone = file_select.clone();
    let file_select_clone2 = file_select.clone();

    // Move handle_delete_message into Rc<RefCell> once
    let delete_message = Rc::new(RefCell::new(handle_delete_message));

    let handle_key_down = move |e: KeyboardEvent| {
        if e.key() == Key::Enter {
            e.prevent_default();
            let mut send_msg = send_message_clone.borrow_mut();
            send_msg();
        }
    };

    let go_back = move |_| {
        navigator.push("/");
    };

    let chat_room_state_clone = chat_room_state.clone();

    use_effect(move || {
        let _ = chat_room_state_clone.read().messages.len();
        scroll_to_bottom("message-container");
    });

    rsx! {
        div { class: "flex flex-col h-screen",
            // Header
            ChatRoomHeader {
                chat_title: chat_room_state.read().chat_name.as_ref().unwrap_or(&"Chat".to_string()).clone(),
                chat_id: chat_room_state.read().chat_id.clone(),
                is_group_chat: chat_room_state.read().is_group_chat,
                is_online: chat_room_state.read().peer.as_ref().map(|p| p.is_online).unwrap_or(false),
                on_go_back: go_back,
            }

            // Messages - takes up remaining space with overflow scroll
            div { id: "message-container", class: "flex-1 overflow-y-auto",
                crate::features::chat::components::message_list::MessageList {
                    messages: chat_room_state.read().messages.clone(),
                    is_group_chat: chat_room_state.read().is_group_chat,
                    on_download_request,
                    on_delete: move |msg_id: String| {
                        let mut delete_msg = delete_message.borrow_mut();
                        delete_msg(msg_id);
                    },
                    on_group_share: handle_group_share,
                }
            }

            // Group Share Notification Modal
            GroupShareNotificationModal {
                is_open: show_group_share_modal,
                group_name,
                sender_name,
                on_close: move |_| show_group_share_modal.set(false),
                on_join: handle_join_group,
                on_ignore: handle_ignore_group,
            }

            // Message Input - fixed at bottom
            div { class: "bg-white border-t border-gray-200",
                div { class: "p-4",
                    div { class: "flex items-center gap-3",
                        button {
                            class: "p-2 text-gray-600 hover:bg-gray-100 rounded-full",
                            onclick: move |_| {
                                let mut file_sel = file_select_clone2.borrow_mut();
                                file_sel();
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
                                    d: "M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12",
                                }
                            }
                        }
                        button {
                            class: "p-2 text-gray-600 hover:bg-gray-100 rounded-full",
                            onclick: move |_| {
                                let mut file_sel = file_select_clone.borrow_mut();
                                file_sel();
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
                                    d: "M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z",
                                }
                            }
                        }
                        input {
                            class: "flex-1 border border-gray-300 rounded-full px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 text-gray-900",
                            placeholder: "Type a message...",
                            value: chat_room_state.read().new_message.clone(),
                            oninput: move |e| {
                                chat_room_state.write().new_message = e.value();
                            },
                            onkeydown: move |e| handle_key_down(e),
                        }
                        button {
                            class: if chat_room_state.read().new_message.is_empty() || chat_room_state.read().sending { "p-2 text-gray-400 cursor-not-allowed" } else { "p-2 text-blue-600 hover:bg-blue-100 rounded-full" },
                            onclick: move |_| {
                                let mut send_msg = send_message.borrow_mut();
                                send_msg();
                            },
                            disabled: chat_room_state.read().new_message.is_empty() || chat_room_state.read().sending,
                            svg {
                                class: "w-5 h-5",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M12 19l9 2-9-18-9 18 9-2zm0 0v-8",
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
