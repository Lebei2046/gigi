use dioxus::prelude::*;

#[component]
pub fn GroupShareNotificationModal(
    is_open: Signal<bool>,
    group_name: Signal<String>,
    sender_name: Signal<String>,
    on_close: EventHandler<()>,
    on_join: EventHandler<()>,
    on_ignore: EventHandler<()>,
) -> Element {
    if !is_open() {
        return Ok(rsx! {
            {}
        }?);
    }

    rsx! {
        div {
            class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            onclick: move |_| on_close.call(()),
            div {
                class: "bg-white rounded-xl shadow-lg w-full max-w-md p-6",
                onclick: move |e| e.stop_propagation(),
                div { class: "text-center mb-6",
                    div { class: "w-16 h-16 bg-blue-100 rounded-full flex items-center justify-center mx-auto mb-4",
                        svg {
                            class: "w-8 h-8 text-blue-600",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z",
                            }
                        }
                    }
                    h3 { class: "text-lg font-semibold text-gray-900 mb-2", "Group Invitation" }
                    p { class: "text-sm text-gray-600",
                        "{sender_name} has invited you to join the group {group_name}"
                    }
                }
                div { class: "flex space-x-3",
                    button {
                        class: "flex-1 py-2 px-4 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 hover:bg-gray-50",
                        onclick: move |_| on_ignore.call(()),
                        "Ignore"
                    }
                    button {
                        class: "flex-1 py-2 px-4 bg-blue-600 border border-transparent rounded-lg text-sm font-medium text-white hover:bg-blue-700",
                        onclick: move |_| on_join.call(()),
                        "Join Group"
                    }
                }
            }
        }
    }
}
