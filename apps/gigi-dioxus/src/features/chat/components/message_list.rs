use dioxus::prelude::*;

use crate::features::chat::chat_state::Message;
use crate::features::chat::components::message_bubble::MessageBubble;

// Message List Component
#[component]
pub fn MessageList(
    messages: Vec<Message>,
    is_group_chat: bool,
    on_download_request: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "flex-1 overflow-y-auto p-4 space-y-3",
            for message in messages {
                MessageBubble { message: message.clone() }
            }
        }
    }
}
