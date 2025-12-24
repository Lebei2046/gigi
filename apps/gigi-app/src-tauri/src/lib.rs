use base64::Engine;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Serialize, Clone)]
pub struct P2pEventFrontend {
    #[serde(rename = "type")]
    event_type: String,
    data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct P2pConfig {
    nickname: String,
    port: u16,
    download_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageArgs {
    nickname: String,
    message: String,
}

#[derive(Debug, Deserialize)]
pub struct SendFileArgs {
    nickname: String,
    image_path: String,
}

#[derive(Debug, Deserialize)]
pub struct GroupMessageArgs {
    group: String,
    message: String,
}

#[derive(Debug, Deserialize)]
pub struct DownloadFileArgs {
    nickname: String,
    share_code: String,
}

type P2pClientWrapper = Arc<tokio::sync::Mutex<Option<gigi_p2p::P2pClient>>>;

#[tauri::command]
async fn initialize_p2p(app: AppHandle, config: P2pConfig) -> Result<(), String> {
    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let download_dir = PathBuf::from(&config.download_dir);

    let (client, mut event_receiver) =
        gigi_p2p::P2pClient::new(keypair, config.nickname.clone(), download_dir)
            .map_err(|e| e.to_string())?;

    let client_wrapper: P2pClientWrapper = Arc::new(tokio::sync::Mutex::new(Some(client)));
    app.manage(client_wrapper.clone());

    let event_sender = app.clone();

    tokio::spawn(async move {
        while let Some(event) = event_receiver.next().await {
            let frontend_event = match event {
                gigi_p2p::P2pEvent::PeerDiscovered {
                    peer_id,
                    nickname,
                    address,
                } => P2pEventFrontend {
                    event_type: "peer_discovered".to_string(),
                    data: serde_json::json!({
                        "peer_id": peer_id.to_string(),
                        "nickname": nickname,
                        "address": address.to_string()
                    }),
                },
                gigi_p2p::P2pEvent::PeerExpired { peer_id, nickname } => P2pEventFrontend {
                    event_type: "peer_expired".to_string(),
                    data: serde_json::json!({
                        "peer_id": peer_id.to_string(),
                        "nickname": nickname
                    }),
                },
                gigi_p2p::P2pEvent::DirectMessage {
                    from_nickname,
                    message,
                    ..
                } => P2pEventFrontend {
                    event_type: "direct_message".to_string(),
                    data: serde_json::json!({
                        "from_nickname": from_nickname,
                        "message": message
                    }),
                },
                gigi_p2p::P2pEvent::DirectImageMessage {
                    from_nickname,
                    filename,
                    data,
                    ..
                } => P2pEventFrontend {
                    event_type: "direct_image_message".to_string(),
                    data: serde_json::json!({
                        "from_nickname": from_nickname,
                        "filename": filename,
                        "data": base64::prelude::BASE64_STANDARD.encode(&data)
                    }),
                },
                gigi_p2p::P2pEvent::GroupMessage {
                    from_nickname,
                    group,
                    message,
                    ..
                } => P2pEventFrontend {
                    event_type: "group_message".to_string(),
                    data: serde_json::json!({
                        "from_nickname": from_nickname,
                        "group": group,
                        "message": message
                    }),
                },
                gigi_p2p::P2pEvent::FileDownloadCompleted { file_id, path, .. } => {
                    P2pEventFrontend {
                        event_type: "file_download_completed".to_string(),
                        data: serde_json::json!({
                            "file_id": file_id,
                            "path": path.to_string_lossy()
                        }),
                    }
                }
                gigi_p2p::P2pEvent::Error(error) => P2pEventFrontend {
                    event_type: "error".to_string(),
                    data: serde_json::json!({
                        "error": error.to_string()
                    }),
                },
                _ => continue,
            };

            let _ = event_sender.emit("p2p-event", frontend_event);
        }
    });

    // Start listening
    {
        let mut client_guard = client_wrapper.lock().await;
        if let Some(client) = client_guard.as_mut() {
            let addr = format!("/ip4/0.0.0.0/tcp/{}", config.port)
                .parse()
                .map_err(|e| format!("Invalid address: {}", e))?;
            client
                .start_listening(addr)
                .map_err(|e| format!("Failed to start listening: {}", e))?;
        }
    }

    Ok(())
}

#[tauri::command]
async fn send_direct_message(app: AppHandle, args: SendMessageArgs) -> Result<(), String> {
    let client_wrapper = app.state::<P2pClientWrapper>();
    let mut client_guard = client_wrapper.lock().await;
    match client_guard.as_mut() {
        Some(client) => client
            .send_direct_message(&args.nickname, args.message)
            .map_err(|e| e.to_string()),
        None => Err("P2P client not initialized".to_string()),
    }
}

#[tauri::command]
async fn send_direct_file(app: AppHandle, args: SendFileArgs) -> Result<(), String> {
    let client_wrapper = app.state::<P2pClientWrapper>();
    let mut client_guard = client_wrapper.lock().await;
    match client_guard.as_mut() {
        Some(client) => client
            .send_direct_file(&args.nickname, &PathBuf::from(args.image_path))
            .map_err(|e| e.to_string()),
        None => Err("P2P client not initialized".to_string()),
    }
}

#[tauri::command]
async fn join_group(app: AppHandle, group: String) -> Result<(), String> {
    let client_wrapper = app.state::<P2pClientWrapper>();
    let mut client_guard = client_wrapper.lock().await;
    match client_guard.as_mut() {
        Some(client) => client.join_group(&group).map_err(|e| e.to_string()),
        None => Err("P2P client not initialized".to_string()),
    }
}

#[tauri::command]
async fn send_group_message(app: AppHandle, args: GroupMessageArgs) -> Result<(), String> {
    let client_wrapper = app.state::<P2pClientWrapper>();
    let mut client_guard = client_wrapper.lock().await;
    match client_guard.as_mut() {
        Some(client) => client
            .send_group_message(&args.group, args.message)
            .map_err(|e| e.to_string()),
        None => Err("P2P client not initialized".to_string()),
    }
}

#[tauri::command]
async fn list_peers(app: AppHandle) -> Result<Vec<serde_json::Value>, String> {
    let client_wrapper = app.state::<P2pClientWrapper>();
    let client_guard = client_wrapper.lock().await;
    match client_guard.as_ref() {
        Some(client) => {
            let peers = client.list_peers();
            Ok(peers
                .into_iter()
                .map(|p| {
                    serde_json::json!({
                        "nickname": p.nickname,
                        "peer_id": p.peer_id.to_string()
                    })
                })
                .collect())
        }
        None => Err("P2P client not initialized".to_string()),
    }
}

#[tauri::command]
async fn share_file(app: AppHandle, file_path: String) -> Result<String, String> {
    let client_wrapper = app.state::<P2pClientWrapper>();
    let mut client_guard = client_wrapper.lock().await;
    match client_guard.as_mut() {
        Some(client) => client
            .share_file(&PathBuf::from(file_path))
            .await
            .map_err(|e| e.to_string()),
        None => Err("P2P client not initialized".to_string()),
    }
}

#[tauri::command]
async fn download_file(app: AppHandle, args: DownloadFileArgs) -> Result<(), String> {
    let client_wrapper = app.state::<P2pClientWrapper>();
    let mut client_guard = client_wrapper.lock().await;
    match client_guard.as_mut() {
        Some(client) => client
            .download_file(&args.nickname, &args.share_code)
            .map_err(|e| e.to_string()),
        None => Err("P2P client not initialized".to_string()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            initialize_p2p,
            send_direct_message,
            send_direct_file,
            join_group,
            send_group_message,
            list_peers,
            share_file,
            download_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
