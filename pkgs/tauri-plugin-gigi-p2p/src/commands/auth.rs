use tauri::State;

use crate::{models::PluginState, Error, Result};

/// Helper to wait for initialization
async fn wait_for_initialization(state: &PluginState) -> Result<()> {
    // Check if already initialized
    {
        let auth_manager = state.auth_manager.lock().await;
        if auth_manager.is_some() {
            return Ok(());
        }
    }

    // Wait for initialization notification
    state.initialized.notified().await;
    Ok(())
}

/// Check if an account exists
#[tauri::command]
pub(crate) async fn auth_check_account(state: State<'_, PluginState>) -> Result<bool> {
    wait_for_initialization(&state).await?;

    let auth_manager = state.auth_manager.lock().await;
    let manager = auth_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Auth manager not initialized".to_string()))?;

    let has_account = manager
        .has_account()
        .await
        .map_err(|e| Error::Io(format!("Failed to check account: {}", e)))?;

    drop(auth_manager);
    Ok(has_account)
}

/// Create a new account
#[tauri::command]
pub(crate) async fn auth_signup(
    mnemonic: String,
    password: String,
    name: String,
    group_name: Option<String>,
    state: State<'_, PluginState>,
) -> Result<gigi_store::AccountInfo> {
    wait_for_initialization(&state).await?;

    let auth_manager = state.auth_manager.lock().await;
    let manager = auth_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Auth manager not initialized".to_string()))?;

    // Check if account already exists
    let has_account = manager
        .has_account()
        .await
        .map_err(|e| Error::Io(format!("Failed to check account: {}", e)))?;

    let account_info = if has_account {
        // Account already exists, try to login and get account info
        match manager.login(&password).await {
            Ok(login_result) => login_result.account_info,
            Err(_) => {
                drop(auth_manager);
                return Err(Error::Io(
                    "Account already exists with different credentials".to_string(),
                ));
            }
        }
    } else {
        // Create new account
        manager
            .create_account(&mnemonic, &password, Some(name))
            .await
            .map_err(|e| Error::Io(format!("Failed to create account: {}", e)))?
    };

    drop(auth_manager);

    // If group_name is provided, create a group
    if let Some(group_name) = group_name {
        let group_manager = state.group_manager.lock().await;
        let manager = group_manager
            .as_ref()
            .ok_or_else(|| Error::CommandFailed("Group manager not initialized".to_string()))?;

        manager
            .add_or_update(&account_info.group_id, &group_name, false)
            .await
            .map_err(|e| Error::Io(format!("Failed to create group: {}", e)))?;

        drop(group_manager);
    }

    Ok(account_info)
}

/// Login with password
#[tauri::command]
pub(crate) async fn auth_login(
    password: String,
    state: State<'_, PluginState>,
) -> Result<gigi_store::LoginResult> {
    wait_for_initialization(&state).await?;

    let auth_manager = state.auth_manager.lock().await;
    let manager = auth_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Auth manager not initialized".to_string()))?;

    let login_result = manager
        .login(&password)
        .await
        .map_err(|e| Error::Io(format!("Login failed: {}", e)))?;

    drop(auth_manager);
    Ok(login_result)
}

/// Get account info (without exposing sensitive data)
#[tauri::command]
pub(crate) async fn auth_get_account_info(
    state: State<'_, PluginState>,
) -> Result<Option<gigi_store::AccountInfo>> {
    wait_for_initialization(&state).await?;

    let auth_manager = state.auth_manager.lock().await;
    let manager = auth_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Auth manager not initialized".to_string()))?;

    let account_info = manager
        .get_account_info()
        .await
        .map_err(|e| Error::Io(format!("Failed to get account info: {}", e)))?;

    drop(auth_manager);
    Ok(account_info)
}

/// Delete account
#[tauri::command]
pub(crate) async fn auth_delete_account(state: State<'_, PluginState>) -> Result<()> {
    wait_for_initialization(&state).await?;

    let auth_manager = state.auth_manager.lock().await;
    let manager = auth_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Auth manager not initialized".to_string()))?;

    manager
        .delete_account()
        .await
        .map_err(|e| Error::Io(format!("Failed to delete account: {}", e)))?;

    drop(auth_manager);
    Ok(())
}

/// Verify password without exposing account data
#[tauri::command]
pub(crate) async fn auth_verify_password(
    password: String,
    state: State<'_, PluginState>,
) -> Result<bool> {
    wait_for_initialization(&state).await?;

    let auth_manager = state.auth_manager.lock().await;
    let manager = auth_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Auth manager not initialized".to_string()))?;

    let is_valid = manager
        .verify_password(&password)
        .await
        .map_err(|e| Error::Io(format!("Failed to verify password: {}", e)))?;

    drop(auth_manager);
    Ok(is_valid)
}
