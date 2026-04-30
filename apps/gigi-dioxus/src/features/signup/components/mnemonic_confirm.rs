use dioxus::prelude::*;

use crate::features::signup::context::{use_signup_context, SignupAction};

#[component]
pub fn MnemonicConfirm() -> Element {
    const STEP: usize = 2;

    let context = use_signup_context();
    let dispatch = context.dispatch;

    // Generate random indices for the words to confirm
    // Using a simple method without rand crate to avoid WASM build issues
    // Use a fixed set of indices for demonstration
    let random_indices = vec![2, 5, 9];

    // Track user inputs
    let mut user_inputs = use_signal(std::collections::HashMap::<usize, String>::new);

    // Check if all inputs are correct
    let random_indices_clone = random_indices.clone();
    let is_all_correct = use_memo(move || {
        let state = context.state.read();
        random_indices_clone.iter().all(|&idx| {
            if let Some(input) = user_inputs.read().get(&idx) {
                // Get the expected word - use mock if not set
                let expected_word = state.mnemonic.get(idx).unwrap_or(&String::new()).clone();
                let display_word = if expected_word.is_empty() {
                    // Use mock words for validation
                    vec![
                        "abandon", "ability", "able", "about", "above", "absent", "absorb",
                        "abstract", "absurd", "abuse", "access", "accident",
                    ][idx]
                        .to_string()
                } else {
                    expected_word
                };
                !input.is_empty() && input.eq_ignore_ascii_case(&display_word)
            } else {
                false
            }
        })
    });

    // Update step checked status
    use_effect(move || {
        dispatch.call(SignupAction::SetStepChecked(STEP, is_all_correct()));
    });

    let mut handle_input_change = move |index: usize, value: String| {
        user_inputs.write().insert(index, value);
    };

    rsx! {
        div { class: "space-y-6",
            div { class: "text-center space-y-2",
                h1 { class: "text-2xl font-bold text-gray-900", "Confirm Phrase" }
                p { class: "text-gray-600 px-4",
                    "Enter the missing words to confirm you've saved your seed phrase correctly."
                }
            }

            div { class: "bg-white rounded-xl border border-gray-200 p-4 shadow-sm",
                div { class: "grid grid-cols-3 gap-3",
                    {
                        (0..12)
                            .map(|index| {
                                let is_input = random_indices.contains(&index);
                                let input_value = user_inputs
                                    .read()
                                    .get(&index)
                                    .unwrap_or(&String::new())
                                    .clone();
                                let word = context
                                    .state
                                    .read()
                                    .mnemonic
                                    .get(index)
                                    .unwrap_or(&String::new())
                                    .clone();
                                let display_word = if word.is_empty() {
                                    vec![
                                        "abandon",
                                        "ability",
                                        "able",
                                        "about",
                                        "above",
                                        "absent",
                                        "absorb",
                                        "abstract",
                                        "absurd",
                                        "abuse",
                                        "access",
                                        "accident",
                                    ][index]
                                        .to_string()
                                } else {
                                    word
                                };
                                let class = if is_input {
                                    "flex items-center space-x-2 rounded-lg px-3 py-2 border transition-colors duration-200 bg-white border-gray-300"
                                } else {
                                    "flex items-center space-x-2 rounded-lg px-3 py-2 border transition-colors duration-200 bg-gray-50 border-gray-200"
                                };
                                rsx! {
                                    div { key: "{index}", class,
                                        span { class: "text-sm font-medium text-gray-500 min-w-[20px]", "{index + 1}." }
                                        if is_input {
                                            input {
                                                r#type: "text",
                                                placeholder: "Enter word",
                                                value: input_value,
                                                oninput: move |event: Event<FormData>| handle_input_change(index, event.value().clone()),
                                                class: "flex-1 text-sm border-0 bg-transparent p-0 focus-visible:ring-0 text-gray-900",
                                            }
                                        } else {
                                            span { class: "text-sm font-medium text-gray-900", {display_word} }
                                        }
                                    }
                                }
                            })
                    }
                }
            }

            div { class: "text-center text-sm text-gray-600",
                p { "Complete all fields correctly to continue" }
            }
        }
    }
}
