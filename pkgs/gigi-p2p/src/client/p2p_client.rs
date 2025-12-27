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
use std::time::Duration;
use tracing::{debug, error, info, instrument, warn};

use super::{
    download_manager::DownloadManager,
    file_sharing::{FileSharingManager, CHUNK_SIZE},
    group_manager::GroupManager,
    peer_manager::PeerManager,
};
use crate::behaviour::{
    create_gossipsub_behaviour, create_gossipsub_config, DirectMessage, FileSharingRequest,
    NicknameRequest, UnifiedBehaviour, UnifiedEvent,
};
use crate::error::P2pError;
use crate::events::{ActiveDownload, ChunkInfo, GroupInfo, P2pEvent, PeerInfo};

/// Main P2P client
pub struct P2pClient {
    swarm: libp2p::swarm::Swarm<UnifiedBehaviour>,
    local_nickname: String,

    // Peer management
    peer_manager: PeerManager,

    // Group management
    group_manager: GroupManager,

    // File sharing
    file_manager: FileSharingManager,

    // Download management
    download_manager: DownloadManager,

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

        let file_sharing = request_response::cbor::Behaviour::new(
            [(StreamProtocol::new("/file/1.0.0"), ProtocolSupport::Full)],
            request_response::Config::default(),
        );

        // Create unified behaviour
        let behaviour = UnifiedBehaviour {
            mdns,
            nickname: nickname_behaviour,
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
            .with_quic()
            .with_behaviour(|_| behaviour)?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(300)))
            .build();

        let file_manager = FileSharingManager::new(shared_file_path.clone());
        let download_manager = DownloadManager::new(output_directory);

        let mut client = Self {
            swarm,
            local_nickname: nickname,
            peer_manager: PeerManager::new(),
            group_manager: GroupManager::new(),
            file_manager,
            download_manager,
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
                self.peer_manager
                    .handle_connection_established(peer_id, &mut self.event_sender);
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                self.peer_manager
                    .handle_connection_closed(peer_id, &mut self.event_sender);
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
            UnifiedEvent::FileSharing(file_event) => self.handle_file_sharing_event(file_event)?,
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
                    self.peer_manager.handle_peer_discovered(
                        peer_id,
                        addr,
                        &mut self.swarm,
                        &self.local_nickname,
                        &mut self.event_sender,
                    )?;
                }
            }
            mdns::Event::Expired(list) => {
                info!("mDNS expired {} peers", list.len());
                for (peer_id, _addr) in list {
                    self.peer_manager
                        .handle_peer_expired(peer_id, &mut self.event_sender)?;
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
                            self.peer_manager.update_peer_nickname(
                                peer,
                                nickname.clone(),
                                &mut self.event_sender,
                            );
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
                            self.peer_manager.update_peer_nickname(
                                peer,
                                nickname,
                                &mut self.event_sender,
                            );
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
            let nickname = self.peer_manager.get_peer_nickname(&peer)?;
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
        // Convert PeerInfo references to owned PeerInfo values for GroupManager
        let peers: HashMap<PeerId, PeerInfo> = self
            .peer_manager
            .list_peers()
            .into_iter()
            .map(|peer| (peer.peer_id, peer.clone()))
            .collect();
        self.group_manager
            .handle_gossipsub_event(event, &peers, &mut self.event_sender)
    }

    /// Handle file sharing events with chunked file support
    fn handle_file_sharing_event(
        &mut self,
        event: request_response::Event<FileSharingRequest, crate::behaviour::FileSharingResponse>,
    ) -> Result<()> {
        if let request_response::Event::Message { message, peer, .. } = event {
            match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    let response = match request {
                        FileSharingRequest::GetFileInfo(file_id) => {
                            let info = self
                                .file_manager
                                .shared_files
                                .get(&file_id)
                                .filter(|f| !f.revoked)
                                .map(|f| f.info.clone());
                            crate::behaviour::FileSharingResponse::FileInfo(info)
                        }
                        FileSharingRequest::GetChunk(file_id, chunk_index) => {
                            if let Some(shared_file) = self.file_manager.shared_files.get(&file_id)
                            {
                                if !shared_file.revoked {
                                    match self.download_manager.read_chunk(
                                        &shared_file.path,
                                        chunk_index,
                                        &file_id,
                                    ) {
                                        Ok(chunk) => crate::behaviour::FileSharingResponse::Chunk(
                                            Some(chunk),
                                        ),
                                        Err(_) => crate::behaviour::FileSharingResponse::Error(
                                            "Failed to read chunk".to_string(),
                                        ),
                                    }
                                } else {
                                    crate::behaviour::FileSharingResponse::Error(
                                        "File has been revoked".to_string(),
                                    )
                                }
                            } else {
                                crate::behaviour::FileSharingResponse::Chunk(None)
                            }
                        }
                        FileSharingRequest::ListFiles => {
                            let files = self
                                .file_manager
                                .shared_files
                                .values()
                                .filter(|f| !f.revoked)
                                .map(|f| f.info.clone())
                                .collect();
                            crate::behaviour::FileSharingResponse::FileList(files)
                        }
                    };
                    let _ = self
                        .swarm
                        .behaviour_mut()
                        .file_sharing
                        .send_response(channel, response);
                }
                request_response::Message::Response { response, .. } => {
                    match response {
                        crate::behaviour::FileSharingResponse::FileInfo(Some(info)) => {
                            // Start download when we receive file info
                            self.download_manager
                                .start_download_file(peer, info.clone())?;
                            self.send_event(P2pEvent::FileInfoReceived {
                                from: peer,
                                info: info.clone(),
                            });

                            // Find and update the active download entry
                            let from_nickname = self
                                .peer_manager
                                .get_peer(&peer)
                                .map(|p| p.nickname.clone())
                                .unwrap_or_else(|| peer.to_string());

                            // Find the share code from pending downloads or use file_id as fallback
                            let share_code = self
                                .download_manager
                                .find_share_code_for_file(&info.id)
                                .unwrap_or_else(|| info.id.clone());

                            // Find the pending download to update it with proper info
                            let pending_download_id = self
                                .download_manager
                                .get_download_by_share_code(&share_code)
                                .map(|download| download.download_id.clone())
                                .unwrap_or_else(|| {
                                    format!(
                                        "pending_{}_{}",
                                        share_code,
                                        std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap()
                                            .as_secs()
                                    )
                                });

                            // Update the pending download with proper file info using DownloadManager
                            let _final_download_id =
                                self.download_manager.update_download_with_file_info(
                                    &pending_download_id,
                                    info.name.clone(),
                                    info.chunk_count,
                                    peer,
                                    from_nickname.clone(),
                                )?;

                            // Send download started event with correct filename
                            self.send_event(P2pEvent::FileDownloadStarted {
                                from: peer,
                                from_nickname: from_nickname,
                                filename: info.name.clone(),
                            });

                            // Start requesting initial chunks with optimized concurrency
                            let file_id = info.id.clone();
                            let total_chunks = info.chunk_count;
                            let initial_requests = std::cmp::min(10, total_chunks); // Start with up to 10 concurrent requests

                            for chunk_index in 0..initial_requests {
                                let _request_id =
                                    self.swarm.behaviour_mut().file_sharing.send_request(
                                        &peer,
                                        FileSharingRequest::GetChunk(file_id.clone(), chunk_index),
                                    );
                            }
                        }
                        crate::behaviour::FileSharingResponse::FileInfo(None) => {
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
                        crate::behaviour::FileSharingResponse::Chunk(Some(chunk)) => {
                            self.handle_chunk_received(
                                peer,
                                chunk.file_id.clone(), // Still use file_id for chunk requests (different concept)
                                chunk.chunk_index,
                                chunk,
                            )?;
                        }
                        crate::behaviour::FileSharingResponse::Chunk(None) => {
                            // Chunk not found
                        }
                        crate::behaviour::FileSharingResponse::FileList(files) => {
                            self.send_event(P2pEvent::FileListReceived { from: peer, files });
                        }
                        crate::behaviour::FileSharingResponse::Error(error) => {
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
        let calculated_hash = self.download_manager.calculate_chunk_hash(&chunk.data);
        calculated_hash == chunk.hash
    }

    /// Get download info from download manager
    fn get_download_info(&self, file_id: &str) -> Option<(usize, PathBuf)> {
        self.download_manager
            .get_downloading_file(file_id)
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

        let offset = chunk_index
            .checked_mul(CHUNK_SIZE)
            .ok_or_else(|| anyhow::anyhow!("Chunk index overflow: {}", chunk_index))?;
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
        _total_chunks: usize,
        download_id: &Option<String>,
    ) -> Result<()> {
        // Update downloading file info
        let (downloaded_count, output_path, expected_hash, total_chunks) =
            if let Some(downloading_file) = self.download_manager.get_downloading_file_mut(file_id)
            {
                downloading_file.downloaded_chunks.insert(chunk_index, true);

                let downloaded_count = downloading_file
                    .downloaded_chunks
                    .values()
                    .filter(|&&v| v)
                    .count();

                let total_chunks = downloading_file.info.chunk_count;

                // Optimized concurrent download strategy
                let max_concurrent_requests: usize = 10;

                // Calculate how many chunks we should have requested by now
                let chunks_we_should_have_requested =
                    std::cmp::min(downloaded_count + max_concurrent_requests, total_chunks);
                let chunks_already_requested = downloading_file.downloaded_chunks.len();

                // Request more chunks if needed
                if chunks_already_requested < chunks_we_should_have_requested {
                    let requests_to_make =
                        chunks_we_should_have_requested - chunks_already_requested;

                    let mut requested = 0;
                    for next_chunk in 0..total_chunks {
                        if !downloading_file.downloaded_chunks.contains_key(&next_chunk) {
                            let _request_id = self.swarm.behaviour_mut().file_sharing.send_request(
                                peer,
                                FileSharingRequest::GetChunk(file_id.to_string(), next_chunk),
                            );
                            downloading_file.downloaded_chunks.insert(next_chunk, false); // Mark as requested (not downloaded)
                            requested += 1;
                            if requested >= requests_to_make {
                                break;
                            }
                        }
                    }
                }

                (
                    downloaded_count,
                    downloading_file.output_path.clone(),
                    downloading_file.info.hash.clone(),
                    total_chunks,
                )
            } else {
                return Ok(());
            };

        // Update active download progress
        if let Some(download_id) = download_id {
            self.download_manager
                .update_download_progress(download_id, downloaded_count);
        }

        // Send progress event
        self.send_progress_event(download_id, downloaded_count, total_chunks);

        // Check if download is complete
        if downloaded_count >= total_chunks {
            let Some(temp_path) = self
                .download_manager
                .get_downloading_file(file_id)
                .map(|f| f.temp_path.clone())
            else {
                return Ok(());
            };

            self.handle_download_complete(&temp_path, &output_path, &expected_hash, download_id)?;

            // Remove from downloading files
            self.download_manager.remove_downloading_file(file_id);
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
        match self.download_manager.calculate_file_hash(temp_path) {
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
        self.download_manager
            .get_download_info_for_event(download_id)
    }

    /// Send download failed event
    fn send_download_failed_event(&mut self, download_id: &str, error: String) {
        // Mark as failed in download manager first
        self.download_manager
            .fail_download(download_id, error.clone());

        // Get info for the event from download manager
        let (actual_download_id, filename, share_code, from_nickname, from_peer_id) = self
            .download_manager
            .get_download_info_for_event(&Some(download_id.to_string()));

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
                // Mark as completed in download manager first
                let completed_download = self
                    .download_manager
                    .complete_download(download_id, output_path.to_path_buf());

                // Get the info for the event
                if let Some(completed_download) = completed_download {
                    (
                        completed_download.download_id.clone(),
                        completed_download.filename.clone(),
                        completed_download.share_code.clone(),
                        completed_download.from_nickname.clone(),
                        completed_download.from_peer_id,
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

    /// Send direct message to peer
    pub fn send_direct_message(&mut self, nickname: &str, message: String) -> Result<()> {
        let peer_id = self
            .peer_manager
            .get_peer_id_by_nickname(nickname)
            .ok_or_else(|| P2pError::NicknameNotFound(nickname.to_string()))?;

        self.swarm
            .behaviour_mut()
            .direct_msg
            .send_request(&peer_id, DirectMessage::Text { message });

        Ok(())
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
        let peer_id = self
            .peer_manager
            .get_peer_id_by_nickname(nickname)
            .ok_or_else(|| P2pError::NicknameNotFound(nickname.to_string()))?;

        // Track download request with DownloadManager
        let _temp_download_id = self.download_manager.start_download(
            peer_id,
            nickname.to_string(),
            share_code.to_string(),
            None, // filename will be updated when file info arrives
        );

        // First request file info
        let _request_id = self.swarm.behaviour_mut().file_sharing.send_request(
            &peer_id,
            FileSharingRequest::GetFileInfo(share_code.to_string()),
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
    fn find_download_id_by_file_id(&self, file_id: &str) -> Option<String> {
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
}
