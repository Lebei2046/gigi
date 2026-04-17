use dioxus::prelude::*;

use crate::features::signup::context::{use_signup_context, SignupAction};

#[component]
pub fn StepNavigation() -> Element {
    let context = use_signup_context();
    let state = context.state.read();
    let dispatch = context.dispatch;
    let save_account_info = context.save_account_info;

    const FINISH_STEP: usize = 4;

    let is_current_step_completed = state.steps[state.current_step];
    let is_on_last_step = state.current_step == FINISH_STEP - 1;

    let handle_next = move |_| {
        if is_current_step_completed {
            if is_on_last_step {
                // Save account info first, then navigate
                save_account_info.call(());
                // Wait a bit to ensure account info is saved before navigating
                spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    dispatch.call(SignupAction::GoToNextStep);
                });
            } else {
                dispatch.call(SignupAction::GoToNextStep);
            }
        }
    };

    let handle_back = move |_| {
        dispatch.call(SignupAction::GoToPrevStep);
    };

    rsx! {
        div { class: "flex justify-between items-center space-x-4 pt-6",
            button {
                class: "flex-1 py-3 rounded-xl font-medium border border-gray-300 text-gray-900 hover:bg-gray-50 transition-all duration-200",
                onclick: handle_back,
                "← Back"
            }
            button {
                class: format!(
                    "flex-1 py-3 rounded-xl font-medium transition-all duration-200 {}",
                    if is_current_step_completed {
                        "bg-blue-600 hover:bg-blue-700 text-white"
                    } else {
                        "bg-gray-300 cursor-not-allowed"
                    },
                ),
                onclick: handle_next,
                disabled: !is_current_step_completed,
                if is_on_last_step {
                    "Complete ✓"
                } else {
                    "Next →"
                }
            }
        }
    }
}
