use crate::services::event_bus::{AppEvent, EventBus};
use anyhow::Result;
use chrono;
use dirs;
use futures_util::stream::StreamExt;
use gigi_logging::tracing;
use gigi_p2p::{Keypair, P2pClient, P2pConfig, P2pEvent, PeerInfo};
use hex;
use image::{imageops, ImageReader};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileShareInfo {
    pub share_code: String,
    pub filename: String,
    pub file_size: u64,
    pub file_type: String,
}

static P2P_CLIENT: Lazy<Arc<Mutex<Option<P2pClient>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));
static LOCAL_NICKNAME: Lazy<Arc<Mutex<Option<String>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

#[derive(Debug)]
enum DbOperation {
    StoreDirectMessage {
        from_nickname: String,
        to_nickname: String,
        message: String,
        is_own: bool,
        from_peer_id: String,
    },
    StoreFileShareMessage {
        from_nickname: String,
        to_nickname: String,
        filename: String,
        share_code: String,
        file_size: u64,
        file_type: String,
        is_own: bool,
        from_peer_id: String,
    },
    StoreGroupShareMessage {
        from_nickname: String,
        to_nickname: String,
        group_id: String,
        group_name: String,
        is_own: bool,
        from_peer_id: String,
    },
    StoreGroupMessage {
        from_nickname: String,
        group_name: String,
        message: String,
        is_own: bool,
    },
    StoreGroupFileShareMessage {
        from_nickname: String,
        group_name: String,
        filename: String,
        share_code: String,
        file_size: u64,
        file_type: String,
        is_own: bool,
    },
    UpsertConversation {
        conv_id: String,
        name: String,
        is_group: bool,
        peer_id: String,
        last_message: Option<String>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    IncrementUnread {
        conv_id: String,
    },
}

type DbSender = mpsc::Sender<DbOperation>;
static DB_WORKER_TX: Lazy<Arc<Mutex<Option<DbSender>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None::<DbSender>)));

async fn db_worker(mut rx: mpsc::Receiver<DbOperation>) {
    while let Some(op) = rx.recv().await {
        match op {
            DbOperation::StoreDirectMessage {
                from_nickname,
                to_nickname,
                message,
                is_own,
                from_peer_id,
            } => {
                if crate::services::persistence_service::PersistenceService::store_direct_message(
                    from_nickname.clone(),
                    to_nickname.clone(),
                    message.clone(),
                    is_own,
                )
                .await
                .is_ok()
                {
                    let (conv_id, conv_name, conv_peer_id) = if is_own {
                        // For own messages, the conversation is with the recipient
                        (from_peer_id.clone(), to_nickname, from_peer_id.clone())
                    } else {
                        // For received messages, the conversation is with the sender
                        (from_peer_id.clone(), from_nickname, from_peer_id.clone())
                    };

                    let _ = crate::services::persistence_service::PersistenceService::upsert_conversation(
                        conv_id,
                        conv_name,
                        false,
                        conv_peer_id,
                        Some(message),
                        Some(chrono::Utc::now()),
                    )
                    .await;
                    // Don't increment unread for own messages
                    if !is_own {
                        let _ = crate::services::persistence_service::PersistenceService::increment_unread(&from_peer_id).await;
                    }
                    let _ = EventBus::send(AppEvent::MessageSaved(from_peer_id));
                }
            }
            DbOperation::StoreFileShareMessage {
                from_nickname,
                to_nickname,
                filename,
                share_code,
                file_size,
                file_type,
                is_own,
                from_peer_id,
            } => {
                let from_nickname_clone = from_nickname.clone();
                let from_peer_id_clone = from_peer_id.clone();
                if crate::services::persistence_service::PersistenceService::store_file_share_message(
                    from_nickname.clone(),
                    to_nickname,
                    filename.clone(),
                    share_code.clone(),
                    file_size,
                    file_type.clone(),
                    is_own,
                )
                .await
                .is_ok()
                {
                    let _ = crate::services::persistence_service::PersistenceService::upsert_conversation(
                        from_peer_id.clone(),
                        from_nickname.clone(),
                        false,
                        from_peer_id.clone(),
                        Some(filename.clone()),
                        Some(chrono::Utc::now()),
                    )
                    .await;
                    let _ = crate::services::persistence_service::PersistenceService::increment_unread(&from_peer_id)
                        .await;
                    let _ = EventBus::send(AppEvent::FileShareReceived {
                        from_peer_id: from_peer_id_clone,
                        from_nickname: from_nickname_clone,
                        share_code,
                        filename,
                        file_size,
                        file_type,
                        conv_id: from_peer_id,
                    });
                }
            }
            DbOperation::StoreGroupShareMessage {
                from_nickname,
                to_nickname,
                group_id,
                group_name,
                is_own,
                from_peer_id,
            } => {
                if crate::services::persistence_service::PersistenceService::store_group_share_message(
                    from_nickname.clone(),
                    to_nickname,
                    group_id.clone(),
                    group_name.clone(),
                    is_own,
                )
                .await
                .is_ok()
                {
                    // Create conversation with the inviter (for the share message)
                    let conv_id = from_peer_id.clone();
                    let _ = crate::services::persistence_service::PersistenceService::upsert_conversation(
                        conv_id.clone(),
                        from_nickname,
                        false,
                        conv_id.clone(),
                        Some(format!("Join group: {}", group_name)),
                        Some(chrono::Utc::now()),
                    )
                    .await;
                    let _ = crate::services::persistence_service::PersistenceService::increment_unread(&conv_id)
                        .await;

                    // Also create a group conversation entry so the group appears in the chat list
                    let group_conv_id = format!("group-{}", group_name);
                    let _ = crate::services::persistence_service::PersistenceService::upsert_conversation(
                        group_conv_id.clone(),
                        group_name.clone(),
                        true,
                        group_id.clone(),
                        Some(format!("Join group: {}", group_name)),
                        Some(chrono::Utc::now()),
                    )
                    .await;

                    // Add the group to auth service so it persists across restarts
                    let _ = tokio::spawn(async move {
                        if let Ok(mut auth_service) = crate::services::auth_service::AuthService::new().await {
                            let _ = auth_service.upsert_group(&group_id, &group_name, false).await;
                        }
                    });

                    let _ = EventBus::send(AppEvent::MessageSaved(conv_id));
                }
            }
            DbOperation::StoreGroupMessage {
                from_nickname,
                group_name,
                message,
                is_own,
            } => {
                if crate::services::persistence_service::PersistenceService::store_group_message(
                    from_nickname.clone(),
                    group_name.clone(),
                    message.clone(),
                    is_own,
                )
                .await
                .is_ok()
                {
                    let conv_id = format!("group-{}", group_name);
                    let _ = crate::services::persistence_service::PersistenceService::upsert_conversation(
                        conv_id.clone(),
                        group_name.clone(),
                        true,
                        group_name,
                        Some(message),
                        Some(chrono::Utc::now()),
                    )
                    .await;
                    let _ =
                        crate::services::persistence_service::PersistenceService::increment_unread(
                            &conv_id,
                        )
                        .await;
                    let _ = EventBus::send(AppEvent::MessageSaved(conv_id));
                }
            }
            DbOperation::StoreGroupFileShareMessage {
                from_nickname,
                group_name,
                filename,
                share_code,
                file_size,
                file_type,
                is_own,
            } => {
                let group_name_clone = group_name.clone();
                if crate::services::persistence_service::PersistenceService::store_group_file_share_message(
                    from_nickname.clone(),
                    group_name.clone(),
                    filename.clone(),
                    share_code.clone(),
                    file_size,
                    file_type.clone(),
                    is_own,
                )
                .await
                .is_ok()
                {
                    let conv_id = format!("group-{}", group_name);
                    let _ = crate::services::persistence_service::PersistenceService::upsert_conversation(
                        conv_id.clone(),
                        group_name.clone(),
                        true,
                        group_name.clone(),
                        Some(filename.clone()),
                        Some(chrono::Utc::now()),
                    )
                    .await;
                    let _ = crate::services::persistence_service::PersistenceService::increment_unread(&conv_id)
                        .await;
                    let _ = EventBus::send(AppEvent::GroupFileShareReceived {
                        from_peer_id: String::new(),
                        from_nickname,
                        share_code,
                        filename,
                        file_size,
                        file_type,
                        group_name: group_name_clone,
                    });
                }
            }
            DbOperation::UpsertConversation {
                conv_id,
                name,
                is_group,
                peer_id,
                last_message,
                timestamp,
            } => {
                let _ =
                    crate::services::persistence_service::PersistenceService::upsert_conversation(
                        conv_id,
                        name,
                        is_group,
                        peer_id,
                        last_message,
                        Some(timestamp),
                    )
                    .await;
            }
            DbOperation::IncrementUnread { conv_id } => {
                let _ = crate::services::persistence_service::PersistenceService::increment_unread(
                    &conv_id,
                )
                .await;
            }
        }
    }
}

async fn send_db_operation(op: DbOperation) {
    if let Some(tx) = DB_WORKER_TX.lock().await.as_ref() {
        let _ = tx.send(op).await;
    }
}

pub struct P2pService;

impl P2pService {
    pub async fn store_own_direct_message(
        from_nickname: String,
        to_nickname: String,
        message: String,
        from_peer_id: String,
    ) {
        send_db_operation(DbOperation::StoreDirectMessage {
            from_nickname,
            to_nickname,
            message,
            is_own: true,
            from_peer_id,
        })
        .await;
    }

    pub async fn store_own_group_message(
        from_nickname: String,
        group_name: String,
        message: String,
    ) {
        send_db_operation(DbOperation::StoreGroupMessage {
            from_nickname,
            group_name,
            message,
            is_own: true,
        })
        .await;
    }
    pub async fn initialize(private_key: &str, nickname: &str) -> Result<()> {
        // Create keypair from private key
        let keypair = Keypair::ed25519_from_bytes(hex::decode(private_key)?)?;

        // Create output directory for downloads
        let data_dir = env::var("GIGI_DATA_DIR").unwrap_or_else(|_| {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("gigi-dioxus")
                .to_string_lossy()
                .to_string()
        });

        // Expand ~ to home directory
        let data_dir_expanded = if data_dir.starts_with('~') {
            if let Some(home) = dirs::home_dir() {
                home.join(data_dir.strip_prefix('~').unwrap_or(""))
            } else {
                PathBuf::from(data_dir)
            }
        } else {
            PathBuf::from(data_dir)
        };

        let output_dir = data_dir_expanded.join("downloads");
        let uploads_dir = data_dir_expanded.join("uploads");

        // Create downloads directory
        std::fs::create_dir_all(&output_dir)?;

        // Create uploads directory
        std::fs::create_dir_all(&uploads_dir)?;

        // Create P2P config without bootstrap nodes (use mDNS for local discovery)
        let p2p_config = P2pConfig {
            bootstrap_nodes: vec![],
            ..Default::default()
        };

        // Create P2P client
        let (mut client, mut event_receiver) =
            P2pClient::new_with_config(keypair, nickname.to_string(), output_dir, p2p_config)?;

        // Start listening
        client.start_listening("/ip4/0.0.0.0/tcp/0".parse()?)?;

        // Store client and local nickname
        *P2P_CLIENT.lock().await = Some(client);
        *LOCAL_NICKNAME.lock().await = Some(nickname.to_string());

        // Start background database worker
        let (db_tx, db_rx) = mpsc::channel(100);
        *DB_WORKER_TX.lock().await = Some(db_tx);
        tokio::spawn(async move {
            db_worker(db_rx).await;
        });

        // Spawn task to handle swarm events
        // This is essential for GigiDnsBehaviour to poll interfaces for peer discovery
        tokio::spawn(async move {
            loop {
                // Get client from global storage and handle swarm event
                if let Ok(Some(mut client_guard)) = Self::get_client().await {
                    if let Some(client) = client_guard.as_mut() {
                        if let Err(e) = client.handle_next_swarm_event().await {
                            gigi_logging::error!("Error handling swarm event: {}", e);
                        }
                    }
                }
            }
        });

        // Start event handling loop for P2P events
        tokio::spawn(async move {
            while let Some(event) = event_receiver.next().await {
                Self::handle_event(event).await;
            }
        });

        Ok(())
    }

    pub async fn get_client() -> Result<Option<tokio::sync::MutexGuard<'static, Option<P2pClient>>>>
    {
        Ok(Some(P2P_CLIENT.lock().await))
    }

    async fn handle_event(event: P2pEvent) {
        let result = EventBus::send(AppEvent::P2P(event.clone()));
        println!("Sent event to event bus: {:?}, result: {:?}", event, result);

        match event {
            P2pEvent::PeerDiscovered {
                peer_id, nickname, ..
            } => {
                println!("Discovered peer: {} ({})", nickname, peer_id);
            }
            P2pEvent::DirectMessage {
                from_nickname,
                message,
                from,
                ..
            } => {
                println!("Message from {}: {}", from_nickname, message);
                let local_nickname = LOCAL_NICKNAME.lock().await.clone().unwrap_or_default();
                send_db_operation(DbOperation::StoreDirectMessage {
                    from_nickname,
                    to_nickname: local_nickname,
                    message,
                    is_own: false,
                    from_peer_id: from.to_string(),
                })
                .await;
            }
            P2pEvent::DirectFileShareMessage {
                from_nickname,
                share_code,
                filename,
                file_size,
                file_type,
                from,
                ..
            } => {
                println!(
                    "File share from {}: {} (code: {})",
                    from_nickname, filename, share_code
                );
                let local_nickname = LOCAL_NICKNAME.lock().await.clone().unwrap_or_default();
                send_db_operation(DbOperation::StoreFileShareMessage {
                    from_nickname,
                    to_nickname: local_nickname,
                    filename,
                    share_code,
                    file_size,
                    file_type,
                    is_own: false,
                    from_peer_id: from.to_string(),
                })
                .await;
            }
            P2pEvent::FileDownloadProgress {
                download_id,
                downloaded_chunks,
                total_chunks,
                filename,
                ..
            } => {
                let progress = (downloaded_chunks * 100) / total_chunks;
                println!("Downloading {}: {}%", filename, progress);
                // Send event to update UI with download progress
                let _ = EventBus::send(AppEvent::FileDownloadProgress {
                    download_id,
                    progress: progress as u8,
                });
            }
            P2pEvent::FileDownloadCompleted {
                download_id,
                path,
                filename,
                ..
            } => {
                println!("File download completed: {} at {:?}", filename, path);

                // Check if it's an image file and generate thumbnail
                let file_ext = filename.split('.').last().unwrap_or("").to_lowercase();

                if matches!(
                    file_ext.as_str(),
                    "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp"
                ) {
                    // Generate thumbnail
                    let thumbnail_path = path
                        .parent()
                        .unwrap_or(&path)
                        .join(format!("{}.thumbnail.jpg", filename));
                    println!(
                        "Generating thumbnail for downloaded file {} at {:?}",
                        filename, thumbnail_path
                    );
                    if let Err(e) = Self::generate_thumbnail(&path, &thumbnail_path) {
                        println!("Failed to generate thumbnail for downloaded image: {:?}", e);
                    } else {
                        println!(
                            "Thumbnail generated successfully for downloaded file at {:?}",
                            thumbnail_path
                        );
                        // Check if the file was actually created and has content
                        if let Ok(metadata) = std::fs::metadata(&thumbnail_path) {
                            println!("Thumbnail file size: {} bytes", metadata.len());
                        }
                    }
                }

                // Send event to update UI with download completion
                let _ = EventBus::send(AppEvent::FileDownloadCompleted { download_id, path });
            }
            P2pEvent::FileDownloadFailed {
                download_id,
                error,
                filename,
                ..
            } => {
                println!("File download failed: {} - {}", filename, error);
                // Send event to update UI with download failure
                let _ = EventBus::send(AppEvent::FileDownloadFailed { download_id, error });
            }
            P2pEvent::DirectGroupShareMessage {
                from,
                from_nickname,
                group_id,
                group_name,
                ..
            } => {
                println!(
                    "Group share from {}: {} (ID: {})",
                    from_nickname, group_name, group_id
                );
                let local_nickname = LOCAL_NICKNAME.lock().await.clone().unwrap_or_default();
                send_db_operation(DbOperation::StoreGroupShareMessage {
                    from_nickname,
                    to_nickname: local_nickname,
                    group_id,
                    group_name,
                    is_own: false,
                    from_peer_id: from.to_string(),
                })
                .await;
            }
            P2pEvent::GroupMessage {
                from: _,
                from_nickname,
                group,
                message,
                ..
            } => {
                println!(
                    "Group message from {} in {}: {}",
                    from_nickname, group, message
                );
                send_db_operation(DbOperation::StoreGroupMessage {
                    from_nickname,
                    group_name: group,
                    message,
                    is_own: false,
                })
                .await;
            }
            P2pEvent::GroupFileShareMessage {
                from: _,
                from_nickname,
                group,
                share_code,
                filename,
                file_size,
                file_type,
                ..
            } => {
                println!(
                    "Group file share from {} in {}: {} (code: {})",
                    from_nickname, group, filename, share_code
                );
                send_db_operation(DbOperation::StoreGroupFileShareMessage {
                    from_nickname,
                    group_name: group,
                    filename,
                    share_code,
                    file_size,
                    file_type,
                    is_own: false,
                })
                .await;
            }
            _ => {
                println!("Other P2P event: {:?}", event);
            }
        }
    }

    pub async fn send_message(to_nickname: &str, message: &str) -> Result<()> {
        if let Ok(Some(mut client_guard)) = Self::get_client().await {
            if let Some(client) = client_guard.as_mut() {
                println!("Sending message to {}: {}", to_nickname, message);
                match client.send_direct_message(to_nickname, message.to_string()) {
                    Ok(_) => println!("Message sent successfully to {}", to_nickname),
                    Err(e) => {
                        println!("Failed to send message to {}: {:?}", to_nickname, e);
                        return Err(e);
                    }
                }
            } else {
                println!("P2P client is not initialized");
                return Err(anyhow::anyhow!("P2P client not initialized"));
            }
        } else {
            println!("Failed to get P2P client");
            return Err(anyhow::anyhow!("Failed to get P2P client"));
        }
        Ok(())
    }

    pub async fn deliver_pending_messages(nickname: &str) -> Result<()> {
        if let Ok(Some(mut client_guard)) = Self::get_client().await {
            if let Some(client) = client_guard.as_mut() {
                println!("Delivering pending messages to {}", nickname);
                match client.send_pending_messages(nickname).await {
                    Ok(count) => {
                        if count > 0 {
                            println!("Delivered {} pending messages to {}", count, nickname);
                        }
                    }
                    Err(e) => {
                        println!(
                            "Failed to deliver pending messages to {}: {:?}",
                            nickname, e
                        );
                        return Err(e);
                    }
                }
            } else {
                println!("P2P client is not initialized");
                return Err(anyhow::anyhow!("P2P client not initialized"));
            }
        } else {
            println!("Failed to get P2P client");
            return Err(anyhow::anyhow!("Failed to get P2P client"));
        }
        Ok(())
    }

    pub async fn send_group_message(group_name: &str, message: &str) -> Result<()> {
        if let Ok(Some(mut client_guard)) = Self::get_client().await {
            if let Some(client) = client_guard.as_mut() {
                client.send_group_message(group_name, message.to_string())?;
            }
        }
        Ok(())
    }

    pub async fn send_group_file(group_name: &str, file_path: &PathBuf) -> Result<FileShareInfo> {
        if let Ok(Some(mut client_guard)) = Self::get_client().await {
            if let Some(client) = client_guard.as_mut() {
                let filename = file_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                let file_size = std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
                let file_type = file_path
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_else(|| "bin".to_string());

                let lower_file_type = file_type.to_lowercase();
                let is_image = lower_file_type.starts_with("image/")
                    || ["png", "jpg", "jpeg", "gif", "bmp", "webp"]
                        .contains(&lower_file_type.as_str());

                if is_image {
                    let data_dir = env::var("GIGI_DATA_DIR").unwrap_or_else(|_| {
                        dirs::data_local_dir()
                            .unwrap_or_else(|| PathBuf::from("."))
                            .join("gigi-dioxus")
                            .to_string_lossy()
                            .to_string()
                    });

                    let data_dir_expanded = if data_dir.starts_with('~') {
                        if let Some(home) = dirs::home_dir() {
                            home.join(data_dir.strip_prefix('~').unwrap_or(""))
                        } else {
                            PathBuf::from(data_dir)
                        }
                    } else {
                        PathBuf::from(data_dir)
                    };

                    let uploads_dir = data_dir_expanded.join("uploads");
                    let thumbnail_path = uploads_dir.join(format!("{}.thumbnail.jpg", filename));

                    println!(
                        "Generating thumbnail for {} at {:?}",
                        filename, thumbnail_path
                    );
                    if let Err(e) = Self::generate_thumbnail(file_path, &thumbnail_path) {
                        println!("Failed to generate thumbnail: {:?}", e);
                    } else {
                        println!("Thumbnail generated successfully at {:?}", thumbnail_path);
                    }
                }

                client.send_group_file(group_name, file_path).await?;

                let share_code = client.share_file(file_path).await?;

                return Ok(FileShareInfo {
                    share_code,
                    filename,
                    file_size,
                    file_type,
                });
            }
        }
        Err(anyhow::anyhow!("P2P client not initialized"))
    }

    pub async fn join_group(group_name: &str) -> Result<()> {
        if let Ok(Some(mut client_guard)) = Self::get_client().await {
            if let Some(client) = client_guard.as_mut() {
                client.join_group(group_name)?;
            }
        }
        Ok(())
    }

    pub async fn get_group_member_count(group_name: &str) -> Result<usize> {
        if let Ok(Some(client_guard)) = Self::get_client().await {
            if let Some(client) = client_guard.as_ref() {
                return Ok(client.get_group_member_count(group_name)?);
            }
        }
        Ok(0)
    }

    pub async fn leave_group(group_name: &str) -> Result<()> {
        if let Ok(Some(mut client_guard)) = Self::get_client().await {
            if let Some(client) = client_guard.as_mut() {
                client.leave_group(group_name)?;
            }
        }
        Ok(())
    }

    pub async fn send_group_share_message(
        to_nicknames: &[&str],
        group_id: &str,
        group_name: &str,
    ) -> Result<()> {
        for nickname in to_nicknames {
            if let Ok(Some(mut client_guard)) = Self::get_client().await {
                if let Some(client) = client_guard.as_mut() {
                    client.send_direct_share_group_message(
                        nickname,
                        group_id.to_string(),
                        group_name.to_string(),
                    )?;
                }
            }
        }
        Ok(())
    }

    pub async fn list_peers() -> Result<Vec<PeerInfo>> {
        if let Ok(Some(client_guard)) = Self::get_client().await {
            if let Some(client) = client_guard.as_ref() {
                let peers = client.list_peers();
                Ok(peers.into_iter().cloned().collect())
            } else {
                Ok(vec![])
            }
        } else {
            Ok(vec![])
        }
    }

    pub async fn get_local_nickname() -> Option<String> {
        let nickname_guard = LOCAL_NICKNAME.lock().await;
        nickname_guard.clone()
    }

    pub async fn share_file(to_nickname: &str, file_path: &PathBuf) -> Result<FileShareInfo> {
        if let Ok(Some(mut client_guard)) = Self::get_client().await {
            if let Some(client) = client_guard.as_mut() {
                let filename = file_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                let file_size = std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
                let file_type = file_path
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_else(|| "bin".to_string());

                // Check if it's an image file
                let lower_file_type = file_type.to_lowercase();
                let is_image = lower_file_type.starts_with("image/")
                    || ["png", "jpg", "jpeg", "gif", "bmp", "webp"]
                        .contains(&lower_file_type.as_str());

                // Generate thumbnail if it's an image
                if is_image {
                    // Get data directory
                    let data_dir = env::var("GIGI_DATA_DIR").unwrap_or_else(|_| {
                        dirs::data_local_dir()
                            .unwrap_or_else(|| PathBuf::from("."))
                            .join("gigi-dioxus")
                            .to_string_lossy()
                            .to_string()
                    });

                    // Expand ~ to home directory
                    let data_dir_expanded = if data_dir.starts_with('~') {
                        if let Some(home) = dirs::home_dir() {
                            home.join(data_dir.strip_prefix('~').unwrap_or(""))
                        } else {
                            PathBuf::from(data_dir)
                        }
                    } else {
                        PathBuf::from(data_dir)
                    };

                    let uploads_dir = data_dir_expanded.join("uploads");
                    let thumbnail_path = uploads_dir.join(format!("{}.thumbnail.jpg", filename));

                    // Generate thumbnail
                    println!(
                        "Generating thumbnail for {} at {:?}",
                        filename, thumbnail_path
                    );
                    if let Err(e) = Self::generate_thumbnail(file_path, &thumbnail_path) {
                        println!("Failed to generate thumbnail: {:?}", e);
                    } else {
                        println!("Thumbnail generated successfully at {:?}", thumbnail_path);
                        // Check if the file was actually created and has content
                        if let Ok(metadata) = std::fs::metadata(&thumbnail_path) {
                            println!("Thumbnail file size: {} bytes", metadata.len());
                        }
                    }
                }

                // Share the file and send it directly to the peer
                client.send_direct_file(to_nickname, file_path).await?;

                // Generate a share code for reference
                let share_code = client.share_file(file_path).await?;

                return Ok(FileShareInfo {
                    share_code,
                    filename,
                    file_size,
                    file_type,
                });
            }
        }
        Err(anyhow::anyhow!("P2P client not initialized"))
    }

    pub async fn download_file(nickname: &str, share_code: &str) -> Result<String> {
        if let Ok(Some(mut client_guard)) = Self::get_client().await {
            if let Some(client) = client_guard.as_mut() {
                let download_id = client.download_file(nickname, share_code)?;
                return Ok(download_id);
            }
        }
        Err(anyhow::anyhow!("P2P client not initialized"))
    }

    pub async fn shutdown() -> Result<()> {
        if let Ok(Some(mut client_guard)) = Self::get_client().await {
            if let Some(client) = client_guard.as_mut() {
                client.shutdown()?;
            }
            *client_guard = None;
        }
        Ok(())
    }

    /// Generate a thumbnail for an image file
    pub fn generate_thumbnail(input_path: &PathBuf, output_path: &PathBuf) -> Result<()> {
        // Open the image file
        let img = ImageReader::open(input_path)?.decode()?;

        // Resize the image to create a thumbnail (max 200x200)
        let thumbnail = imageops::resize(&img, 200, 200, imageops::FilterType::Lanczos3);

        // Create the output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Convert to RGB if needed (JPEG doesn't support alpha channel)
        let mut rgb_thumbnail = image::ImageBuffer::new(thumbnail.width(), thumbnail.height());
        for (x, y, pixel) in thumbnail.enumerate_pixels() {
            rgb_thumbnail.put_pixel(x, y, image::Rgb([pixel.0[0], pixel.0[1], pixel.0[2]]));
        }

        // Save the thumbnail as JPEG
        rgb_thumbnail.save(output_path)?;

        Ok(())
    }
}
