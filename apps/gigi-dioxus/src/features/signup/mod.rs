use dioxus::prelude::*;

pub mod components;
pub mod context;
pub mod pages;

use components::{
    MnemonicConfirm, MnemonicDisplay, MnemonicInput, SignupInfoInput, StepNavigation, TermsOfUse,
};
use context::{use_signup_context, SignupProvider, SignupType};
use pages::{SignupFinish, Welcome};

#[component]
pub fn Signup() -> Element {
    rsx! {
        SignupProvider { SignupContent {} }
    }
}

#[component]
fn SignupContent() -> Element {
    let context = use_signup_context();
    let state = context.state.read();

    const FINISH_STEP: usize = 4;

    if state.signup_type == SignupType::None {
        rsx! {
            Welcome {}
        }
    } else if state.current_step == FINISH_STEP {
        rsx! {
            div { class: "min-h-screen flex items-center justify-center bg-gray-50 p-4",
                div { class: "w-full max-w-2xl", SignupFinish {} }
            }
        }
    } else {
        rsx! {
            div { class: "min-h-screen flex items-center justify-center bg-gray-50 p-4",
                div { class: "w-full max-w-2xl", Stepper {} }
            }
        }
    }
}

#[component]
fn Stepper() -> Element {
    let context = use_signup_context();
    let state = context.state.read();

    match state.current_step {
        0 => rsx! {
            TermsOfUse {}
            StepNavigation {}
        },
        1 => rsx! {
            match state.signup_type {
                SignupType::Create => rsx! {
                    MnemonicDisplay {}
                },
                SignupType::Import => rsx! {
                    MnemonicInput {}
                },
                SignupType::None => rsx! {
                    Welcome {}
                },
            }
            StepNavigation {}
        },
        2 => rsx! {
            MnemonicConfirm {}
            StepNavigation {}
        },
        3 => rsx! {
            SignupInfoInput {}
            StepNavigation {}
        },
        _ => rsx! {
            Welcome {}
        },
    }
}
