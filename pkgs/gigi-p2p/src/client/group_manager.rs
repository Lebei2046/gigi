//! Group management functionality

use anyhow::Result;
use futures::channel::mpsc;
use libp2p::{gossipsub::IdentTopic, PeerId, Swarm};
use std::collections::HashMap;
use tracing::{debug, info, instrument, warn};

use crate::behaviour::UnifiedBehaviour;
use crate::error::P2pError;
use crate::events::{GroupInfo, GroupMessage, P2pEvent};

/// Group management functionality
pub struct GroupManager {
    groups: HashMap<String, GroupInfo>,
}

impl GroupManager {
    /// Create a new group manager
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    /// Join a group
    #[instrument(skip(self, swarm))]
    pub fn join_group(
        &mut self,
        swarm: &mut Swarm<UnifiedBehaviour>,
        group_name: &str,
        event_sender: &mut mpsc::UnboundedSender<P2pEvent>,
    ) -> Result<()> {
        info!("Joining group: {}", group_name);
        let topic = IdentTopic::new(group_name);

        // Check if already subscribed
        if self.groups.contains_key(group_name) {
            warn!("Already subscribed to group: {}", group_name);
            return Ok(());
        }

        swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

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
    pub fn leave_group(
        &mut self,
        swarm: &mut Swarm<UnifiedBehaviour>,
        group_name: &str,
    ) -> Result<()> {
        if let Some(group) = self.groups.remove(group_name) {
            swarm.behaviour_mut().gossipsub.unsubscribe(&group.topic);
        } else {
            return Err(P2pError::GroupNotFound(group_name.to_string()).into());
        }

        Ok(())
    }

    /// Send message to group
    #[instrument(skip(self, swarm, message))]
    pub fn send_group_message(
        &mut self,
        swarm: &mut Swarm<UnifiedBehaviour>,
        group_name: &str,
        message: String,
        local_nickname: &str,
    ) -> Result<()> {
        debug!("Sending group message to: {}", group_name);

        let group = self
            .groups
            .get(group_name)
            .ok_or_else(|| P2pError::GroupNotFound(group_name.to_string()))?;

        let group_message = GroupMessage {
            sender_nickname: local_nickname.to_string(),
            content: message,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(std::time::Duration::from_secs(0))
                .as_secs(),
            has_file_share: false,
            share_code: None,
            filename: None,
            file_size: None,
            file_type: None,
        };

        let data = serde_json::to_vec(&group_message)?;

        swarm
            .behaviour_mut()
            .gossipsub
            .publish(group.topic.clone(), data)?;

        debug!("Group message published successfully");
        Ok(())
    }

    /// Send file to group using file sharing
    pub async fn send_group_file(
        &mut self,
        swarm: &mut Swarm<UnifiedBehaviour>,
        group_name: &str,
        file_path: &std::path::Path,
        file_manager: &mut super::file_transfer::FileTransferManager,
        local_nickname: &str,
    ) -> Result<()> {
        let group_topic = {
            let group = self
                .groups
                .get(group_name)
                .ok_or_else(|| P2pError::GroupNotFound(group_name.to_string()))?;
            group.topic.clone()
        };

        // 1. Share the file to get a share code
        let share_code = file_manager.share_file(file_path).await?;
        let shared_file = file_manager
            .shared_files
            .get(&share_code)
            .ok_or_else(|| crate::error::P2pError::FileNotFound(file_path.to_path_buf()))?;

        let filename = shared_file.info.name.clone();
        let file_size = shared_file.info.size;
        let file_type = mime_guess::from_path(file_path)
            .first_or_octet_stream()
            .to_string();

        // 2. Send message with file share information
        let group_message = GroupMessage {
            sender_nickname: local_nickname.to_string(),
            content: format!("Shared file: {}", filename),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            has_file_share: true,
            share_code: Some(shared_file.share_code.clone()),
            filename: Some(filename),
            file_size: Some(file_size),
            file_type: Some(file_type),
        };

        let msg_data = serde_json::to_vec(&group_message)?;

        swarm
            .behaviour_mut()
            .gossipsub
            .publish(group_topic, msg_data)?;

        debug!("Group image message published successfully");
        Ok(())
    }

    /// Get joined groups
    pub fn list_groups(&self) -> Vec<&GroupInfo> {
        self.groups.values().collect()
    }

    /// Handle gossipsub events related to groups
    pub fn handle_gossipsub_event(
        &mut self,
        event: libp2p::gossipsub::Event,
        peers: &std::collections::HashMap<PeerId, crate::events::PeerInfo>,
        event_sender: &mut mpsc::UnboundedSender<P2pEvent>,
    ) -> Result<()> {
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
                    let nickname = peers
                        .get(&peer_id)
                        .map(|p| p.nickname.clone())
                        .unwrap_or_else(|| peer_id.to_string());

                    debug!("Parsed group message successfully:");
                    debug!("   - From: {} ({})", nickname, peer_id);
                    debug!("   - Group: {}", group_name);
                    debug!("   - Content: {}", group_message.content);
                    debug!("   - Timestamp: {}", group_message.timestamp);

                    if group_message.has_file_share {
                        if let (
                            Some(share_code),
                            Some(filename),
                            Some(file_size),
                            Some(file_type),
                        ) = (
                            group_message.share_code,
                            group_message.filename,
                            group_message.file_size,
                            group_message.file_type,
                        ) {
                            let _ = event_sender.unbounded_send(P2pEvent::GroupFileShareMessage {
                                from: peer_id,
                                from_nickname: nickname,
                                group: group_name,
                                share_code,
                                filename,
                                file_size,
                                file_type,
                                message: group_message.content.clone(),
                            });
                        }
                    } else {
                        let group_name_clone = group_name.clone();
                        let _ = event_sender.unbounded_send(P2pEvent::GroupMessage {
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
                let _ = event_sender.unbounded_send(P2pEvent::GroupJoined { group: group_name });
            }
            libp2p::gossipsub::Event::Unsubscribed { topic, .. } => {
                let group_name = topic.to_string();
                let _ = event_sender.unbounded_send(P2pEvent::GroupLeft { group: group_name });
            }
            _ => {}
        }
        Ok(())
    }
}

impl Default for GroupManager {
    fn default() -> Self {
        Self::new()
    }
}
