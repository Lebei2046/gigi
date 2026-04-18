use dioxus::prelude::*;

#[component]
pub fn Chat() -> Element {
    rsx! {
        div { class: "flex flex-col h-full bg-gray-50",
            div { class: "bg-white border-b border-gray-200 px-4 py-4",
                h2 { class: "text-2xl font-bold text-gray-900", "Messages" }
            }

            div { class: "flex-1 overflow-y-auto px-4 py-4",
                div { class: "flex items-center justify-center h-full",
                    div { class: "text-center space-y-4",
                        div { class: "w-24 h-24 bg-gray-200 rounded-full flex items-center justify-center mx-auto",
                            svg {
                                class: "w-12 h-12 text-gray-400",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
                                }
                            }
                        }
                        h3 { class: "text-xl font-semibold text-gray-900", "Chat" }
                        p { class: "text-gray-600 text-sm px-6",
                            "Connect with peers to start chatting"
                        }
                    }
                }
            }
        }
    }
}