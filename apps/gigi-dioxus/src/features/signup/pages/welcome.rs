use dioxus::prelude::*;

use crate::features::signup::context::{use_signup_context, SignupAction, SignupType};

#[component]
pub fn Welcome() -> Element {
    let context = use_signup_context();
    let dispatch = context.dispatch;

    rsx! {
        div { class: "min-h-screen flex flex-col items-center justify-center bg-gray-50 p-4",
            div { class: "w-full max-w-2xl space-y-6",
                div { class: "text-center",
                    h1 { class: "text-3xl font-bold text-gray-900 mb-2", "Let's set up your account" }
                    p { class: "text-gray-600", "Pick an option below to get started" }
                }

                div { class: "bg-white rounded-xl border border-gray-200 p-6 shadow-sm",
                    div { class: "space-y-2",
                        h3 { class: "text-lg font-semibold text-gray-900", "Create new account" }
                        p { class: "text-gray-600 text-sm",
                            "Create a fresh account and generate a new seed phrase"
                        }
                        button {
                            class: "mt-4 text-blue-600 hover:text-blue-800 text-sm font-medium",
                            onclick: move |_| dispatch.call(SignupAction::InitSignup(SignupType::Create)),
                            "Create"
                        }
                    }
                }

                div { class: "bg-white rounded-xl border border-gray-200 p-6 shadow-sm",
                    div { class: "space-y-2",
                        h3 { class: "text-lg font-semibold text-gray-900", "Import seed phrase" }
                        p { class: "text-gray-600 text-sm",
                            "Restore an existing account using your seed phrase"
                        }
                        button {
                            class: "mt-4 text-blue-600 hover:text-blue-800 text-sm font-medium",
                            onclick: move |_| dispatch.call(SignupAction::InitSignup(SignupType::Import)),
                            "Import"
                        }
                    }
                }
            }
        }
    }
}
