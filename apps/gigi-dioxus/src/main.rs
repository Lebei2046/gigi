use dioxus::prelude::*;
use dioxus_router::{use_navigator, Routable, Router};

mod features;
mod services;

#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/chat/:id")]
    ChatRoom { id: String },
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
    // Initialize event bus
    crate::services::event_bus::EventBus::init();

    // Initialize persistence service
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        if let Err(e) = crate::services::persistence_service::PersistenceService::initialize().await
        {
            eprintln!("Failed to initialize persistence service: {:?}", e);
        }
    });

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

/// Home page with tabbed navigation (Chat + Me)
#[component]
pub fn Home() -> Element {
    let auth_state = crate::services::auth_context::AuthContext::get_state();
    let navigator = use_navigator();

    if !matches!(
        auth_state,
        crate::services::auth_context::AuthState::Authenticated(_)
    ) {
        spawn(async move {
            match crate::services::auth_service::AuthService::new().await {
                Ok(mut auth_service) => match auth_service.has_account().await {
                    Ok(exists) => {
                        if exists {
                            // Auto login for testing
                            match auth_service.login("password").await {
                                Ok(login_result) => {
                                    match auth_service.get_account_info().await {
                                        Ok(Some(info)) => {
                                            let name = info.name.clone();
                                            let account_info =
                                                crate::services::auth_context::AccountInfo {
                                                    name: info.name,
                                                    peer_id: info.peer_id,
                                                    address: info.address,
                                                };
                                            crate::services::auth_context::AuthContext::set_authenticated(account_info);

                                            // Initialize P2P network
                                            if let Err(err) = crate::services::p2p_service::P2pService::initialize(&login_result.private_key, &name).await {
                                                println!("Failed to initialize P2P network: {:?}", err);
                                            } else {
                                                println!("P2P network initialized successfully");
                                            }
                                        }
                                        _ => {
                                            println!("Failed to get account info");
                                        }
                                    }
                                }
                                Err(err) => {
                                    println!("Login error: {:?}", err);
                                    navigator.push("/unlock");
                                }
                            }
                        } else {
                            navigator.push("/signup");
                        }
                    }
                    Err(_) => {
                        navigator.push("/signup");
                    }
                },
                Err(_) => {
                    navigator.push("/signup");
                }
            }
        });

        rsx! {
            div { class: "flex items-center justify-center min-h-screen",
                div { class: "text-2xl font-semibold text-gray-700", "Loading..." }
            }
        }
    } else {
        rsx! {
            features::home::Home {}
        }
    }
}

/// Chat page
#[component]
pub fn Chat() -> Element {
    rsx! {
        features::chat::Chat {}
    }
}

/// Chat Room page
#[component]
pub fn ChatRoom(id: String) -> Element {
    rsx! {
        features::chat::chat_room::ChatRoom { id }
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
