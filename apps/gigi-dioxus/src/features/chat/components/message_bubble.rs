use dioxus::prelude::*;

use crate::features::chat::chat_state::{Message, MessageType};

// Message Bubble Component
#[component]
pub fn MessageBubble(message: Message) -> Element {
    match message.message_type {
        MessageType::Text => rsx! {
            TextMessageBubble { message: message }
        },
        MessageType::Image => rsx! {
            ImageMessageBubble { message: message }
        },
        MessageType::File => rsx! {
            FileMessageBubble { message: message }
        },
    }
}

// Text Message Bubble
#[component]
fn TextMessageBubble(message: Message) -> Element {
    rsx! {
        div { 
            class: if message.is_own {
                "flex justify-end"
            } else {
                "flex"
            },
            div { 
                class: if message.is_own {
                    "bg-blue-100 rounded-lg rounded-tr-none p-3 max-w-[80%]"
                } else {
                    "bg-white rounded-lg rounded-tl-none p-3 max-w-[80%] border border-gray-200"
                },
                if !message.is_own {
                    div { class: "text-xs font-medium text-gray-500 mb-1", "{message.sender}" }
                }
                div { class: "text-sm text-gray-900", "{message.content}" }
                div { 
                    class: if message.is_own {
                        "text-xs text-blue-600 mt-1 text-right"
                    } else {
                        "text-xs text-gray-500 mt-1"
                    },
                    "{message.timestamp}"
                }
            }
        }
    }
}

// Image Message Bubble
#[component]
fn ImageMessageBubble(message: Message) -> Element {
    rsx! {
        div { 
            class: if message.is_own {
                "flex justify-end"
            } else {
                "flex"
            },
            div { 
                class: if message.is_own {
                    "bg-blue-100 rounded-lg rounded-tr-none p-3 max-w-[80%]"
                } else {
                    "bg-white rounded-lg rounded-tl-none p-3 max-w-[80%] border border-gray-200"
                },
                if !message.is_own {
                    div { class: "text-sm text-gray-900 mb-2", "{message.sender}:" }
                }
                div { class: "bg-gray-100 rounded p-2 mb-2",
                    // In a real app, this would be an actual image
                    div { class: "w-48 h-48 bg-gray-200 rounded flex items-center justify-center",
                        span { class: "text-gray-500", "Image" }
                    }
                }
                div { 
                    class: if message.is_own {
                        "text-xs text-blue-600 mt-1 text-right"
                    } else {
                        "text-xs text-gray-500 mt-1"
                    },
                    "{message.timestamp}"
                }
            }
        }
    }
}

// File Message Bubble
#[component]
fn FileMessageBubble(message: Message) -> Element {
    rsx! {
        div { 
            class: if message.is_own {
                "flex justify-end"
            } else {
                "flex"
            },
            div { 
                class: if message.is_own {
                    "bg-blue-100 rounded-lg rounded-tr-none p-3 max-w-[80%]"
                } else {
                    "bg-white rounded-lg rounded-tl-none p-3 max-w-[80%] border border-gray-200"
                },
                if !message.is_own {
                    div { class: "text-sm text-gray-900 mb-2", "{message.sender}:" }
                }
                div { class: "bg-gray-100 rounded p-3 mb-2 flex items-center gap-3",
                    div { class: "w-10 h-10 bg-gray-200 rounded flex items-center justify-center",
                        svg { 
                            class: "w-5 h-5 text-gray-500",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path { 
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                            }
                        }
                    }
                    div { class: "flex-1 min-w-0",
                        div { class: "text-sm font-medium text-gray-900 truncate", "{message.content}" }
                        div { class: "text-xs text-gray-500", "File attachment" }
                    }
                    button { 
                        class: "text-blue-600 hover:text-blue-800",
                        svg { 
                            class: "w-5 h-5",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path { 
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
                            }
                        }
                    }
                }
                div { 
                    class: if message.is_own {
                        "text-xs text-blue-600 mt-1 text-right"
                    } else {
                        "text-xs text-gray-500 mt-1"
                    },
                    "{message.timestamp}"
                }
            }
        }
    }
}
