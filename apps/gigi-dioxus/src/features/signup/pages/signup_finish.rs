use crate::features::signup::context::use_signup_context;
use dioxus::prelude::*;

#[component]
pub fn SignupFinish() -> Element {
    let context = use_signup_context();
    let state = context.state.read();
    let navigator = use_navigator();

    rsx! {
        div { class: "text-center space-y-6",
            h1 { class: "text-3xl font-bold text-gray-900", "Account Created Successfully!" }
            p { class: "text-gray-600", "Your Gigi account is ready to use" }
            div { class: "bg-white rounded-xl border border-gray-200 p-6 shadow-sm mt-4",
                div { class: "space-y-4",
                    div { class: "text-left",
                        p { class: "text-sm text-gray-600", "Account Name" }
                        p { class: "font-medium text-gray-900", "{state.name}" }
                    }
                    div { class: "text-left",
                        p { class: "text-sm text-gray-600", "Address" }
                        p { class: "font-medium text-gray-900 break-all", "{state.address}" }
                    }
                    div { class: "text-left",
                        p { class: "text-sm text-gray-600", "Peer ID" }
                        p { class: "font-medium text-gray-900 break-all", "{state.peer_id}" }
                    }
                    if state.create_group {
                        div { class: "text-left",
                            p { class: "text-sm text-gray-600", "Group Name" }
                            p { class: "font-medium text-gray-900", "{state.group_name}" }
                        }
                    }
                }
            }
            button {
                class: "mt-6 bg-blue-600 hover:bg-blue-700 text-white font-medium py-3 px-6 rounded-xl transition-colors duration-200",
                onclick: move |_| {
                    navigator.push("/unlock");
                },
                "Login"
            }
        }
    }
}
