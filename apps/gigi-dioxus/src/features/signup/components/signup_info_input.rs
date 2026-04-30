use dioxus::prelude::*;

use crate::features::signup::context::{use_signup_context, SignupAction};

#[component]
pub fn SignupInfoInput() -> Element {
    const STEP: usize = 3;

    let context = use_signup_context();
    let dispatch = context.dispatch;

    // Track local state for inputs
    let mut name = use_signal(|| context.state.read().name.clone());
    let mut password = use_signal(|| context.state.read().password.clone());
    let mut confirm_password = use_signal(String::new);
    let mut create_group = use_signal(|| context.state.read().create_group);
    let mut group_name = use_signal(|| context.state.read().group_name.clone());

    // Calculate password strength
    let password_strength = use_memo(move || std::cmp::min(password().len() * 10, 100));

    // Check if passwords match
    let passwords_match = use_memo(move || password() == confirm_password());

    // Show warning if passwords don't match
    let show_warning = use_memo(move || !confirm_password().is_empty() && !passwords_match());

    // Check if all required fields are filled
    let all_fields_filled = use_memo(move || {
        !name().is_empty()
            && !password().is_empty()
            && !confirm_password().is_empty()
            && passwords_match()
            && (!create_group() || !group_name().is_empty())
    });

    // Update step checked status
    use_effect(move || {
        dispatch.call(SignupAction::SetStepChecked(STEP, all_fields_filled()));
    });

    let handle_name_change = move |event: Event<FormData>| {
        let value = event.value().clone();
        name.set(value.clone());
        dispatch.call(SignupAction::SetName(value));
    };

    let handle_password_change = move |event: Event<FormData>| {
        let value = event.value().clone();
        password.set(value.clone());
        dispatch.call(SignupAction::SetPassword(value));
    };

    let handle_confirm_password_change = move |event: Event<FormData>| {
        confirm_password.set(event.value().clone());
    };

    let handle_create_group_change = move |event: Event<FormData>| {
        let checked = event.value().parse::<bool>().unwrap_or(false);
        create_group.set(checked);
        dispatch.call(SignupAction::SetCreateGroup(checked));
        if !checked {
            group_name.set(String::new());
            dispatch.call(SignupAction::SetGroupName(String::new()));
        }
    };

    let handle_group_name_change = move |event: Event<FormData>| {
        let value = event.value().clone();
        group_name.set(value.clone());
        dispatch.call(SignupAction::SetGroupName(value));
    };

    rsx! {
        div { class: "space-y-6",
            div { class: "text-center space-y-2",
                h1 { class: "text-2xl font-bold text-gray-900", "Create Account" }
                p { class: "text-gray-600", "Set up your account password and preferences" }
            }

            div { class: "space-y-4",
                div { class: "space-y-2",
                    label { class: "text-sm font-medium text-gray-700", "Account Name" }
                    input {
                        r#type: "text",
                        placeholder: "Enter your account name",
                        value: name(),
                        oninput: handle_name_change,
                        class: "w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-blue-500 focus:border-blue-500 text-gray-900",
                    }
                }

                div { class: "space-y-2",
                    label { class: "text-sm font-medium text-gray-700", "Password" }
                    input {
                        r#type: "password",
                        placeholder: "Enter your password",
                        value: password(),
                        oninput: handle_password_change,
                        class: "w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-blue-500 focus:border-blue-500 text-gray-900",
                    }
                    div { class: "space-y-1",
                        div { class: "w-full bg-gray-200 rounded-full h-2",
                            div {
                                class: if password_strength() > 70 { "h-2 rounded-full transition-all duration-300 bg-green-500" } else if password_strength() > 40 { "h-2 rounded-full transition-all duration-300 bg-yellow-500" } else { "h-2 rounded-full transition-all duration-300 bg-red-500" },
                                style: format!("width: {}%", password_strength()),
                            }
                        }
                        p { class: "text-xs text-gray-600",
                            "Password strength: "
                            span { class: if password_strength() > 70 { "font-medium text-green-600" } else if password_strength() > 40 { "font-medium text-yellow-600" } else { "font-medium text-red-600" },
                                if password_strength() > 70 {
                                    " Strong"
                                } else if password_strength() > 40 {
                                    " Medium"
                                } else {
                                    " Weak"
                                }
                            }
                        }
                    }
                }

                div { class: "space-y-2",
                    label { class: "text-sm font-medium text-gray-700", "Confirm Password" }
                    input {
                        r#type: "password",
                        placeholder: "Confirm your password",
                        value: confirm_password(),
                        oninput: handle_confirm_password_change,
                        class: format!(
                            "w-full border {} rounded-md px-3 py-2 focus:ring-blue-500 focus:border-blue-500 text-gray-900",
                            if show_warning() { "border-red-300" } else { "border-gray-300" },
                        ),
                    }
                    if show_warning() {
                        p { class: "text-sm text-red-600 flex items-center gap-1",
                            "⚠️ Passwords do not match!"
                        }
                    }
                }

                div { class: "bg-gray-50 rounded-lg p-4 border border-gray-200",
                    div { class: "flex items-center space-x-3",
                        input {
                            r#type: "checkbox",
                            id: "createGroup",
                            checked: create_group(),
                            onchange: handle_create_group_change,
                            class: "w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 focus:ring-2",
                        }
                        label {
                            r#for: "createGroup",
                            class: "text-sm font-medium text-gray-700",
                            "Create the first chat group"
                        }
                    }

                    if create_group() {
                        div { class: "mt-4 space-y-2",
                            label { class: "text-sm font-medium text-gray-700", "Group Name" }
                            input {
                                r#type: "text",
                                placeholder: "Enter your group name",
                                value: group_name(),
                                oninput: handle_group_name_change,
                                class: "w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-blue-500 focus:border-blue-500 text-gray-900 bg-white",
                            }
                        }
                    }
                }
            }
        }
    }
}
