use dioxus::prelude::*;
use dioxus_router::{use_navigator, Routable, Router};

mod features;
mod services;

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

/// Home page with tabbed navigation (Chat + Me)
#[component]
pub fn Home() -> Element {
    let auth_state = crate::services::auth_context::AuthContext::get_state();
    let navigator = use_navigator();

    if !matches!(auth_state, crate::services::auth_context::AuthState::Authenticated(_)) {
        spawn(async move {
            match crate::services::auth_service::AuthService::new().await {
                Ok(auth_service) => match auth_service.has_account().await {
                    Ok(exists) => {
                        if exists {
                            navigator.push("/unlock");
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