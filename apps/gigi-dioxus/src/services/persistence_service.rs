use anyhow::Result;
use dirs;
use gigi_store::{ConversationStore, MessageStore, PersistenceConfig, StoredMessage};
use once_cell::sync::Lazy;
use std::env;
use std::path::PathBuf;
use tokio::sync::Mutex;

// Singleton instances for all store managers
static MESSAGE_STORE: Lazy<Mutex<Option<MessageStore>>> = Lazy::new(|| Mutex::new(None));
static CONVERSATION_STORE: Lazy<Mutex<Option<ConversationStore>>> = Lazy::new(|| Mutex::new(None));

pub struct PersistenceService;

impl PersistenceService {
    pub async fn initialize() -> Result<()> {
        let mut message_store_guard = MESSAGE_STORE.lock().await;
        let mut conversation_store_guard = CONVERSATION_STORE.lock().await;

        // Check if stores are already initialized
        if message_store_guard.is_some() && conversation_store_guard.is_some() {
            return Ok(());
        }

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

        let db_path = data_dir_expanded.join("gigi-store.db");

        // Create parent directories if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let config = PersistenceConfig {
            db_path: db_path.clone(),
            ..Default::default()
        };

        // Initialize all stores
        let message_store = MessageStore::with_config(config.clone()).await?;
        let conversation_store = ConversationStore::new(db_path.clone()).await?;

        // Store singletons
        *message_store_guard = Some(message_store);
        *conversation_store_guard = Some(conversation_store);

        Ok(())
    }

    // Helper methods
    pub async fn store_direct_message(
        from_nickname: String,
        to_nickname: String,
        message: String,
        is_own: bool,
    ) -> Result<String> {
        let msg_id = uuid::Uuid::new_v4().to_string();
        let stored_msg = StoredMessage {
            id: msg_id.clone(),
            msg_type: gigi_store::MessageType::Direct,
            direction: if is_own {
                gigi_store::MessageDirection::Sent
            } else {
                gigi_store::MessageDirection::Received
            },
            content: gigi_store::MessageContent::Text { text: message },
            sender_nickname: from_nickname.clone(),
            recipient_nickname: Some(to_nickname),
            group_name: None,
            peer_id: from_nickname.to_string(),
            timestamp: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
            delivered: false,
            delivered_at: None,
            read: false,
            read_at: None,
            sync_status: gigi_store::SyncStatus::Pending,
            sync_attempts: 0,
            last_sync_attempt: None,
            expires_at: chrono::Utc::now() + chrono::Duration::days(7),
        };

        let mut store_guard = MESSAGE_STORE.lock().await;
        if let Some(store) = store_guard.as_mut() {
            store.store_message(stored_msg).await?;
        }

        Ok(msg_id)
    }

    pub async fn store_file_share_message(
        from_nickname: String,
        to_nickname: String,
        filename: String,
        share_code: String,
        file_size: u64,
        file_type: String,
        is_own: bool,
    ) -> Result<String> {
        let msg_id = uuid::Uuid::new_v4().to_string();
        let stored_msg = StoredMessage {
            id: msg_id.clone(),
            msg_type: gigi_store::MessageType::Direct,
            direction: if is_own {
                gigi_store::MessageDirection::Sent
            } else {
                gigi_store::MessageDirection::Received
            },
            content: gigi_store::MessageContent::FileShare {
                filename,
                share_code,
                file_size,
                file_type,
            },
            sender_nickname: from_nickname.clone(),
            recipient_nickname: Some(to_nickname),
            group_name: None,
            peer_id: from_nickname.to_string(),
            timestamp: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
            delivered: false,
            delivered_at: None,
            read: false,
            read_at: None,
            sync_status: gigi_store::SyncStatus::Pending,
            sync_attempts: 0,
            last_sync_attempt: None,
            expires_at: chrono::Utc::now() + chrono::Duration::days(7),
        };

        let mut store_guard = MESSAGE_STORE.lock().await;
        if let Some(store) = store_guard.as_mut() {
            store.store_message(stored_msg).await?;
        }

        Ok(msg_id)
    }

    pub async fn store_group_file_share_message(
        from_nickname: String,
        group_name: String,
        filename: String,
        share_code: String,
        file_size: u64,
        file_type: String,
        is_own: bool,
    ) -> Result<String> {
        let msg_id = uuid::Uuid::new_v4().to_string();
        let stored_msg = StoredMessage {
            id: msg_id.clone(),
            msg_type: gigi_store::MessageType::Group,
            direction: if is_own {
                gigi_store::MessageDirection::Sent
            } else {
                gigi_store::MessageDirection::Received
            },
            content: gigi_store::MessageContent::FileShare {
                filename,
                share_code,
                file_size,
                file_type,
            },
            sender_nickname: from_nickname.clone(),
            recipient_nickname: None,
            group_name: Some(group_name),
            peer_id: from_nickname.to_string(),
            timestamp: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
            delivered: false,
            delivered_at: None,
            read: false,
            read_at: None,
            sync_status: gigi_store::SyncStatus::Pending,
            sync_attempts: 0,
            last_sync_attempt: None,
            expires_at: chrono::Utc::now() + chrono::Duration::days(7),
        };

        let mut store_guard = MESSAGE_STORE.lock().await;
        if let Some(store) = store_guard.as_mut() {
            store.store_message(stored_msg).await?;
        }

        Ok(msg_id)
    }

    pub async fn store_group_share_message(
        from_nickname: String,
        to_nickname: String,
        group_id: String,
        group_name: String,
        is_own: bool,
    ) -> Result<String> {
        let msg_id = uuid::Uuid::new_v4().to_string();
        let stored_msg = StoredMessage {
            id: msg_id.clone(),
            msg_type: gigi_store::MessageType::Direct,
            direction: if is_own {
                gigi_store::MessageDirection::Sent
            } else {
                gigi_store::MessageDirection::Received
            },
            content: gigi_store::MessageContent::ShareGroup {
                group_id,
                group_name,
                inviter_nickname: from_nickname.clone(),
            },
            sender_nickname: from_nickname.clone(),
            recipient_nickname: Some(to_nickname),
            group_name: None,
            peer_id: from_nickname.to_string(),
            timestamp: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
            delivered: false,
            delivered_at: None,
            read: false,
            read_at: None,
            sync_status: gigi_store::SyncStatus::Pending,
            sync_attempts: 0,
            last_sync_attempt: None,
            expires_at: chrono::Utc::now() + chrono::Duration::days(7),
        };

        let mut store_guard = MESSAGE_STORE.lock().await;
        if let Some(store) = store_guard.as_mut() {
            store.store_message(stored_msg).await?;
        }

        Ok(msg_id)
    }

    pub async fn store_group_message(
        from_nickname: String,
        group_name: String,
        message: String,
        is_own: bool,
    ) -> Result<String> {
        let msg_id = uuid::Uuid::new_v4().to_string();
        Self::store_group_message_with_id(
            msg_id.clone(),
            from_nickname,
            group_name,
            message,
            is_own,
        )
        .await?;
        Ok(msg_id)
    }

    pub async fn store_group_message_with_id(
        msg_id: String,
        from_nickname: String,
        group_name: String,
        message: String,
        is_own: bool,
    ) -> Result<String> {
        println!("store_group_message_with_id called - msg_id: {}, from_nickname: {}, group_name: {}, message: {}", msg_id, from_nickname, group_name, message);

        let stored_msg = StoredMessage {
            id: msg_id.clone(),
            msg_type: gigi_store::MessageType::Group,
            direction: if is_own {
                gigi_store::MessageDirection::Sent
            } else {
                gigi_store::MessageDirection::Received
            },
            content: gigi_store::MessageContent::Text { text: message },
            sender_nickname: from_nickname.clone(),
            recipient_nickname: None,
            group_name: Some(group_name.clone()),
            peer_id: group_name.clone(),
            timestamp: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
            delivered: false,
            delivered_at: None,
            read: false,
            read_at: None,
            sync_status: gigi_store::SyncStatus::Pending,
            sync_attempts: 0,
            last_sync_attempt: None,
            expires_at: chrono::Utc::now() + chrono::Duration::days(7),
        };

        let mut store_guard = MESSAGE_STORE.lock().await;
        if let Some(store) = store_guard.as_mut() {
            println!("Storing group message to database...");
            store.store_message(stored_msg).await?;
            println!("Group message stored successfully");
        } else {
            println!("WARNING: MESSAGE_STORE is None, message not stored!");
        }

        Ok(msg_id)
    }

    pub async fn load_conversations() -> Result<Vec<gigi_store::Conversation>> {
        let store_guard = CONVERSATION_STORE.lock().await;
        if let Some(store) = store_guard.as_ref() {
            store
                .get_conversations()
                .await
                .map_err(|e| anyhow::anyhow!(e))
        } else {
            Err(anyhow::anyhow!("Persistence service not initialized"))
        }
    }

    pub async fn load_messages(
        peer_nickname: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<StoredMessage>> {
        println!(
            "load_messages called - peer_nickname: {}, limit: {}, offset: {}",
            peer_nickname, limit, offset
        );

        let store_guard = MESSAGE_STORE.lock().await;
        if let Some(store) = store_guard.as_ref() {
            let result = store
                .get_conversation(peer_nickname, limit, offset)
                .await
                .map_err(|e| anyhow::anyhow!(e));

            if let Ok(messages) = &result {
                println!(
                    "Loaded {} direct messages for '{}'",
                    messages.len(),
                    peer_nickname
                );
                for (i, msg) in messages.iter().enumerate() {
                    println!(
                        "  Message {}: id={}, sender={}, content={}",
                        i,
                        msg.id,
                        msg.sender_nickname,
                        match &msg.content {
                            gigi_store::MessageContent::Text { text } => text.clone(),
                            _ => "non-text".to_string(),
                        }
                    );
                }
            }

            result
        } else {
            println!("WARNING: MESSAGE_STORE is None, returning empty");
            Err(anyhow::anyhow!("Persistence service not initialized"))
        }
    }

    pub async fn load_group_messages(
        group_name: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<StoredMessage>> {
        println!(
            "load_group_messages called - group_name: {}, limit: {}, offset: {}",
            group_name, limit, offset
        );

        let store_guard = MESSAGE_STORE.lock().await;
        if let Some(store) = store_guard.as_ref() {
            let result = store
                .get_group_messages(group_name, limit, offset)
                .await
                .map_err(|e| anyhow::anyhow!(e));

            if let Ok(messages) = &result {
                println!(
                    "Loaded {} group messages for '{}'",
                    messages.len(),
                    group_name
                );
                for (i, msg) in messages.iter().enumerate() {
                    println!(
                        "  Message {}: id={}, sender={}, content={}",
                        i,
                        msg.id,
                        msg.sender_nickname,
                        match &msg.content {
                            gigi_store::MessageContent::Text { text } => text.clone(),
                            _ => "non-text".to_string(),
                        }
                    );
                }
            }

            result
        } else {
            println!("WARNING: MESSAGE_STORE is None, returning empty");
            Err(anyhow::anyhow!("Persistence service not initialized"))
        }
    }

    // Clear messages for a conversation
    pub async fn clear_conversation(peer_nickname: &str) -> Result<usize> {
        let store_guard = MESSAGE_STORE.lock().await;
        if let Some(store) = store_guard.as_ref() {
            store
                .clear_conversation(peer_nickname)
                .await
                .map_err(|e| anyhow::anyhow!(e))
        } else {
            Err(anyhow::anyhow!("Persistence service not initialized"))
        }
    }

    pub async fn upsert_conversation(
        id: String,
        name: String,
        is_group: bool,
        peer_id: String,
        last_message: Option<String>,
        last_message_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<()> {
        let store_guard = CONVERSATION_STORE.lock().await;
        if let Some(store) = store_guard.as_ref() {
            store
                .upsert_conversation(
                    id,
                    name,
                    is_group,
                    peer_id,
                    last_message,
                    last_message_timestamp,
                )
                .await
                .map_err(|e| anyhow::anyhow!(e))
        } else {
            Err(anyhow::anyhow!("Persistence service not initialized"))
        }
    }

    pub async fn increment_unread(id: &str) -> Result<()> {
        let store_guard = CONVERSATION_STORE.lock().await;
        if let Some(store) = store_guard.as_ref() {
            store
                .increment_unread(id)
                .await
                .map_err(|e| anyhow::anyhow!(e))
        } else {
            Err(anyhow::anyhow!("Persistence service not initialized"))
        }
    }
}
