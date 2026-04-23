use dioxus::prelude::*;

// Chat Room Input Component
#[component]
pub fn ChatRoomInput(
    new_message: String,
    sending: bool,
    is_group_chat: bool,
    chat_name: String,
    on_send_message: EventHandler<()>,
    on_file_select: EventHandler<()>,
    on_image_select: EventHandler<()>,
    on_message_change: EventHandler<String>,
    on_key_down: EventHandler<KeyboardEvent>,
) -> Element {
    rsx! {
        div { class: "p-4",
            div { class: "flex items-center gap-3",
                button {
                    class: "p-2 text-gray-600 hover:bg-gray-100 rounded-full transition-colors",
                    title: "Attach file",
                    onclick: move |_| on_file_select.call(()),
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
                    class: "p-2 text-gray-600 hover:bg-gray-100 rounded-full transition-colors",
                    title: "Attach image",
                    onclick: move |_| on_image_select.call(()),
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
                div { class: "flex-1",
                    input {
                        class: "w-full border border-gray-300 rounded-full px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 text-gray-900 transition-all",
                        placeholder: "Type a message...",
                        value: new_message.clone(),
                        oninput: move |e| on_message_change.call(e.value()),
                        onkeydown: move |e| on_key_down.call(e),
                    }
                }
                button {
                    class: if new_message.is_empty() || sending { "p-2 text-gray-400 cursor-not-allowed" } else { "p-2 text-blue-600 hover:bg-blue-100 rounded-full transition-colors" },
                    onclick: move |_| on_send_message.call(()),
                    disabled: new_message.is_empty() || sending,
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
