use crate::features::chat::chat_state::Message;
use crate::services::persistence_service::PersistenceService;

// Message service for handling message persistence and history loading
pub struct MessageService;

impl MessageService {
    pub fn new() -> Self {
        Self
    }

    // Load message history for a chat
    pub async fn load_message_history(
        &self,
        peer_nickname: &str,
        limit: u32,
        offset: u32,
    ) -> Vec<Message> {
        println!(
            "Loading message history for chat: {} with limit: {}, offset: {}",
            peer_nickname, limit, offset
        );

        match PersistenceService::load_messages(peer_nickname, limit as usize, offset as usize)
            .await
        {
            Ok(stored_messages) => stored_messages.iter().map(|m| m.into()).collect(),
            Err(e) => {
                println!("Error loading message history: {}", e);
                vec![]
            }
        }
    }

    // Save a message to persistence (new signature with full params)
    pub async fn save_message_full(
        &self,
        message: &Message,
        chat_id: &str,
        from_nickname: &str,
        to_nickname: &str,
        is_own: bool,
    ) -> Result<(), String> {
        println!("Saving message to chat {}: {:?}", chat_id, message);

        // Use persistence service to store message
        let _ = PersistenceService::store_direct_message(
            from_nickname.to_string(),
            to_nickname.to_string(),
            message.content.clone(),
            is_own,
        )
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    // Save a message to persistence (backward compatible wrapper)
    pub async fn save_message(&self, message: &Message, chat_id: &str) -> Result<(), String> {
        self.save_message_full(message, chat_id, "You", "", true)
            .await
    }

    // Mark messages as read
    pub async fn mark_messages_as_read(&self, chat_id: &str) -> Result<(), String> {
        // In a real app, this would mark messages as read in the backend
        println!("Marking messages as read for chat: {}", chat_id);
        Ok(())
    }

    // Clear messages for a chat
    pub async fn clear_messages(&self, chat_id: &str) -> Result<(), String> {
        // In a real app, this would clear messages in the backend
        println!("Clearing messages for chat: {}", chat_id);
        Ok(())
    }
}
