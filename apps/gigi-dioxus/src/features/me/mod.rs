use crate::services::auth_context::AccountInfo;
use crate::services::auth_context::AuthContext;
use dioxus::prelude::*;

pub mod contact_list;
pub mod qr_code;

fn get_default_name(info: Option<&AccountInfo>) -> String {
    info.map(|i| i.name.clone())
        .unwrap_or_else(|| "Anonymous User".to_string())
}

fn get_default_peer_id(info: Option<&AccountInfo>) -> String {
    info.map(|i| i.peer_id.clone())
        .unwrap_or_else(|| "Unknown".to_string())
}

#[component]
pub fn Me() -> Element {
    let auth_state = AuthContext::get_state();
    let info = auth_state.get_account_info();
    let mut active_tab = use_signal(|| "contacts".to_string());

    let default_name = get_default_name(info);
    let default_peer_id = get_default_peer_id(info);

    let contacts_class = if active_tab() == "contacts" {
        "flex-1 py-2 px-4 rounded-md text-center text-sm font-medium transition-colors duration-200 bg-white text-blue-600"
    } else {
        "flex-1 py-2 px-4 rounded-md text-center text-sm font-medium transition-colors duration-200 text-white hover:bg-white/30"
    };

    let qr_code_class = if active_tab() == "qr-code" {
        "flex-1 py-2 px-4 rounded-md text-center text-sm font-medium transition-colors duration-200 bg-white text-blue-600"
    } else {
        "flex-1 py-2 px-4 rounded-md text-center text-sm font-medium transition-colors duration-200 text-white hover:bg-white/30"
    };

    let settings_class = if active_tab() == "settings" {
        "flex-1 py-2 px-4 rounded-md text-center text-sm font-medium transition-colors duration-200 bg-white text-blue-600"
    } else {
        "flex-1 py-2 px-4 rounded-md text-center text-sm font-medium transition-colors duration-200 text-white hover:bg-white/30"
    };

    rsx! {
        div { class: "flex flex-col h-full bg-gray-50",
            div { class: "bg-gradient-to-br from-blue-600 to-purple-700 p-6",
                div { class: "flex items-center space-x-4",
                    div { class: "w-16 h-16 bg-white/20 rounded-full flex items-center justify-center",
                        svg {
                            class: "w-8 h-8 text-white",
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
                    }

                    div { class: "flex-1 min-w-0",
                        h1 { class: "text-2xl font-bold text-white truncate mb-1",
                            "{default_name}"
                        }
                        div { class: "flex items-center gap-2",
                            span { class: "text-blue-100 text-sm", "Peer ID:" }
                            p { class: "text-white text-sm font-mono bg-white/20 px-2 py-1 rounded truncate max-w-[200px]",
                                "{default_peer_id}"
                            }
                        }
                    }
                }

                div { class: "mt-6 flex bg-white/20 rounded-lg p-1",
                    button {
                        class: "{contacts_class}",
                        onclick: move |_| active_tab.set("contacts".to_string()),
                        "Contacts"
                    }
                    button {
                        class: "{qr_code_class}",
                        onclick: move |_| active_tab.set("qr-code".to_string()),
                        "QR Code"
                    }
                    button {
                        class: "{settings_class}",
                        onclick: move |_| active_tab.set("settings".to_string()),
                        "Settings"
                    }
                }
            }

            div { class: "flex-1 bg-white rounded-t-3xl -mt-4 relative overflow-y-auto",
                div { class: "bg-white rounded-t-3xl",
                    div { class: "w-12 h-1 bg-gray-300 rounded-full mx-auto mt-3 mb-4" }

                    if active_tab() == "contacts" {
                        div { class: "h-full", contact_list::ContactList {} }
                    } else if active_tab() == "qr-code" {
                        div { class: "h-full",
                            qr_code::QRCodeTab {
                                name: default_name.clone(),
                                peer_id: default_peer_id.clone(),
                            }
                        }
                    } else if active_tab() == "settings" {
                        div { class: "p-6 h-full",
                            div { class: "bg-gray-50 rounded-xl p-6 text-center",
                                div { class: "w-16 h-16 bg-gray-200 rounded-full flex items-center justify-center mx-auto mb-4",
                                    svg {
                                        class: "w-8 h-8 text-gray-400",
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
                                h3 { class: "text-lg font-semibold text-gray-900 mb-2",
                                    "Settings Coming Soon"
                                }
                                p { class: "text-gray-600 text-sm",
                                    "Settings and preferences will be available in the next update."
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
