use dioxus::prelude::*;

use crate::features::signup::components::agree_to_continue::AgreeToContinue;

#[component]
pub fn TermsOfUse() -> Element {
    const STEP: usize = 0;

    rsx! {
        div { class: "space-y-6",
            div { class: "text-center space-y-2",
                h1 { class: "text-2xl font-bold text-gray-900", "Terms of Use" }
                p { class: "text-gray-600 px-4",
                    "Please read the following terms and conditions carefully"
                }
            }

            div { class: "bg-white rounded-xl border border-gray-200 shadow-sm",
                div { class: "p-4 max-h-64 overflow-y-auto",
                    div { class: "text-sm text-gray-700 leading-relaxed",
                        h2 { class: "text-lg font-semibold mb-2", "Terms of Service" }
                        p { class: "mb-2",
                            "By using the Gigi P2P network, you agree to the following terms and conditions..."
                        }
                        p { class: "mb-2",
                            "1. You are responsible for maintaining the security of your account and seed phrase."
                        }
                        p { class: "mb-2",
                            "2. Gigi does not store your seed phrase or have access to your account."
                        }
                        p { class: "mb-2",
                            "3. You agree to use the network in accordance with applicable laws."
                        }
                        p { class: "mb-2",
                            "4. Gigi is not responsible for any loss of funds or data."
                        }
                        p { "5. These terms may be updated from time to time." }
                    }
                }
            }

            AgreeToContinue {
                id: "termsOfUseAgreement".to_string(),
                label: "I agree to the Terms of Use Agreement".to_string(),
                step: STEP,
                disabled: None,
            }
        }
    }
}
