use futures::StreamExt;
use tauri::{Emitter, Manager, State};

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

/// Login with password and initialize P2P client in one step
#[tauri::command]
pub(crate) async fn auth_login_with_p2p<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    password: String,
    state: State<'_, PluginState>,
) -> Result<gigi_store::AccountInfo> {
    use gigi_p2p::P2pClient;
    use hex;

    wait_for_initialization(&state).await?;

    // Get auth manager and login
    let auth_manager = state.auth_manager.lock().await;
    let manager = auth_manager
        .as_ref()
        .ok_or_else(|| Error::CommandFailed("Auth manager not initialized".to_string()))?;

    let login_result = manager
        .login(&password)
        .await
        .map_err(|e| Error::Io(format!("Login failed: {}", e)))?;

    let account_info = login_result.account_info.clone();
    let private_key_hex = login_result.private_key;
    drop(auth_manager);

    // Update nickname in config before creating P2P client
    let nickname = account_info.name.clone();
    {
        let mut config_guard = state.config.write().await;
        config_guard.nickname = nickname.clone();
    }

    // Update config with download directory
    let download_dir = app
        .path()
        .download_dir()
        .map_err(|e| Error::Io(format!("Failed to get app data directory: {}", e)))?;

    #[cfg(target_os = "android")]
    let gigi_download_dir = download_dir;

    #[cfg(not(target_os = "android"))]
    let gigi_download_dir = download_dir.join("gigi");

    let mut config_guard = state.config.write().await;
    config_guard.download_folder = gigi_download_dir.to_string_lossy().to_string();
    drop(config_guard);

    // Get config for download directory
    let config_guard = state.config.read().await;
    let output_dir = std::path::PathBuf::from(&config_guard.download_folder);
    let final_nickname = config_guard.nickname.clone();
    drop(config_guard);

    // Create downloads directory at runtime
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| Error::Io(format!("Failed to create download directory: {}", e)))?;

    // Get unified database path for P2P client
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| Error::Io(format!("Failed to get app data dir: {}", e)))?;
    let db_path = app_data_dir.join("gigi/gigi.db");

    // Create persistence config
    let persistence_config = gigi_store::PersistenceConfig {
        db_path,
        ..Default::default()
    };

    // Create P2P keypair from private key
    let private_key_bytes = hex::decode(&private_key_hex)
        .map_err(|e| Error::CommandFailed(format!("Failed to decode private key hex: {}", e)))?;
    let keypair = libp2p::identity::Keypair::ed25519_from_bytes(private_key_bytes)
        .map_err(|e| Error::CommandFailed(format!("Failed to create keypair: {}", e)))?;

    let peer_id = keypair.public().to_peer_id();

    // Capture public key hex before moving keypair
    let public_key_hex = hex::encode(
        keypair
            .to_protobuf_encoding()
            .map_err(|e| Error::CommandFailed(format!("Failed to encode public key: {}", e)))?,
    );

    // Create P2P client
    match P2pClient::new_with_config_and_persistence(
        keypair,
        final_nickname,
        output_dir,
        Some(persistence_config),
    ) {
        Ok((mut client, event_receiver)) => {
            // Use existing database connection
            let db_conn = {
                let conn_guard = state.db_connection.read().await;
                conn_guard
                    .as_ref()
                    .ok_or_else(|| Error::Io("Database not initialized".to_string()))?
                    .clone()
            };

            // Initialize stores
            let file_sharing_store = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { gigi_store::FileSharingStore::new(db_conn.clone()).await })
            })
            .map_err(|e| Error::Io(format!("Failed to create file sharing store: {}", e)))?;

            let mut file_sharing_store_guard = state.file_sharing_store.write().await;
            *file_sharing_store_guard = Some(file_sharing_store);
            drop(file_sharing_store_guard);

            let thumbnail_store = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { gigi_store::ThumbnailStore::new(db_conn.clone()).await })
            })
            .map_err(|e| Error::Io(format!("Failed to create thumbnail store: {}", e)))?;

            let mut thumbnail_store_guard = state.thumbnail_store.write().await;
            *thumbnail_store_guard = Some(thumbnail_store);
            drop(thumbnail_store_guard);

            let message_store = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    gigi_store::MessageStore::with_connection(db_conn.clone()).await
                })
            })
            .map_err(|e| Error::Io(format!("Failed to create message store: {}", e)))?;

            let mut message_store_guard = state.message_store.write().await;
            *message_store_guard = Some(message_store);
            drop(message_store_guard);

            let conversation_store = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    gigi_store::ConversationStore::with_connection(db_conn.clone()).await
                })
            })
            .map_err(|e| Error::Io(format!("Failed to create conversation store: {}", e)))?;

            let mut conversation_store_guard = state.conversation_store.write().await;
            *conversation_store_guard = Some(conversation_store);
            drop(conversation_store_guard);

            // Start listening
            let addr = "/ip4/0.0.0.0/tcp/0"
                .parse()
                .map_err(|e| Error::P2p(format!("Failed to parse address: {}", e)))?;
            if let Err(e) = client.start_listening(addr) {
                return Err(Error::P2p(format!("Failed to start listening: {}", e)));
            }

            // Store client and receiver
            let mut client_guard = state.p2p_client.lock().await;
            *client_guard = Some(client);
            drop(client_guard);

            let mut receiver_guard = state.event_receiver.lock().await;
            *receiver_guard = Some(event_receiver);
            drop(receiver_guard);

            // Setup file sharing chunk reader for Android
            #[cfg(target_os = "android")]
            {
                use gigi_p2p::events::FilePath;
                let app_handle = app.clone();

                let chunk_reader = std::sync::Arc::new({
                    let app_handle = app_handle.clone();
                    move |file_path: &FilePath,
                          offset: u64,
                          size: usize|
                          -> Result<Vec<u8>, String> {
                        match file_path {
                            FilePath::Path(path) => {
                                use std::io::{Read, Seek};
                                let mut file = std::fs::File::open(path)
                                    .map_err(|e| format!("Failed to open file: {}", e))?;
                                file.seek(std::io::SeekFrom::Start(offset))
                                    .map_err(|e| format!("Failed to seek file: {}", e))?;
                                let mut buffer = vec![0u8; size];
                                let bytes_read = file
                                    .read(&mut buffer)
                                    .map_err(|e| format!("Failed to read file: {}", e))?;
                                buffer.truncate(bytes_read);
                                Ok(buffer)
                            }
                            FilePath::Url(url) => {
                                use tauri_plugin_android_fs::{
                                    AndroidFsExt, FileAccessMode, FileUri,
                                };
                                use tauri_plugin_fs::FilePath as TauriFilePath;

                                let android_api = app_handle.android_fs();
                                let file_uri = FileUri::from(TauriFilePath::Url(url.clone()));

                                match android_api.open_file(&file_uri, FileAccessMode::Read) {
                                    Ok(mut file) => {
                                        use std::io::{Read, Seek};
                                        file.seek(std::io::SeekFrom::Start(offset))
                                            .map_err(|e| format!("Failed to seek file: {}", e))?;
                                        let mut buffer = vec![0u8; size];
                                        let bytes_read = file
                                            .read(&mut buffer)
                                            .map_err(|e| format!("Failed to read file: {}", e))?;
                                        buffer.truncate(bytes_read);
                                        Ok(buffer)
                                    }
                                    Err(e) => Err(format!("Failed to open content URI: {}", e)),
                                }
                            }
                        }
                    }
                });

                let mut client_guard = state.p2p_client.lock().await;
                if let Some(ref mut client) = *client_guard {
                    client.set_chunk_reader(chunk_reader);
                }
            }

            // Start event handling loop
            let p2p_client = state.p2p_client.clone();
            let app_handle_clone = app.clone();
            let receiver = {
                let mut guard = state.event_receiver.lock().await;
                guard.take().unwrap()
            };

            // Task 1: Poll swarm events
            let p2p_client_for_events = p2p_client.clone();
            tokio::spawn(async move {
                loop {
                    let client_ready = {
                        let client_guard = p2p_client_for_events.lock().await;
                        client_guard.as_ref().is_some()
                    };

                    if client_ready {
                        let result =
                            tokio::time::timeout(tokio::time::Duration::from_millis(100), async {
                                let mut client_guard = p2p_client_for_events.lock().await;
                                if let Some(client) = client_guard.as_mut() {
                                    client.handle_next_swarm_event().await
                                } else {
                                    Ok(())
                                }
                            })
                            .await;

                        if let Err(e) = result {
                            if !matches!(e, tokio::time::error::Elapsed { .. }) {
                                log::error!("Error handling swarm event: {:?}", e);
                            }
                        }
                    } else {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            });

            // Task 2: Handle P2P events
            let active_downloads = state.active_downloads.clone();
            let state_for_events = (*state).clone();
            tokio::spawn(async move {
                let mut receiver = receiver;
                while let Some(event) = receiver.next().await {
                    if let Err(e) = crate::events::handle_p2p_event(
                        event,
                        &p2p_client,
                        &active_downloads,
                        &app_handle_clone,
                        &state_for_events,
                    )
                    .await
                    {
                        log::error!("Failed to handle P2P event: {}", e);
                    }
                }
            });

            // Emit events
            app.emit("peer-id-changed", &peer_id.to_string())
                .map_err(|e| {
                    Error::CommandFailed(format!("Failed to emit peer-id-changed: {}", e))
                })?;
            app.emit("public-key-changed", &public_key_hex)
                .map_err(|e| {
                    Error::CommandFailed(format!("Failed to emit public-key-changed: {}", e))
                })?;
            app.emit("nickname-changed", &nickname).map_err(|e| {
                Error::CommandFailed(format!("Failed to emit nickname-changed: {}", e))
            })?;

            Ok(account_info)
        }
        Err(e) => Err(Error::P2p(format!("Failed to create P2P client: {}", e))),
    }
}
