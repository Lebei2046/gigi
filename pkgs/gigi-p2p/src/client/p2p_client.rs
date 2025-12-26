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

use super::{
    file_transfer::{FileTransferManager, CHUNK_SIZE},
    group_manager::GroupManager,
};
use crate::behaviour::{
    create_gossipsub_behaviour, create_gossipsub_config, DirectMessage, FileTransferRequest,
    NicknameRequest, UnifiedBehaviour, UnifiedEvent,
};
use crate::error::P2pError;
use crate::events::{ActiveDownload, ChunkInfo, GroupInfo, P2pEvent, PeerInfo};

/// Main P2P client
pub struct P2pClient {
    swarm: libp2p::swarm::Swarm<UnifiedBehaviour>,
    local_nickname: String,

    // Peer management
    peers: HashMap<PeerId, PeerInfo>,
    nickname_to_peer: HashMap<String, PeerId>,

    // Group management
    group_manager: GroupManager,

    // File sharing
    file_manager: FileTransferManager,

    // Active download tracking for mobile apps
    active_downloads: HashMap<String, ActiveDownload>,
    download_share_codes: HashMap<String, String>, // download_id -> share_code mapping

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
            group_manager: GroupManager::new(),
            file_manager,
            active_downloads: HashMap::new(),
            download_share_codes: HashMap::new(),
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
        event: request_response::Event<NicknameRequest, crate::behaviour::NicknameResponse>,
    ) -> Result<()> {
        use crate::behaviour::NicknameResponse;

        if let request_response::Event::Message { message, peer, .. } = event {
            match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    let response = match request {
                        NicknameRequest::AnnounceNickname { nickname } => {
                            self.update_peer_nickname(peer, nickname.clone());
                            NicknameResponse::Ack {
                                nickname: self.local_nickname.clone(),
                            }
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
                    // Handle Ack or Error responses
                    match response {
                        crate::behaviour::NicknameResponse::Ack { nickname } => {
                            debug!("Peer {} acknowledged our nickname announcement with their nickname: {}", peer, nickname);
                            // Update peer with their nickname from the Ack response
                            self.update_peer_nickname(peer, nickname);
                        }
                        crate::behaviour::NicknameResponse::Error(error) => {
                            warn!("Peer {} reported nickname error: {}", peer, error);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Handle direct message events
    fn handle_direct_message_event(
        &mut self,
        event: request_response::Event<DirectMessage, crate::behaviour::DirectResponse>,
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
                DirectMessage::FileShare {
                    share_code,
                    filename,
                    file_size,
                    file_type,
                } => {
                    self.send_event(P2pEvent::DirectFileShareMessage {
                        from: peer,
                        from_nickname: nickname,
                        share_code,
                        filename,
                        file_size,
                        file_type,
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
                .send_response(channel, crate::behaviour::DirectResponse::Ack);
        }
        Ok(())
    }

    /// Handle gossipsub events
    fn handle_gossipsub_event(&mut self, event: libp2p::gossipsub::Event) -> Result<()> {
        self.group_manager
            .handle_gossipsub_event(event, &self.peers, &mut self.event_sender)
    }

    /// Handle file transfer events with chunked file support
    fn handle_file_transfer_event(
        &mut self,
        event: request_response::Event<FileTransferRequest, crate::behaviour::FileTransferResponse>,
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
                            crate::behaviour::FileTransferResponse::FileInfo(info)
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
                                        Ok(chunk) => crate::behaviour::FileTransferResponse::Chunk(
                                            Some(chunk),
                                        ),
                                        Err(_) => crate::behaviour::FileTransferResponse::Error(
                                            "Failed to read chunk".to_string(),
                                        ),
                                    }
                                } else {
                                    crate::behaviour::FileTransferResponse::Error(
                                        "File has been revoked".to_string(),
                                    )
                                }
                            } else {
                                crate::behaviour::FileTransferResponse::Chunk(None)
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
                            crate::behaviour::FileTransferResponse::FileList(files)
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
                        crate::behaviour::FileTransferResponse::FileInfo(Some(info)) => {
                            // Start download when we receive file info
                            self.file_manager.start_download(peer, info.clone())?;
                            self.send_event(P2pEvent::FileInfoReceived {
                                from: peer,
                                info: info.clone(),
                            });

                            // Find and update the active download entry
                            let from_nickname = self
                                .peers
                                .get(&peer)
                                .map(|p| p.nickname.clone())
                                .unwrap_or_else(|| peer.to_string());

                            // Find the share code from pending downloads or use file_id as fallback
                            let share_code = self
                                .find_share_code_for_file(&info.id)
                                .unwrap_or_else(|| info.id.clone());

                            // Create unique download_id for this specific download
                            let download_id = format!(
                                "dl_{}_{}_{}",
                                info.id,
                                peer,
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs()
                            );

                            // Store share code mapping
                            self.download_share_codes
                                .insert(download_id.clone(), share_code.clone());

                            // Update or create active download entry
                            let active_download = ActiveDownload {
                                download_id: download_id.clone(),
                                filename: info.name.clone(),
                                share_code: share_code.clone(),
                                from_peer_id: peer,
                                from_nickname: from_nickname.clone(),
                                total_chunks: info.chunk_count,
                                downloaded_chunks: 0,
                                started_at: std::time::Instant::now(),
                                completed: false,
                                failed: false,
                                error_message: None,
                                final_path: None,
                            };
                            self.active_downloads
                                .insert(download_id.clone(), active_download);

                            // Send download started event
                            self.send_event(P2pEvent::FileDownloadStarted {
                                from: peer,
                                from_nickname: from_nickname,
                                filename: info.name.clone(),
                            });

                            // Start requesting initial chunks
                            let file_id = info.id.clone();
                            let total_chunks = info.chunk_count;
                            let initial_requests = std::cmp::min(5, total_chunks); // Start with up to 5 concurrent requests

                            for chunk_index in 0..initial_requests {
                                let _request_id =
                                    self.swarm.behaviour_mut().file_transfer.send_request(
                                        &peer,
                                        FileTransferRequest::GetChunk(file_id.clone(), chunk_index),
                                    );
                            }
                        }
                        crate::behaviour::FileTransferResponse::FileInfo(None) => {
                            // File not found - send error event
                            self.send_event(P2pEvent::FileDownloadFailed {
                                download_id: "unknown".to_string(),
                                filename: "Unknown".to_string(),
                                share_code: "unknown".to_string(),
                                from_peer_id: libp2p::PeerId::random(),
                                from_nickname: "Unknown".to_string(),
                                error: "File not found".to_string(),
                            });
                        }
                        crate::behaviour::FileTransferResponse::Chunk(Some(chunk)) => {
                            self.handle_chunk_received(
                                peer,
                                chunk.file_id.clone(), // Still use file_id for chunk requests (different concept)
                                chunk.chunk_index,
                                chunk,
                            )?;
                        }
                        crate::behaviour::FileTransferResponse::Chunk(None) => {
                            // Chunk not found
                        }
                        crate::behaviour::FileTransferResponse::FileList(files) => {
                            self.send_event(P2pEvent::FileListReceived { from: peer, files });
                        }
                        crate::behaviour::FileTransferResponse::Error(error) => {
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
        // Find the download_id for this file_id
        let download_id = self.find_download_id_by_file_id(&file_id);

        // Verify chunk hash
        if !self.verify_chunk_hash(&chunk) {
            self.send_download_failed_event(
                download_id.as_deref().unwrap_or("unknown"),
                format!("Chunk {} hash mismatch", chunk_index),
            );
            return Ok(());
        }

        // Extract necessary data before mutable borrow
        let Some((total_chunks, temp_path)) = self.get_download_info(&file_id) else {
            return Ok(());
        };

        // Write chunk to temp file
        if let Err(e) = self.write_chunk_to_file(&temp_path, chunk_index, &chunk.data) {
            self.send_download_failed_event(
                download_id.as_deref().unwrap_or("unknown"),
                format!("Failed to write chunk: {}", e),
            );
            return Ok(());
        }

        // Update download progress
        self.update_download_progress(&file_id, &peer, chunk_index, total_chunks, &download_id)
    }

    /// Verify chunk hash
    fn verify_chunk_hash(&self, chunk: &ChunkInfo) -> bool {
        let calculated_hash = self.file_manager.calculate_chunk_hash(&chunk.data);
        calculated_hash == chunk.hash
    }

    /// Get download info from file manager
    fn get_download_info(&self, file_id: &str) -> Option<(usize, PathBuf)> {
        self.file_manager
            .downloading_files
            .get(file_id)
            .map(|downloading_file| {
                (
                    downloading_file.info.chunk_count,
                    downloading_file.temp_path.clone(),
                )
            })
    }

    /// Write chunk data to file at specific offset
    fn write_chunk_to_file(&self, temp_path: &Path, chunk_index: usize, data: &[u8]) -> Result<()> {
        use std::io::{Seek, Write};

        let offset = chunk_index * CHUNK_SIZE;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(temp_path)?;

        file.seek(std::io::SeekFrom::Start(offset as u64))?;
        file.write_all(data)?;
        file.flush()?;
        Ok(())
    }

    /// Update download progress and handle completion
    fn update_download_progress(
        &mut self,
        file_id: &str,
        peer: &PeerId,
        chunk_index: usize,
        total_chunks: usize,
        download_id: &Option<String>,
    ) -> Result<()> {
        // Update downloading file info
        let (downloaded_count, output_path, expected_hash) =
            if let Some(downloading_file) = self.file_manager.downloading_files.get_mut(file_id) {
                downloading_file.downloaded_chunks.insert(chunk_index, true);

                let downloaded_count = downloading_file
                    .downloaded_chunks
                    .values()
                    .filter(|&&v| v)
                    .count();

                // Request next chunk if available (sliding window)
                if downloading_file.next_chunk_to_request < total_chunks {
                    let next_chunk = downloading_file.next_chunk_to_request;
                    downloading_file.next_chunk_to_request += 1;

                    let _request_id = self.swarm.behaviour_mut().file_transfer.send_request(
                        peer,
                        FileTransferRequest::GetChunk(file_id.to_string(), next_chunk),
                    );
                }

                (
                    downloaded_count,
                    downloading_file.output_path.clone(),
                    downloading_file.info.hash.clone(),
                )
            } else {
                return Ok(());
            };

        // Update active download progress
        if let Some(download_id) = download_id {
            if let Some(active_download) = self.active_downloads.get_mut(download_id) {
                active_download.downloaded_chunks = downloaded_count;
            }
        }

        // Send progress event
        self.send_progress_event(download_id, downloaded_count, total_chunks);

        // Check if download is complete
        if downloaded_count == total_chunks {
            let Some(temp_path) = self
                .file_manager
                .downloading_files
                .get(file_id)
                .map(|f| f.temp_path.clone())
            else {
                return Ok(());
            };

            self.handle_download_complete(&temp_path, &output_path, &expected_hash, download_id)?;

            // Remove from downloading files
            self.file_manager.downloading_files.remove(file_id);
        }

        Ok(())
    }

    /// Handle download completion and verification
    fn handle_download_complete(
        &mut self,
        temp_path: &Path,
        output_path: &Path,
        expected_hash: &str,
        download_id: &Option<String>,
    ) -> Result<()> {
        // Verify file hash
        match self.file_manager.calculate_file_hash(temp_path) {
            Ok(file_hash) => {
                if file_hash == expected_hash {
                    // Rename temp file to final name
                    match std::fs::rename(temp_path, output_path) {
                        Ok(_) => {
                            self.send_download_completed_event(download_id, output_path);
                        }
                        Err(e) => {
                            self.send_download_failed_event(
                                download_id.as_deref().unwrap_or("unknown"),
                                format!("Failed to rename file: {}", e),
                            );
                        }
                    }
                } else {
                    self.send_download_failed_event(
                        download_id.as_deref().unwrap_or("unknown"),
                        "File hash verification failed".to_string(),
                    );
                }
            }
            Err(e) => {
                self.send_download_failed_event(
                    download_id.as_deref().unwrap_or("unknown"),
                    format!("Failed to calculate file hash: {}", e),
                );
            }
        }
        Ok(())
    }

    /// Helper to get download info for events
    fn get_download_info_for_event(
        &self,
        download_id: &Option<String>,
    ) -> (String, String, String, String, libp2p::PeerId) {
        if let Some(download_id) = download_id {
            if let Some(active_download) = self.active_downloads.get(download_id) {
                (
                    active_download.download_id.clone(),
                    active_download.filename.clone(),
                    active_download.share_code.clone(),
                    active_download.from_nickname.clone(),
                    active_download.from_peer_id,
                )
            } else {
                (
                    download_id.clone(),
                    "Unknown".to_string(),
                    "Unknown".to_string(),
                    "Unknown".to_string(),
                    libp2p::PeerId::random(),
                )
            }
        } else {
            (
                "unknown".to_string(),
                "Unknown".to_string(),
                "Unknown".to_string(),
                "Unknown".to_string(),
                libp2p::PeerId::random(),
            )
        }
    }

    /// Send download failed event
    fn send_download_failed_event(&mut self, download_id: &str, error: String) {
        let (actual_download_id, filename, share_code, from_nickname, from_peer_id) =
            if let Some(active_download) = self.active_downloads.get(download_id) {
                (
                    active_download.download_id.clone(),
                    active_download.filename.clone(),
                    active_download.share_code.clone(),
                    active_download.from_nickname.clone(),
                    active_download.from_peer_id,
                )
            } else {
                (
                    download_id.to_string(),
                    "Unknown".to_string(),
                    "Unknown".to_string(),
                    "Unknown".to_string(),
                    libp2p::PeerId::random(),
                )
            };

        self.send_event(P2pEvent::FileDownloadFailed {
            download_id: actual_download_id,
            filename,
            share_code,
            from_peer_id,
            from_nickname,
            error,
        });
    }

    /// Send progress event
    fn send_progress_event(
        &mut self,
        download_id: &Option<String>,
        downloaded_count: usize,
        total_chunks: usize,
    ) {
        let (actual_download_id, filename, share_code, from_nickname, from_peer_id) =
            self.get_download_info_for_event(download_id);

        self.send_event(P2pEvent::FileDownloadProgress {
            download_id: actual_download_id,
            filename,
            share_code,
            from_peer_id,
            from_nickname,
            downloaded_chunks: downloaded_count,
            total_chunks,
        });
    }

    /// Send download completed event
    fn send_download_completed_event(&mut self, download_id: &Option<String>, output_path: &Path) {
        let (actual_download_id, filename, share_code, from_nickname, from_peer_id) =
            if let Some(download_id) = download_id {
                if let Some(mut active_download) = self.active_downloads.remove(download_id) {
                    active_download.completed = true;
                    active_download.final_path = Some(output_path.to_path_buf());
                    (
                        active_download.download_id,
                        active_download.filename,
                        active_download.share_code,
                        active_download.from_nickname,
                        active_download.from_peer_id,
                    )
                } else {
                    (
                        download_id.clone(),
                        "Unknown".to_string(),
                        "Unknown".to_string(),
                        "Unknown".to_string(),
                        libp2p::PeerId::random(),
                    )
                }
            } else {
                (
                    "unknown".to_string(),
                    "Unknown".to_string(),
                    "Unknown".to_string(),
                    "Unknown".to_string(),
                    libp2p::PeerId::random(),
                )
            };

        self.send_event(P2pEvent::FileDownloadCompleted {
            download_id: actual_download_id,
            filename,
            share_code,
            from_peer_id,
            from_nickname,
            path: output_path.to_path_buf(),
        });
    }

    /// Handle peer discovery
    pub fn handle_peer_discovered(&mut self, peer_id: PeerId, addr: Multiaddr) -> Result<()> {
        if !self.peers.contains_key(&peer_id) {
            let nickname = peer_id.to_string(); // Default nickname

            self.nickname_to_peer.insert(nickname.clone(), peer_id);

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

    /// Send file to peer using file sharing
    pub async fn send_direct_file(&mut self, nickname: &str, file_path: &Path) -> Result<()> {
        let peer_id = *self
            .nickname_to_peer
            .get(nickname)
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
    pub fn download_file(&mut self, nickname: &str, share_code: &str) -> Result<()> {
        let peer_id = *self
            .nickname_to_peer
            .get(nickname)
            .ok_or_else(|| P2pError::NicknameNotFound(nickname.to_string()))?;

        // Track download request with temporary entry
        let temp_download_id = format!(
            "pending_{}_{}",
            share_code,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        let active_download = ActiveDownload {
            download_id: temp_download_id.clone(),
            filename: "Loading...".to_string(), // Will be updated when file info arrives
            share_code: share_code.to_string(),
            from_peer_id: peer_id,
            from_nickname: nickname.to_string(),
            total_chunks: 0,
            downloaded_chunks: 0,
            started_at: std::time::Instant::now(),
            completed: false,
            failed: false,
            error_message: None,
            final_path: None,
        };
        self.active_downloads
            .insert(temp_download_id.clone(), active_download);

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
        self.group_manager.list_groups()
    }

    // ===== Active Download Tracking Methods for Mobile Apps =====

    /// Get all active downloads
    pub fn get_active_downloads(&self) -> Vec<&ActiveDownload> {
        self.active_downloads.values().collect()
    }

    /// Get active download by download_id
    pub fn get_active_download(&self, download_id: &str) -> Option<&ActiveDownload> {
        self.active_downloads.get(download_id)
    }

    /// Get active download by share code
    pub fn get_download_by_share_code(&self, share_code: &str) -> Option<&ActiveDownload> {
        self.active_downloads
            .values()
            .find(|download| download.share_code == share_code)
    }

    /// Remove completed or failed downloads (cleanup)
    pub fn cleanup_downloads(&mut self) {
        self.active_downloads
            .retain(|_, download| !download.completed && !download.failed);
    }

    /// Helper to find share code for a download (looks in pending downloads first)
    fn find_share_code_for_file(&self, file_id: &str) -> Option<String> {
        // First check if we have it mapped
        if let Some(share_code) = self.download_share_codes.get(file_id) {
            return Some(share_code.clone());
        }

        // Look for pending downloads with this file_id pattern
        self.active_downloads
            .values()
            .find(|download| {
                download.download_id.contains(file_id)
                    || download.download_id.starts_with("pending_")
            })
            .map(|download| download.share_code.clone())
    }

    /// Helper to find download_id by file_id
    fn find_download_id_by_file_id(&self, file_id: &str) -> Option<String> {
        // Look through active downloads to find the one with this file_id
        self.active_downloads
            .values()
            .find(|download| download.download_id.contains(file_id))
            .map(|download| download.download_id.clone())
    }

    /// Get downloads from a specific peer
    pub fn get_downloads_from_peer(&self, peer_nickname: &str) -> Vec<&ActiveDownload> {
        self.active_downloads
            .values()
            .filter(|download| download.from_nickname == peer_nickname)
            .collect()
    }

    /// Get completed downloads (useful for UI history)
    pub fn get_recent_downloads(&self, limit: usize) -> Vec<&ActiveDownload> {
        let mut downloads: Vec<&ActiveDownload> = self
            .active_downloads
            .values()
            .filter(|download| download.completed || download.failed)
            .collect();

        // Sort by started time (most recent first)
        downloads.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        downloads.truncate(limit);
        downloads
    }
}
