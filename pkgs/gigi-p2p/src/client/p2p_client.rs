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
use tracing::{error, info, instrument, warn};

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
///
/// This is the primary entry point for the gigi-p2p library. It provides:
/// - Peer discovery and management via GigiDns
/// - Direct messaging between peers
/// - Group messaging via GossipSub pub-sub protocol
/// - File sharing with chunked transfer and hash verification
/// - Download tracking for mobile applications
/// - Optional message persistence for offline messaging
///
/// The client uses libp2p as the underlying networking framework and combines
/// multiple behaviours into a unified P2P swarm.
pub struct P2pClient {
    /// The libp2p swarm that manages all network connections and behaviours
    pub(super) swarm: libp2p::swarm::Swarm<UnifiedBehaviour>,
    /// Local peer's nickname for display purposes
    pub(super) local_nickname: String,

    // Peer management
    /// Manages peer discovery, nickname resolution, and connection tracking
    /// Maintains dual mapping between PeerId and nickname for quick lookups
    pub(super) peer_manager: PeerManager,

    // Group management
    /// Manages GossipSub group subscriptions and message broadcasting
    /// Handles group join/leave operations and group message distribution
    pub(super) group_manager: GroupManager,

    // File sharing
    /// Manages shared files, generates share codes, and handles chunked file transfers
    /// Supports both local files and mobile content URIs (Android content://, iOS file://)
    pub(super) file_manager: FileSharingManager,

    // Download management
    /// Tracks active downloads for mobile UI integration
    /// Provides progress updates and download state management
    pub(super) download_manager: DownloadManager,

    // Event handling
    /// Channel for sending P2P events to the application layer
    /// Applications receive events through the corresponding receiver
    pub(super) event_sender: mpsc::UnboundedSender<P2pEvent>,

    // Persistence (optional)
    /// Optional message store for offline messaging and conversation history
    /// When enabled, messages are persisted to SQLite database
    #[allow(dead_code)]
    pub(super) message_store: Option<Arc<MessageStore>>,
    /// Optional sync manager for handling offline message delivery
    /// Manages message synchronization when peers come back online
    #[allow(dead_code)]
    pub(super) sync_manager: Option<SyncManager>,
}

impl P2pClient {
    /// Create a new P2P client
    ///
    /// Creates a P2P client with all behaviours enabled but without persistence.
    /// Use this for scenarios where message persistence is not required.
    ///
    /// # Arguments
    /// * `keypair` - The cryptographic keypair for this peer's identity
    /// * `nickname` - Display name for this peer in the network
    /// * `output_directory` - Directory where downloaded files will be saved
    ///
    /// # Returns
    /// A tuple containing the P2pClient instance and an event receiver
    ///
    /// # Example
    /// ```rust
    /// use gigi_p2p::{Keypair, P2pClient};
    /// use std::path::PathBuf;
    /// # async fn example() -> anyhow::Result<()> {
    /// let keypair = Keypair::generate_ed25519();
    /// let (client, mut event_rx) = P2pClient::new(
    ///     keypair,
    ///     "alice".to_string(),
    ///     PathBuf::from("./downloads"),
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(keypair))]
    pub fn new(
        keypair: Keypair,
        nickname: String,
        output_directory: PathBuf,
    ) -> Result<(Self, mpsc::UnboundedReceiver<P2pEvent>)> {
        Self::new_with_config_and_persistence(keypair, nickname, output_directory, None)
    }

    /// Create a new P2P client with persistence enabled
    ///
    /// Creates a P2P client with optional message persistence for offline messaging.
    /// When persistence is enabled, messages are stored in SQLite and can be
    /// delivered when peers come back online.
    ///
    /// # Arguments
    /// * `keypair` - The cryptographic keypair for this peer's identity
    /// * `nickname` - Display name for this peer in the network
    /// * `output_directory` - Directory where downloaded files will be saved
    /// * `persistence_config` - Optional configuration for message persistence
    ///
    /// # Returns
    /// A tuple containing the P2pClient instance and an event receiver
    #[instrument(skip(keypair))]
    pub fn new_with_config_and_persistence(
        keypair: Keypair,
        nickname: String,
        output_directory: PathBuf,
        persistence_config: Option<PersistenceConfig>,
    ) -> Result<(Self, mpsc::UnboundedReceiver<P2pEvent>)> {
        let (event_sender, event_receiver) = mpsc::unbounded();

        // Create gigi-dns config
        // GigiDns enables peer discovery through a distributed DNS-like service
        // Peers announce their presence with nicknames and capabilities
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
        // Direct messaging: 1:1 peer-to-peer communication using request/response pattern
        let direct_msg = request_response::cbor::Behaviour::new(
            [(StreamProtocol::new("/direct/1.0.0"), ProtocolSupport::Full)],
            request_response::Config::default(),
        );

        // GossipSub: pub-sub protocol for group messaging
        // Messages are propagated through the mesh network with message deduplication
        let gossipsub_config = create_gossipsub_config(&keypair)
            .map_err(|e| anyhow::anyhow!("Failed to create gossipsub config: {}", e))?;
        let gossipsub = create_gossipsub_behaviour(keypair.clone(), gossipsub_config)?;

        // File sharing: request/response protocol for chunked file transfers
        // Files are split into chunks, transferred sequentially, and verified with BLAKE3 hashes
        let file_sharing = request_response::cbor::Behaviour::new(
            [(StreamProtocol::new("/file/1.0.0"), ProtocolSupport::Full)],
            request_response::Config::default(),
        );

        // Create unified behaviour
        // Combines all protocols into a single libp2p behaviour
        // Each protocol handles its own events and message types
        let behaviour = UnifiedBehaviour {
            gigi_dns,
            direct_msg,
            gossipsub,
            file_sharing,
        };

        // Build swarm
        // Configure transport: TCP + Noise (encryption) + Yamux (multiplexing)
        let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default(),
                libp2p::noise::Config::new, // Transport encryption using Noise protocol
                libp2p::yamux::Config::default, // Stream multiplexing
            )?
            .with_behaviour(|_| behaviour)?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(300)))
            .build();

        // Log peer ID when swarm starts
        info!("P2pClient started with peer ID: {}", swarm.local_peer_id());

        let file_manager = FileSharingManager::new();
        let download_manager = DownloadManager::new(output_directory);

        // Initialize persistence if config provided
        // This enables offline messaging, conversation history, and shared file persistence
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
            // Shared files are persisted so they remain available after app restart
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
        // This allows shared files to be restored after app restart
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
    ///
    /// Begins accepting incoming connections on the specified address.
    /// Call this after creating the client to make it discoverable by other peers.
    ///
    /// # Arguments
    /// * `addr` - The multiaddr to listen on (e.g., "/ip4/0.0.0.0/tcp/0")
    ///
    /// # Example
    /// ```rust,ignore
    /// client.start_listening("/ip4/0.0.0.0/tcp/0".parse()?)?;
    /// ```
    pub fn start_listening(&mut self, addr: Multiaddr) -> Result<()> {
        self.swarm
            .listen_on(addr)
            .map_err(|e| P2pError::NetworkError(e.to_string()))?;
        Ok(())
    }

    /// Handle the next swarm event (convenient method)
    ///
    /// Waits for and processes the next event from the libp2p swarm.
    /// This is the main event loop for P2P networking.
    ///
    /// # Example
    /// ```rust,ignore
    /// loop {
    ///     client.handle_next_swarm_event().await?;
    /// }
    /// ```
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

    // ===== Peer Management Methods =====
    // These methods provide peer discovery, lookup, and management functionality

    /// Get peer by nickname
    ///
    /// Retrieves peer information by their display nickname.
    ///
    /// # Arguments
    /// * `nickname` - The peer's display name
    ///
    /// # Returns
    /// A reference to the PeerInfo if found
    pub fn get_peer_by_nickname(&self, nickname: &str) -> Result<&PeerInfo> {
        self.peer_manager.get_peer_by_nickname(nickname)
    }

    /// Get peer nickname
    ///
    /// Retrieves the display nickname for a given PeerId.
    /// Useful for displaying peer information in the UI.
    ///
    /// # Arguments
    /// * `peer_id` - The peer's unique identifier
    ///
    /// # Returns
    /// The peer's nickname if found
    pub fn get_peer_nickname(&self, peer_id: &PeerId) -> Result<String> {
        self.peer_manager.get_peer_nickname(peer_id)
    }

    /// Get peer info
    ///
    /// Retrieves detailed information about a peer.
    ///
    /// # Arguments
    /// * `peer_id` - The peer's unique identifier
    ///
    /// # Returns
    /// PeerInfo containing peer details if found
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<&PeerInfo> {
        self.peer_manager.get_peer(peer_id)
    }

    /// Get peer ID by nickname
    ///
    /// Looks up a peer's unique identifier by their display name.
    ///
    /// # Arguments
    /// * `nickname` - The peer's display name
    ///
    /// # Returns
    /// The PeerId if found
    pub fn get_peer_id_by_nickname(&self, nickname: &str) -> Option<PeerId> {
        self.peer_manager.get_peer_id_by_nickname(nickname)
    }

    /// Remove a peer from the peer list
    ///
    /// Removes a peer from the internal peer tracking.
    /// This does not disconnect the peer if they are still connected.
    ///
    /// # Arguments
    /// * `peer_id` - The peer's unique identifier
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.peer_manager.remove_peer(peer_id);
    }

    /// Gracefully shutdown the client and notify all peers
    ///
    /// Sends a shutdown notification to all connected peers
    /// and cleans up internal state.
    ///
    /// # Returns
    /// Ok on successful shutdown
    pub fn shutdown(&mut self) -> Result<()> {
        self.peer_manager.shutdown(&mut self.event_sender)
    }

    /// List all discovered peers
    ///
    /// Returns information about all peers that have been discovered
    /// through GigiDns, regardless of connection status.
    ///
    /// # Returns
    /// Vector of peer references
    pub fn list_peers(&self) -> Vec<&PeerInfo> {
        self.peer_manager.list_peers()
    }

    /// Get all connected peers
    ///
    /// Returns information about peers that are currently connected.
    ///
    /// # Returns
    /// Vector of connected peer references
    pub fn get_connected_peers(&self) -> Vec<&PeerInfo> {
        self.peer_manager.get_connected_peers()
    }

    /// Get peers count
    ///
    /// Returns the total number of discovered peers.
    ///
    /// # Returns
    /// Count of all discovered peers
    pub fn peers_count(&self) -> usize {
        self.peer_manager.peers_count()
    }

    /// Get connected peers count
    ///
    /// Returns the number of currently connected peers.
    ///
    /// # Returns
    /// Count of connected peers
    pub fn connected_peers_count(&self) -> usize {
        self.peer_manager.connected_peers_count()
    }

    // ===== Direct Messaging Methods =====
    // These methods handle 1:1 peer-to-peer messaging

    /// Send persistent message to peer
    ///
    /// Sends a message to a peer with persistence support.
    /// If the peer is online, the message is sent immediately and stored.
    /// If the peer is offline, the message is stored for later delivery.
    /// Requires persistence to be enabled.
    ///
    /// # Arguments
    /// * `nickname` - The recipient's display name
    /// * `message` - The message text to send
    ///
    /// # Returns
    /// Ok if sent successfully or stored for delivery
    /// Error if peer not found and persistence is disabled
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
    ///
    /// Sends a direct message to a peer without waiting for persistence.
    /// If the peer is online, the message is sent immediately.
    /// If the peer is offline and persistence is enabled, the message is stored.
    ///
    /// # Arguments
    /// * `nickname` - The recipient's display name
    /// * `message` - The message text to send
    ///
    /// # Returns
    /// Ok if sent successfully or stored for delivery
    /// Error if peer not found
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
    ///
    /// Shares a file with a peer by sending a share code.
    /// The file is registered with the file sharing system and
    /// the share code is sent to the recipient who can then download it.
    ///
    /// # Arguments
    /// * `nickname` - The recipient's display name
    /// * `file_path` - Path to the file to share
    ///
    /// # Returns
    /// Ok on success
    ///
    /// # Note
    /// This method sends a share code rather than the file data directly.
    /// The recipient will download the file using the share code.
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
    ///
    /// Sends a group invitation to a peer.
    /// The recipient can use this to join the specified group.
    ///
    /// # Arguments
    /// * `nickname` - The recipient's display name
    /// * `group_id` - The unique group identifier
    /// * `group_name` - The group's display name
    ///
    /// # Returns
    /// Ok on success
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

    // ===== Group Messaging Methods =====
    // These methods handle GossipSub-based group communication

    /// Join a group
    ///
    /// Subscribes to a GossipSub group and starts receiving group messages.
    ///
    /// # Arguments
    /// * `group_name` - The name of the group to join
    ///
    /// # Returns
    /// Ok on successful subscription
    ///
    /// # Note
    /// Groups are identified by name. Multiple peers can join the same group
    /// to exchange messages in a many-to-many fashion.
    pub fn join_group(&mut self, group_name: &str) -> Result<()> {
        self.group_manager
            .join_group(&mut self.swarm, group_name, &mut self.event_sender)
    }

    /// Leave a group
    ///
    /// Unsubscribes from a GossipSub group and stops receiving group messages.
    ///
    /// # Arguments
    /// * `group_name` - The name of the group to leave
    ///
    /// # Returns
    /// Ok on successful unsubscription
    pub fn leave_group(&mut self, group_name: &str) -> Result<()> {
        self.group_manager.leave_group(&mut self.swarm, group_name)
    }

    /// Send message to group
    ///
    /// Sends a message to all members of a group.
    /// The message is propagated through the GossipSub mesh network.
    ///
    /// # Arguments
    /// * `group_name` - The name of the group
    /// * `message` - The message text to send
    ///
    /// # Returns
    /// Ok on success
    pub fn send_group_message(&mut self, group_name: &str, message: String) -> Result<()> {
        self.group_manager.send_group_message(
            &mut self.swarm,
            group_name,
            message,
            &self.local_nickname,
        )
    }

    /// Send file to group using file sharing
    ///
    /// Shares a file with all members of a group.
    /// The file is registered and the share code is broadcast to the group.
    ///
    /// # Arguments
    /// * `group_name` - The name of the group
    /// * `file_path` - Path to the file to share
    ///
    /// # Returns
    /// Ok on success
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

    // ===== File Sharing Methods =====
    // These methods handle file sharing with chunked transfers and hash verification

    /// Share a file
    ///
    /// Registers a file for sharing and generates a share code.
    /// The file can then be downloaded by peers using the share code.
    ///
    /// # Arguments
    /// * `file_path` - Path to the file to share
    ///
    /// # Returns
    /// The share code that can be used to download this file
    pub async fn share_file(&mut self, file_path: &Path) -> Result<String> {
        self.file_manager.share_file(file_path).await
    }

    /// Set the chunk reader callback for URI-based files
    ///
    /// Sets a callback function for reading chunks from mobile content URIs.
    /// This is required for sharing files on Android (content://) and iOS (file://).
    ///
    /// # Arguments
    /// * `reader` - A callback that reads file chunks by offset and size
    ///
    /// # Example
    /// ```rust,ignore
    /// client.set_chunk_reader(Box::new(|path, offset, size| {
    ///     // Platform-specific file reading implementation
    ///     Ok(data)
    /// }));
    /// ```
    pub fn set_chunk_reader(&mut self, reader: super::file_sharing::FileChunkReader) {
        self.file_manager.set_chunk_reader(reader.clone());
        self.download_manager.set_chunk_reader(reader);
    }

    /// Share a content URI (Android content:// or iOS file://)
    ///
    /// Registers a mobile content URI for sharing.
    /// This is used on mobile platforms where files are accessed through URIs.
    ///
    /// # Arguments
    /// * `uri` - The content URI to share (e.g., "content://...")
    /// * `name` - The display name for the file
    /// * `size` - The file size in bytes
    ///
    /// # Returns
    /// The share code that can be used to download this file
    ///
    /// # Note
    /// Requires `set_chunk_reader` to be called first to provide file reading capability.
    pub async fn share_content_uri(&mut self, uri: &str, name: &str, size: u64) -> Result<String> {
        self.file_manager.share_content_uri(uri, name, size).await
    }

    /// List shared files
    ///
    /// Returns information about all files currently shared by this peer.
    ///
    /// # Returns
    /// Vector of SharedFile references
    pub fn list_shared_files(&self) -> Vec<&crate::events::SharedFile> {
        self.file_manager.list_shared_files()
    }

    /// Unshare a file by share code
    ///
    /// Stops sharing a file and revokes the share code.
    /// Peers will no longer be able to download the file.
    ///
    /// # Arguments
    /// * `share_code` - The share code of the file to unshare
    ///
    /// # Returns
    /// Ok on success
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

    // ===== Download Methods =====
    // These methods handle downloading files from peers with progress tracking

    /// Download file from peer
    ///
    /// Initiates a file download from a peer using a share code.
    /// The download is tracked and progress events are emitted.
    ///
    /// # Arguments
    /// * `nickname` - The peer sharing the file
    /// * `share_code` - The share code of the file to download
    ///
    /// # Returns
    /// The download_id for tracking this download
    ///
    /// # Download Flow
    /// 1. Request file info from peer
    /// 2. Download chunks sequentially
    /// 3. Verify each chunk's BLAKE3 hash
    /// 4. Assemble chunks into final file
    /// 5. Verify final SHA256 hash
    ///
    /// # Events
    /// The client will emit `P2pEvent` updates for download progress.
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
    ///
    /// Sends a P2pEvent to the application's event channel.
    /// Used internally by the client to notify the application of P2P events.
    ///
    /// # Arguments
    /// * `event` - The event to send
    pub fn send_event(&self, event: P2pEvent) {
        if let Err(e) = self.event_sender.unbounded_send(event) {
            error!("Failed to send P2P event: {}", e);
        }
    }

    /// Get local peer ID
    ///
    /// Returns the unique identifier for this peer.
    ///
    /// # Returns
    /// The local PeerId
    pub fn local_peer_id(&self) -> PeerId {
        *self.swarm.local_peer_id()
    }

    /// Get local nickname
    ///
    /// Returns the display nickname for this peer.
    ///
    /// # Returns
    /// The local nickname string
    pub fn local_nickname(&self) -> &str {
        &self.local_nickname
    }

    /// Get joined groups
    ///
    /// Returns information about all groups this peer has joined.
    ///
    /// # Returns
    /// Vector of GroupInfo references
    pub fn list_groups(&self) -> Vec<&GroupInfo> {
        self.group_manager.list_groups()
    }

    // ===== Active Download Tracking Methods for Mobile Apps =====
    // These methods provide download state tracking for UI integration

    /// Get all active downloads
    ///
    /// Returns all downloads currently in progress, completed, or failed.
    /// Useful for displaying a download list in the UI.
    ///
    /// # Returns
    /// Vector of ActiveDownload references
    pub fn get_active_downloads(&self) -> Vec<&ActiveDownload> {
        self.download_manager.get_active_downloads()
    }

    /// Get active download by download_id
    ///
    /// Retrieves a specific download by its unique identifier.
    ///
    /// # Arguments
    /// * `download_id` - The download's unique identifier
    ///
    /// # Returns
    /// The ActiveDownload if found
    pub fn get_active_download(&self, download_id: &str) -> Option<&ActiveDownload> {
        self.download_manager.get_active_download(download_id)
    }

    /// Get active download by share code
    ///
    /// Finds a download by the file's share code.
    /// Useful when you only have the share code from a peer.
    ///
    /// # Arguments
    /// * `share_code` - The share code of the file being downloaded
    ///
    /// # Returns
    /// The ActiveDownload if found
    pub fn get_download_by_share_code(&self, share_code: &str) -> Option<&ActiveDownload> {
        self.download_manager.get_download_by_share_code(share_code)
    }

    /// Remove completed or failed downloads (cleanup)
    ///
    /// Cleans up the download list by removing completed and failed downloads.
    /// Call this periodically to prevent memory growth.
    pub fn cleanup_downloads(&mut self) {
        self.download_manager.cleanup_downloads();
    }

    /// Helper to find download_id by file_id
    ///
    /// Finds the download_id corresponding to a file_id (share_code).
    ///
    /// # Arguments
    /// * `file_id` - The file identifier (share_code)
    ///
    /// # Returns
    /// The download_id if found
    pub fn find_download_id_by_file_id(&self, file_id: &str) -> Option<String> {
        self.download_manager.find_download_id_by_file_id(file_id)
    }

    /// Get downloads from a specific peer
    ///
    /// Returns all downloads initiated from a specific peer.
    /// Useful for filtering downloads by source.
    ///
    /// # Arguments
    /// * `peer_nickname` - The nickname of the peer
    ///
    /// # Returns
    /// Vector of ActiveDownload references
    pub fn get_downloads_from_peer(&self, peer_nickname: &str) -> Vec<&ActiveDownload> {
        self.download_manager.get_downloads_from_peer(peer_nickname)
    }

    /// Get completed downloads (useful for UI history)
    ///
    /// Returns the most recent completed or failed downloads.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of downloads to return
    ///
    /// # Returns
    /// Vector of ActiveDownload references, sorted by most recent first
    pub fn get_recent_downloads(&self, limit: usize) -> Vec<&ActiveDownload> {
        self.download_manager.get_recent_downloads(limit)
    }

    // ===== Persistence Methods =====
    // These methods provide message persistence and offline messaging support

    /// Check if persistence is enabled
    ///
    /// Returns whether message persistence is currently enabled.
    ///
    /// # Returns
    /// true if persistence is enabled, false otherwise
    #[allow(dead_code)]
    pub fn is_persistence_enabled(&self) -> bool {
        self.message_store.is_some()
    }

    /// Get conversation history with a peer
    ///
    /// Retrieves the message history with a specific peer.
    ///
    /// # Arguments
    /// * `nickname` - The peer's nickname
    ///
    /// # Returns
    /// Vector of StoredMessage records
    ///
    /// # Note
    /// Requires persistence to be enabled
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
    ///
    /// Marks a specific message as read in the message store.
    ///
    /// # Arguments
    /// * `message_id` - The unique ID of the message
    ///
    /// # Returns
    /// Ok on success
    ///
    /// # Note
    /// Requires persistence to be enabled
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
    ///
    /// Marks all messages from a peer as read.
    ///
    /// # Arguments
    /// * `nickname` - The peer's nickname
    ///
    /// # Returns
    /// Ok on success
    ///
    /// # Note
    /// Requires persistence to be enabled
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
    ///
    /// Marks a message as delivered to the recipient.
    ///
    /// # Arguments
    /// * `message_id` - The unique ID of the message
    ///
    /// # Returns
    /// Ok on success
    ///
    /// # Note
    /// Requires persistence to be enabled
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
    ///
    /// Stores a message in the message store.
    ///
    /// # Arguments
    /// * `msg` - The message to store
    ///
    /// # Returns
    /// Ok on success
    ///
    /// # Note
    /// Requires persistence to be enabled
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
    ///
    /// Returns the number of unread messages from a peer.
    ///
    /// # Arguments
    /// * `peer_nickname` - The peer's nickname
    ///
    /// # Returns
    /// The unread message count
    ///
    /// # Note
    /// Requires persistence to be enabled
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
    ///
    /// Delivers all queued messages to a peer that has come back online.
    /// Call this when receiving a PeerConnected event for a peer.
    ///
    /// # Arguments
    /// * `nickname` - The peer's nickname
    ///
    /// # Returns
    /// The number of messages sent
    ///
    /// # Note
    /// Requires persistence to be enabled
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

                // Mark message as sent in the queue
                let _ = message_store.mark_message_sent(&msg.id).await;

                sent_count += 1;
            }
        }

        Ok(sent_count)
    }

    /// Clear conversation history with a peer
    ///
    /// Deletes all messages from a specific peer.
    ///
    /// # Arguments
    /// * `nickname` - The peer's nickname
    ///
    /// # Returns
    /// The number of messages deleted
    ///
    /// # Note
    /// Requires persistence to be enabled
    pub async fn clear_conversation(&self, nickname: &str) -> Result<usize> {
        let message_store = self
            .message_store
            .as_ref()
            .ok_or_else(|| P2pError::PersistenceNotEnabled)?;

        message_store
            .clear_conversation(nickname)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to clear conversation: {}", e))
    }
}
