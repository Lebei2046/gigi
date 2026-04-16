use dioxus::prelude::*;

use crate::features::signup::components::agree_to_continue::AgreeToContinue;
use crate::features::signup::context::use_signup_context;

#[component]
pub fn MnemonicDisplay() -> Element {
    const STEP: usize = 1;

    let context = use_signup_context();
    let state = context.state.read();

    // Generate a mock mnemonic phrase for demonstration
    // In a real implementation, this would call the authGenerateMnemonic function
    let mnemonic = state.mnemonic.clone();

    // If mnemonic is empty, generate a mock one
    let display_mnemonic = if mnemonic.iter().all(|word| word.is_empty()) {
        vec![
            "abandon", "ability", "able", "about", "above", "absent", "absorb", "abstract",
            "absurd", "abuse", "access", "accident",
        ]
        .iter()
        .map(|&s| s.to_string())
        .collect()
    } else {
        mnemonic
    };

    let display_mnemonic_clone = display_mnemonic.clone();
    let handle_copy = move |_| {
        let mnemonic_string = display_mnemonic_clone.join(" ");
        // In a real implementation, this would copy to clipboard
        // For now, we'll just log it
        println!("Copied to clipboard: {}", mnemonic_string);
    };

    rsx! {
        div { class: "space-y-6",
            div { class: "text-center space-y-2",
                h1 { class: "text-2xl font-bold text-gray-900", "Seed Phrase" }
                p { class: "text-gray-600 px-4",
                    "Please write down your seed phrase in the correct order and keep it in a safe place."
                }
            }

            div { class: "bg-white rounded-xl border border-gray-200 p-4 shadow-sm",
                div { class: "grid grid-cols-3 gap-3",
                    {display_mnemonic.iter().enumerate().map(|(index, word)| rsx! {
                        div {
                            key: "{index}",
                            class: "flex items-center space-x-2 bg-gray-50 rounded-lg px-3 py-2 border border-gray-200",
                            span { class: "text-sm font-medium text-gray-500 min-w-[20px]", "{index + 1}." }
                            span { class: "text-sm font-medium text-gray-900", {word.clone()} }
                        }
                    })}
                }
            }

            div { class: "flex justify-center",
                button {
                    class: "w-full max-w-xs bg-blue-600 hover:bg-blue-700 text-white font-medium py-3 rounded-xl transition-colors duration-200",
                    onclick: handle_copy,
                    "Copy Seed Phrase"
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
                    label: "I have written down my seed phrase on paper and stored it securely".to_string(),
                    step: STEP,
                    disabled: None,
                }
            }
        }
    }
}
