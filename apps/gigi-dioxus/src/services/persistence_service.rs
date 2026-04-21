
use anyhow::Result;
use gigi_store::{PersistenceConfig, MessageStore, ConversationStore, StoredMessage};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use std::path::PathBuf;
use dirs;
use std::env;

// Singleton instances for all store managers
static MESSAGE_STORE: Lazy<Mutex<Option<MessageStore>>> = Lazy::new(|| Mutex::new(None));
static CONVERSATION_STORE: Lazy<Mutex<Option<ConversationStore>>> = Lazy::new(|| Mutex::new(None));

pub struct PersistenceService;

impl PersistenceService {
    pub async fn initialize() -> Result<()> {
        // Get data directory
        let data_dir = env::var("GIGI_DATA_DIR")
            .unwrap_or_else(|_| {
                dirs::data_local_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("gigi-dioxus")
                    .to_string_lossy()
                    .to_string()
            });
        
        let db_path = PathBuf::from(data_dir).join("gigi-store.db");
        
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
        *MESSAGE_STORE.lock().await = Some(message_store);
        *CONVERSATION_STORE.lock().await = Some(conversation_store);
        
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
            direction: if is_own { gigi_store::MessageDirection::Sent } else { gigi_store::MessageDirection::Received },
            content: gigi_store::MessageContent::Text { text: message },
            sender_nickname: from_nickname,
            recipient_nickname: Some(to_nickname),
            group_name: None,
            peer_id: "".to_string(),
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
    
    pub async fn load_conversations() -> Result<Vec<gigi_store::Conversation>> {
        let store_guard = CONVERSATION_STORE.lock().await;
        if let Some(store) = store_guard.as_ref() {
            store.get_conversations().await.map_err(|e| anyhow::anyhow!(e))
        } else {
            Err(anyhow::anyhow!("Persistence service not initialized"))
        }
    }
    
    pub async fn load_messages(peer_nickname: &str, limit: usize, offset: usize) -> Result<Vec<StoredMessage>> {
        let store_guard = MESSAGE_STORE.lock().await;
        if let Some(store) = store_guard.as_ref() {
            store.get_conversation(peer_nickname, limit, offset).await.map_err(|e| anyhow::anyhow!(e))
        } else {
            Err(anyhow::anyhow!("Persistence service not initialized"))
        }
    }
}
