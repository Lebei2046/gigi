use dioxus::prelude::*;
use dioxus_router::use_route;

fn get_tab_class(is_active: bool) -> &'static str {
    if is_active {
        "flex-1 flex flex-col items-center justify-center py-2 text-blue-600 transition-colors duration-200"
    } else {
        "flex-1 flex flex-col items-center justify-center py-2 text-gray-500 hover:text-gray-400 transition-colors duration-200"
    }
}

#[component]
pub fn Home() -> Element {
    let mut active_tab = use_signal(|| "chat".to_string());
    let route = use_route::<crate::Route>();

    let chat_class = get_tab_class(active_tab() == "chat");
    let me_class = get_tab_class(active_tab() == "me");

    rsx! {
        div { class: "flex flex-col w-full h-screen bg-gray-50",
            // Main content area
            div { class: "flex-1 w-full",
                // Only render tab content if not in a chat room route
                if !matches!(route, crate::Route::ChatRoom { .. }) {
                    if active_tab() == "chat" {
                        crate::features::chat::Chat {}
                    } else {
                        crate::features::me::Me {}
                    }
                }
            }

            div { class: "h-16 bg-white border-t border-gray-200 shadow-lg",
                div { class: "flex h-full",
                    button {
                        class: "{chat_class}",
                        onclick: move |_| active_tab.set("chat".to_string()),
                        svg {
                            class: "w-5 h-5 mb-1",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z",
                            }
                        }
                        span { class: "text-xs font-medium", "Chat" }
                    }

                    button {
                        class: "{me_class}",
                        onclick: move |_| active_tab.set("me".to_string()),
                        svg {
                            class: "w-5 h-5 mb-1",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z",
                            }
                        }
                        span { class: "text-xs font-medium", "Me" }
                    }
                }
            }
        }
    }
}
