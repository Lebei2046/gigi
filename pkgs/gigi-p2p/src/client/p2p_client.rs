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

use std::path::{Path, PathBuf};
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
}

impl P2pClient {
    /// Create a new P2P client
    #[instrument(skip(keypair))]
    pub fn new(
        keypair: Keypair,
        nickname: String,
        output_directory: PathBuf,
    ) -> Result<(Self, mpsc::UnboundedReceiver<P2pEvent>)> {
        let shared_file_path = output_directory.join("shared.json");
        Self::new_with_config(keypair, nickname, output_directory, shared_file_path)
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

    /// Set the chunk reader callback for URI-based files
    pub fn set_chunk_reader(&mut self, reader: super::file_sharing::FileChunkReader) {
        self.file_manager.set_chunk_reader(reader.clone());
        self.download_manager.set_chunk_reader(reader);
    }

    /// Share a content URI (Android content:// or iOS file://)
    pub fn share_content_uri(&mut self, uri: &str, name: &str, size: u64) -> Result<String> {
        self.file_manager.share_content_uri(uri, name, size)
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
}
