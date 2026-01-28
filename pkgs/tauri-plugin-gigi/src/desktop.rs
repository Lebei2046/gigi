//! Desktop-specific initialization for the Gigi Tauri plugin.
//!
//! This module handles the desktop platform initialization, including database
//! setup, running migrations, and initializing the various store managers.

use sea_orm_migration::MigratorTrait;
use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Manager, Runtime};

use crate::models::PluginState;
use sea_orm::{Database, DatabaseConnection};

/// Initializes the Gigi plugin for desktop platforms.
///
/// This function sets up the plugin state and asynchronously initializes the
/// database and store managers. The initialization runs in the background to
/// avoid blocking the main thread.
///
/// # Arguments
///
/// * `app` - A reference to the Tauri AppHandle
/// * `_api` - The plugin API (unused but required for the trait)
///
/// # Returns
///
/// A `Result` containing the initialized `Gigi<R>` instance or an error.
///
/// # Database Initialization
///
/// The database is initialized asynchronously:
/// - Creates the gigi directory in the app data folder
/// - Connects to a SQLite database at `{app_data_dir}/gigi/gigi.db`
/// - Runs all pending migrations
/// - Initializes AuthManager, GroupManager, and ContactManager
/// - Notifies waiting threads when initialization is complete
pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Gigi<R>> {
    let state = PluginState::new();
    app.manage(state);

    // Initialize database connection and managers asynchronously
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = initialize_database_and_managers(&app_handle).await {
            tracing::error!("Failed to initialize database and managers: {}", e);
        }
    });

    Ok(Gigi(app.clone()))
}

/// Initializes the database connection and all store managers.
///
/// This is an internal function that performs the heavy lifting of setting up
/// the persistence layer. It runs in a background task to avoid blocking.
///
/// # Steps
///
/// 1. Get the app data directory from Tauri
/// 2. Create the gigi subdirectory if it doesn't exist
/// 3. Connect to the SQLite database (creates if it doesn't exist)
/// 4. Run all database migrations
/// 5. Initialize managers with the database connection
/// 6. Update the PluginState with the initialized components
/// 7. Notify waiting threads that initialization is complete
///
/// # Arguments
///
/// * `app` - A reference to the Tauri AppHandle
///
/// # Returns
///
/// A `Result` indicating success or failure.
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

    tracing::info!("Connecting to database at: {}", db_url);

    // Connect to database
    let db: DatabaseConnection = Database::connect(&db_url)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Run migrations
    tracing::info!("Running database migrations...");
    gigi_store::migration::Migrator::up(&db, None)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    tracing::info!("Database migrations completed");

    // Initialize AuthManager for user authentication
    let auth_manager = gigi_store::AuthManager::new(db.clone());

    // Initialize GroupManager for group chat management
    let group_manager = gigi_store::GroupManager::new(db.clone());

    // Initialize ContactManager for contact management
    let contact_manager = gigi_store::ContactManager::new(db.clone());

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

    let mut contact_manager_guard = state.contact_manager.lock().await;
    *contact_manager_guard = Some(contact_manager);
    drop(contact_manager_guard);

    // Notify that initialization is complete
    state.initialized.notify_one();

    tracing::info!("Database and managers initialized successfully");
    Ok(())
}

/// Access to the gigi APIs.
///
/// This struct provides access to the Gigi plugin's state and functionality.
/// It is managed by Tauri and can be accessed via the `GigiExt` trait.
pub struct Gigi<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Gigi<R> {
    /// Gets the plugin state.
    ///
    /// Returns a clone of the PluginState, which contains all the managers,
    /// stores, and configuration needed by the plugin.
    ///
    /// # Returns
    ///
    /// A cloned `PluginState` instance.
    pub fn get_state(&self) -> PluginState {
        self.0.state::<PluginState>().inner().clone()
    }
}
