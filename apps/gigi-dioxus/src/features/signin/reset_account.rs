use dioxus::prelude::*;
use dioxus_router::{navigator, Link};

use crate::services::auth_service::AuthService;

#[component]
pub fn ResetAccount() -> Element {
    let mut checked = use_signal(|| false);
    let mut is_loading = use_signal(|| false);
    let mut error = use_signal(|| Option::<String>::None);

    let mut handle_reset = move |_| {
        if !*checked.read() {
            return;
        }

        is_loading.set(true);
        error.set(None);

        spawn(async move {
            match AuthService::new().await {
                Ok(mut auth_service) => {
                    match auth_service.delete_account().await {
                        Ok(_) => {
                            println!("Account deleted successfully");
                            is_loading.set(false);
                            // Navigate back to signup page
                            navigator().push("/signup");
                        }
                        Err(err) => {
                            println!("Error deleting account: {:?}", err);
                            error.set(Some("Failed to delete account".to_string()));
                            is_loading.set(false);
                        }
                    }
                }
                Err(err) => {
                    println!("Error creating auth service: {:?}", err);
                    error.set(Some("Failed to initialize authentication".to_string()));
                    is_loading.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "min-h-screen flex items-center justify-center bg-gradient-to-br from-red-50 to-gray-50 px-4",
            div { class: "w-full max-w-md",
                div { class: "text-center mb-8",
                    div { class: "mx-auto w-16 h-16 bg-red-100 rounded-full flex items-center justify-center mb-4",
                        svg {
                            class: "w-8 h-8 text-red-600",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z",
                            }
                        }
                    }
                    h1 { class: "text-3xl font-bold text-gray-900 mb-2", "Reset Account" }
                    p { class: "text-gray-600", "You are about to permanently delete your account" }
                }

                div { class: "bg-white rounded-2xl shadow-lg border border-gray-100 p-6 space-y-6",
                    div { class: "bg-red-50 border border-red-200 rounded-lg p-4",
                        div { class: "flex items-center gap-2",
                            svg {
                                class: "w-5 h-5 text-red-800",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z",
                                }
                            }
                            h3 { class: "text-red-800 font-semibold", "Warning: Destructive Action" }
                        }
                        p { class: "text-red-700 text-sm mt-2",
                            "Resetting your account will permanently delete all your data, including your account information and transaction history. This action cannot be undone. Please ensure you have backed up your recovery phrase before proceeding."
                        }
                    }

                    div { class: "space-y-4",
                        div { class: "bg-gray-50 rounded-lg p-4 border border-gray-200",
                            h3 { class: "text-sm font-semibold text-gray-900 mb-2",
                                "What will be deleted:"
                            }
                            ul { class: "text-sm text-gray-600 space-y-1",
                                li { class: "flex items-center gap-2",
                                    span { class: "text-red-500", "✕" }
                                    "Account information"
                                }
                                li { class: "flex items-center gap-2",
                                    span { class: "text-red-500", "✕" }
                                    "Transaction history"
                                }
                                li { class: "flex items-center gap-2",
                                    span { class: "text-red-500", "✕" }
                                    "Chat messages"
                                }
                                li { class: "flex items-center gap-2",
                                    span { class: "text-red-500", "✕" }
                                    "Group memberships"
                                }
                            }
                        }

                        div { class: "flex items-start space-x-3 p-4 bg-amber-50 rounded-lg border border-amber-200",
                            input {
                                r#type: "checkbox",
                                id: "accept-risk",
                                checked: *checked.read(),
                                onchange: move |e| checked.set(e.checked()),
                                class: "mt-1 w-4 h-4 text-red-600 bg-gray-100 border-gray-300 rounded focus:ring-red-500 focus:ring-2",
                            }
                            label {
                                r#for: "accept-risk",
                                class: "text-sm font-medium text-gray-700 leading-relaxed",
                                "I understand this action is permanent and irreversible. I have backed up my recovery phrase and accept all risks."
                            }
                        }
                    }

                    {error.read().as_ref().map(|err| rsx! {
                        div { class: "bg-red-50 border border-red-200 rounded-lg p-3",
                            p { class: "text-red-600 text-sm", "⚠️ {err}" }
                        }
                    })}

                    div { class: "flex gap-3",
                        Link {
                            to: "/unlock",
                            class: "flex-1 py-3 border border-gray-300 text-gray-700 hover:bg-gray-50 rounded-lg text-center font-medium",
                            "Cancel"
                        }
                        button {
                            class: "flex-1 py-3 bg-red-600 hover:bg-red-700 text-white font-medium rounded-lg transition-all duration-200 disabled:bg-gray-300 disabled:cursor-not-allowed",
                            disabled: !*checked.read() || *is_loading.read(),
                            onclick: handle_reset,
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
                                            "Resetting..."
                                        }
                                    }
                                } else {
                                    rsx! { "Reset Account" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
