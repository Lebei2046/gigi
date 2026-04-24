use dioxus::prelude::*;

use crate::features::chat::chat_state::Message;
use crate::features::chat::components::message_bubble::MessageBubble;

#[component]
pub fn MessageList(
    messages: Vec<Message>,
    is_group_chat: bool,
    on_download_request: EventHandler<(String, String, String)>,
    on_delete: EventHandler<String>,
) -> Element {
    let mut last_msg_count = use_signal(|| 0);
    let messages_clone = messages.clone();

    use_effect(move || {
        let current_count = messages.len();
        if current_count != *last_msg_count.read() {
            last_msg_count.set(current_count);
        }
    });

    rsx! {
        div { class: "w-full h-full p-4 space-y-3", id: "message-list",
            for message in messages_clone {
                MessageBubble {
                    message: message.clone(),
                    on_delete: on_delete.clone(),
                    on_download_request: Some(on_download_request.clone()),
                }
            }
        }
    }
}
