use tauri::{AppHandle, Manager};

use crate::{Error, Result};

/// Clear all app data
#[tauri::command]
pub(crate) async fn clear_app_data<R: tauri::Runtime>(app: AppHandle<R>) -> Result<()> {
    use std::fs;

    // Get app data directory
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| Error::Io(format!("Failed to get app data directory: {}", e)))?;

    // Remove app data directory if it exists
    if app_data_dir.exists() {
        fs::remove_dir_all(&app_data_dir)
            .map_err(|e| Error::Io(format!("Failed to remove app data directory: {}", e)))?;
    }

    // Get local app data directory
    let local_data_dir = app
        .path()
        .local_data_dir()
        .map_err(|e| Error::Io(format!("Failed to get local data directory: {}", e)))?;

    // Remove local app data if it exists
    if local_data_dir.exists() {
        fs::remove_dir_all(&local_data_dir)
            .map_err(|e| Error::Io(format!("Failed to remove local data directory: {}", e)))?;
    }

    Ok(())
}
