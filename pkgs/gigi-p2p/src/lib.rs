//! Gigi P2P - A comprehensive peer-to-peer networking library
//!
//! This library provides unified P2P functionality including:
//! - Auto Discovery via mDNS
//! - Nickname Exchange via request-response
//! - Direct Messaging via request-response  
//! - Group Messaging via Gossipsub
//! - File Transfer via request-response
//! - Unified event system

use anyhow::Result;
use blake3::Hasher;
use chrono::{DateTime, Utc};
use futures::channel::mpsc;

use libp2p::{
    gossipsub::{self, IdentTopic, MessageAuthenticity, MessageId, ValidationMode},
    identity::Keypair,
    mdns::{self, Config as MdnsConfig},
    multiaddr::Multiaddr,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    PeerId, StreamProtocol,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::fs;
use tracing::error;

/// Constants for chunked file transfer
pub const CHUNK_SIZE: usize = 256 * 1024; // 256KB chunks for better performance

/// Errors that can occur in P2P operations
#[derive(Error, Debug)]
pub enum P2pError {
    #[error("Peer not found: {0}")]
    PeerNotFound(PeerId),
    #[error("Nickname not found: {0}")]
    NicknameNotFound(String),
    #[error("Group not found: {0}")]
    GroupNotFound(String),
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    #[error("Share code invalid: {0}")]
    InvalidShareCode(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Timeout: {0}")]
    Timeout(String),
}

/// Unified P2P event
#[derive(Debug, Clone)]
pub enum P2pEvent {
    // Discovery events
    PeerDiscovered {
        peer_id: PeerId,
        nickname: String,
        address: Multiaddr,
    },
    PeerExpired {
        peer_id: PeerId,
        nickname: String,
    },
    NicknameUpdated {
        peer_id: PeerId,
        nickname: String,
    },

    // Direct messaging events
    DirectMessage {
        from: PeerId,
        from_nickname: String,
        message: String,
    },
    DirectImageMessage {
        from: PeerId,
        from_nickname: String,
        filename: String,
        data: Vec<u8>,
    },

    // Group messaging events
    GroupMessage {
        from: PeerId,
        from_nickname: String,
        group: String,
        message: String,
    },
    GroupImageMessage {
        from: PeerId,
        from_nickname: String,
        group: String,
        filename: String,
        data: Vec<u8>,
        message: String,
    },
    GroupJoined {
        group: String,
    },
    GroupLeft {
        group: String,
    },

    // File transfer events
    FileShareRequest {
        from: PeerId,
        from_nickname: String,
        share_code: String,
        filename: String,
        size: u64,
    },
    FileShared {
        file_id: String,
        info: FileInfo,
    },
    FileRevoked {
        file_id: String,
    },
    FileInfoReceived {
        from: PeerId,
        info: FileInfo,
    },
    ChunkReceived {
        from: PeerId,
        file_id: String,
        chunk_index: usize,
        chunk: ChunkInfo,
    },
    FileListReceived {
        from: PeerId,
        files: Vec<FileInfo>,
    },
    FileDownloadStarted {
        from: PeerId,
        from_nickname: String,
        filename: String,
    },
    FileDownloadProgress {
        file_id: String,
        downloaded_chunks: usize,
        total_chunks: usize,
    },
    FileDownloadCompleted {
        file_id: String,
        path: PathBuf,
    },
    FileDownloadFailed {
        file_id: String,
        error: String,
    },

    // System events
    ListeningOn {
        address: Multiaddr,
    },
    Connected {
        peer_id: PeerId,
        nickname: String,
    },
    Disconnected {
        peer_id: PeerId,
        nickname: String,
    },
    Error(String),
}

// Message types for request-response protocols

/// Nickname exchange messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NicknameRequest {
    GetNickname,
    AnnounceNickname { nickname: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NicknameResponse {
    Nickname { peer_id: String, nickname: String },
    Ack,
    Error(String),
}

/// Direct messaging messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DirectMessage {
    Text { message: String },
    Image { filename: String, data: Vec<u8> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DirectResponse {
    Ack,
    Error(String),
}

/// File transfer messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub hash: String,
    pub chunk_count: usize,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    pub file_id: String,
    pub chunk_index: usize,
    pub data: Vec<u8>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileTransferRequest {
    GetFileInfo(String),
    GetChunk(String, usize),
    ListFiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileTransferResponse {
    FileInfo(Option<FileInfo>),
    Chunk(Option<ChunkInfo>),
    FileList(Vec<FileInfo>),
    Error(String),
}

/// File sharing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedFile {
    pub info: FileInfo,
    pub path: PathBuf,
    pub share_code: String,
    pub revoked: bool,
}

/// Downloading file information
#[derive(Debug, Clone)]
pub struct DownloadingFile {
    pub info: FileInfo,
    pub output_path: PathBuf,
    pub temp_path: PathBuf,
    pub downloaded_chunks: HashMap<usize, bool>,
    pub started_at: Instant,
    pub next_chunk_to_request: usize,
    pub max_concurrent_requests: usize,
    pub peer_id: PeerId,
}

/// Download info for tracking active downloads
#[derive(Debug, Clone)]
pub struct DownloadInfo {
    pub peer_id: PeerId,
    pub filename: String,
    pub share_code: String,
    pub expected_hash: String,
    pub temp_path: PathBuf,
    pub final_path: PathBuf,
    pub started_at: Instant,
}

/// Peer information
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub nickname: String,
    pub addresses: Vec<Multiaddr>,
    pub last_seen: Instant,
    pub connected: bool,
}

/// Group information
#[derive(Debug, Clone)]
pub struct GroupInfo {
    pub name: String,
    pub topic: IdentTopic,
    pub joined_at: DateTime<Utc>,
}

/// Unified network behaviour combining all protocols
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "UnifiedEvent")]
pub struct UnifiedBehaviour {
    pub mdns: mdns::tokio::Behaviour,
    pub nickname: request_response::cbor::Behaviour<NicknameRequest, NicknameResponse>,
    pub direct_msg: request_response::cbor::Behaviour<DirectMessage, DirectResponse>,
    pub gossipsub: gossipsub::Behaviour,
    pub file_transfer: request_response::cbor::Behaviour<FileTransferRequest, FileTransferResponse>,
}

/// Unified event from network behaviour
#[derive(Debug)]
pub enum UnifiedEvent {
    Mdns(mdns::Event),
    Nickname(request_response::Event<NicknameRequest, NicknameResponse>),
    DirectMessage(request_response::Event<DirectMessage, DirectResponse>),
    Gossipsub(gossipsub::Event),
    FileTransfer(request_response::Event<FileTransferRequest, FileTransferResponse>),
}

impl From<mdns::Event> for UnifiedEvent {
    fn from(event: mdns::Event) -> Self {
        Self::Mdns(event)
    }
}

impl From<request_response::Event<NicknameRequest, NicknameResponse>> for UnifiedEvent {
    fn from(event: request_response::Event<NicknameRequest, NicknameResponse>) -> Self {
        Self::Nickname(event)
    }
}

impl From<request_response::Event<DirectMessage, DirectResponse>> for UnifiedEvent {
    fn from(event: request_response::Event<DirectMessage, DirectResponse>) -> Self {
        Self::DirectMessage(event)
    }
}

impl From<gossipsub::Event> for UnifiedEvent {
    fn from(event: gossipsub::Event) -> Self {
        Self::Gossipsub(event)
    }
}

impl From<request_response::Event<FileTransferRequest, FileTransferResponse>> for UnifiedEvent {
    fn from(event: request_response::Event<FileTransferRequest, FileTransferResponse>) -> Self {
        Self::FileTransfer(event)
    }
}

/// Main P2P client
pub struct P2pClient {
    pub swarm: libp2p::swarm::Swarm<UnifiedBehaviour>,
    pub local_nickname: String,

    // Peer management
    pub peers: HashMap<PeerId, PeerInfo>,
    pub nickname_to_peer: HashMap<String, PeerId>,

    // Group management
    pub groups: HashMap<String, GroupInfo>,

    // File sharing
    pub shared_files: HashMap<String, SharedFile>,
    pub downloading_files: HashMap<String, DownloadingFile>,
    pub active_downloads: HashMap<String, DownloadInfo>,
    pub output_directory: PathBuf,
    pub persistent_dir: PathBuf,
    pub shared_file_path: PathBuf,

    // Event handling
    pub event_sender: mpsc::UnboundedSender<P2pEvent>,
}

/// Group message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    pub sender_nickname: String,
    pub content: String,
    pub timestamp: u64,
    pub is_image: bool,
    pub filename: Option<String>,
    pub data: Option<Vec<u8>>,
}

impl P2pClient {
    /// Create a new P2P client
    pub fn new(
        keypair: Keypair,
        nickname: String,
        output_directory: PathBuf,
    ) -> Result<(Self, mpsc::UnboundedReceiver<P2pEvent>)> {
        Self::new_with_config(
            keypair,
            nickname,
            output_directory,
            PathBuf::from("shared.json"),
        )
    }

    /// Create a new P2P client with custom shared file path
    pub fn new_with_config(
        keypair: Keypair,
        nickname: String,
        output_directory: PathBuf,
        shared_file_path: PathBuf,
    ) -> Result<(Self, mpsc::UnboundedReceiver<P2pEvent>)> {
        let (event_sender, event_receiver) = mpsc::unbounded();

        let persistent_dir = output_directory.join(".gigi");

        // Create behaviours
        let mdns =
            mdns::tokio::Behaviour::new(MdnsConfig::default(), keypair.public().to_peer_id())?;

        let nickname_behaviour = request_response::cbor::Behaviour::new(
            [(
                StreamProtocol::new("/nickname/1.0.0"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default(),
        );

        let direct_msg = request_response::cbor::Behaviour::new(
            [(StreamProtocol::new("/direct/1.0.0"), ProtocolSupport::Full)],
            request_response::Config::default(),
        );

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(ValidationMode::Strict)
            .message_id_fn(|message| {
                let mut hasher = Hasher::new();
                hasher.update(&message.data);
                MessageId::from(hasher.finalize().as_bytes())
            })
            .build()
            .expect("Valid config");

        let gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(keypair.clone()),
            gossipsub_config,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create gossipsub behaviour: {}", e))?;

        let file_transfer = request_response::cbor::Behaviour::new(
            [(StreamProtocol::new("/file/1.0.0"), ProtocolSupport::Full)],
            request_response::Config::default(),
        );

        // Create unified behaviour
        let behaviour = UnifiedBehaviour {
            mdns,
            nickname: nickname_behaviour,
            direct_msg,
            gossipsub,
            file_transfer,
        };

        // Build swarm
        let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default(),
                libp2p::noise::Config::new,
                libp2p::yamux::Config::default,
            )?
            .with_quic()
            .with_behaviour(|_| behaviour)?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(300)))
            .build();

        let mut client = Self {
            swarm,
            local_nickname: nickname,
            peers: HashMap::new(),
            nickname_to_peer: HashMap::new(),
            groups: HashMap::new(),
            shared_files: HashMap::new(),
            downloading_files: HashMap::new(),
            active_downloads: HashMap::new(),
            output_directory,
            persistent_dir,
            shared_file_path,
            event_sender,
        };

        // Load existing shared files
        client.load_shared_files()?;

        Ok((client, event_receiver))
    }

    /// Start listening on given address
    pub fn start_listening(&mut self, addr: Multiaddr) -> Result<()> {
        self.swarm
            .listen_on(addr)
            .map_err(|e| P2pError::NetworkError(e.to_string()))?;
        Ok(())
    }

    /// Handle a single swarm event
    pub fn handle_event(&mut self, event: SwarmEvent<UnifiedEvent>) -> Result<()> {
        match event {
            SwarmEvent::Behaviour(unified_event) => {
                self.handle_unified_event(unified_event)?;
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                self.send_event(P2pEvent::ListeningOn { address });
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                if let Some(peer) = self.peers.get(&peer_id) {
                    self.send_event(P2pEvent::Connected {
                        peer_id,
                        nickname: peer.nickname.clone(),
                    });
                }
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                if let Some(peer) = self.peers.get(&peer_id) {
                    self.send_event(P2pEvent::Disconnected {
                        peer_id,
                        nickname: peer.nickname.clone(),
                    });
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle unified network events
    fn handle_unified_event(&mut self, event: UnifiedEvent) -> Result<()> {
        match event {
            UnifiedEvent::Mdns(mdns_event) => self.handle_mdns_event(mdns_event)?,
            UnifiedEvent::Nickname(nickname_event) => self.handle_nickname_event(nickname_event)?,
            UnifiedEvent::DirectMessage(dm_event) => self.handle_direct_message_event(dm_event)?,
            UnifiedEvent::Gossipsub(gossip_event) => self.handle_gossipsub_event(gossip_event)?,
            UnifiedEvent::FileTransfer(file_event) => {
                self.handle_file_transfer_event(file_event)?
            }
        }
        Ok(())
    }

    /// Handle mDNS events
    fn handle_mdns_event(&mut self, event: mdns::Event) -> Result<()> {
        match event {
            mdns::Event::Discovered(list) => {
                println!("🔍 mDNS discovered {} peers", list.len());
                for (peer_id, addr) in list {
                    println!("🔍 Found peer: {} at {}", peer_id, addr);
                    self.handle_peer_discovered(peer_id, addr)?;
                }
            }
            mdns::Event::Expired(list) => {
                println!("👋 mDNS expired {} peers", list.len());
                for (peer_id, _addr) in list {
                    self.handle_peer_expired(peer_id)?;
                }
            }
        }
        Ok(())
    }

    /// Handle nickname events
    fn handle_nickname_event(
        &mut self,
        event: request_response::Event<NicknameRequest, NicknameResponse>,
    ) -> Result<()> {
        if let request_response::Event::Message { message, peer, .. } = event {
            match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    let response = match request {
                        NicknameRequest::GetNickname => NicknameResponse::Nickname {
                            peer_id: self.swarm.local_peer_id().to_string(),
                            nickname: self.local_nickname.clone(),
                        },
                        NicknameRequest::AnnounceNickname { nickname } => {
                            self.update_peer_nickname(peer, nickname.clone());
                            NicknameResponse::Ack
                        }
                    };
                    let _ = self
                        .swarm
                        .behaviour_mut()
                        .nickname
                        .send_response(channel, response);
                }
                request_response::Message::Response { response, .. } => {
                    if let NicknameResponse::Nickname { nickname, .. } = response {
                        self.update_peer_nickname(peer, nickname);
                    }
                }
            }
        }
        Ok(())
    }

    /// Handle direct message events
    fn handle_direct_message_event(
        &mut self,
        event: request_response::Event<DirectMessage, DirectResponse>,
    ) -> Result<()> {
        if let request_response::Event::Message {
            message:
                request_response::Message::Request {
                    request, channel, ..
                },
            peer,
            ..
        } = event
        {
            let nickname = self.get_peer_nickname(&peer)?;
            match request {
                DirectMessage::Text { message } => {
                    self.send_event(P2pEvent::DirectMessage {
                        from: peer,
                        from_nickname: nickname,
                        message,
                    });
                }
                DirectMessage::Image { filename, data } => {
                    self.send_event(P2pEvent::DirectImageMessage {
                        from: peer,
                        from_nickname: nickname,
                        filename,
                        data,
                    });
                }
            }
            let _ = self
                .swarm
                .behaviour_mut()
                .direct_msg
                .send_response(channel, DirectResponse::Ack);
        }
        Ok(())
    }

    /// Handle gossipsub events
    fn handle_gossipsub_event(&mut self, event: gossipsub::Event) -> Result<()> {
        match event {
            gossipsub::Event::Message {
                propagation_source: peer_id,
                message,
                ..
            } => {
                if let Ok(group_message) = serde_json::from_slice::<GroupMessage>(&message.data) {
                    let group_name = message.topic.to_string();
                    let nickname = self.get_peer_nickname(&peer_id)?;

                    if group_message.is_image {
                        self.send_event(P2pEvent::GroupImageMessage {
                            from: peer_id,
                            from_nickname: nickname,
                            group: group_name,
                            filename: group_message.filename.unwrap_or_default(),
                            data: group_message.data.unwrap_or_default(),
                            message: group_message.content.clone(),
                        });
                    } else {
                        self.send_event(P2pEvent::GroupMessage {
                            from: peer_id,
                            from_nickname: nickname,
                            group: group_name,
                            message: group_message.content,
                        });
                    }
                }
            }
            gossipsub::Event::Subscribed { topic, .. } => {
                let group_name = topic.to_string();
                self.send_event(P2pEvent::GroupJoined { group: group_name });
            }
            gossipsub::Event::Unsubscribed { topic, .. } => {
                let group_name = topic.to_string();
                self.send_event(P2pEvent::GroupLeft { group: group_name });
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle file transfer events with chunked file support
    fn handle_file_transfer_event(
        &mut self,
        event: request_response::Event<FileTransferRequest, FileTransferResponse>,
    ) -> Result<()> {
        if let request_response::Event::Message { message, peer, .. } = event {
            match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    let response = match request {
                        FileTransferRequest::GetFileInfo(file_id) => {
                            let info = self
                                .shared_files
                                .get(&file_id)
                                .filter(|f| !f.revoked)
                                .map(|f| f.info.clone());
                            FileTransferResponse::FileInfo(info)
                        }
                        FileTransferRequest::GetChunk(file_id, chunk_index) => {
                            if let Some(shared_file) = self.shared_files.get(&file_id) {
                                if !shared_file.revoked {
                                    match self.read_chunk(&shared_file.path, chunk_index, &file_id)
                                    {
                                        Ok(chunk) => FileTransferResponse::Chunk(Some(chunk)),
                                        Err(_) => FileTransferResponse::Error(
                                            "Failed to read chunk".to_string(),
                                        ),
                                    }
                                } else {
                                    FileTransferResponse::Error("File has been revoked".to_string())
                                }
                            } else {
                                FileTransferResponse::Chunk(None)
                            }
                        }
                        FileTransferRequest::ListFiles => {
                            let files = self
                                .shared_files
                                .values()
                                .filter(|f| !f.revoked)
                                .map(|f| f.info.clone())
                                .collect();
                            FileTransferResponse::FileList(files)
                        }
                    };
                    let _ = self
                        .swarm
                        .behaviour_mut()
                        .file_transfer
                        .send_response(channel, response);
                }
                request_response::Message::Response { response, .. } => {
                    match response {
                        FileTransferResponse::FileInfo(Some(info)) => {
                            // Start download when we receive file info
                            self.start_download(peer, info.clone())?;
                            self.send_event(P2pEvent::FileInfoReceived { from: peer, info });
                        }
                        FileTransferResponse::FileInfo(None) => {
                            // File not found
                        }
                        FileTransferResponse::Chunk(Some(chunk)) => {
                            self.handle_chunk_received(
                                peer,
                                chunk.file_id.clone(),
                                chunk.chunk_index,
                                chunk,
                            )?;
                        }
                        FileTransferResponse::Chunk(None) => {
                            // Chunk not found
                        }
                        FileTransferResponse::FileList(files) => {
                            self.send_event(P2pEvent::FileListReceived { from: peer, files });
                        }
                        FileTransferResponse::Error(error) => {
                            self.send_event(P2pEvent::Error(error));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Handle peer discovery
    pub fn handle_peer_discovered(&mut self, peer_id: PeerId, addr: Multiaddr) -> Result<()> {
        if !self.peers.contains_key(&peer_id) {
            let nickname = peer_id.to_string(); // Default nickname

            self.nickname_to_peer.insert(nickname.clone(), peer_id);

            // Request nickname from peer
            println!("📤 Sending GetNickname request to {}", peer_id);
            self.swarm
                .behaviour_mut()
                .nickname
                .send_request(&peer_id, NicknameRequest::GetNickname);

            // Announce our nickname
            println!(
                "📤 Sending AnnounceNickname request to {} as {}",
                peer_id, self.local_nickname
            );
            self.swarm.behaviour_mut().nickname.send_request(
                &peer_id,
                NicknameRequest::AnnounceNickname {
                    nickname: self.local_nickname.clone(),
                },
            );

            self.send_event(P2pEvent::PeerDiscovered {
                peer_id,
                nickname: nickname.clone(),
                address: addr.clone(),
            });

            let peer_info = PeerInfo {
                peer_id,
                nickname,
                addresses: vec![addr],
                last_seen: Instant::now(),
                connected: false,
            };

            self.peers.insert(peer_id, peer_info);
        }
        Ok(())
    }

    /// Handle peer expiration
    fn handle_peer_expired(&mut self, peer_id: PeerId) -> Result<()> {
        if let Some(peer) = self.peers.remove(&peer_id) {
            self.nickname_to_peer.remove(&peer.nickname);

            self.send_event(P2pEvent::PeerExpired {
                peer_id,
                nickname: peer.nickname,
            });
        }
        Ok(())
    }

    /// Update peer nickname
    pub fn update_peer_nickname(&mut self, peer_id: PeerId, nickname: String) {
        if let Some(peer) = self.peers.get_mut(&peer_id) {
            let old_nickname = peer.nickname.clone();
            if old_nickname != nickname {
                self.nickname_to_peer.remove(&old_nickname);
                self.nickname_to_peer.insert(nickname.clone(), peer_id);
                peer.nickname = nickname.clone();

                self.send_event(P2pEvent::NicknameUpdated { peer_id, nickname });
            }
        }
    }

    /// Get peer by nickname
    pub fn get_peer_by_nickname(&self, nickname: &str) -> Result<&PeerInfo> {
        let peer_id = *self
            .nickname_to_peer
            .get(nickname)
            .ok_or_else(|| P2pError::NicknameNotFound(nickname.to_string()))?;
        self.peers
            .get(&peer_id)
            .ok_or_else(|| P2pError::PeerNotFound(peer_id).into())
    }

    /// Get peer nickname
    pub fn get_peer_nickname(&self, peer_id: &PeerId) -> Result<String> {
        self.peers
            .get(peer_id)
            .map(|p| p.nickname.clone())
            .ok_or_else(|| P2pError::PeerNotFound(*peer_id).into())
    }

    /// Remove a peer from the peer list
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        if let Some(peer) = self.peers.remove(peer_id) {
            self.nickname_to_peer.remove(&peer.nickname);
        }
    }

    /// List all discovered peers
    pub fn list_peers(&self) -> Vec<&PeerInfo> {
        self.peers.values().collect()
    }

    /// Send direct message to peer
    pub fn send_direct_message(&mut self, nickname: &str, message: String) -> Result<()> {
        let peer_id = *self
            .nickname_to_peer
            .get(nickname)
            .ok_or_else(|| P2pError::NicknameNotFound(nickname.to_string()))?;

        self.swarm
            .behaviour_mut()
            .direct_msg
            .send_request(&peer_id, DirectMessage::Text { message });

        Ok(())
    }

    /// Send image to peer
    pub fn send_direct_image(&mut self, nickname: &str, image_path: &Path) -> Result<()> {
        let peer_id = *self
            .nickname_to_peer
            .get(nickname)
            .ok_or_else(|| P2pError::NicknameNotFound(nickname.to_string()))?;

        let data = std::fs::read(image_path)?;
        let filename = image_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| P2pError::FileNotFound(image_path.to_path_buf()))?
            .to_string();

        self.swarm
            .behaviour_mut()
            .direct_msg
            .send_request(&peer_id, DirectMessage::Image { filename, data });

        Ok(())
    }

    /// Join a group
    pub fn join_group(&mut self, group_name: &str) -> Result<()> {
        let topic = IdentTopic::new(group_name);

        self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

        let group_info = GroupInfo {
            name: group_name.to_string(),
            topic,
            joined_at: Utc::now(),
        };

        self.groups.insert(group_name.to_string(), group_info);

        Ok(())
    }

    /// Leave a group
    pub fn leave_group(&mut self, group_name: &str) -> Result<()> {
        if let Some(group) = self.groups.remove(group_name) {
            self.swarm
                .behaviour_mut()
                .gossipsub
                .unsubscribe(&group.topic);
        } else {
            return Err(P2pError::GroupNotFound(group_name.to_string()).into());
        }

        Ok(())
    }

    /// Send message to group
    pub fn send_group_message(&mut self, group_name: &str, message: String) -> Result<()> {
        let group = self
            .groups
            .get(group_name)
            .ok_or_else(|| P2pError::GroupNotFound(group_name.to_string()))?;

        let group_message = GroupMessage {
            sender_nickname: self.local_nickname.clone(),
            content: message,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            is_image: false,
            filename: None,
            data: None,
        };

        let data = serde_json::to_vec(&group_message)?;

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(group.topic.clone(), data)?;

        Ok(())
    }

    /// Send image to group
    pub async fn send_group_image(&mut self, group_name: &str, image_path: &Path) -> Result<()> {
        let group_topic = {
            let group = self
                .groups
                .get(group_name)
                .ok_or_else(|| P2pError::GroupNotFound(group_name.to_string()))?;
            group.topic.clone()
        };

        // First share the file to get a share code
        let share_code = self.share_file(image_path).await?;

        let filename = image_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| P2pError::FileNotFound(image_path.to_path_buf()))?
            .to_string();

        // Send a message with the share code instead of the raw image data
        let group_message = GroupMessage {
            sender_nickname: self.local_nickname.clone(),
            content: format!("/download {} 🖼️", share_code),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            is_image: true,
            filename: Some(filename),
            data: None, // No raw data, just share code in content
        };

        let msg_data = serde_json::to_vec(&group_message)?;

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(group_topic, msg_data)?;

        Ok(())
    }

    /// Share a file
    pub async fn share_file(&mut self, file_path: &Path) -> Result<String> {
        let path = file_path
            .canonicalize()
            .map_err(|_| P2pError::FileNotFound(file_path.to_path_buf()))?;

        let metadata = fs::metadata(&path).await?;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| P2pError::FileNotFound(path.clone()))?
            .to_string();

        // Calculate file hash
        let hash = self.calculate_file_hash(&path)?;

        // Check if file is already shared
        if let Some((existing_share_code, existing_shared_file)) = self
            .shared_files
            .iter()
            .find(|(_, shared_file)| shared_file.path == path)
        {
            // File already shared, check if it has changed
            if existing_shared_file.info.hash == hash {
                // File unchanged, return existing share code
                println!(
                    "📂 File '{}' already shared with code: {} (unchanged)",
                    filename, existing_share_code
                );
                return Ok(existing_share_code.clone());
            } else {
                // File changed, update the existing entry
                let updated_info = FileInfo {
                    id: existing_shared_file.info.id.clone(),
                    name: filename.clone(),
                    size: metadata.len(),
                    hash: hash.clone(),
                    chunk_count: (metadata.len() as usize + CHUNK_SIZE - 1) / CHUNK_SIZE,
                    created_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs(),
                };

                let share_code = existing_share_code.clone();
                let updated_shared_file = SharedFile {
                    info: updated_info,
                    path: path.clone(),
                    share_code: share_code.clone(),
                    revoked: false,
                };

                self.shared_files
                    .insert(share_code.clone(), updated_shared_file);
                self.save_shared_files()?;

                println!(
                    "📂 Updated file '{}' (hash: {}) with existing code: {}",
                    filename,
                    &hash[..8],
                    share_code
                );
                return Ok(share_code);
            }
        }

        // New file, create new entry
        let share_code = self.generate_share_code(&filename);
        let file_id = share_code.clone();

        // Calculate chunk count
        let chunk_count = (metadata.len() as usize + CHUNK_SIZE - 1) / CHUNK_SIZE;

        // Create FileInfo
        let file_info = FileInfo {
            id: file_id.clone(),
            name: filename.clone(),
            size: metadata.len(),
            hash: hash.clone(),
            chunk_count,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        };

        let shared_file = SharedFile {
            info: file_info,
            path: path.clone(),
            share_code: share_code.clone(),
            revoked: false,
        };

        self.shared_files.insert(share_code.clone(), shared_file);

        // Save to persistent storage
        self.save_shared_files()?;

        println!(
            "📂 Shared file '{}' (hash: {}) with code: {}",
            filename,
            &hash[..8],
            share_code
        );

        Ok(share_code)
    }

    /// List shared files
    pub fn list_shared_files(&self) -> Vec<&SharedFile> {
        self.shared_files.values().collect()
    }

    /// Download file from peer
    pub fn download_file(&mut self, nickname: &str, share_code: &str) -> Result<()> {
        let peer_id = *self
            .nickname_to_peer
            .get(nickname)
            .ok_or_else(|| P2pError::NicknameNotFound(nickname.to_string()))?;

        // First request file info
        let _request_id = self.swarm.behaviour_mut().file_transfer.send_request(
            &peer_id,
            FileTransferRequest::GetFileInfo(share_code.to_string()),
        );

        Ok(())
    }

    /// Generate share code for file
    fn generate_share_code(&self, filename: &str) -> String {
        let mut hasher = Hasher::new();
        hasher.update(filename.as_bytes());
        hasher.update(
            &std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_le_bytes(),
        );

        format!("{}", hasher.finalize().to_hex())[..8].to_string()
    }

    /// Send event to event receiver
    fn send_event(&self, event: P2pEvent) {
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
        self.groups.values().collect()
    }

    /// Load shared files from persistent storage
    fn load_shared_files(&mut self) -> Result<()> {
        // For relative paths, use current working directory, not persistent_dir
        let shared_files_path = if self.shared_file_path.is_absolute() {
            self.shared_file_path.clone()
        } else {
            std::env::current_dir()?.join(&self.shared_file_path)
        };

        if shared_files_path.exists() {
            let content = std::fs::read_to_string(&shared_files_path)?;

            // Try to deserialize as new format first
            let loaded_files: Result<HashMap<String, SharedFile>, _> =
                serde_json::from_str(&content);

            let loaded_files = match loaded_files {
                Ok(files) => files,
                Err(_) => {
                    // Try to deserialize as old format and migrate
                    #[derive(Debug, Deserialize)]
                    struct OldSharedFile {
                        path: PathBuf,
                        filename: String,
                        size: u64,
                        share_code: String,
                        hash: String,
                        created_at: String,
                    }

                    let old_files: HashMap<String, OldSharedFile> = serde_json::from_str(&content)?;
                    old_files
                        .into_iter()
                        .map(|(share_code, old_file)| {
                            // Parse the created_at timestamp
                            let created_at = old_file
                                .created_at
                                .parse::<DateTime<Utc>>()
                                .unwrap_or_else(|_| Utc::now())
                                .timestamp() as u64;

                            let shared_file = SharedFile {
                                info: FileInfo {
                                    id: share_code.clone(),
                                    name: old_file.filename,
                                    size: old_file.size,
                                    hash: old_file.hash,
                                    chunk_count: ((old_file.size + CHUNK_SIZE as u64 - 1)
                                        / CHUNK_SIZE as u64)
                                        as usize,
                                    created_at,
                                },
                                path: old_file.path,
                                share_code: old_file.share_code,
                                revoked: false,
                            };
                            (share_code, shared_file)
                        })
                        .collect()
                }
            };

            // Only load files that still exist on disk
            for (share_code, shared_file) in loaded_files {
                if shared_file.path.exists() {
                    self.shared_files.insert(share_code, shared_file);
                }
            }

            println!(
                "📂 Loaded {} shared files from storage",
                self.shared_files.len()
            );
        }

        Ok(())
    }

    /// Save shared files to persistent storage
    fn save_shared_files(&self) -> Result<()> {
        let shared_files_path = if self.shared_file_path.is_absolute() {
            self.shared_file_path
                .parent()
                .map(|p| std::fs::create_dir_all(p))
                .transpose()?
                .unwrap_or(());
            self.shared_file_path.clone()
        } else {
            // For relative paths, use current working directory
            let path = std::env::current_dir()?.join(&self.shared_file_path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            path
        };

        let content = serde_json::to_string_pretty(&self.shared_files)?;
        std::fs::write(&shared_files_path, content)?;

        Ok(())
    }

    /// Calculate SHA256 hash of a file
    fn calculate_file_hash(&self, file_path: &Path) -> Result<String> {
        let mut file = std::fs::File::open(file_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Calculate hash of data chunk
    fn calculate_chunk_hash(&self, data: &[u8]) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        hasher.finalize().to_hex().to_string()
    }

    /// Verify downloaded file hash
    #[allow(dead_code)]
    fn verify_file_hash(&self, file_path: &Path, expected_hash: &str) -> Result<bool> {
        let actual_hash = self.calculate_file_hash(file_path)?;
        Ok(actual_hash == expected_hash)
    }

    /// Read a chunk from a file
    fn read_chunk(&self, file_path: &Path, chunk_index: usize, file_id: &str) -> Result<ChunkInfo> {
        use std::io::{Read, Seek};

        let mut file = std::fs::File::open(file_path)?;
        let offset = chunk_index * CHUNK_SIZE;
        file.seek(std::io::SeekFrom::Start(offset as u64))?;

        let mut buffer = vec![0u8; CHUNK_SIZE];
        let bytes_read = file.read(&mut buffer)?;
        buffer.truncate(bytes_read);

        let hash = self.calculate_chunk_hash(&buffer);

        Ok(ChunkInfo {
            file_id: file_id.to_string(),
            chunk_index,
            data: buffer,
            hash,
        })
    }

    /// Handle a received chunk
    fn handle_chunk_received(
        &mut self,
        peer: PeerId,
        file_id: String,
        chunk_index: usize,
        chunk: ChunkInfo,
    ) -> Result<()> {
        // Verify chunk hash
        let calculated_hash = self.calculate_chunk_hash(&chunk.data);
        if calculated_hash != chunk.hash {
            self.send_event(P2pEvent::FileDownloadFailed {
                file_id: file_id.clone(),
                error: format!("Chunk {} hash mismatch", chunk_index),
            });
            return Ok(());
        }

        // Extract necessary data before mutable borrow
        let (should_process, total_chunks) =
            if let Some(downloading_file) = self.downloading_files.get(&file_id) {
                (true, downloading_file.info.chunk_count)
            } else {
                (false, 0)
            };

        if should_process {
            // Write chunk to temp file
            let offset = chunk_index * CHUNK_SIZE;
            let temp_path = self.downloading_files[&file_id].temp_path.clone();
            match std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(&temp_path)
            {
                Ok(mut file) => {
                    use std::io::{Seek, Write};
                    file.seek(std::io::SeekFrom::Start(offset as u64))?;
                    file.write_all(&chunk.data)?;
                    file.flush()?;

                    // Update downloading file info
                    if let Some(downloading_file) = self.downloading_files.get_mut(&file_id) {
                        downloading_file.downloaded_chunks.insert(chunk_index, true);

                        let downloaded_count = downloading_file
                            .downloaded_chunks
                            .values()
                            .filter(|&&v| v)
                            .count();

                        let next_chunk_to_request = downloading_file.next_chunk_to_request;
                        let should_request_next = next_chunk_to_request < total_chunks;

                        // Request next chunk if available (sliding window)
                        if should_request_next {
                            let next_chunk = next_chunk_to_request;
                            downloading_file.next_chunk_to_request += 1;

                            // Request next chunk
                            let _request_id =
                                self.swarm.behaviour_mut().file_transfer.send_request(
                                    &peer,
                                    FileTransferRequest::GetChunk(file_id.clone(), next_chunk),
                                );
                        }

                        // Check if download is complete
                        if downloaded_count == total_chunks {
                            let output_path = downloading_file.output_path.clone();
                            let expected_hash = downloading_file.info.hash.clone();

                            // Drop the mutable borrow before calculating hash
                            let _ = downloading_file;

                            // Send progress event
                            self.send_event(P2pEvent::FileDownloadProgress {
                                file_id: file_id.clone(),
                                downloaded_chunks: downloaded_count,
                                total_chunks,
                            });

                            // Verify file hash
                            match self.calculate_file_hash(&temp_path) {
                                Ok(file_hash) => {
                                    if file_hash == expected_hash {
                                        // Rename temp file to final name
                                        match std::fs::rename(&temp_path, &output_path) {
                                            Ok(_) => {
                                                self.send_event(P2pEvent::FileDownloadCompleted {
                                                    file_id: file_id.clone(),
                                                    path: output_path,
                                                });
                                            }
                                            Err(e) => {
                                                self.send_event(P2pEvent::FileDownloadFailed {
                                                    file_id: file_id.clone(),
                                                    error: format!("Failed to rename file: {}", e),
                                                });
                                            }
                                        }
                                    } else {
                                        self.send_event(P2pEvent::FileDownloadFailed {
                                            file_id: file_id.clone(),
                                            error: "File hash verification failed".to_string(),
                                        });
                                    }
                                }
                                Err(e) => {
                                    self.send_event(P2pEvent::FileDownloadFailed {
                                        file_id: file_id.clone(),
                                        error: format!("Failed to calculate file hash: {}", e),
                                    });
                                }
                            }

                            // Remove from downloading files
                            self.downloading_files.remove(&file_id);
                        } else {
                            // Send progress event for incomplete download
                            self.send_event(P2pEvent::FileDownloadProgress {
                                file_id: file_id.clone(),
                                downloaded_chunks: downloaded_count,
                                total_chunks,
                            });
                        }
                    }
                }
                Err(e) => {
                    self.send_event(P2pEvent::FileDownloadFailed {
                        file_id: file_id.clone(),
                        error: format!("Failed to write chunk: {}", e),
                    });
                }
            }
        }

        self.send_event(P2pEvent::ChunkReceived {
            from: peer,
            file_id: file_id.clone(),
            chunk_index,
            chunk,
        });

        Ok(())
    }

    /// Start downloading a file after receiving file info
    fn start_download(&mut self, peer_id: PeerId, info: FileInfo) -> Result<()> {
        // Find available filename
        let filename = self.find_available_filename(&info.name);
        let output_path = self.output_directory.join(&filename);
        let temp_path = self
            .output_directory
            .join(format!("{}.downloading", info.id));

        // Create downloading file entry
        let downloading_file = DownloadingFile {
            info: info.clone(),
            output_path: output_path.clone(),
            temp_path: temp_path.clone(),
            downloaded_chunks: HashMap::new(),
            started_at: Instant::now(),
            next_chunk_to_request: 0,
            max_concurrent_requests: 5, // Allow 5 concurrent chunk requests
            peer_id,
        };

        self.downloading_files
            .insert(info.id.clone(), downloading_file);

        // Send download started event
        let nickname = self.get_peer_nickname(&peer_id).unwrap_or_default();
        self.send_event(P2pEvent::FileDownloadStarted {
            from: peer_id,
            from_nickname: nickname,
            filename: info.name.clone(),
        });

        // Start requesting chunks (initial sliding window)
        let initial_requests = std::cmp::min(5, info.chunk_count);
        for i in 0..initial_requests {
            let _request_id = self
                .swarm
                .behaviour_mut()
                .file_transfer
                .send_request(&peer_id, FileTransferRequest::GetChunk(info.id.clone(), i));
        }

        // Update next chunk to request
        if let Some(downloading) = self.downloading_files.get_mut(&info.id) {
            downloading.next_chunk_to_request = initial_requests;
        }

        Ok(())
    }

    /// Find available filename (append number if exists)
    fn find_available_filename(&self, filename: &str) -> String {
        let path = self.output_directory.join(filename);

        if !path.exists() {
            return filename.to_string();
        }

        // Extract name and extension
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        for i in 1..1000 {
            let candidate = if extension.is_empty() {
                format!("{}_{}", stem, i)
            } else {
                format!("{}_{}.{}", stem, i, extension)
            };

            if !self.output_directory.join(&candidate).exists() {
                return candidate;
            }
        }

        // Fallback to timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!("{}_{}.{}", stem, timestamp, extension)
    }
}
