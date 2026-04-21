use dioxus::prelude::*;
use dioxus_router::use_navigator;
use std::cell::RefCell;
use std::rc::Rc;

use crate::features::chat::components::{ChatRoomHeader, ChatRoomInput, MessageList};
use crate::features::chat::hooks::{use_chat_initialization, use_chat_room_initialization, use_message_actions};

// Chat Room Component
#[component]
pub fn ChatRoom(id: String) -> Element {
    let navigator = use_navigator();
    let chat_state = use_chat_initialization();
    let mut chat_room_state = use_chat_room_initialization(id, chat_state);
    let (
        mut handle_send_message,
        handle_image_select,
        handle_file_select,
        handle_file_download_request,
        _handle_share_file,
    ) = use_message_actions(chat_room_state.clone());

    // Create a shared reference to the send message closure
    let send_message = Rc::new(RefCell::new(handle_send_message));
    let send_message_clone = send_message.clone();

    let handle_input_change = move |value: String| {
        chat_room_state.write().new_message = value;
    };

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

    rsx! {
        div { class: "flex flex-col h-full",
            // Header
            ChatRoomHeader {
                chat_title: chat_room_state.read().chat_name.as_ref().unwrap_or(&"Chat".to_string()).clone(),
                chat_id: chat_room_state.read().chat_id.clone(),
                is_group_chat: chat_room_state.read().is_group_chat,
                on_go_back: go_back,
            }

            // Messages
            MessageList {
                messages: chat_room_state.read().messages.clone(),
                is_group_chat: chat_room_state.read().is_group_chat,
                on_download_request: handle_file_download_request,
            }

            // Message Input
            ChatRoomInput {
                new_message: chat_room_state.read().new_message.clone(),
                sending: chat_room_state.read().sending,
                is_group_chat: chat_room_state.read().is_group_chat,
                chat_name: chat_room_state.read().chat_name.as_ref().unwrap_or(&"Chat".to_string()).clone(),
                on_send_message: move || {
                    let mut send_msg = send_message.borrow_mut();
                    send_msg();
                },
                on_file_select: handle_file_select,
                on_image_select: handle_image_select,
                on_message_change: handle_input_change,
                on_key_down: handle_key_down,
            }
        }
    }
}
