//! Event handling for P2P client swarm events

use anyhow::Result;
use libp2p::{swarm::SwarmEvent, PeerId};
use tracing::{debug, info, warn};

use super::P2pClient;
use crate::behaviour::UnifiedEvent;
use crate::events::{P2pEvent, PeerInfo};

/// Handles all swarm-level events
pub struct SwarmEventHandler<'a> {
    client: &'a mut P2pClient,
}

impl<'a> SwarmEventHandler<'a> {
    pub fn new(client: &'a mut P2pClient) -> Self {
        Self { client }
    }

    /// Handle a single swarm event
    pub fn handle_event(&mut self, event: SwarmEvent<UnifiedEvent>) -> Result<()> {
        match event {
            SwarmEvent::Behaviour(unified_event) => {
                self.handle_unified_event(unified_event)?;
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                self.client.send_event(P2pEvent::ListeningOn { address });
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                self.client
                    .peer_manager
                    .handle_connection_established(peer_id, &mut self.client.event_sender);
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                self.client
                    .peer_manager
                    .handle_connection_closed(peer_id, &mut self.client.event_sender);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle unified network events by delegating to specific handlers
    fn handle_unified_event(&mut self, event: UnifiedEvent) -> Result<()> {
        match event {
            UnifiedEvent::Mdns(mdns_event) => {
                MdnsEventHandler::new(self.client).handle_event(mdns_event)?
            }
            UnifiedEvent::Nickname(nickname_event) => {
                NicknameEventHandler::new(self.client).handle_event(nickname_event)?
            }
            UnifiedEvent::DirectMessage(dm_event) => {
                DirectMessageEventHandler::new(self.client).handle_event(dm_event)?
            }
            UnifiedEvent::Gossipsub(gossip_event) => {
                GossipsubEventHandler::new(self.client).handle_event(gossip_event)?
            }
            UnifiedEvent::FileSharing(file_event) => {
                FileSharingEventHandler::new(self.client).handle_event(file_event)?
            }
        }
        Ok(())
    }
}

/// Handles mDNS discovery events
pub struct MdnsEventHandler<'a> {
    client: &'a mut P2pClient,
}

impl<'a> MdnsEventHandler<'a> {
    pub fn new(client: &'a mut P2pClient) -> Self {
        Self { client }
    }

    pub fn handle_event(&mut self, event: libp2p::mdns::Event) -> Result<()> {
        match event {
            libp2p::mdns::Event::Discovered(list) => {
                info!("mDNS discovered {} peers", list.len());
                for (peer_id, addr) in list {
                    debug!("Found peer: {} at {}", peer_id, addr);
                    self.client.peer_manager.handle_peer_discovered(
                        peer_id,
                        addr,
                        &mut self.client.swarm,
                        &self.client.local_nickname,
                        &mut self.client.event_sender,
                    )?;
                }
            }
            libp2p::mdns::Event::Expired(list) => {
                info!("mDNS expired {} peers", list.len());
                for (peer_id, _addr) in list {
                    self.client
                        .peer_manager
                        .handle_peer_expired(peer_id, &mut self.client.event_sender)?;
                }
            }
        }
        Ok(())
    }
}

/// Handles nickname protocol events
pub struct NicknameEventHandler<'a> {
    client: &'a mut P2pClient,
}

impl<'a> NicknameEventHandler<'a> {
    pub fn new(client: &'a mut P2pClient) -> Self {
        Self { client }
    }

    pub fn handle_event(
        &mut self,
        event: libp2p::request_response::Event<
            crate::behaviour::NicknameRequest,
            crate::behaviour::NicknameResponse,
        >,
    ) -> Result<()> {
        use crate::behaviour::{NicknameRequest, NicknameResponse};

        if let libp2p::request_response::Event::Message { message, peer, .. } = event {
            match message {
                libp2p::request_response::Message::Request {
                    request, channel, ..
                } => {
                    let response = match request {
                        NicknameRequest::AnnounceNickname { nickname } => {
                            self.client.peer_manager.update_peer_nickname(
                                peer,
                                nickname.clone(),
                                &mut self.client.event_sender,
                            );
                            NicknameResponse::Ack {
                                nickname: self.client.local_nickname.clone(),
                            }
                        }
                    };
                    debug!("Sending nickname response to {}: {:?}", peer, response);
                    let _ = self
                        .client
                        .swarm
                        .behaviour_mut()
                        .nickname
                        .send_response(channel, response);
                }
                libp2p::request_response::Message::Response { response, .. } => {
                    debug!("Received nickname response from {}: {:?}", peer, response);
                    match response {
                        NicknameResponse::Ack { nickname } => {
                            debug!("Peer {} acknowledged our nickname announcement with their nickname: {}", peer, nickname);
                            self.client.peer_manager.update_peer_nickname(
                                peer,
                                nickname,
                                &mut self.client.event_sender,
                            );
                        }
                        NicknameResponse::Error(error) => {
                            warn!("Peer {} reported nickname error: {}", peer, error);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// Handles direct message events
pub struct DirectMessageEventHandler<'a> {
    client: &'a mut P2pClient,
}

impl<'a> DirectMessageEventHandler<'a> {
    pub fn new(client: &'a mut P2pClient) -> Self {
        Self { client }
    }

    pub fn handle_event(
        &mut self,
        event: libp2p::request_response::Event<
            crate::behaviour::DirectMessage,
            crate::behaviour::DirectResponse,
        >,
    ) -> Result<()> {
        use crate::behaviour::{DirectMessage, DirectResponse};

        if let libp2p::request_response::Event::Message {
            message:
                libp2p::request_response::Message::Request {
                    request, channel, ..
                },
            peer,
            ..
        } = event
        {
            let nickname = self.client.peer_manager.get_peer_nickname(&peer)?;
            match request {
                DirectMessage::Text { message } => {
                    self.client.send_event(P2pEvent::DirectMessage {
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
                    self.client.send_event(P2pEvent::DirectFileShareMessage {
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
                    self.client.send_event(P2pEvent::DirectGroupShareMessage {
                        from: peer,
                        from_nickname: nickname,
                        group_id,
                        group_name,
                    });
                }
            }
            let _ = self
                .client
                .swarm
                .behaviour_mut()
                .direct_msg
                .send_response(channel, DirectResponse::Ack);
        }
        Ok(())
    }
}

/// Handles gossipsub events
pub struct GossipsubEventHandler<'a> {
    client: &'a mut P2pClient,
}

impl<'a> GossipsubEventHandler<'a> {
    pub fn new(client: &'a mut P2pClient) -> Self {
        Self { client }
    }

    pub fn handle_event(&mut self, event: libp2p::gossipsub::Event) -> Result<()> {
        // Convert PeerInfo references to owned PeerInfo values for GroupManager
        let peers: std::collections::HashMap<PeerId, PeerInfo> = self
            .client
            .peer_manager
            .list_peers()
            .into_iter()
            .map(|peer| (peer.peer_id, peer.clone()))
            .collect();
        self.client.group_manager.handle_gossipsub_event(
            event,
            &peers,
            &mut self.client.event_sender,
        )
    }
}

/// Handles file sharing events
pub struct FileSharingEventHandler<'a> {
    client: &'a mut P2pClient,
}

impl<'a> FileSharingEventHandler<'a> {
    pub fn new(client: &'a mut P2pClient) -> Self {
        Self { client }
    }

    pub fn handle_event(
        &mut self,
        event: libp2p::request_response::Event<
            crate::behaviour::FileSharingRequest,
            crate::behaviour::FileSharingResponse,
        >,
    ) -> Result<()> {
        use crate::behaviour::{FileSharingRequest, FileSharingResponse};

        if let libp2p::request_response::Event::Message { message, peer, .. } = event {
            match message {
                libp2p::request_response::Message::Request {
                    request, channel, ..
                } => {
                    let response = match request {
                        FileSharingRequest::GetFileInfo(file_id) => {
                            let info = self
                                .client
                                .file_manager
                                .shared_files
                                .get(&file_id)
                                .filter(|f| !f.revoked)
                                .map(|f| f.info.clone());
                            FileSharingResponse::FileInfo(info)
                        }
                        FileSharingRequest::GetChunk(file_id, chunk_index) => {
                            if let Some(shared_file) =
                                self.client.file_manager.shared_files.get(&file_id)
                            {
                                if !shared_file.revoked {
                                    match self.client.download_manager.read_chunk(
                                        &shared_file.path,
                                        chunk_index,
                                        &file_id,
                                    ) {
                                        Ok(chunk) => FileSharingResponse::Chunk(Some(chunk)),
                                        Err(_) => FileSharingResponse::Error(
                                            "Failed to read chunk".to_string(),
                                        ),
                                    }
                                } else {
                                    FileSharingResponse::Error("File has been revoked".to_string())
                                }
                            } else {
                                FileSharingResponse::Chunk(None)
                            }
                        }
                        FileSharingRequest::ListFiles => {
                            let files = self
                                .client
                                .file_manager
                                .shared_files
                                .values()
                                .filter(|f| !f.revoked)
                                .map(|f| f.info.clone())
                                .collect();
                            FileSharingResponse::FileList(files)
                        }
                    };
                    let _ = self
                        .client
                        .swarm
                        .behaviour_mut()
                        .file_sharing
                        .send_response(channel, response);
                }
                libp2p::request_response::Message::Response { response, .. } => {
                    self.handle_file_response(response, peer)?;
                }
            }
        }
        Ok(())
    }

    fn handle_file_response(
        &mut self,
        response: crate::behaviour::FileSharingResponse,
        peer: PeerId,
    ) -> Result<()> {
        use crate::behaviour::FileSharingResponse;

        match response {
            FileSharingResponse::FileInfo(Some(info)) => {
                self.handle_file_info_response(info, peer)?;
            }
            FileSharingResponse::FileInfo(None) => {
                self.client.send_event(P2pEvent::FileDownloadFailed {
                    download_id: "unknown".to_string(),
                    filename: "Unknown".to_string(),
                    share_code: "unknown".to_string(),
                    from_peer_id: libp2p::PeerId::random(),
                    from_nickname: "Unknown".to_string(),
                    error: "File not found".to_string(),
                });
            }
            FileSharingResponse::Chunk(Some(chunk)) => {
                self.handle_chunk_response(peer, chunk)?;
            }
            FileSharingResponse::Chunk(None) => {
                // Chunk not found
            }
            FileSharingResponse::FileList(files) => {
                self.client
                    .send_event(P2pEvent::FileListReceived { from: peer, files });
            }
            FileSharingResponse::Error(error) => {
                self.client.send_event(P2pEvent::Error(error));
            }
        }
        Ok(())
    }

    fn handle_file_info_response(
        &mut self,
        info: crate::events::FileInfo,
        peer: PeerId,
    ) -> Result<()> {
        use crate::behaviour::FileSharingRequest;

        // Start download when we receive file info
        self.client
            .download_manager
            .start_download_file(peer, info.clone())?;
        self.client.send_event(P2pEvent::FileInfoReceived {
            from: peer,
            info: info.clone(),
        });

        // Find and update the active download entry
        let from_nickname = self
            .client
            .peer_manager
            .get_peer(&peer)
            .map(|p| p.nickname.clone())
            .unwrap_or_else(|| peer.to_string());

        // Find the share code from pending downloads or use file_id as fallback
        let share_code = self
            .client
            .download_manager
            .find_share_code_for_file(&info.id)
            .unwrap_or_else(|| info.id.clone());

        // Find the pending download to update it with proper info
        let pending_download_id = self
            .client
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
        let _final_download_id = self
            .client
            .download_manager
            .update_download_with_file_info(
                &pending_download_id,
                info.name.clone(),
                info.chunk_count,
                peer,
                from_nickname.clone(),
            )?;

        // Send download started event with correct filename
        self.client.send_event(P2pEvent::FileDownloadStarted {
            from: peer,
            from_nickname: from_nickname,
            filename: info.name.clone(),
        });

        // Start requesting initial chunks with optimized concurrency
        let file_id = info.id.clone();
        let initial_requests = std::cmp::min(10, info.chunk_count);
        let initial_chunk_indices: Vec<usize> = (0..initial_requests).collect();

        // Mark initial chunks as requested
        self.client
            .download_manager
            .mark_chunks_requested(&file_id, &initial_chunk_indices)?;

        // Send initial requests
        for chunk_index in initial_chunk_indices {
            let _request_id = self.client.swarm.behaviour_mut().file_sharing.send_request(
                &peer,
                FileSharingRequest::GetChunk(file_id.clone(), chunk_index),
            );
        }

        Ok(())
    }

    fn handle_chunk_response(
        &mut self,
        peer: PeerId,
        chunk: crate::events::ChunkInfo,
    ) -> Result<()> {
        use crate::behaviour::FileSharingRequest;

        // Find download_id for this file_id
        let download_id = self.client.find_download_id_by_file_id(&chunk.file_id);

        // Process chunk through DownloadManager
        match self.client.download_manager.process_received_chunk(
            &chunk.file_id,
            chunk.chunk_index,
            &chunk,
        )? {
            super::download_manager::ChunkProcessResult::Success {
                downloaded_count,
                total_chunks,
                is_complete,
                output_path,
                temp_path,
                expected_hash,
            } => {
                // Update active download progress
                if let Some(download_id) = &download_id {
                    self.client
                        .download_manager
                        .update_download_progress(download_id, downloaded_count);
                }

                // Send progress event
                self.send_progress_event(&download_id, downloaded_count, total_chunks);

                // Check if download is complete
                if is_complete {
                    self.handle_download_complete(
                        &temp_path,
                        &output_path,
                        &expected_hash,
                        &download_id,
                    )?;
                    // Remove from downloading files
                    self.client
                        .download_manager
                        .remove_downloading_file(&chunk.file_id);
                } else {
                    // Get next chunks to request
                    if let Some(next_chunks) = self
                        .client
                        .download_manager
                        .get_next_chunks_to_request(&chunk.file_id, 10)
                    {
                        // Mark them as requested
                        self.client
                            .download_manager
                            .mark_chunks_requested(&chunk.file_id, &next_chunks)?;

                        // Send requests
                        for chunk_idx in next_chunks {
                            let _request_id =
                                self.client.swarm.behaviour_mut().file_sharing.send_request(
                                    &peer,
                                    FileSharingRequest::GetChunk(chunk.file_id.clone(), chunk_idx),
                                );
                        }
                    }
                }
            }
            super::download_manager::ChunkProcessResult::HashMismatch => {
                self.send_download_failed_event(
                    download_id.as_deref().unwrap_or("unknown"),
                    format!("Chunk {} hash mismatch", chunk.chunk_index),
                );
            }
            super::download_manager::ChunkProcessResult::WriteFailed(error) => {
                self.send_download_failed_event(
                    download_id.as_deref().unwrap_or("unknown"),
                    format!("Failed to write chunk: {}", error),
                );
            }
        }

        Ok(())
    }

    fn handle_download_complete(
        &mut self,
        temp_path: &std::path::Path,
        output_path: &std::path::Path,
        expected_hash: &str,
        download_id: &Option<String>,
    ) -> Result<()> {
        // Verify file hash
        match self.client.download_manager.calculate_file_hash(temp_path) {
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

    fn send_progress_event(
        &mut self,
        download_id: &Option<String>,
        downloaded_count: usize,
        total_chunks: usize,
    ) {
        let (actual_download_id, filename, share_code, from_nickname, from_peer_id) = self
            .client
            .download_manager
            .get_download_info_for_event(download_id);

        self.client.send_event(P2pEvent::FileDownloadProgress {
            download_id: actual_download_id,
            filename,
            share_code,
            from_peer_id,
            from_nickname,
            downloaded_chunks: downloaded_count,
            total_chunks,
        });
    }

    fn send_download_failed_event(&mut self, download_id: &str, error: String) {
        // Mark as failed in download manager first
        self.client
            .download_manager
            .fail_download(download_id, error.clone());

        // Get info for the event from download manager
        let (actual_download_id, filename, share_code, from_nickname, from_peer_id) = self
            .client
            .download_manager
            .get_download_info_for_event(&Some(download_id.to_string()));

        self.client.send_event(P2pEvent::FileDownloadFailed {
            download_id: actual_download_id,
            filename,
            share_code,
            from_peer_id,
            from_nickname,
            error,
        });
    }

    fn send_download_completed_event(
        &mut self,
        download_id: &Option<String>,
        output_path: &std::path::Path,
    ) {
        let (actual_download_id, filename, share_code, from_nickname, from_peer_id) =
            if let Some(download_id) = download_id {
                // Mark as completed in download manager first
                let completed_download = self
                    .client
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

        self.client.send_event(P2pEvent::FileDownloadCompleted {
            download_id: actual_download_id,
            filename,
            share_code,
            from_peer_id,
            from_nickname,
            path: output_path.to_path_buf(),
        });
    }
}
