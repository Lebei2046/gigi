use dioxus::prelude::*;

use crate::features::chat::chat_state::{Message, MessageType};

// Message Bubble Component
#[component]
pub fn MessageBubble(
    message: Message,
    on_delete: EventHandler<String>,
    on_download_request: Option<EventHandler<(String, String, String)>>,
) -> Element {
    match message.message_type {
        MessageType::Text => rsx! {
            TextMessageBubble { message, on_delete }
        },
        MessageType::Image => rsx! {
            ImageMessageBubble { message, on_delete, on_download_request }
        },
        MessageType::File => rsx! {
            FileMessageBubble { message, on_delete, on_download_request }
        },
    }
}

// Text Message Bubble
#[component]
fn TextMessageBubble(message: Message, on_delete: EventHandler<String>) -> Element {
    rsx! {
        div { class: if message.is_own { "flex justify-end" } else { "flex" },
            div { class: if message.is_own { "bg-blue-100 rounded-lg rounded-tr-none p-3 max-w-[80%] relative" } else { "bg-white rounded-lg rounded-tl-none p-3 max-w-[80%] border border-gray-200 relative" },
                if message.is_own {
                    div { class: "absolute top-1 right-1",
                        button {
                            class: "text-xs text-gray-500 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-opacity",
                            onclick: move |_| on_delete.call(message.id.clone()),
                            title: "Delete message",
                            svg {
                                class: "w-4 h-4",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16",
                                }
                            }
                        }
                    }
                }
                if !message.is_own {
                    div { class: "text-xs font-medium text-gray-500 mb-1", "{message.sender}" }
                }
                div { class: "text-sm text-gray-900", "{message.content}" }
                div { class: if message.is_own { "text-xs text-blue-600 mt-1 text-right" } else { "text-xs text-gray-500 mt-1" },
                    "{message.timestamp}"
                }
            }
        }
    }
}

// Image Message Bubble
#[component]
fn ImageMessageBubble(
    message: Message,
    on_delete: EventHandler<String>,
    on_download_request: Option<EventHandler<(String, String, String)>>,
) -> Element {
    println!(
        "ImageMessageBubble called for message: {:?}, message_type: {:?}",
        message.id, message.message_type
    );
    let is_downloading = message.is_downloading;
    let download_progress = message.download_progress;
    let file_path = message.file_path.clone();
    let filename = message.filename.clone();
    let file_size = message.file_size;
    let message_content = message.content.clone();
    let message_sender = message.sender.clone();
    let message_timestamp = message.timestamp.clone();
    let is_own = message.is_own;
    let message_id = message.id.clone();
    let share_code = message.share_code.clone();

    let message_id_for_click = message_id.clone();
    let share_code_for_click = share_code.clone();
    let filename_for_click = filename.clone();

    let handle_download_click = move |_| {
        if let (Some(on_download), Some(code), Some(name)) = (
            on_download_request,
            share_code_for_click.clone(),
            filename_for_click.clone(),
        ) {
            on_download.call((message_id_for_click.clone(), code, name));
        }
    };

    let format_file_size = |size: Option<u64>| match size {
        Some(bytes) => {
            if bytes < 1024 {
                format!("{} Bytes", bytes)
            } else if bytes < 1024 * 1024 {
                format!("{:.2} KB", bytes as f64 / 1024.0)
            } else {
                format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
            }
        }
        None => "Unknown size".to_string(),
    };

    println!(
        "MessageBubble rendered for message: {:?}, is_own={}, message_type={:?}, file_path={:?}",
        message.id, is_own, message.message_type, file_path
    );
    let has_local_file = file_path.is_some();
    let is_downloadable = !is_own && share_code.is_some() && !is_downloading && !has_local_file;
    let file_exists = file_path
        .as_ref()
        .map(|p| {
            let exists = std::path::Path::new(p).exists();
            println!("File path {} exists: {}", p, exists);
            exists
        })
        .unwrap_or(false);
    // For sender, if thumbnail doesn't exist, use the original file path
    let effective_file_path =
        if is_own && !file_exists && message.message_type == MessageType::Image {
            println!("Sender image message, file_exists={}", file_exists);
            // Try to use the original file path if available
            // First check if the current file_path exists (it might be the original path)
            if let Some(path) = &file_path {
                let path_exists = std::path::Path::new(path).exists();
                println!("Current file path {} exists: {}", path, path_exists);
                if path_exists {
                    println!("Using current file path");
                    file_path.clone()
                } else {
                    // If not, try to find the file in common locations
                    let filename = message.content.clone();
                    let home_dir = dirs::home_dir();
                    let common_dirs = ["图片", "Pictures", "Downloads", "桌面", "Desktop"];

                    let mut found_path = None;
                    if let Some(home) = home_dir {
                        for dir in common_dirs {
                            let test_path = home.join(dir).join(filename.clone());
                            println!("Checking common location: {:?}", test_path);
                            if test_path.exists() {
                                println!("Found file at common location: {:?}", test_path);
                                found_path = Some(test_path.to_string_lossy().to_string());
                                break;
                            }
                        }
                    }

                    if found_path.is_some() {
                        println!("Using found path: {:?}", found_path);
                        found_path
                    } else {
                        println!("No original file found, using thumbnail path");
                        file_path.clone()
                    }
                }
            } else {
                file_path.clone()
            }
        } else {
            file_path.clone()
        };
    let effective_file_exists = effective_file_path
        .as_ref()
        .map(|p| {
            let exists = std::path::Path::new(p).exists();
            println!("Effective file path {} exists: {}", p, exists);
            exists
        })
        .unwrap_or(false);
    println!(
        "Effective file path: {:?}, effective_file_exists: {}",
        effective_file_path, effective_file_exists
    );
    // crate::services::logging::debug(format!("MessageBubble: is_own={}, message_type={:?}, file_path={:?}, effective_file_path={:?}, file_exists={}, effective_file_exists={}, is_downloading={}, is_downloadable={}",
    //     is_own, message.message_type, file_path, effective_file_path, file_exists, effective_file_exists, is_downloading, is_downloadable));

    rsx! {
        div { class: if is_own { "flex justify-end" } else { "flex" },
            div { class: if is_own { "bg-blue-100 rounded-lg rounded-tr-none p-3 max-w-[80%] relative" } else { "bg-white rounded-lg rounded-tl-none p-3 max-w-[80%] border border-gray-200 relative" },
                if is_own {
                    div { class: "absolute top-1 right-1",
                        button {
                            class: "text-xs text-gray-500 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-opacity",
                            onclick: move |_| on_delete.call(message_id.clone()),
                            title: "Delete message",
                            svg {
                                class: "w-4 h-4",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16",
                                }
                            }
                        }
                    }
                }
                if !is_own {
                    div { class: "text-xs font-medium text-gray-500 mb-1", "{message_sender}" }
                }
                if is_downloading {
                    div { class: "bg-gray-100 rounded p-3 mb-2",
                        div { class: "w-64 h-64 bg-gray-200 rounded flex flex-col items-center justify-center",
                            div { class: "text-gray-500 mb-2", "Downloading..." }
                            div { class: "w-48 h-2 bg-gray-300 rounded-full",
                                div {
                                    class: "bg-blue-500 h-2 rounded-full transition-all duration-300",
                                    style: format!("width: {}%", download_progress.unwrap_or(0)),
                                }
                            }
                            div { class: "text-xs text-gray-500 mt-2",
                                "{download_progress.unwrap_or(0)}%"
                            }
                        }
                    }
                } else if let Some(path) = &effective_file_path {
                    if effective_file_exists {
                        div { class: "bg-gray-100 rounded p-2 mb-2",
                            img {
                                class: "max-w-64 max-h-64 rounded object-contain",
                                src: format!("{}", effective_file_path.as_ref().unwrap()),
                                alt: format!("{}", filename.as_ref().unwrap_or(&message_content)),
                            }
                        }
                    } else if is_downloadable {
                        div {
                            class: "bg-gray-100 rounded p-3 mb-2 flex items-center gap-3 cursor-pointer hover:bg-gray-200 transition-colors",
                            onclick: handle_download_click,
                            div { class: "w-16 h-16 bg-gray-200 rounded flex items-center justify-center",
                                svg {
                                    class: "w-8 h-8 text-gray-500",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z",
                                    }
                                }
                            }
                            div { class: "flex-1 min-w-0",
                                div { class: "text-sm font-medium text-gray-900 truncate",
                                    "{filename.as_ref().unwrap_or(&message_content)}"
                                }
                                div { class: "text-xs text-gray-500", "{format_file_size(file_size)}" }
                                div { class: "text-xs text-blue-600 mt-1", "Tap to download" }
                            }
                        }
                    } else {
                        div { class: "bg-gray-100 rounded p-2 mb-2",
                            div { class: "w-48 h-48 bg-gray-200 rounded flex items-center justify-center",
                                span { class: "text-gray-500", "Image" }
                            }
                        }
                    }
                } else if is_downloadable {
                    div {
                        class: "bg-gray-100 rounded p-3 mb-2 flex items-center gap-3 cursor-pointer hover:bg-gray-200 transition-colors",
                        onclick: handle_download_click,
                        div { class: "w-16 h-16 bg-gray-200 rounded flex items-center justify-center",
                            svg {
                                class: "w-8 h-8 text-gray-500",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z",
                                }
                            }
                        }
                        div { class: "flex-1 min-w-0",
                            div { class: "text-sm font-medium text-gray-900 truncate",
                                "{filename.as_ref().unwrap_or(&message_content)}"
                            }
                            div { class: "text-xs text-gray-500", "{format_file_size(file_size)}" }
                            div { class: "text-xs text-blue-600 mt-1", "Tap to download" }
                        }
                    }
                } else {
                    div { class: "bg-gray-100 rounded p-2 mb-2",
                        div { class: "w-48 h-48 bg-gray-200 rounded flex items-center justify-center",
                            span { class: "text-gray-500", "Image" }
                        }
                    }
                }
                div { class: if is_own { "text-xs text-blue-600 mt-1 text-right" } else { "text-xs text-gray-500 mt-1" },
                    "{message_timestamp}"
                }
            }
        }
    }
}

// File Message Bubble
#[component]
fn FileMessageBubble(
    message: Message,
    on_delete: EventHandler<String>,
    on_download_request: Option<EventHandler<(String, String, String)>>,
) -> Element {
    let message_id = message.id.clone();
    let share_code = message.share_code.clone();
    let filename = message.filename.clone();
    let file_type = message.file_type.clone();
    let is_own = message.is_own;
    let is_downloading = message.is_downloading;
    let download_progress = message.download_progress;
    let file_path = message.file_path.clone();
    let message_sender = message.sender.clone();
    let message_content = message.content.clone();
    let message_file_size = message.file_size;
    let message_timestamp = message.timestamp.clone();

    let filename_for_icon = filename.clone();
    let filename_for_click = filename.clone();
    let share_code_for_click = share_code.clone();
    let message_id_for_click = message_id.clone();
    let message_id_for_delete = message_id.clone();
    let file_path_for_display = file_path.clone();

    let is_image = file_type
        .as_ref()
        .map(|ft| {
            let ext = ft.to_lowercase();
            ext.starts_with("image/")
                || ["png", "jpg", "jpeg", "gif", "bmp", "webp"].contains(&ext.as_str())
        })
        .unwrap_or(false);

    let has_local_file = file_path.is_some();
    let is_downloaded = !is_own && has_local_file;
    let is_downloadable = !is_own && share_code.is_some() && !is_downloading && !is_downloaded;
    let file_exists = file_path
        .as_ref()
        .map(|p| std::path::Path::new(p).exists())
        .unwrap_or(false);

    let handle_download_click = move |_| {
        if let (Some(on_download), Some(code), Some(name)) = (
            on_download_request,
            share_code_for_click.clone(),
            filename_for_click.clone(),
        ) {
            on_download.call((message_id_for_click.clone(), code, name));
        }
    };

    let get_file_icon = move || {
        let ext = filename_for_icon
            .as_ref()
            .and_then(|f| f.split('.').last())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        if ["png", "jpg", "jpeg", "gif", "bmp", "webp"].contains(&ext.as_str()) {
            rsx! {
                svg {
                    class: "w-5 h-5 text-gray-500",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z",
                    }
                }
            }
        } else if ["mp4", "avi", "mov", "mkv", "webm"].contains(&ext.as_str()) {
            rsx! {
                svg {
                    class: "w-5 h-5 text-gray-500",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z",
                    }
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
                    }
                }
            }
        } else if ["mp3", "wav", "flac", "aac", "ogg"].contains(&ext.as_str()) {
            rsx! {
                svg {
                    class: "w-5 h-5 text-gray-500",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z",
                    }
                }
            }
        } else if ["pdf", "doc", "docx", "txt", "rtf"].contains(&ext.as_str()) {
            rsx! {
                svg {
                    class: "w-5 h-5 text-gray-500",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z",
                    }
                }
            }
        } else if ["zip", "rar", "7z", "tar", "gz"].contains(&ext.as_str()) {
            rsx! {
                svg {
                    class: "w-5 h-5 text-gray-500",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4",
                    }
                }
            }
        } else if [
            "toml", "json", "js", "ts", "jsx", "tsx", "css", "html", "xml", "yaml", "yml",
        ]
        .contains(&ext.as_str())
        {
            rsx! {
                svg {
                    class: "w-5 h-5 text-gray-500",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z",
                    }
                }
            }
        } else {
            rsx! {
                svg {
                    class: "w-5 h-5 text-gray-500",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4",
                    }
                }
            }
        }
    };

    let format_file_size = |size: Option<u64>| match size {
        Some(bytes) => {
            if bytes < 1024 {
                format!("{} Bytes", bytes)
            } else if bytes < 1024 * 1024 {
                format!("{:.2} KB", bytes as f64 / 1024.0)
            } else {
                format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
            }
        }
        None => "Unknown size".to_string(),
    };

    rsx! {
        div { class: if is_own { "flex justify-end" } else { "flex" },
            div { class: if is_own { "bg-blue-100 rounded-lg rounded-tr-none p-3 max-w-[80%] relative" } else { "bg-white rounded-lg rounded-tl-none p-3 max-w-[80%] border border-gray-200 relative" },
                if is_own {
                    div { class: "absolute top-1 right-1",
                        button {
                            class: "text-xs text-gray-500 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-opacity",
                            onclick: move |_| on_delete.call(message_id_for_delete.clone()),
                            title: "Delete message",
                            svg {
                                class: "w-4 h-4",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16",
                                }
                            }
                        }
                    }
                }
                if !is_own {
                    div { class: "text-sm text-gray-900 mb-2", "{message_sender}:" }
                }

                // Image with thumbnail display or download progress
                if is_image {
                    if is_downloading {
                        div { class: "bg-gray-100 rounded p-3 mb-2",
                            div { class: "w-64 h-64 bg-gray-200 rounded flex flex-col items-center justify-center",
                                div { class: "text-gray-500 mb-2", "Downloading..." }
                                div { class: "w-48 h-2 bg-gray-300 rounded-full",
                                    div {
                                        class: "bg-blue-500 h-2 rounded-full transition-all duration-300",
                                        style: format!("width: {}%", download_progress.unwrap_or(0)),
                                    }
                                }
                                div { class: "text-xs text-gray-500 mt-2",
                                    "{download_progress.unwrap_or(0)}%"
                                }
                            }
                        }
                    } else if has_local_file {
                        if file_exists {
                            div { class: "bg-gray-100 rounded p-2 mb-2",
                                img {
                                    class: "max-w-64 max-h-64 rounded object-contain",
                                    src: format!("file://{}", file_path_for_display.as_ref().unwrap()),
                                    alt: format!("{}", filename.as_ref().unwrap_or(&message_content)),
                                }
                            }
                        } else if is_downloadable {
                            div {
                                class: "bg-gray-100 rounded p-3 mb-2 flex items-center gap-3 cursor-pointer hover:bg-gray-200 transition-colors",
                                onclick: handle_download_click,
                                div { class: "w-16 h-16 bg-gray-200 rounded flex items-center justify-center",
                                    {get_file_icon()}
                                }
                                div { class: "flex-1 min-w-0",
                                    div { class: "text-sm font-medium text-gray-900 truncate",
                                        "{filename.as_ref().unwrap_or(&message_content)}"
                                    }
                                    div { class: "text-xs text-gray-500",
                                        "{format_file_size(message_file_size)}"
                                    }
                                    div { class: "text-xs text-blue-600 mt-1", "Tap to download" }
                                }
                            }
                        } else {
                            div { class: "bg-gray-100 rounded p-3 mb-2 flex items-center gap-3",
                                div { class: "w-10 h-10 bg-gray-200 rounded flex items-center justify-center",
                                    {get_file_icon()}
                                }
                                div { class: "flex-1 min-w-0",
                                    div { class: "text-sm font-medium text-gray-900 truncate",
                                        "{filename.as_ref().unwrap_or(&message_content)}"
                                    }
                                    div { class: "text-xs text-gray-500",
                                        "{format_file_size(message_file_size)}"
                                    }
                                }
                            }
                        }
                    } else if is_downloadable {
                        div {
                            class: "bg-gray-100 rounded p-3 mb-2 flex items-center gap-3 cursor-pointer hover:bg-gray-200 transition-colors",
                            onclick: handle_download_click,
                            div { class: "w-16 h-16 bg-gray-200 rounded flex items-center justify-center",
                                {get_file_icon()}
                            }
                            div { class: "flex-1 min-w-0",
                                div { class: "text-sm font-medium text-gray-900 truncate",
                                    "{filename.as_ref().unwrap_or(&message_content)}"
                                }
                                div { class: "text-xs text-gray-500",
                                    "{format_file_size(message_file_size)}"
                                }
                                div { class: "text-xs text-blue-600 mt-1", "Tap to download" }
                            }
                        }
                    } else {
                        div { class: "bg-gray-100 rounded p-3 mb-2 flex items-center gap-3",
                            div { class: "w-10 h-10 bg-gray-200 rounded flex items-center justify-center",
                                {get_file_icon()}
                            }
                            div { class: "flex-1 min-w-0",
                                div { class: "text-sm font-medium text-gray-900 truncate",
                                    "{filename.as_ref().unwrap_or(&message_content)}"
                                }
                                div { class: "text-xs text-gray-500",
                                    "{format_file_size(message_file_size)}"
                                }
                            }
                        }
                    }
                } else {
                    // Non-image file attachment display
                    div { class: "bg-gray-100 rounded p-3 mb-2 flex items-center gap-3",
                        div { class: "w-10 h-10 bg-gray-200 rounded flex items-center justify-center",
                            {get_file_icon()}
                        }
                        div { class: "flex-1 min-w-0",
                            div { class: "text-sm font-medium text-gray-900 truncate",
                                "{filename.as_ref().unwrap_or(&message_content)}"
                            }
                            div { class: "text-xs text-gray-500",
                                "{format_file_size(message_file_size)}"
                            }
                            if is_downloading {
                                div { class: "mt-2",
                                    div { class: "w-full bg-gray-200 rounded-full h-2",
                                        div {
                                            class: "bg-blue-500 h-2 rounded-full transition-all duration-300",
                                            style: format!("width: {}%", download_progress.unwrap_or(0)),
                                        }
                                    }
                                    div { class: "text-xs text-blue-600 mt-1",
                                        "Downloading... {download_progress.unwrap_or(0)}%"
                                    }
                                }
                            }
                            if is_downloaded {
                                div { class: "mt-2",
                                    div { class: "flex items-center gap-1 text-green-600",
                                        svg {
                                            class: "w-4 h-4",
                                            fill: "none",
                                            stroke: "currentColor",
                                            view_box: "0 0 24 24",
                                            path {
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                stroke_width: "2",
                                                d: "M5 13l4 4L19 7",
                                            }
                                        }
                                        span { class: "text-xs", "Downloaded" }
                                    }
                                }
                            }
                        }
                        if is_downloadable {
                            button {
                                class: "p-2 text-blue-600 hover:bg-blue-50 rounded-lg transition-colors",
                                onclick: handle_download_click,
                                title: "Download file",
                                svg {
                                    class: "w-5 h-5",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z",
                                    }
                                }
                            }
                        }
                    }
                }
                if message_content != filename.as_deref().unwrap_or("").to_string() {
                    div { class: "text-sm text-gray-600 mt-1", "{message_content}" }
                }
                div { class: if message.is_own { "text-xs text-blue-600 mt-1 text-right" } else { "text-xs text-gray-500 mt-1" },
                    "{message.timestamp}"
                }
            }
        }
    }
}
