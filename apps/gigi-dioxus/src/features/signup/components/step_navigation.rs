use dioxus::prelude::*;

use crate::features::signup::context::{use_signup_context, SignupAction};

#[component]
pub fn StepNavigation() -> Element {
    let context = use_signup_context();
    let state = context.state.read();
    let dispatch = context.dispatch;

    // Check if current step is completed
    let is_current_step_completed = state.steps[state.current_step];

    // Handle next button click
    let handle_next = move |_| {
        if is_current_step_completed {
            dispatch.call(SignupAction::GoToNextStep);
        }
    };

    // Handle back button click
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
                "Next →"
            }
        }
    }
}
