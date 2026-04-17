use dioxus::prelude::*;
use dioxus_router::{use_navigator, Routable, Router};

mod features;
mod services;

use services::auth_context::{AuthContext, AuthState};

#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/signup")]
    Signup {},
    #[route("/unlock")]
    Unlock {},
    #[route("/reset")]
    ResetAccount {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    let navigator = use_navigator();
    let mut checked = use_signal(|| false);

    let current_state = AuthContext::get_state();
    if matches!(current_state, AuthState::Authenticated(_)) {
        if let Some(info) = AuthContext::get_state().get_account_info() {
            return rsx! {
                div { class: "min-h-screen bg-gray-50",
                    div { class: "max-w-4xl mx-auto py-12 px-4",
                        div { class: "bg-white rounded-2xl shadow-lg border border-gray-100 p-8",
                            h1 { class: "text-3xl font-bold text-gray-900 mb-2", "Welcome to Gigi" }
                            p { class: "text-gray-600 mb-8", "Your P2P network is ready" }

                            div { class: "space-y-6",
                                div { class: "bg-gradient-to-br from-blue-50 to-indigo-50 rounded-xl p-6 border border-blue-100",
                                    h2 { class: "text-lg font-semibold text-gray-900 mb-4",
                                        "Account Information"
                                    }
                                    div { class: "space-y-3",
                                        div { class: "flex items-center justify-between",
                                            span { class: "text-sm font-medium text-gray-500",
                                                "Account Name"
                                            }
                                            span { class: "text-lg font-semibold text-gray-900",
                                                "{info.name}"
                                            }
                                        }
                                        div { class: "flex items-center justify-between",
                                            span { class: "text-sm font-medium text-gray-500",
                                                "Peer ID"
                                            }
                                            span { class: "text-sm font-mono text-gray-700 bg-gray-100 px-3 py-1 rounded",
                                                "{info.peer_id}"
                                            }
                                        }
                                        div { class: "flex items-center justify-between",
                                            span { class: "text-sm font-medium text-gray-500",
                                                "Address"
                                            }
                                            span { class: "text-sm font-mono text-gray-700 bg-gray-100 px-3 py-1 rounded break-all",
                                                "{info.address}"
                                            }
                                        }
                                    }
                                }

                                div { class: "mt-8 pt-6 border-t border-gray-200",
                                    h3 { class: "text-lg font-semibold text-gray-900 mb-4",
                                        "Quick Actions"
                                    }
                                    div { class: "grid grid-cols-2 gap-4",
                                        button {
                                            class: "bg-blue-600 hover:bg-blue-700 text-white font-medium py-3 px-6 rounded-xl transition-colors duration-200",
                                            onclick: move |_| {
                                                println!("Open chat");
                                            },
                                            "Open Chat"
                                        }
                                        button {
                                            class: "bg-white hover:bg-gray-50 text-gray-700 font-medium py-3 px-6 rounded-xl border border-gray-300 transition-colors duration-200",
                                            onclick: move |_| {
                                                println!("Settings");
                                            },
                                            "Settings"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            };
        }
    }

    use_effect(move || {
        if !*checked.read() {
            checked.set(true);
            spawn(async move {
                match services::auth_service::AuthService::new().await {
                    Ok(auth_service) => match auth_service.has_account().await {
                        Ok(exists) => {
                            if exists {
                                AuthContext::set_unauthenticated();
                                navigator.push("/unlock");
                            } else {
                                AuthContext::set_unregistered();
                                navigator.push("/signup");
                            }
                        }
                        Err(_) => {
                            AuthContext::set_unregistered();
                            navigator.push("/signup");
                        }
                    },
                    Err(_) => {
                        AuthContext::set_unregistered();
                        navigator.push("/signup");
                    }
                }
            });
        }
    });

    rsx! {
        div { class: "flex items-center justify-center min-h-screen",
            div { class: "text-2xl font-semibold text-gray-700", "Loading..." }
        }
    }
}

/// Signup page
#[component]
pub fn Signup() -> Element {
    rsx! {
        features::signup::Signup {}
    }
}

/// Unlock page
#[component]
pub fn Unlock() -> Element {
    rsx! {
        features::signin::unlock::Unlock {}
    }
}

/// Reset Account page
#[component]
pub fn ResetAccount() -> Element {
    rsx! {
        features::signin::reset_account::ResetAccount {}
    }
}
