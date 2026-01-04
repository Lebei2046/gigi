use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn clear_app_data(app_handle: AppHandle) -> Result<(), String> {
    use std::fs;

    // Get app data directory
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // Remove app data directory if it exists
    if app_data_dir.exists() {
        fs::remove_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to remove app data directory: {}", e))?;
    }

    // Get local app data directory
    let local_data_dir = app_handle
        .path()
        .local_data_dir()
        .map_err(|e| format!("Failed to get local data directory: {}", e))?;

    // Remove local app data if it exists
    if local_data_dir.exists() {
        fs::remove_dir_all(&local_data_dir)
            .map_err(|e| format!("Failed to remove local data directory: {}", e))?;
    }

    Ok(())
}
