use dioxus::prelude::*;

use crate::features::signup::components::agree_to_continue::AgreeToContinue;
use crate::features::signup::context::{use_signup_context, SignupAction};

#[component]
pub fn MnemonicInput() -> Element {
    const STEP: usize = 1;

    let context = use_signup_context();
    let dispatch = context.dispatch;

    let handle_change = use_callback(move |(index, value): (usize, String)| {
        // Create a new action to update the specific word
        // We'll handle this in the reducer
        dispatch.call(SignupAction::SetMnemonicWord(index, value));
    });

    // Check if all fields are filled
    let is_all_filled = use_memo(move || {
        let state = context.state.read();
        state.mnemonic.iter().all(|word| !word.is_empty())
    });

    rsx! {
        div { class: "space-y-6",
            div { class: "text-center space-y-2",
                h1 { class: "text-2xl font-bold text-gray-900", "Recover Account" }
                p { class: "text-gray-600 px-4",
                    "Enter your existing seed phrase to restore your account."
                }
            }

            div { class: "bg-white rounded-xl border border-gray-200 p-4 shadow-sm",
                div { class: "grid grid-cols-2 gap-3",
                    {
                        (0..12)
                            .map(|index| {
                                let state = context.state.read();
                                rsx! {
                                    div { key: "{index}", class: "flex items-center space-x-2",
                                        span { class: "text-sm font-medium text-gray-500 min-w-[20px]", "{index + 1}." }
                                        input {
                                            r#type: "text",
                                            placeholder: "word",
                                            value: state.mnemonic[index].clone(),
                                            oninput: move |event: Event<FormData>| handle_change((index, event.value().clone())),
                                            class: "flex-1 text-sm border border-gray-300 rounded-md px-3 py-2 focus:ring-blue-500 focus:border-blue-500 text-gray-900 bg-white",
                                        }
                                    }
                                }
                            })
                    }
                }
            }

            div { class: "bg-amber-50 border border-amber-200 rounded-lg p-4",
                div { class: "text-amber-800 font-semibold mb-1", "⚠️ Important" }
                div { class: "text-amber-700 text-sm",
                    "Anyone with access to your recovery phrase can access your account. Store it securely. Gigi does not keep a backup of your 12-word phrase."
                }
            }

            div { class: "pt-4",
                AgreeToContinue {
                    id: "seedPhraseConfirmation".to_string(),
                    label: "I have entered my seed phrase correctly".to_string(),
                    step: STEP,
                    disabled: Some(!is_all_filled()),
                }
            }
        }
    }
}
