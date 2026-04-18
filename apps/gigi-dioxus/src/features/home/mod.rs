use dioxus::prelude::*;

pub mod chat;
pub mod me;

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

    let chat_class = get_tab_class(active_tab() == "chat");
    let me_class = get_tab_class(active_tab() == "me");

    rsx! {
        div { class: "flex flex-col w-full h-screen bg-gray-50",
            div { class: "flex-grow w-full overflow-hidden",
                if active_tab() == "chat" {
                    div { class: "h-full", chat::Chat {} }
                } else if active_tab() == "me" {
                    div { class: "h-full", me::Me {} }
                }
            }

            div { class: "fixed inset-x-0 bottom-0 h-[calc(4rem+env(safe-area-inset-bottom))] bg-white border-t border-gray-200 shadow-lg",
                div { class: "h-full flex items-end pb-[env(safe-area-inset-bottom)]",
                    div { class: "flex w-full h-full bg-transparent border-none",
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
                                    d: "M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
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
                                    d: "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                                }
                            }
                            span { class: "text-xs font-medium", "Me" }
                        }
                    }
                }
            }
        }
    }
}