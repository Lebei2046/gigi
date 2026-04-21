use anyhow::Result;
use gigi_p2p::{Keypair, P2pClient, P2pConfig, P2pEvent};
use futures_util::stream::StreamExt;
use once_cell::sync::Lazy;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use dirs;
use hex;
use crate::services::event_bus::{EventBus, AppEvent};

static P2P_CLIENT: Lazy<Arc<Mutex<Option<P2pClient>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

pub struct P2pService;

impl P2pService {
    pub async fn initialize(private_key: &str, nickname: &str) -> Result<()> {
        EventBus::init();
        // Create keypair from private key
        let keypair = Keypair::from_protobuf_encoding(
            &hex::decode(private_key)?
        )?;

        // Create output directory for downloads
        let data_dir = env::var("GIGI_DATA_DIR")
            .unwrap_or_else(|_| {
                dirs::data_local_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("gigi-dioxus")
                    .to_string_lossy()
                    .to_string()
            });
        
        let output_dir = PathBuf::from(data_dir).join("downloads");

        if let Some(parent) = output_dir.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create P2P config with bootstrap nodes
        let p2p_config = P2pConfig {
            bootstrap_nodes: vec![
                "/ip4/127.0.0.1/tcp/3000/p2p/12D3KooWJf7PzDq9J5vV78QyQ3X6q2n8X4Y7Z9W8U7V6T5R4E3W2Q1A".to_string(),
            ],
            ..Default::default()
        };

        // Create P2P client
        let (mut client, mut event_receiver) = P2pClient::new_with_config(
            keypair,
            nickname.to_string(),
            output_dir,
            p2p_config,
        )?;

        // Start listening
        client.start_listening("/ip4/0.0.0.0/tcp/0".parse()?)?;

        // Store client
        *P2P_CLIENT.lock().unwrap() = Some(client);

        // Start event handling loop
        tokio::spawn(async move {
            while let Some(event) = event_receiver.next().await {
                Self::handle_event(event).await;
            }
        });

        Ok(())
    }

    pub fn get_client() -> Result<Option<std::sync::MutexGuard<'static, Option<P2pClient>>>> {
        Ok(P2P_CLIENT.lock().ok())
    }

    async fn handle_event(event: P2pEvent) {
        let _ = EventBus::send(AppEvent::P2P(event.clone()));
        
        match event {
            P2pEvent::PeerDiscovered { peer_id, nickname, .. } => {
                println!("Discovered peer: {} ({})", nickname, peer_id);
            }
            P2pEvent::DirectMessage { from_nickname, message, .. } => {
                println!("Message from {}: {}", from_nickname, message);
                let _ = crate::services::persistence_service::PersistenceService::store_direct_message(
                    from_nickname,
                    "".to_string(),
                    message,
                    false,
                ).await;
            }
            P2pEvent::FileDownloadProgress { downloaded_chunks, total_chunks, filename, .. } => {
                let progress = (downloaded_chunks * 100) / total_chunks;
                println!("Downloading {}: {}%", filename, progress);
            }
            _ => {}
        }
    }

    pub async fn send_message(to_nickname: &str, message: &str) -> Result<()> {
        if let Some(mut client_guard) = Self::get_client()? {
            if let Some(client) = client_guard.as_mut() {
                client.send_direct_message(to_nickname, message.to_string())?;
            }
        }
        Ok(())
    }

    pub async fn send_group_message(group_name: &str, message: &str) -> Result<()> {
        if let Some(mut client_guard) = Self::get_client()? {
            if let Some(client) = client_guard.as_mut() {
                client.send_group_message(group_name, message.to_string())?;
            }
        }
        Ok(())
    }

    pub async fn join_group(group_name: &str) -> Result<()> {
        if let Some(mut client_guard) = Self::get_client()? {
            if let Some(client) = client_guard.as_mut() {
                client.join_group(group_name)?;
            }
        }
        Ok(())
    }

    pub async fn leave_group(group_name: &str) -> Result<()> {
        if let Some(mut client_guard) = Self::get_client()? {
            if let Some(client) = client_guard.as_mut() {
                client.leave_group(group_name)?;
            }
        }
        Ok(())
    }

    pub fn list_peers() -> Result<Vec<String>> {
        if let Some(client_guard) = Self::get_client()? {
            if let Some(client) = client_guard.as_ref() {
                let peers = client.list_peers();
                Ok(peers.iter().map(|p| p.nickname.clone()).collect())
            } else {
                Ok(vec![])
            }
        } else {
            Ok(vec![])
        }
    }

    pub async fn share_file(to_nickname: &str, file_path: &PathBuf) -> Result<String> {
        if let Some(mut client_guard) = Self::get_client()? {
            if let Some(client) = client_guard.as_mut() {
                let share_code = client.share_file(file_path).await?;
                client.send_direct_message(to_nickname, format!("File share: {}", share_code))?;
                return Ok(share_code);
            }
        }
        Err(anyhow::anyhow!("P2P client not initialized"))
    }

    pub async fn download_file(nickname: &str, share_code: &str) -> Result<String> {
        if let Some(mut client_guard) = Self::get_client()? {
            if let Some(client) = client_guard.as_mut() {
                let download_id = client.download_file(nickname, share_code)?;
                return Ok(download_id);
            }
        }
        Err(anyhow::anyhow!("P2P client not initialized"))
    }

    pub fn shutdown() -> Result<()> {
        if let Some(mut client_guard) = Self::get_client()? {
            if let Some(client) = client_guard.as_mut() {
                client.shutdown()?;
            }
            *client_guard = None;
        }
        Ok(())
    }
}
