use dioxus::prelude::*;

// Chat Room Header Component
#[component]
pub fn ChatRoomHeader(
    chat_title: String,
    chat_id: Option<String>,
    is_group_chat: bool,
    is_online: bool,
    on_go_back: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "bg-white border-b border-gray-200 px-4 py-4",
            div { class: "flex items-center",
                button {
                    class: "mr-4 text-gray-600",
                    onclick: move |_| on_go_back.call(()),
                    svg {
                        class: "w-6 h-6",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M15 19l-7-7 7-7",
                        }
                    }
                }
                h2 { class: "text-xl font-bold text-gray-900", "{chat_title}" }
                if is_group_chat {
                    div { class: "ml-auto flex items-center gap-3",
                        div { class: "text-sm text-gray-500", "5 members" }
                        button { class: "text-gray-600",
                            svg {
                                class: "w-6 h-6",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z",
                                }
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M15 12a3 3 0 11-6 0 3 3 0 016 0z",
                                }
                            }
                        }
                    }
                } else {
                    div { class: "ml-auto flex items-center gap-3",
                        div { class: if is_online { "w-2 h-2 bg-green-500 rounded-full" } else { "w-2 h-2 bg-gray-400 rounded-full" } }
                        div { class: "text-sm text-gray-500", if is_online { "Online" } else { "Offline" } }
                        button { class: "text-gray-600",
                            svg {
                                class: "w-6 h-6",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M15 12a3 3 0 11-6 0 3 3 0 016 0z",
                                }
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z",
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
