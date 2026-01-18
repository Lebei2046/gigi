use tauri::State;

use crate::{models::PluginState, Error, Result};
use gigi_store::Conversation;

/// Get all conversations
#[tauri::command]
pub(crate) async fn get_conversations(
    state: State<'_, PluginState>,
) -> Result<Vec<Conversation>> {
    let conversation_store = state.conversation_store.read().await;
    let store = conversation_store
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Conversation store not initialized".to_string()))?;

    let conversations = store.get_conversations().await.map_err(|e| {
        Error::Io(format!("Failed to get conversations: {}", e))
    })?;

    drop(conversation_store);
    Ok(conversations)
}

/// Get a specific conversation by ID
#[tauri::command]
pub(crate) async fn get_conversation(
    id: String,
    state: State<'_, PluginState>,
) -> Result<Option<Conversation>> {
    let conversation_store = state.conversation_store.read().await;
    let store = conversation_store
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Conversation store not initialized".to_string()))?;

    let conversation = store.get_conversation(&id).await.map_err(|e| {
        Error::Io(format!("Failed to get conversation: {}", e))
    })?;

    drop(conversation_store);
    Ok(conversation)
}

/// Create or update a conversation
#[tauri::command]
pub(crate) async fn upsert_conversation(
    id: String,
    name: String,
    is_group: bool,
    peer_id: String,
    last_message: Option<String>,
    last_message_timestamp: Option<i64>,
    state: State<'_, PluginState>,
) -> Result<()> {
    let conversation_store = state.conversation_store.read().await;
    let store = conversation_store
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Conversation store not initialized".to_string()))?;

    let last_ts = last_message_timestamp.and_then(|ts| chrono::DateTime::from_timestamp_millis(ts));

    store.upsert_conversation(id, name, is_group, peer_id, last_message, last_ts)
        .await
        .map_err(|e| Error::Io(format!("Failed to upsert conversation: {}", e)))?;

    drop(conversation_store);
    Ok(())
}

/// Update last message for a conversation
#[tauri::command]
pub(crate) async fn update_conversation_last_message(
    id: String,
    last_message: String,
    last_message_timestamp: i64,
    state: State<'_, PluginState>,
) -> Result<()> {
    let conversation_store = state.conversation_store.read().await;
    let store = conversation_store
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Conversation store not initialized".to_string()))?;

    let ts = chrono::DateTime::from_timestamp_millis(last_message_timestamp)
        .ok_or_else(|| Error::CommandFailed("Invalid timestamp".to_string()))?;

    store.update_last_message(&id, last_message, ts)
        .await
        .map_err(|e| Error::Io(format!("Failed to update conversation last message: {}", e)))?;

    drop(conversation_store);
    Ok(())
}

/// Increment unread count for a conversation
#[tauri::command]
pub(crate) async fn increment_conversation_unread(
    id: String,
    state: State<'_, PluginState>,
) -> Result<()> {
    let conversation_store = state.conversation_store.read().await;
    let store = conversation_store
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Conversation store not initialized".to_string()))?;

    store.increment_unread(&id)
        .await
        .map_err(|e| Error::Io(format!("Failed to increment unread count: {}", e)))?;

    drop(conversation_store);
    Ok(())
}

/// Mark conversation as read
#[tauri::command]
pub(crate) async fn mark_conversation_as_read(
    id: String,
    state: State<'_, PluginState>,
) -> Result<()> {
    let conversation_store = state.conversation_store.read().await;
    let store = conversation_store
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Conversation store not initialized".to_string()))?;

    store.mark_as_read(&id)
        .await
        .map_err(|e| Error::Io(format!("Failed to mark conversation as read: {}", e)))?;

    drop(conversation_store);
    Ok(())
}

/// Delete a conversation
#[tauri::command]
pub(crate) async fn delete_conversation(
    id: String,
    state: State<'_, PluginState>,
) -> Result<()> {
    let conversation_store = state.conversation_store.read().await;
    let store = conversation_store
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Conversation store not initialized".to_string()))?;

    store.delete_conversation(&id)
        .await
        .map_err(|e| Error::Io(format!("Failed to delete conversation: {}", e)))?;

    drop(conversation_store);
    Ok(())
}
