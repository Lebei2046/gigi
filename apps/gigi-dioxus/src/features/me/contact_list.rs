use dioxus::prelude::*;

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Contact {
    pub id: String,
    pub name: String,
}

fn group_and_sort_contacts(contacts: Vec<Contact>, search_term: String) -> Vec<(String, Vec<Contact>)> {
    let filtered: Vec<Contact> = contacts
        .into_iter()
        .filter(|contact| contact.name.to_lowercase().contains(&search_term.to_lowercase()))
        .collect();

    let mut grouped: std::collections::HashMap<char, Vec<Contact>> = std::collections::HashMap::new();
    for contact in filtered {
        if let Some(first_char) = contact.name.chars().next() {
            let key = first_char.to_ascii_uppercase();
            grouped.entry(key).or_default().push(contact);
        }
    }

    let mut entries: Vec<(String, Vec<Contact>)> = grouped
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

#[component]
pub fn ContactList() -> Element {
    let mut contacts = use_signal(|| Vec::<Contact>::new());
    let mut search_term = use_signal(|| String::new());

    let filtered_groups = use_memo(move || {
        group_and_sort_contacts(contacts(), search_term())
    });

    rsx! {
        div { class: "flex flex-col h-full bg-gray-50",
            div { class: "sticky top-0 z-10 bg-white border-b border-gray-200 px-4 py-3",
                div { class: "flex items-center bg-gray-100 rounded-xl px-4 py-3 focus-within:bg-white focus-within:ring-2 focus-within:ring-blue-500 focus-within:border-transparent transition-all",
                    svg {
                        class: "w-5 h-5 text-gray-400 mr-3",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                        }
                    }
                    input {
                        r#type: "text",
                        placeholder: "Search contacts...",
                        class: "flex-1 bg-transparent outline-none text-gray-900 placeholder-gray-500 text-sm",
                        value: "{search_term}",
                        oninput: move |e: Event<FormData>| {
                            search_term.set(e.value().to_string());
                        }
                    }
                }
            }

            div { class: "flex-1 overflow-y-auto",
                if filtered_groups().is_empty() {
                    div { class: "flex items-center justify-center h-full",
                        div { class: "text-center py-12 px-6",
                            div { class: "w-20 h-20 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4",
                                svg {
                                    class: "w-10 h-10 text-gray-400",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
                                    }
                                }
                            }
                            h3 { class: "text-lg font-semibold text-gray-900 mb-2", "No Contacts Found" }
                            p { class: "text-gray-600 text-sm",
                                if search_term().is_empty() {
                                    "You haven't added any contacts yet."
                                } else {
                                    "No contacts match your search."
                                }
                            }
                        }
                    }
                } else {
                    div { class: "px-4 py-2",
                        for (letter, group) in filtered_groups().iter() {
                            div { key: "{letter}", class: "mb-4",
                                div { class: "sticky top-14 z-10 bg-blue-600 text-white px-3 py-2 rounded-lg mb-2 text-sm font-semibold shadow-sm",
                                    "{letter}"
                                }
                                div { class: "bg-white rounded-xl shadow-sm overflow-hidden",
                                    for contact in group {
                                        div { key: "{contact.id}", class: "flex items-center px-4 py-3 border-b border-gray-100 last:border-b-0 hover:bg-gray-50 transition-colors cursor-pointer",
                                            div { class: "w-10 h-10 bg-gradient-to-br from-blue-500 to-purple-600 rounded-full flex items-center justify-center text-white font-semibold text-sm mr-3",
                                                "{contact.name.chars().next().unwrap_or('?').to_ascii_uppercase()}"
                                            }
                                            div { class: "flex-1 min-w-0",
                                                span { class: "text-gray-900 font-medium text-sm truncate block",
                                                    "{contact.name}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}