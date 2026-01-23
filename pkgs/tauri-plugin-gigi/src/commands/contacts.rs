use tauri::State;

use crate::{models::PluginState, Error, Result};

/// Helper to wait for initialization
async fn wait_for_initialization(state: &PluginState) -> Result<()> {
    // Check if already initialized
    {
        let contact_manager = state.contact_manager.lock().await;
        if contact_manager.is_some() {
            return Ok(());
        }
    }

    // Wait for initialization notification
    state.initialized.notified().await;
    Ok(())
}

/// Add a contact
#[tauri::command]
pub(crate) async fn contact_add(
    peer_id: String,
    name: String,
    state: State<'_, PluginState>,
) -> Result<gigi_store::ContactInfo> {
    wait_for_initialization(&state).await?;

    let contact_manager = state.contact_manager.lock().await;
    let manager = contact_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Contact manager not initialized".to_string()))?;

    manager
        .add(&peer_id, &name)
        .await
        .map_err(|e| Error::Io(format!("Failed to add contact: {}", e)))?;

    drop(contact_manager);

    // Return the contact info
    Ok(gigi_store::ContactInfo {
        peer_id,
        name,
        added_at: chrono::Utc::now().timestamp_millis(),
    })
}

/// Remove a contact
#[tauri::command]
pub(crate) async fn contact_remove(peer_id: String, state: State<'_, PluginState>) -> Result<()> {
    wait_for_initialization(&state).await?;

    let contact_manager = state.contact_manager.lock().await;
    let manager = contact_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Contact manager not initialized".to_string()))?;

    manager
        .remove(&peer_id)
        .await
        .map_err(|e| Error::Io(format!("Failed to remove contact: {}", e)))?;

    Ok(())
}

/// Update a contact's name
#[tauri::command]
pub(crate) async fn contact_update(
    peer_id: String,
    name: String,
    state: State<'_, PluginState>,
) -> Result<gigi_store::ContactInfo> {
    wait_for_initialization(&state).await?;

    let contact_manager = state.contact_manager.lock().await;
    let manager = contact_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Contact manager not initialized".to_string()))?;

    manager
        .update_name(&peer_id, &name)
        .await
        .map_err(|e| Error::Io(format!("Failed to update contact: {}", e)))?;

    drop(contact_manager);

    // Get the updated contact
    let contact_manager = state.contact_manager.lock().await;
    let manager = contact_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Contact manager not initialized".to_string()))?;

    let contact = manager
        .get(&peer_id)
        .await
        .map_err(|e| Error::Io(format!("Failed to get updated contact: {}", e)))?;

    drop(contact_manager);

    contact.ok_or_else(|| Error::CommandFailed("Contact not found after update".to_string()))
}

/// Get a specific contact by peer ID
#[tauri::command]
pub(crate) async fn contact_get(
    peer_id: String,
    state: State<'_, PluginState>,
) -> Result<Option<gigi_store::ContactInfo>> {
    wait_for_initialization(&state).await?;

    let contact_manager = state.contact_manager.lock().await;
    let manager = contact_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Contact manager not initialized".to_string()))?;

    let contact = manager
        .get(&peer_id)
        .await
        .map_err(|e| Error::Io(format!("Failed to get contact: {}", e)))?;

    drop(contact_manager);
    Ok(contact)
}

/// Get all contacts
#[tauri::command]
pub(crate) async fn contact_get_all(
    state: State<'_, PluginState>,
) -> Result<Vec<gigi_store::ContactInfo>> {
    wait_for_initialization(&state).await?;

    let contact_manager = state.contact_manager.lock().await;
    let manager = contact_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Contact manager not initialized".to_string()))?;

    let contacts = manager
        .get_all()
        .await
        .map_err(|e| Error::Io(format!("Failed to get contacts: {}", e)))?;

    drop(contact_manager);
    Ok(contacts)
}
