use dioxus::prelude::*;
use dioxus_router::{use_navigator, Link};

use crate::services::auth_context::{AccountInfo, AuthContext};
use crate::services::auth_service::AuthService;

#[component]
pub fn Unlock() -> Element {
    let mut password = use_signal(|| String::new());
    let mut is_loading = use_signal(|| false);
    let mut error = use_signal(|| Option::<String>::None);
    let navigator = use_navigator();

    let mut handle_unlock = move |_| {
        let password_clone = password.read().clone();
        is_loading.set(true);
        error.set(None);

        spawn(async move {
            match AuthService::new().await {
                Ok(mut auth_service) => match auth_service.login(&password_clone).await {
                    Ok(_) => {
                        match auth_service.get_account_info().await {
                            Ok(Some(info)) => {
                                let account_info = AccountInfo {
                                    name: info.name,
                                    peer_id: info.peer_id,
                                    address: info.address,
                                };
                                AuthContext::set_authenticated(account_info);
                                navigator.push("/");
                            }
                            _ => {
                                error.set(Some("Failed to get account info".to_string()));
                            }
                        }
                        is_loading.set(false);
                    }
                    Err(err) => {
                        println!("Login error: {:?}", err);
                        error.set(Some("Invalid password".to_string()));
                        is_loading.set(false);
                    }
                },
                Err(err) => {
                    println!("Error creating auth service: {:?}", err);
                    error.set(Some("Failed to initialize authentication".to_string()));
                    is_loading.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-gray-50 px-4",
            div { class: "w-full max-w-md",
                div { class: "text-center mb-8",
                    h1 { class: "text-3xl font-bold text-gray-900 mb-2", "Welcome Back" }
                    p { class: "text-gray-600", "Enter your password to unlock your account" }
                }

                div { class: "bg-white rounded-2xl shadow-lg border border-gray-100 p-6 space-y-6",
                    div { class: "space-y-4",
                        div { class: "space-y-2",
                            label { class: "text-sm font-medium text-gray-700", "Password" }
                            input {
                                id: "password",
                                r#type: "password",
                                placeholder: "Enter your password",
                                oninput: move |event| {
                                    let value = event.value();
                                    password.set(value.clone());
                                    if error.read().is_some() {
                                        error.set(None);
                                    }
                                },
                                onkeydown: move |e: KeyboardEvent| {
                                    if e.key() == Key::Enter {
                                        handle_unlock(());
                                    }
                                },
                                disabled: *is_loading.read(),
                                style: "color: #111827; background-color: #ffffff; width: 100%; padding: 0.75rem 1rem; border: 1px solid #d1d5db; border-radius: 0.5rem; font-size: 1rem;",
                            }
                            {error.read().as_ref().map(|err| rsx! {
                                div { class: "bg-red-50 border border-red-200 rounded-lg p-3",
                                    p { class: "text-red-600 text-sm", "⚠️ {err}" }
                                }
                            })}
                        }

                        button {
                            r#type: "button",
                            disabled: *is_loading.read() || password().is_empty(),
                            class: "w-full py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed text-white font-medium rounded-lg transition-all duration-200",
                            onclick: move |e: MouseEvent| {
                                e.prevent_default();
                                handle_unlock(());
                            },
                            {
                                if *is_loading.read() {
                                    rsx! {
                                        span { class: "flex items-center justify-center",
                                            svg {
                                                class: "animate-spin -ml-1 mr-3 h-5 w-5 text-white",
                                                xmlns: "http://www.w3.org/2000/svg",
                                                fill: "none",
                                                view_box: "0 0 24 24",
                                                circle {
                                                    class: "opacity-25",
                                                    cx: "12",
                                                    cy: "12",
                                                    r: "10",
                                                    stroke: "currentColor",
                                                    stroke_width: "4",
                                                }
                                                path {
                                                    class: "opacity-75",
                                                    fill: "currentColor",
                                                    d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                                                }
                                            }
                                            "Unlocking..."
                                        }
                                    }
                                } else {
                                    rsx! { "Unlock Account" }
                                }
                            }
                        }
                    }

                    div { class: "text-center pt-4 border-t border-gray-100",
                        Link {
                            to: "/reset",
                            class: "text-blue-600 hover:text-blue-700 text-sm font-medium",
                            "Forgot password?"
                        }
                    }
                }
            }
        }
    }
}