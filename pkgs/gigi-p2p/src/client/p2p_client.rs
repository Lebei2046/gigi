//! Main P2P client implementation

use anyhow::Result;
use futures::channel::mpsc;
use gigi_dns::GigiDnsConfig;
use libp2p::{
    identity::Keypair,
    multiaddr::Multiaddr,
    request_response::{self, ProtocolSupport},
    swarm::SwarmEvent,
    PeerId, StreamProtocol,
};

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, instrument, warn};

use super::{
    download_manager::DownloadManager, event_handler::SwarmEventHandler,
    file_sharing::FileSharingManager, group_manager::GroupManager, peer_manager::PeerManager,
};
use crate::behaviour::{
    create_gossipsub_behaviour, create_gossipsub_config, DirectMessage, FileSharingRequest,
    UnifiedBehaviour, UnifiedEvent,
};
use crate::error::P2pError;
use crate::events::{ActiveDownload, GroupInfo, P2pEvent, PeerInfo};
use gigi_store::migration::MigratorTrait;
use gigi_store::{MessageStore, PersistenceConfig, SyncManager};

/// Main P2P client
pub struct P2pClient {
    pub(super) swarm: libp2p::swarm::Swarm<UnifiedBehaviour>,
    pub(super) local_nickname: String,

    // Peer management
    pub(super) peer_manager: PeerManager,

    // Group management
    pub(super) group_manager: GroupManager,

    // File sharing
    pub(super) file_manager: FileSharingManager,

    // Download management
    pub(super) download_manager: DownloadManager,

    // Event handling
    pub(super) event_sender: mpsc::UnboundedSender<P2pEvent>,

    // Persistence (optional)
    #[allow(dead_code)]
    pub(super) message_store: Option<Arc<MessageStore>>,
    #[allow(dead_code)]
    pub(super) sync_manager: Option<SyncManager>,
}

impl P2pClient {
    /// Create a new P2P client
    #[instrument(skip(keypair))]
    pub fn new(
        keypair: Keypair,
        nickname: String,
        output_directory: PathBuf,
    ) -> Result<(Self, mpsc::UnboundedReceiver<P2pEvent>)> {
        Self::new_with_config_and_persistence(keypair, nickname, output_directory, None)
    }

    /// Create a new P2P client with custom shared file path (deprecated, no longer used)
    #[instrument(skip(keypair))]
    #[deprecated(
        since = "0.0.1",
        note = "Use new() instead - shared files are now stored in gigi-store"
    )]
    pub fn new_with_config(
        keypair: Keypair,
        nickname: String,
        output_directory: PathBuf,
        _shared_file_path: Option<PathBuf>,
    ) -> Result<(Self, mpsc::UnboundedReceiver<P2pEvent>)> {
        Self::new_with_config_and_persistence(keypair, nickname, output_directory, None)
    }

    /// Create a new P2P client with persistence enabled
    #[instrument(skip(keypair))]
    pub fn new_with_config_and_persistence(
        keypair: Keypair,
        nickname: String,
        output_directory: PathBuf,
        persistence_config: Option<PersistenceConfig>,
    ) -> Result<(Self, mpsc::UnboundedReceiver<P2pEvent>)> {
        let (event_sender, event_receiver) = mpsc::unbounded();

        // Create gigi-dns config
        let dns_config = GigiDnsConfig {
            nickname: nickname.clone(),
            capabilities: vec!["chat".to_string(), "file_sharing".to_string()],
            ttl: Duration::from_secs(360),
            query_interval: Duration::from_secs(300),
            announce_interval: Duration::from_secs(15),
            cleanup_interval: Duration::from_secs(30),
            enable_ipv6: false,
            ..Default::default()
        };

        // Create gigi-dns behaviour
        let gigi_dns = gigi_dns::GigiDnsBehaviour::new(keypair.public().to_peer_id(), dns_config)?;

        // Create other behaviours
        let direct_msg = request_response::cbor::Behaviour::new(
            [(StreamProtocol::new("/direct/1.0.0"), ProtocolSupport::Full)],
            request_response::Config::default(),
        );

        let gossipsub_config = create_gossipsub_config(&keypair)
            .map_err(|e| anyhow::anyhow!("Failed to create gossipsub config: {}", e))?;
        let gossipsub = create_gossipsub_behaviour(keypair.clone(), gossipsub_config)?;

        let file_sharing = request_response::cbor::Behaviour::new(
            [(StreamProtocol::new("/file/1.0.0"), ProtocolSupport::Full)],
            request_response::Config::default(),
        );

        // Create unified behaviour
        let behaviour = UnifiedBehaviour {
            gigi_dns,
            direct_msg,
            gossipsub,
            file_sharing,
        };

        // Build swarm
        let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default(),
                libp2p::noise::Config::new,
                libp2p::yamux::Config::default,
            )?
            .with_behaviour(|_| behaviour)?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(300)))
            .build();

        let file_manager = FileSharingManager::new();
        let download_manager = DownloadManager::new(output_directory);

        // Initialize persistence if config provided
        let (message_store, sync_manager, file_sharing_store) = if let Some(config) =
            persistence_config
        {
            let store = Arc::new(tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { MessageStore::new(config.db_path.clone()).await })
            })?);
            let sync_state_path = config.db_path.with_extension("sync");
            let sync = SyncManager::new(store.clone(), nickname.clone(), sync_state_path);

            // Create file sharing store using the same database
            let db_conn = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    sea_orm::Database::connect(format!(
                        "sqlite://{}?mode=rwc",
                        config.db_path.display()
                    ))
                    .await
                })
            })?;
            // Run migrations to ensure shared_files table exists
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { gigi_store::migration::Migrator::up(&db_conn, None).await })
            })?;
            let file_store = Arc::new(tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { gigi_store::FileSharingStore::new(db_conn).await })
            })?);

            (Some(store), Some(sync), Some(file_store))
        } else {
            (None, None, None)
        };

        // Attach file sharing store to file manager if available
        let file_manager = match &file_sharing_store {
            Some(store) => file_manager.with_store(Arc::clone(store)),
            None => file_manager,
        };

        let mut client = Self {
            swarm,
            local_nickname: nickname,
            peer_manager: PeerManager::new(),
            group_manager: GroupManager::new(),
            file_manager,
            download_manager,
            event_sender,
            message_store,
            sync_manager,
        };

        // Load existing shared files from store if available
        if file_sharing_store.is_some() {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { client.file_manager.load_from_store().await })
            })?;
        }

        Ok((client, event_receiver))
    }

    /// Start listening on given address
    pub fn start_listening(&mut self, addr: Multiaddr) -> Result<()> {
        self.swarm
            .listen_on(addr)
            .map_err(|e| P2pError::NetworkError(e.to_string()))?;
        Ok(())
    }

    /// Handle the next swarm event (convenient method)
    pub async fn handle_next_swarm_event(&mut self) -> Result<()> {
        use futures::StreamExt;
        let event = self.swarm.select_next_some().await;
        self.handle_event(event)?;
        Ok(())
    }

    /// Handle a single swarm event
    fn handle_event(&mut self, event: SwarmEvent<UnifiedEvent>) -> Result<()> {
        SwarmEventHandler::new(self).handle_event(event)
    }

    /// Get peer by nickname
    pub fn get_peer_by_nickname(&self, nickname: &str) -> Result<&PeerInfo> {
        self.peer_manager.get_peer_by_nickname(nickname)
    }

    /// Get peer nickname
    pub fn get_peer_nickname(&self, peer_id: &PeerId) -> Result<String> {
        self.peer_manager.get_peer_nickname(peer_id)
    }

    /// Get peer info
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<&PeerInfo> {
        self.peer_manager.get_peer(peer_id)
    }

    /// Get peer ID by nickname
    pub fn get_peer_id_by_nickname(&self, nickname: &str) -> Option<PeerId> {
        self.peer_manager.get_peer_id_by_nickname(nickname)
    }

    /// Remove a peer from the peer list
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.peer_manager.remove_peer(peer_id);
    }

    /// Gracefully shutdown the client and notify all peers
    pub fn shutdown(&mut self) -> Result<()> {
        self.peer_manager.shutdown(&mut self.event_sender)
    }

    /// List all discovered peers
    pub fn list_peers(&self) -> Vec<&PeerInfo> {
        self.peer_manager.list_peers()
    }

    /// Get all connected peers
    pub fn get_connected_peers(&self) -> Vec<&PeerInfo> {
        self.peer_manager.get_connected_peers()
    }

    /// Get peers count
    pub fn peers_count(&self) -> usize {
        self.peer_manager.peers_count()
    }

    /// Get connected peers count
    pub fn connected_peers_count(&self) -> usize {
        self.peer_manager.connected_peers_count()
    }

    /// Send persistent message to peer
    pub async fn send_persistent_message(&mut self, nickname: &str, message: String) -> Result<()> {
        let peer_id = self.peer_manager.get_peer_id_by_nickname(nickname);

        match peer_id {
            Some(peer_id) => {
                // Peer is online, store message and send via P2P
                if let Some(ref message_store) = self.message_store {
                    use gigi_store::{
                        MessageContent, MessageDirection, MessageType, StoredMessage, SyncStatus,
                    };
                    let peer_id_str = peer_id.to_string();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            let stored_msg = StoredMessage {
                                id: uuid::Uuid::new_v4().to_string(),
                                msg_type: MessageType::Direct,
                                direction: MessageDirection::Sent,
                                content: MessageContent::Text {
                                    text: message.clone(),
                                },
                                sender_nickname: self.local_nickname.clone(),
                                recipient_nickname: Some(nickname.to_string()),
                                group_name: None,
                                peer_id: peer_id_str.clone(),
                                timestamp: chrono::Utc::now(),
                                created_at: chrono::Utc::now(),
                                delivered: false,
                                delivered_at: None,
                                read: false,
                                read_at: None,
                                sync_status: SyncStatus::Pending,
                                sync_attempts: 0,
                                last_sync_attempt: None,
                                expires_at: chrono::Utc::now() + chrono::Duration::days(7),
                            };
                            message_store.store_message(stored_msg).await
                        })
                    })
                    .map_err(|e| anyhow::anyhow!("Failed to store message: {}", e))?;
                }

                // Send the message via P2P
                self.swarm
                    .behaviour_mut()
                    .direct_msg
                    .send_request(&peer_id, DirectMessage::Text { message });

                Ok(())
            }
            None => {
                // Peer is offline, store message for later delivery
                if let Some(message_store) = &self.message_store {
                    use gigi_store::{MessageContent, MessageDirection, MessageType, SyncStatus};

                    // Create stored message
                    let message_id = uuid::Uuid::new_v4().to_string();
                    let stored_msg = gigi_store::StoredMessage {
                        id: message_id.clone(),
                        msg_type: MessageType::Direct,
                        direction: MessageDirection::Sent,
                        content: MessageContent::Text {
                            text: message.clone(),
                        },
                        sender_nickname: self.local_nickname.clone(),
                        recipient_nickname: Some(nickname.to_string()),
                        group_name: None,
                        peer_id: String::new(), // Empty string since we don't know peer_id
                        timestamp: chrono::Utc::now(),
                        created_at: chrono::Utc::now(),
                        delivered: false,
                        delivered_at: None,
                        read: false,
                        read_at: None,
                        sync_status: SyncStatus::Pending,
                        sync_attempts: 0,
                        last_sync_attempt: None,
                        expires_at: chrono::Utc::now() + chrono::Duration::days(7),
                    };

                    // Store message and add to offline queue
                    let msg_clone = message_store.clone();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            msg_clone.store_message(stored_msg).await?;
                            msg_clone
                                .enqueue_offline(message_id, nickname.to_string())
                                .await?;
                            Ok::<(), anyhow::Error>(())
                        })
                    })?;
                }

                Err(anyhow::anyhow!(
                    "Peer '{}' is not online. Message saved for later delivery.",
                    nickname
                ))
            }
        }
    }

    /// Send direct message to peer
    pub fn send_direct_message(&mut self, nickname: &str, message: String) -> Result<()> {
        let peer_id = self.peer_manager.get_peer_id_by_nickname(nickname);

        match peer_id {
            Some(peer_id) => {
                // Peer is online, send immediately
                self.swarm
                    .behaviour_mut()
                    .direct_msg
                    .send_request(&peer_id, DirectMessage::Text { message });
                Ok(())
            }
            None => {
                // Peer is offline, store message for later delivery
                if let Some(message_store) = &self.message_store {
                    use chrono::Utc;
                    use gigi_store::MessageContent;
                    use gigi_store::MessageDirection;

                    // Create stored message
                    let message_id = uuid::Uuid::new_v4().to_string();
                    let stored_msg = gigi_store::StoredMessage {
                        id: message_id.clone(),
                        msg_type: gigi_store::MessageType::Direct,
                        direction: MessageDirection::Sent,
                        content: MessageContent::Text {
                            text: message.clone(),
                        },
                        sender_nickname: self.local_nickname.clone(),
                        recipient_nickname: Some(nickname.to_string()),
                        group_name: None,
                        peer_id: String::new(), // Empty string since we don't know peer_id
                        timestamp: Utc::now(),
                        created_at: Utc::now(),

                        delivered: false,
                        delivered_at: None,

                        read: false,
                        read_at: None,

                        sync_status: gigi_store::SyncStatus::Pending,
                        sync_attempts: 0,
                        last_sync_attempt: None,

                        expires_at: Utc::now() + chrono::Duration::days(7), // Expire after 7 days
                    };

                    // Store message and add to offline queue
                    let msg_clone = message_store.clone();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            msg_clone.store_message(stored_msg).await?;
                            msg_clone
                                .enqueue_offline(message_id, nickname.to_string())
                                .await?;
                            Ok::<(), anyhow::Error>(())
                        })
                    })?;
                }

                Err(anyhow::anyhow!(
                    "Peer '{}' is not online. Message saved for later delivery.",
                    nickname
                ))
            }
        }
    }

    /// Send file to peer using file sharing
    pub async fn send_direct_file(&mut self, nickname: &str, file_path: &Path) -> Result<()> {
        let peer_id = self
            .peer_manager
            .get_peer_id_by_nickname(nickname)
            .ok_or_else(|| P2pError::NicknameNotFound(nickname.to_string()))?;

        // 1. Add file to file sharing system
        let share_code = self.file_manager.share_file(file_path).await?;
        let shared_file = self
            .file_manager
            .shared_files
            .get(&share_code)
            .ok_or_else(|| P2pError::FileNotFound(file_path.to_path_buf()))?;

        // 2. Detect file type
        let file_type = mime_guess::from_path(file_path)
            .first_or_octet_stream()
            .to_string();

        // 3. Send share code instead of raw data
        self.swarm.behaviour_mut().direct_msg.send_request(
            &peer_id,
            DirectMessage::FileShare {
                share_code: shared_file.share_code.clone(),
                filename: shared_file.info.name.clone(),
                file_size: shared_file.info.size,
                file_type,
            },
        );

        Ok(())
    }

    /// Send group share message to peer
    pub fn send_direct_share_group_message(
        &mut self,
        nickname: &str,
        group_id: String,
        group_name: String,
    ) -> Result<()> {
        let peer_id = self
            .peer_manager
            .get_peer_id_by_nickname(nickname)
            .ok_or_else(|| P2pError::NicknameNotFound(nickname.to_string()))?;

        self.swarm.behaviour_mut().direct_msg.send_request(
            &peer_id,
            DirectMessage::ShareGroup {
                group_id,
                group_name,
                inviter_nickname: self.local_nickname.clone(),
            },
        );

        Ok(())
    }

    /// Join a group
    pub fn join_group(&mut self, group_name: &str) -> Result<()> {
        self.group_manager
            .join_group(&mut self.swarm, group_name, &mut self.event_sender)
    }

    /// Leave a group
    pub fn leave_group(&mut self, group_name: &str) -> Result<()> {
        self.group_manager.leave_group(&mut self.swarm, group_name)
    }

    /// Send message to group
    pub fn send_group_message(&mut self, group_name: &str, message: String) -> Result<()> {
        self.group_manager.send_group_message(
            &mut self.swarm,
            group_name,
            message,
            &self.local_nickname,
        )
    }

    /// Send file to group using file sharing
    pub async fn send_group_file(&mut self, group_name: &str, file_path: &Path) -> Result<()> {
        self.group_manager
            .send_group_file(
                &mut self.swarm,
                group_name,
                file_path,
                &mut self.file_manager,
                &self.local_nickname,
            )
            .await
    }

    /// Share a file
    pub async fn share_file(&mut self, file_path: &Path) -> Result<String> {
        self.file_manager.share_file(file_path).await
    }

    /// Set the chunk reader callback for URI-based files
    pub fn set_chunk_reader(&mut self, reader: super::file_sharing::FileChunkReader) {
        self.file_manager.set_chunk_reader(reader.clone());
        self.download_manager.set_chunk_reader(reader);
    }

    /// Share a content URI (Android content:// or iOS file://)
    pub async fn share_content_uri(&mut self, uri: &str, name: &str, size: u64) -> Result<String> {
        self.file_manager.share_content_uri(uri, name, size).await
    }

    /// List shared files
    pub fn list_shared_files(&self) -> Vec<&crate::events::SharedFile> {
        self.file_manager.list_shared_files()
    }

    /// Unshare a file by share code
    pub fn unshare_file(&mut self, share_code: &str) -> Result<()> {
        let shared_file = self.file_manager.shared_files.get(share_code);
        if let Some(shared_file) = shared_file {
            // Send event that file is revoked
            self.send_event(P2pEvent::FileRevoked {
                file_id: shared_file.info.id.clone(),
            });

            // Save updated shared files
            self.file_manager.unshare_file(share_code)?;
        } else {
            return Err(P2pError::InvalidShareCode(share_code.to_string()).into());
        }
        Ok(())
    }

    /// Download file from peer
    pub fn download_file(&mut self, nickname: &str, share_code: &str) -> Result<String> {
        let peer_id = self
            .peer_manager
            .get_peer_id_by_nickname(nickname)
            .ok_or_else(|| P2pError::NicknameNotFound(nickname.to_string()))?;

        // Track download request with DownloadManager and get the download_id
        let download_id = self.download_manager.start_download(
            peer_id,
            nickname.to_string(),
            share_code.to_string(),
            None, // filename will be updated when file info arrives
        );

        // First request file info
        let request_id = self.swarm.behaviour_mut().file_sharing.send_request(
            &peer_id,
            FileSharingRequest::GetFileInfo(share_code.to_string()),
        );

        // Map request_id to download_id so we can match the response
        self.download_manager
            .map_request_to_download(request_id.to_string(), download_id.clone());

        Ok(download_id)
    }

    /// Send event to event receiver
    pub fn send_event(&self, event: P2pEvent) {
        if let Err(e) = self.event_sender.unbounded_send(event) {
            error!("Failed to send P2P event: {}", e);
        }
    }

    /// Get local peer ID
    pub fn local_peer_id(&self) -> PeerId {
        *self.swarm.local_peer_id()
    }

    /// Get local nickname
    pub fn local_nickname(&self) -> &str {
        &self.local_nickname
    }

    /// Get joined groups
    pub fn list_groups(&self) -> Vec<&GroupInfo> {
        self.group_manager.list_groups()
    }

    // ===== Active Download Tracking Methods for Mobile Apps =====

    /// Get all active downloads
    pub fn get_active_downloads(&self) -> Vec<&ActiveDownload> {
        self.download_manager.get_active_downloads()
    }

    /// Get active download by download_id
    pub fn get_active_download(&self, download_id: &str) -> Option<&ActiveDownload> {
        self.download_manager.get_active_download(download_id)
    }

    /// Get active download by share code
    pub fn get_download_by_share_code(&self, share_code: &str) -> Option<&ActiveDownload> {
        self.download_manager.get_download_by_share_code(share_code)
    }

    /// Remove completed or failed downloads (cleanup)
    pub fn cleanup_downloads(&mut self) {
        self.download_manager.cleanup_downloads();
    }

    /// Helper to find download_id by file_id
    pub fn find_download_id_by_file_id(&self, file_id: &str) -> Option<String> {
        self.download_manager.find_download_id_by_file_id(file_id)
    }

    /// Get downloads from a specific peer
    pub fn get_downloads_from_peer(&self, peer_nickname: &str) -> Vec<&ActiveDownload> {
        self.download_manager.get_downloads_from_peer(peer_nickname)
    }

    /// Get completed downloads (useful for UI history)
    pub fn get_recent_downloads(&self, limit: usize) -> Vec<&ActiveDownload> {
        self.download_manager.get_recent_downloads(limit)
    }

    // ===== Persistence Methods =====

    /// Check if persistence is enabled
    #[allow(dead_code)]
    pub fn is_persistence_enabled(&self) -> bool {
        self.message_store.is_some()
    }

    /// Get conversation history with a peer
    pub async fn get_conversation_history(
        &self,
        nickname: &str,
    ) -> Result<Vec<gigi_store::StoredMessage>> {
        let message_store = self
            .message_store
            .as_ref()
            .ok_or_else(|| P2pError::PersistenceNotEnabled)?;

        // We don't require the peer to be online to view history
        // Just check if there are any messages in the database
        message_store
            .get_conversation(nickname, 100, 0)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get conversation history: {}", e))
    }

    /// Mark a message as read
    pub async fn mark_message_as_read(&self, message_id: &str) -> Result<()> {
        let message_store = self
            .message_store
            .as_ref()
            .ok_or_else(|| P2pError::PersistenceNotEnabled)?;

        message_store
            .mark_read(message_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to mark message as read: {}", e))
    }

    /// Mark all messages in a conversation as read
    pub async fn mark_conversation_read(&self, nickname: &str) -> Result<()> {
        let message_store = self
            .message_store
            .as_ref()
            .ok_or_else(|| P2pError::PersistenceNotEnabled)?;

        message_store
            .mark_conversation_read(nickname)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to mark conversation as read: {}", e))
    }

    /// Mark a message as delivered
    pub async fn mark_message_as_delivered(&self, message_id: &str) -> Result<()> {
        let message_store = self
            .message_store
            .as_ref()
            .ok_or_else(|| P2pError::PersistenceNotEnabled)?;

        message_store
            .mark_delivered(message_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to mark message as delivered: {}", e))
    }

    /// Store a message
    pub async fn store_message(&self, msg: gigi_store::StoredMessage) -> Result<()> {
        let message_store = self
            .message_store
            .as_ref()
            .ok_or_else(|| P2pError::PersistenceNotEnabled)?;

        message_store
            .store_message(msg)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to store message: {}", e))
    }

    /// Get unread message count for a peer
    #[allow(dead_code)]
    pub async fn get_unread_count(&self, peer_nickname: &str) -> Result<u64> {
        let sync_manager = self
            .sync_manager
            .as_ref()
            .ok_or_else(|| P2pError::PersistenceNotEnabled)?;

        sync_manager
            .get_unread_count(peer_nickname)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get unread count: {}", e))
    }

    /// Send pending messages to a peer that just came online
    pub async fn send_pending_messages(&mut self, nickname: &str) -> Result<usize> {
        let message_store = self
            .message_store
            .as_ref()
            .ok_or_else(|| P2pError::PersistenceNotEnabled)?;

        // Get the peer_id for this nickname
        let peer_id = self
            .peer_manager
            .get_peer_id_by_nickname(nickname)
            .ok_or_else(|| anyhow::anyhow!("Peer not found: {}", nickname))?;

        // Get pending messages
        let pending = message_store
            .get_pending_messages(nickname, 100)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get pending messages: {}", e))?;

        if pending.is_empty() {
            return Ok(0);
        }

        // Send each pending message
        let mut sent_count = 0;
        for msg in pending {
            if let gigi_store::MessageContent::Text { text } = msg.content {
                self.swarm.behaviour_mut().direct_msg.send_request(
                    &peer_id,
                    crate::behaviour::DirectMessage::Text { message: text },
                );

                // Update peer_id if it was empty
                if msg.peer_id.is_empty() {
                    let _ = message_store
                        .update_message_peer_id(&msg.id, peer_id.to_string())
                        .await;
                }

                sent_count += 1;
            }
        }

        Ok(sent_count)
    }
}
