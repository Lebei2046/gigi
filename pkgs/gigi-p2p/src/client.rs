//! Main P2P client implementation

use anyhow::Result;
use futures::channel::mpsc;
use libp2p::{
    identity::Keypair,
    mdns::{self, Config as MdnsConfig},
    multiaddr::Multiaddr,
    request_response::{self, ProtocolSupport},
    swarm::SwarmEvent,
    PeerId, StreamProtocol,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, instrument, warn};

use super::behaviour::{
    create_gossipsub_behaviour, create_gossipsub_config, DirectMessage, FileTransferRequest,
    NicknameRequest, UnifiedBehaviour, UnifiedEvent,
};
use super::error::P2pError;
use super::events::{ChunkInfo, GroupInfo, GroupMessage, P2pEvent, PeerInfo};
use super::file_transfer::{FileTransferManager, CHUNK_SIZE};

/// Main P2P client
pub struct P2pClient {
    swarm: libp2p::swarm::Swarm<UnifiedBehaviour>,
    local_nickname: String,

    // Peer management
    peers: HashMap<PeerId, PeerInfo>,
    nickname_to_peer: HashMap<String, PeerId>,

    // Group management
    groups: HashMap<String, GroupInfo>,

    // File sharing
    file_manager: FileTransferManager,

    // Event handling
    event_sender: mpsc::UnboundedSender<P2pEvent>,
}

impl P2pClient {
    /// Create a new P2P client
    #[instrument(skip(keypair))]
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
    #[instrument(skip(keypair))]
    pub fn new_with_config(
        keypair: Keypair,
        nickname: String,
        output_directory: PathBuf,
        shared_file_path: PathBuf,
    ) -> Result<(Self, mpsc::UnboundedReceiver<P2pEvent>)> {
        let (event_sender, event_receiver) = mpsc::unbounded();

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

        let gossipsub_config = create_gossipsub_config(&keypair)
            .map_err(|e| anyhow::anyhow!("Failed to create gossipsub config: {}", e))?;
        let gossipsub = create_gossipsub_behaviour(keypair.clone(), gossipsub_config)?;

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

        let file_manager =
            FileTransferManager::new(output_directory.clone(), shared_file_path.clone());

        let mut client = Self {
            swarm,
            local_nickname: nickname,
            peers: HashMap::new(),
            nickname_to_peer: HashMap::new(),
            groups: HashMap::new(),
            file_manager,
            event_sender,
        };

        // Load existing shared files
        client.file_manager.load_shared_files()?;

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
                if let Some(peer) = self.peers.remove(&peer_id) {
                    self.nickname_to_peer.remove(&peer.nickname);
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
    fn handle_mdns_event(&mut self, event: libp2p::mdns::Event) -> Result<()> {
        match event {
            mdns::Event::Discovered(list) => {
                info!("mDNS discovered {} peers", list.len());
                for (peer_id, addr) in list {
                    debug!("Found peer: {} at {}", peer_id, addr);
                    self.handle_peer_discovered(peer_id, addr)?;
                }
            }
            mdns::Event::Expired(list) => {
                info!("mDNS expired {} peers", list.len());
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
        event: request_response::Event<NicknameRequest, super::behaviour::NicknameResponse>,
    ) -> Result<()> {
        use super::behaviour::NicknameResponse;

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
                    debug!("Sending nickname response to {}: {:?}", peer, response);
                    let _ = self
                        .swarm
                        .behaviour_mut()
                        .nickname
                        .send_response(channel, response);
                }
                request_response::Message::Response { response, .. } => {
                    debug!("Received nickname response from {}: {:?}", peer, response);
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
        event: request_response::Event<DirectMessage, super::behaviour::DirectResponse>,
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
                DirectMessage::ShareGroup {
                    group_id,
                    group_name,
                    inviter_nickname: _,
                } => {
                    self.send_event(P2pEvent::DirectGroupShareMessage {
                        from: peer,
                        from_nickname: nickname,
                        group_id,
                        group_name,
                    });
                }
            }
            let _ = self
                .swarm
                .behaviour_mut()
                .direct_msg
                .send_response(channel, super::behaviour::DirectResponse::Ack);
        }
        Ok(())
    }

    /// Handle gossipsub events
    fn handle_gossipsub_event(&mut self, event: libp2p::gossipsub::Event) -> Result<()> {
        match event {
            libp2p::gossipsub::Event::Message {
                propagation_source: peer_id,
                message,
                ..
            } => {
                debug!("Raw gossipsub message received from: {}", peer_id);
                debug!("Topic: {}", message.topic);
                debug!("Data length: {} bytes", message.data.len());

                if let Ok(group_message) = serde_json::from_slice::<GroupMessage>(&message.data) {
                    let group_name = message.topic.to_string();
                    let nickname = self.get_peer_nickname(&peer_id)?;

                    debug!("Parsed group message successfully:");
                    debug!("   - From: {} ({})", nickname, peer_id);
                    debug!("   - Group: {}", group_name);
                    debug!("   - Content: {}", group_message.content);
                    debug!("   - Timestamp: {}", group_message.timestamp);

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
                        let group_name_clone = group_name.clone();
                        self.send_event(P2pEvent::GroupMessage {
                            from: peer_id,
                            from_nickname: nickname,
                            group: group_name,
                            message: group_message.content,
                        });
                        debug!("Emitted GroupMessage event for group: {}", group_name_clone);
                    }
                } else {
                    warn!("Failed to parse group message from gossipsub data");
                    debug!("Raw data: {:?}", String::from_utf8(message.data));
                }
            }
            libp2p::gossipsub::Event::Subscribed { topic, .. } => {
                let group_name = topic.to_string();
                info!("Successfully subscribed to group topic: {}", group_name);
                self.send_event(P2pEvent::GroupJoined { group: group_name });
            }
            libp2p::gossipsub::Event::Unsubscribed { topic, .. } => {
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
        event: request_response::Event<FileTransferRequest, super::behaviour::FileTransferResponse>,
    ) -> Result<()> {
        if let request_response::Event::Message { message, peer, .. } = event {
            match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    let response = match request {
                        FileTransferRequest::GetFileInfo(file_id) => {
                            let info = self
                                .file_manager
                                .shared_files
                                .get(&file_id)
                                .filter(|f| !f.revoked)
                                .map(|f| f.info.clone());
                            super::behaviour::FileTransferResponse::FileInfo(info)
                        }
                        FileTransferRequest::GetChunk(file_id, chunk_index) => {
                            if let Some(shared_file) = self.file_manager.shared_files.get(&file_id)
                            {
                                if !shared_file.revoked {
                                    match self.file_manager.read_chunk(
                                        &shared_file.path,
                                        chunk_index,
                                        &file_id,
                                    ) {
                                        Ok(chunk) => super::behaviour::FileTransferResponse::Chunk(
                                            Some(chunk),
                                        ),
                                        Err(_) => super::behaviour::FileTransferResponse::Error(
                                            "Failed to read chunk".to_string(),
                                        ),
                                    }
                                } else {
                                    super::behaviour::FileTransferResponse::Error(
                                        "File has been revoked".to_string(),
                                    )
                                }
                            } else {
                                super::behaviour::FileTransferResponse::Chunk(None)
                            }
                        }
                        FileTransferRequest::ListFiles => {
                            let files = self
                                .file_manager
                                .shared_files
                                .values()
                                .filter(|f| !f.revoked)
                                .map(|f| f.info.clone())
                                .collect();
                            super::behaviour::FileTransferResponse::FileList(files)
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
                        super::behaviour::FileTransferResponse::FileInfo(Some(info)) => {
                            // Start download when we receive file info
                            self.file_manager.start_download(peer, info.clone())?;
                            self.send_event(P2pEvent::FileInfoReceived { from: peer, info });
                        }
                        super::behaviour::FileTransferResponse::FileInfo(None) => {
                            // File not found
                        }
                        super::behaviour::FileTransferResponse::Chunk(Some(chunk)) => {
                            self.handle_chunk_received(
                                peer,
                                chunk.file_id.clone(),
                                chunk.chunk_index,
                                chunk,
                            )?;
                        }
                        super::behaviour::FileTransferResponse::Chunk(None) => {
                            // Chunk not found
                        }
                        super::behaviour::FileTransferResponse::FileList(files) => {
                            self.send_event(P2pEvent::FileListReceived { from: peer, files });
                        }
                        super::behaviour::FileTransferResponse::Error(error) => {
                            self.send_event(P2pEvent::Error(error));
                        }
                    }
                }
            }
        }
        Ok(())
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
        let calculated_hash = self.file_manager.calculate_chunk_hash(&chunk.data);
        if calculated_hash != chunk.hash {
            self.send_event(P2pEvent::FileDownloadFailed {
                file_id: file_id.clone(),
                error: format!("Chunk {} hash mismatch", chunk_index),
            });
            return Ok(());
        }

        // Extract necessary data before mutable borrow
        let (should_process, total_chunks) =
            if let Some(downloading_file) = self.file_manager.downloading_files.get(&file_id) {
                (true, downloading_file.info.chunk_count)
            } else {
                (false, 0)
            };

        if should_process {
            // Write chunk to temp file
            let offset = chunk_index * CHUNK_SIZE;
            let temp_path = self.file_manager.downloading_files[&file_id]
                .temp_path
                .clone();
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
                    if let Some(downloading_file) =
                        self.file_manager.downloading_files.get_mut(&file_id)
                    {
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
                            match self.file_manager.calculate_file_hash(&temp_path) {
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
                            self.file_manager.downloading_files.remove(&file_id);
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

    /// Handle peer discovery
    pub fn handle_peer_discovered(&mut self, peer_id: PeerId, addr: Multiaddr) -> Result<()> {
        if !self.peers.contains_key(&peer_id) {
            let nickname = peer_id.to_string(); // Default nickname

            self.nickname_to_peer.insert(nickname.clone(), peer_id);

            // Request nickname from peer
            debug!("Sending GetNickname request to {}", peer_id);
            self.swarm
                .behaviour_mut()
                .nickname
                .send_request(&peer_id, NicknameRequest::GetNickname);

            // Announce our nickname
            debug!(
                "Sending AnnounceNickname request to {} as {}",
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

    /// Gracefully shutdown the client and notify all peers
    pub fn shutdown(&mut self) -> Result<()> {
        // Close all connections and notify peers
        let connected_peers: Vec<PeerId> = self.peers.keys().copied().collect();
        for peer_id in connected_peers {
            if let Some(peer) = self.peers.remove(&peer_id) {
                self.nickname_to_peer.remove(&peer.nickname);
                self.send_event(P2pEvent::Disconnected {
                    peer_id,
                    nickname: peer.nickname.clone(),
                });
            }
        }

        Ok(())
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

    /// Send group share message to peer
    pub fn send_direct_share_group_message(
        &mut self,
        nickname: &str,
        group_id: String,
        group_name: String,
    ) -> Result<()> {
        let peer_id = *self
            .nickname_to_peer
            .get(nickname)
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
    #[instrument(skip(self))]
    pub fn join_group(&mut self, group_name: &str) -> Result<()> {
        info!("Joining group: {}", group_name);
        use libp2p::gossipsub::IdentTopic;
        let topic = IdentTopic::new(group_name);

        // Check if already subscribed
        if self.groups.contains_key(group_name) {
            warn!("Already subscribed to group: {}", group_name);
            return Ok(());
        }

        self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

        let group_info = GroupInfo {
            name: group_name.to_string(),
            topic,
            joined_at: chrono::Utc::now(),
        };

        self.groups.insert(group_name.to_string(), group_info);
        info!("Successfully joined group: {}", group_name);

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
    #[instrument(skip(self, message))]
    pub fn send_group_message(&mut self, group_name: &str, message: String) -> Result<()> {
        debug!("Sending group message to: {}", group_name);

        let group = self
            .groups
            .get(group_name)
            .ok_or_else(|| P2pError::GroupNotFound(group_name.to_string()))?;

        let group_message = GroupMessage {
            sender_nickname: self.local_nickname.clone(),
            content: message,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
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

        debug!("Group message published successfully");
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
        let share_code = self.file_manager.share_file(image_path).await?;

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
        self.file_manager.share_file(file_path).await
    }

    /// List shared files
    pub fn list_shared_files(&self) -> Vec<&super::events::SharedFile> {
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
}
