//! Event handling for P2P client swarm events

use anyhow::Result;
use libp2p::{swarm::SwarmEvent, PeerId};
use tracing::info;

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
                info!("Connection established with peer: {}", peer_id);
                self.client
                    .peer_manager
                    .handle_connection_established(peer_id, &mut self.client.event_sender);

                // Trigger sync if persistence is enabled (simplified - no async for now)
                if let Some(ref _sync_manager) = self.client.sync_manager {
                    if let Ok(nickname) = self.client.peer_manager.get_peer_nickname(&peer_id) {
                        info!("Sending PendingMessagesAvailable event for {}", nickname);
                        // TODO: Implement proper async sync triggering
                        self.client.send_event(P2pEvent::PendingMessagesAvailable {
                            peer: peer_id,
                            nickname,
                        });
                    }
                }
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                self.client
                    .peer_manager
                    .handle_connection_closed(peer_id, &mut self.client.event_sender);

                // Notify sync manager if persistence is enabled (simplified - no async for now)
                if let Some(ref _sync_manager) = self.client.sync_manager {
                    // TODO: Implement proper async offline notification
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle unified network events by delegating to specific handlers
    fn handle_unified_event(&mut self, event: UnifiedEvent) -> Result<()> {
        match event {
            UnifiedEvent::GigiDns(gigi_dns_event) => {
                GigiDnsEventHandler::new(self.client).handle_event(gigi_dns_event)?
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

/// Handles gigi-dns discovery events
pub struct GigiDnsEventHandler<'a> {
    client: &'a mut P2pClient,
}

impl<'a> GigiDnsEventHandler<'a> {
    pub fn new(client: &'a mut P2pClient) -> Self {
        Self { client }
    }

    pub fn handle_event(&mut self, event: gigi_dns::GigiDnsEvent) -> Result<()> {
        match event {
            gigi_dns::GigiDnsEvent::Discovered(peer_info) => {
                info!(
                    "gigi-dns discovered peer: {} ({})",
                    peer_info.nickname, peer_info.peer_id
                );
                self.client.peer_manager.handle_peer_discovered(
                    peer_info.peer_id,
                    peer_info.multiaddr.clone(),
                    &mut self.client.swarm,
                    &peer_info.nickname,
                    &mut self.client.event_sender,
                )?;
            }
            gigi_dns::GigiDnsEvent::Updated {
                peer_id, new_info, ..
            } => {
                info!("gigi-dns updated peer: {} ({})", new_info.nickname, peer_id);
                self.client.peer_manager.update_peer_nickname(
                    peer_id,
                    new_info.nickname.clone(),
                    &mut self.client.event_sender,
                );
            }
            gigi_dns::GigiDnsEvent::Expired { peer_id, info } => {
                info!("gigi-dns expired peer: {} ({})", info.nickname, peer_id);
                self.client
                    .peer_manager
                    .handle_peer_expired(peer_id, &mut self.client.event_sender)?;
            }
            gigi_dns::GigiDnsEvent::Offline { peer_id, info } => {
                info!("gigi-dns peer offline: {} ({})", info.nickname, peer_id);
                self.client
                    .peer_manager
                    .handle_peer_expired(peer_id, &mut self.client.event_sender)?;
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
                    // Note: Message storage is handled by the plugin event handler (handle_direct_message in events.rs)
                    // to avoid duplicates and ensure consistent UUID across storage and event

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
                    request,
                    channel,
                    request_id: _,
                    ..
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
                libp2p::request_response::Message::Response {
                    response,
                    request_id,
                    ..
                } => {
                    self.handle_file_response(response, peer, request_id.to_string())?;
                }
            }
        }
        Ok(())
    }

    fn handle_file_response(
        &mut self,
        response: crate::behaviour::FileSharingResponse,
        peer: PeerId,
        request_id: String,
    ) -> Result<()> {
        use crate::behaviour::FileSharingResponse;

        match response {
            FileSharingResponse::FileInfo(Some(info)) => {
                self.handle_file_info_response(info, peer, request_id)?;
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
                self.handle_chunk_response(peer, chunk, request_id)?;
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
        request_id: String,
    ) -> Result<()> {
        use crate::behaviour::FileSharingRequest;

        // Find the pending download_id using the request_id
        let pending_download_id = self
            .client
            .download_manager
            .get_download_by_request_id(&request_id)
            .unwrap_or_else(|| {
                format!(
                    "pending_unknown_{}_{}",
                    request_id,
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                )
            });

        // Clean up the request mapping
        self.client
            .download_manager
            .cleanup_request_mapping(&request_id);

        // Get the download entry to extract share_code
        let share_code = self
            .client
            .download_manager
            .get_active_download(&pending_download_id)
            .map(|d| d.share_code.clone())
            .unwrap_or_else(|| info.id.clone());

        // Get peer nickname
        let from_nickname = self
            .client
            .peer_manager
            .get_peer(&peer)
            .map(|p| p.nickname.clone())
            .unwrap_or_else(|| peer.to_string());

        // Start download when we receive file info, using the pending_download_id for unique temp path
        self.client.download_manager.start_download_file(
            peer,
            info.clone(),
            Some(&pending_download_id),
        )?;
        self.client.send_event(P2pEvent::FileInfoReceived {
            from: peer,
            info: info.clone(),
        });

        // Update the pending download with proper file info using DownloadManager
        let final_download_id = self
            .client
            .download_manager
            .update_download_with_file_info(
                &pending_download_id,
                info.name.clone(),
                info.chunk_count,
                peer,
                from_nickname.clone(),
            )?;

        // Send download started event with correct filename and download_id
        self.client.send_event(P2pEvent::FileDownloadStarted {
            from: peer,
            from_nickname: from_nickname,
            filename: info.name.clone(),
            download_id: final_download_id.clone(),
            share_code: share_code,
        });

        // Start requesting initial chunks with optimized concurrency
        let file_id = info.id.clone();
        let initial_requests = std::cmp::min(10, info.chunk_count);
        let initial_chunk_indices: Vec<usize> = (0..initial_requests).collect();

        // Mark initial chunks as requested using download_id
        self.client
            .download_manager
            .mark_chunks_requested(&final_download_id, &initial_chunk_indices)?;

        // Send initial requests and map request_id to download_id
        for chunk_index in initial_chunk_indices {
            let request_id = self.client.swarm.behaviour_mut().file_sharing.send_request(
                &peer,
                FileSharingRequest::GetChunk(file_id.clone(), chunk_index),
            );
            // Map this chunk request to the download_id so we can route the response correctly
            self.client
                .download_manager
                .map_request_to_download(request_id.to_string(), final_download_id.clone());
        }

        Ok(())
    }

    fn handle_chunk_response(
        &mut self,
        peer: PeerId,
        chunk: crate::events::ChunkInfo,
        request_id: String,
    ) -> Result<()> {
        use crate::behaviour::FileSharingRequest;

        // Find download_id using the request_id mapping
        let download_id = self
            .client
            .download_manager
            .get_download_by_request_id(&request_id)
            .ok_or_else(|| anyhow::anyhow!("No download found for request_id: {}", request_id))?;

        // Clean up the request_id mapping after finding the download_id
        self.client
            .download_manager
            .cleanup_request_mapping(&request_id);

        // Process chunk through DownloadManager using download_id
        match self.client.download_manager.process_received_chunk(
            &download_id,
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
                self.client
                    .download_manager
                    .update_download_progress(&download_id, downloaded_count);

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
                        .remove_downloading_file(&download_id);
                } else {
                    // Get next chunks to request
                    if let Some(next_chunks) = self
                        .client
                        .download_manager
                        .get_next_chunks_to_request(&download_id, 10)
                    {
                        // Mark them as requested
                        self.client
                            .download_manager
                            .mark_chunks_requested(&download_id, &next_chunks)?;

                        // Send requests
                        for chunk_idx in next_chunks {
                            let request_id =
                                self.client.swarm.behaviour_mut().file_sharing.send_request(
                                    &peer,
                                    FileSharingRequest::GetChunk(chunk.file_id.clone(), chunk_idx),
                                );
                            // Map this chunk request to download_id
                            self.client.download_manager.map_request_to_download(
                                request_id.to_string(),
                                download_id.clone(),
                            );
                        }
                    }
                }
            }
            super::download_manager::ChunkProcessResult::HashMismatch => {
                self.send_download_failed_event(
                    &download_id,
                    format!("Chunk {} hash mismatch", chunk.chunk_index),
                );
            }
            super::download_manager::ChunkProcessResult::WriteFailed(error) => {
                self.send_download_failed_event(
                    &download_id,
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
        download_id: &str,
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
                                download_id,
                                format!("Failed to rename file: {}", e),
                            );
                        }
                    }
                } else {
                    self.send_download_failed_event(
                        download_id,
                        "File hash verification failed".to_string(),
                    );
                }
            }
            Err(e) => {
                self.send_download_failed_event(
                    download_id,
                    format!("Failed to calculate file hash: {}", e),
                );
            }
        }
        Ok(())
    }

    fn send_progress_event(
        &mut self,
        download_id: &str,
        downloaded_count: usize,
        total_chunks: usize,
    ) {
        let (actual_download_id, filename, share_code, from_nickname, from_peer_id) = self
            .client
            .download_manager
            .get_download_info_for_event(&Some(download_id.to_string()));

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

    fn send_download_completed_event(&mut self, download_id: &str, output_path: &std::path::Path) {
        let (actual_download_id, filename, share_code, from_nickname, from_peer_id) = self
            .client
            .download_manager
            .get_download_info_for_event(&Some(download_id.to_string()));

        // Mark as completed in download manager
        let _completed_download = self
            .client
            .download_manager
            .complete_download(&actual_download_id, output_path.to_path_buf());

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
