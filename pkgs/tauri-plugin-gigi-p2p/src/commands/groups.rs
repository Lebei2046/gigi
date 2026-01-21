use tauri::State;

use crate::{models::PluginState, Error, Result};

/// Helper to wait for initialization
async fn wait_for_initialization(state: &PluginState) -> Result<()> {
    // Check if already initialized
    {
        let group_manager = state.group_manager.lock().await;
        if group_manager.is_some() {
            return Ok(());
        }
    }

    // Wait for initialization notification
    state.initialized.notified().await;
    Ok(())
}

/// Create a new group
#[tauri::command]
pub(crate) async fn group_create(
    group_id: String,
    group_name: String,
    joined: bool,
    state: State<'_, PluginState>,
) -> Result<gigi_store::GroupInfo> {
    wait_for_initialization(&state).await?;

    let group_manager = state.group_manager.lock().await;
    let manager = group_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Group manager not initialized".to_string()))?;

    manager
        .add_or_update(&group_id, &group_name, joined)
        .await
        .map_err(|e| Error::Io(format!("Failed to create group: {}", e)))?;

    drop(group_manager);

    // Return the group info
    Ok(gigi_store::GroupInfo {
        group_id,
        name: group_name,
        joined,
        created_at: chrono::Utc::now().timestamp_millis(),
    })
}

/// Join an existing group
#[tauri::command]
pub(crate) async fn group_join(
    group_id: String,
    group_name: String,
    state: State<'_, PluginState>,
) -> Result<gigi_store::GroupInfo> {
    wait_for_initialization(&state).await?;

    let group_manager = state.group_manager.lock().await;
    let manager = group_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Group manager not initialized".to_string()))?;

    manager
        .add_or_update(&group_id, &group_name, true)
        .await
        .map_err(|e| Error::Io(format!("Failed to join group: {}", e)))?;

    drop(group_manager);

    Ok(gigi_store::GroupInfo {
        group_id,
        name: group_name,
        joined: true,
        created_at: chrono::Utc::now().timestamp_millis(),
    })
}

/// Get all groups
#[tauri::command]
pub(crate) async fn group_get_all(
    state: State<'_, PluginState>,
) -> Result<Vec<gigi_store::GroupInfo>> {
    wait_for_initialization(&state).await?;

    let group_manager = state.group_manager.lock().await;
    let manager = group_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Group manager not initialized".to_string()))?;

    let groups = manager
        .get_all()
        .await
        .map_err(|e| Error::Io(format!("Failed to get groups: {}", e)))?;

    drop(group_manager);
    Ok(groups)
}

/// Get a specific group by ID
#[tauri::command]
pub(crate) async fn group_get(
    group_id: String,
    state: State<'_, PluginState>,
) -> Result<Option<gigi_store::GroupInfo>> {
    wait_for_initialization(&state).await?;

    let group_manager = state.group_manager.lock().await;
    let manager = group_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Group manager not initialized".to_string()))?;

    let group = manager
        .get(&group_id)
        .await
        .map_err(|e| Error::Io(format!("Failed to get group: {}", e)))?;

    drop(group_manager);
    Ok(group)
}

/// Delete a group
#[tauri::command]
pub(crate) async fn group_delete(group_id: String, state: State<'_, PluginState>) -> Result<()> {
    wait_for_initialization(&state).await?;

    let group_manager = state.group_manager.lock().await;
    let manager = group_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Group manager not initialized".to_string()))?;

    manager
        .delete(&group_id)
        .await
        .map_err(|e| Error::Io(format!("Failed to delete group: {}", e)))?;

    drop(group_manager);
    Ok(())
}

/// Update a group
#[tauri::command]
pub(crate) async fn group_update(
    group_id: String,
    group_name: Option<String>,
    joined: Option<bool>,
    state: State<'_, PluginState>,
) -> Result<gigi_store::GroupInfo> {
    wait_for_initialization(&state).await?;

    let group_manager = state.group_manager.lock().await;
    let manager = group_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Group manager not initialized".to_string()))?;

    // Update name if provided
    if let Some(name) = group_name {
        manager
            .update_name(&group_id, &name)
            .await
            .map_err(|e| Error::Io(format!("Failed to update group name: {}", e)))?;
    }

    // Update join status if provided
    if let Some(is_joined) = joined {
        manager
            .update_join_status(&group_id, is_joined)
            .await
            .map_err(|e| Error::Io(format!("Failed to update join status: {}", e)))?;
    }

    drop(group_manager);

    // Get the updated group
    let group_manager = state.group_manager.lock().await;
    let manager = group_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Group manager not initialized".to_string()))?;

    let group = manager
        .get(&group_id)
        .await
        .map_err(|e| Error::Io(format!("Failed to get updated group: {}", e)))?;

    drop(group_manager);

    group.ok_or_else(|| Error::CommandFailed("Group not found after update".to_string()))
}
