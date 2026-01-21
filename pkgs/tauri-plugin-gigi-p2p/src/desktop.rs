use sea_orm_migration::MigratorTrait;
use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Manager, Runtime};

use crate::models::PluginState;
use sea_orm::{Database, DatabaseConnection};

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<GigiP2p<R>> {
    let state = PluginState::new();
    app.manage(state);

    // Initialize database connection and managers
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = initialize_database_and_managers(&app_handle).await {
            tracing::error!("Failed to initialize database and managers: {}", e);
        }
    });

    Ok(GigiP2p(app.clone()))
}

async fn initialize_database_and_managers<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get app data directory
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Create gigi directory if it doesn't exist
    let gigi_dir = app_data_dir.join("gigi");
    std::fs::create_dir_all(&gigi_dir).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Database path
    let db_path = gigi_dir.join("gigi.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

    // Connect to database
    let db: DatabaseConnection = Database::connect(&db_url)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Run migrations
    gigi_store::migration::Migrator::up(&db, None)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Initialize AuthManager
    let auth_manager = gigi_store::AuthManager::new(db.clone());

    // Initialize GroupManager
    let group_manager = gigi_store::GroupManager::new(db.clone());

    // Get state and update it
    let state = app.state::<PluginState>();
    let mut db_connection_guard = state.db_connection.write().await;
    *db_connection_guard = Some(db);
    drop(db_connection_guard);

    let mut auth_manager_guard = state.auth_manager.lock().await;
    *auth_manager_guard = Some(auth_manager);
    drop(auth_manager_guard);

    let mut group_manager_guard = state.group_manager.lock().await;
    *group_manager_guard = Some(group_manager);
    drop(group_manager_guard);

    // Notify that initialization is complete
    state.initialized.notify_one();

    tracing::info!("Database and managers initialized successfully");
    Ok(())
}

/// Access to the gigi-p2p APIs.
pub struct GigiP2p<R: Runtime>(AppHandle<R>);

impl<R: Runtime> GigiP2p<R> {
    pub fn get_state(&self) -> PluginState {
        self.0.state::<PluginState>().inner().clone()
    }
}
