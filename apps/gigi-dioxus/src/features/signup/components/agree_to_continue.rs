use dioxus::prelude::*;

use crate::features::signup::context::{use_signup_context, SignupAction};

#[derive(Props, Clone, PartialEq)]
pub struct AgreeToContinueProps {
    id: String,
    label: String,
    step: usize,
    disabled: Option<bool>,
}

#[component]
pub fn AgreeToContinue(props: AgreeToContinueProps) -> Element {
    let context = use_signup_context();
    let dispatch = context.dispatch;
    let state = context.state.read();

    let disabled = props.disabled.unwrap_or(false);

    let handle_on_change = move |event: Event<FormData>| {
        let checked = event.value().parse::<bool>().unwrap_or(false);
        dispatch.call(SignupAction::SetStepChecked(props.step, checked));
    };

    rsx! {
        div { class: "flex items-start space-x-3 p-4 bg-gray-50 rounded-lg border border-gray-200",
            input {
                r#type: "checkbox",
                id: props.id.clone(),
                class: "mt-1 w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 focus:ring-2",
                checked: state.steps[props.step],
                disabled,
                onchange: handle_on_change,
            }
            label {
                r#for: props.id.clone(),
                class: "text-sm font-medium text-gray-700 leading-relaxed",
                {props.label}
            }
        }
    }
}
