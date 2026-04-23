use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ConfirmationDialogProps {
    is_open: bool,
    title: String,
    message: String,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
}

#[component]
pub fn ConfirmationDialog(props: ConfirmationDialogProps) -> Element {
    if !props.is_open {
        return rsx! {
            {}
        };
    }

    rsx! {
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            onclick: move |_| props.on_cancel.call(()),
            div {
                class: "bg-white rounded-lg shadow-xl p-6 max-w-md w-full",
                onclick: move |e| e.stop_propagation(),
                h3 { class: "text-lg font-semibold text-gray-900 mb-2", "{props.title}" }
                p { class: "text-gray-600 mb-4", "{props.message}" }
                div { class: "flex justify-end gap-3",
                    button {
                        class: "px-4 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50 transition-colors",
                        onclick: move |_| props.on_cancel.call(()),
                        "Cancel"
                    }
                    button {
                        class: "px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors",
                        onclick: move |_| props.on_confirm.call(()),
                        "Confirm"
                    }
                }
            }
        }
    }
}
