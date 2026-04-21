use dioxus::prelude::*;
use qrcode::render::svg;
use qrcode::QrCode;

#[derive(Props, Clone, PartialEq)]
pub struct QRCodeTabProps {
    pub name: String,
    pub peer_id: String,
}

fn format_short_peer_id(peer_id: &str) -> String {
    if peer_id.len() > 12 {
        format!("{}...{}", &peer_id[..6], &peer_id[peer_id.len() - 4..])
    } else {
        peer_id.to_string()
    }
}

#[component]
pub fn QRCodeTab(props: QRCodeTabProps) -> Element {
    let qr_data = format!(
        "{{\"name\":\"{}\",\"peerId\":\"{}\"}}",
        props.name, props.peer_id
    );

    let svg_content = use_memo(move || {
        let code = QrCode::new(qr_data.as_bytes());
        match code {
            Ok(qr) => {
                let rendered = qr
                    .render::<svg::Color>()
                    .min_dimensions(200, 200)
                    .max_dimensions(200, 200)
                    .build();
                rendered
            }
            Err(_) => String::new(),
        }
    });

    rsx! {
        div { class: "p-6 bg-gray-50 h-full",
            div { class: "text-center space-y-6",
                div { class: "space-y-2",
                    h2 { class: "text-xl font-bold text-gray-900", "Share Your QR Code" }
                    p { class: "text-gray-600 text-sm px-4",
                        "Show this QR code to friends so they can add you as a contact"
                    }
                }

                div { class: "bg-white rounded-2xl shadow-lg border border-gray-100 p-6 space-y-4",
                    div { class: "flex justify-center",
                        div { class: "p-4 bg-white rounded-xl border-2 border-gray-200",
                            if !svg_content.is_empty() {
                                div {
                                    class: "w-[200px] h-[200px]",
                                    dangerous_inner_html: "{svg_content}",
                                }
                            } else {
                                div { class: "w-[200px] h-[200px] flex items-center justify-center bg-gray-100",
                                    span { class: "text-gray-500 text-sm", "Generating..." }
                                }
                            }
                        }
                    }

                    div { class: "text-center space-y-2",
                        div { class: "font-semibold text-gray-900", "{props.name}" }
                        div { class: "text-xs text-gray-500 font-mono bg-gray-100 px-3 py-2 rounded-lg inline-block max-w-[200px] truncate",
                            "{format_short_peer_id(&props.peer_id)}"
                        }
                    }
                }

                div { class: "space-y-3",
                    button {
                        class: "w-full py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-xl transition-colors duration-200 flex items-center justify-center gap-2",
                        onclick: move |_| {
                            tracing::info!("Scan QR Code clicked - scanner not implemented");
                        },
                        svg {
                            class: "w-5 h-5",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z",
                            }
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M15 13a3 3 0 11-6 0 3 3 0 016 0z",
                            }
                        }
                        "Scan Friend's QR Code"
                    }
                }
            }
        }
    }
}
