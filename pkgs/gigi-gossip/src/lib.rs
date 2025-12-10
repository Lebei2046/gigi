use anyhow::{anyhow, Result};
use futures::channel::mpsc;
use gigi_mdns::{
    NicknameBehaviour, NicknameBehaviourEvent, NicknameManager, NicknameRequest, NicknameResponse,
};
use libp2p::{
    gossipsub::{self, Event as GossipsubEvent, IdentTopic, MessageAuthenticity, MessageId},
    identity::Keypair,
    request_response::{self},
    swarm::{NetworkBehaviour, Swarm},
    PeerId,
};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Text {
        content: String,
        timestamp: u64,
    },
    Image {
        data: Vec<u8>,
        filename: String,
        timestamp: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub sender: String,
    pub message: Message,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub enum GossipEvent {
    MessageReceived {
        from: PeerId,
        sender: String,
        message: Message,
    },
    PeerJoined {
        peer_id: PeerId,
        nickname: String,
    },
    PeerLeft {
        peer_id: PeerId,
        nickname: String,
    },
    Error(String),
}

/// NetworkBehaviour for gossip messaging (always includes mDNS)
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "GossipBehaviourEvent")]
pub struct GossipBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: NicknameBehaviour,
}

#[derive(Debug)]
pub enum GossipBehaviourEvent {
    Gossipsub(GossipsubEvent),
    Mdns(NicknameBehaviourEvent),
}

impl From<GossipsubEvent> for GossipBehaviourEvent {
    fn from(event: GossipsubEvent) -> Self {
        GossipBehaviourEvent::Gossipsub(event)
    }
}

impl From<NicknameBehaviourEvent> for GossipBehaviourEvent {
    fn from(event: NicknameBehaviourEvent) -> Self {
        GossipBehaviourEvent::Mdns(event)
    }
}

/// High-level interface for gossip messaging (always includes mDNS)
pub struct GossipChat {
    pub swarm: Swarm<GossipBehaviour>,
    nickname: String,
    topic: IdentTopic,
    pub event_sender: mpsc::UnboundedSender<GossipEvent>,
    #[allow(dead_code)]
    known_peers: std::collections::HashSet<PeerId>,
    peer_nicknames: std::collections::HashMap<PeerId, String>,
}

/// Create a gossip behaviour for external swarm creation (always includes mDNS)
pub fn create_behaviour(keypair: Keypair) -> Result<GossipBehaviour> {
    // Create message ID function
    let message_id_fn = |message: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        MessageId::from(s.finish().to_string())
    };

    // Configure gossipsub with larger message size for images
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(std::time::Duration::from_secs(10))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .message_id_fn(message_id_fn)
        .max_transmit_size(2 * 1024 * 1024) // 2MB max message size
        .mesh_n_low(1)
        .mesh_n_high(5)
        .history_gossip(3)
        .build()
        .map_err(|e| anyhow!("Failed to build gossipsub config: {}", e))?;

    // Create gossipsub behaviour
    let gossipsub = gossipsub::Behaviour::new(
        MessageAuthenticity::Signed(keypair.clone()),
        gossipsub_config,
    )
    .map_err(|e| anyhow!("Failed to create gossipsub behaviour: {}", e))?;

    // Create mDNS behaviour using gigi-mdns
    let mdns = NicknameManager::create_behaviour(
        keypair.public().to_peer_id(),
        libp2p::mdns::Config::default(),
        libp2p::request_response::Config::default(),
    )
    .map_err(|e| anyhow!("Failed to create mDNS behaviour: {}", e))?;

    Ok(GossipBehaviour { gossipsub, mdns })
}

impl GossipChat {
    /// Create a gossip chat instance with existing swarm (always includes mDNS)
    pub fn with_swarm(
        swarm: Swarm<GossipBehaviour>,
        nickname: String,
        topic: String,
    ) -> Result<(Self, mpsc::UnboundedReceiver<GossipEvent>)> {
        let (event_sender, event_receiver) = mpsc::unbounded();

        let topic = IdentTopic::new(topic);

        let mut chat = Self {
            swarm,
            nickname,
            topic,
            event_sender,
            known_peers: std::collections::HashSet::new(),
            peer_nicknames: std::collections::HashMap::new(),
        };

        // Subscribe to the topic
        chat.subscribe_to_topic()?;

        Ok((chat, event_receiver))
    }

    /// Handle a single event and send to event receiver
    pub fn handle_event(&mut self, event: GossipBehaviourEvent) -> Result<()> {
        let events = match event {
            GossipBehaviourEvent::Gossipsub(gossipsub_event) => {
                self.handle_gossipsub_event(gossipsub_event)?
            }
            GossipBehaviourEvent::Mdns(mdns_event) => self.handle_mdns_event(mdns_event)?,
        };

        // Send all events to the event receiver
        for event in events {
            if let Err(e) = self.event_sender.unbounded_send(event) {
                tracing::error!("Failed to send event to receiver: {}", e);
            }
        }

        Ok(())
    }

    fn handle_gossipsub_event(&mut self, event: GossipsubEvent) -> Result<Vec<GossipEvent>> {
        let mut events = Vec::<GossipEvent>::new();

        match event {
            gossipsub::Event::Message {
                propagation_source: peer_id,
                message,
                ..
            } => {
                if let Ok(chat_message) = serde_json::from_slice::<ChatMessage>(&message.data) {
                    events.push(GossipEvent::MessageReceived {
                        from: peer_id,
                        sender: chat_message.sender,
                        message: chat_message.message,
                    });
                }
            }
            gossipsub::Event::Subscribed { peer_id, topic } => {
                tracing::debug!("Peer {} subscribed to topic {}", peer_id, topic);
            }
            gossipsub::Event::Unsubscribed { peer_id, topic } => {
                tracing::debug!("Peer {} unsubscribed from topic {}", peer_id, topic);
            }
            gossipsub::Event::GossipsubNotSupported { peer_id } => {
                tracing::debug!("Peer {} does not support gossipsub", peer_id);
            }
            gossipsub::Event::SlowPeer { peer_id, .. } => {
                tracing::debug!("Peer {} is slow", peer_id);
            }
        }

        Ok(events)
    }

    fn handle_mdns_event(&mut self, event: NicknameBehaviourEvent) -> Result<Vec<GossipEvent>> {
        let mut events = Vec::<GossipEvent>::new();

        match event {
            NicknameBehaviourEvent::Mdns(mdns_event) => {
                match mdns_event {
                    libp2p::mdns::Event::Discovered(list) => {
                        for (peer_id, _multiaddr) in list {
                            // Only add peer if we haven't seen them before
                            if self.known_peers.insert(peer_id) {
                                // Add peer to gossipsub
                                self.swarm
                                    .behaviour_mut()
                                    .gossipsub
                                    .add_explicit_peer(&peer_id);

                                // Announce our nickname to the new peer
                                if let Err(e) = self.announce_nickname(peer_id) {
                                    tracing::error!(
                                        "Failed to announce nickname to {}: {}",
                                        peer_id,
                                        e
                                    );
                                }

                                // Request their nickname
                                if let Err(e) = self.request_peer_nickname(peer_id) {
                                    tracing::error!(
                                        "Failed to request nickname from {}: {}",
                                        peer_id,
                                        e
                                    );
                                }
                            }
                        }
                    }
                    libp2p::mdns::Event::Expired(list) => {
                        for (peer_id, _multiaddr) in list {
                            // Only remove peer if we know about them
                            if self.known_peers.remove(&peer_id) {
                                // Remove peer from gossipsub
                                self.swarm
                                    .behaviour_mut()
                                    .gossipsub
                                    .remove_explicit_peer(&peer_id);

                                // Use stored nickname or fallback to peer_id
                                let nickname = self
                                    .peer_nicknames
                                    .get(&peer_id)
                                    .cloned()
                                    .unwrap_or_else(|| peer_id.to_string());

                                // Remove from nickname storage
                                self.peer_nicknames.remove(&peer_id);

                                events.push(GossipEvent::PeerLeft { peer_id, nickname });
                            }
                        }
                    }
                }
            }
            NicknameBehaviourEvent::RequestResponse(req_resp_event) => {
                // Handle nickname exchange requests
                if let Some(event) = self.handle_request_response_event(req_resp_event)? {
                    events.push(event);
                }
            }
        }

        Ok(events)
    }

    /// Handle nickname exchange request-response events
    fn handle_request_response_event(
        &mut self,
        event: request_response::Event<NicknameRequest, NicknameResponse>,
    ) -> Result<Option<GossipEvent>> {
        match event {
            request_response::Event::Message { message, peer, .. } => {
                match message {
                    request_response::Message::Request {
                        request, channel, ..
                    } => {
                        // Handle incoming nickname requests
                        let response = match request {
                            NicknameRequest::GetNickname => NicknameResponse::Nickname {
                                peer_id: self.swarm.local_peer_id().to_string(),
                                nickname: self.nickname.clone(),
                            },
                            NicknameRequest::AnnounceNickname { nickname } => {
                                // Store the announced nickname
                                self.peer_nicknames.insert(peer, nickname.clone());

                                // Generate event for nickname update
                                let event = Some(GossipEvent::PeerJoined {
                                    peer_id: peer,
                                    nickname: nickname.clone(),
                                });

                                // Send acknowledgment
                                let _ = self
                                    .swarm
                                    .behaviour_mut()
                                    .mdns
                                    .request_response
                                    .send_response(channel, NicknameResponse::Ack);

                                return Ok(event);
                            }
                            _ => NicknameResponse::Error("Not supported".to_string()),
                        };

                        // Send response
                        let _ = self
                            .swarm
                            .behaviour_mut()
                            .mdns
                            .request_response
                            .send_response(channel, response);

                        Ok(None)
                    }
                    request_response::Message::Response {
                        request_id: _,
                        response,
                        ..
                    } => {
                        // Handle nickname responses
                        match response {
                            NicknameResponse::Nickname {
                                peer_id: _,
                                nickname,
                            } => {
                                // Update stored nickname
                                self.peer_nicknames.insert(peer, nickname.clone());

                                // Generate event if this is a new nickname
                                if !self.known_peers.contains(&peer) {
                                    Ok(Some(GossipEvent::PeerJoined {
                                        peer_id: peer,
                                        nickname: nickname.clone(),
                                    }))
                                } else {
                                    // Nickname update event
                                    Ok(Some(GossipEvent::PeerJoined {
                                        peer_id: peer,
                                        nickname: nickname.clone(),
                                    }))
                                }
                            }
                            _ => Ok(None),
                        }
                    }
                }
            }
            _ => Ok(None),
        }
    }

    /// Request nickname from a newly discovered peer
    fn request_peer_nickname(&mut self, peer_id: PeerId) -> Result<()> {
        self.swarm
            .behaviour_mut()
            .mdns
            .request_response
            .send_request(&peer_id, NicknameRequest::GetNickname);
        Ok(())
    }

    /// Announce our nickname to a peer
    fn announce_nickname(&mut self, peer_id: PeerId) -> Result<()> {
        self.swarm
            .behaviour_mut()
            .mdns
            .request_response
            .send_request(
                &peer_id,
                NicknameRequest::AnnounceNickname {
                    nickname: self.nickname.clone(),
                },
            );
        Ok(())
    }

    fn subscribe_to_topic(&mut self) -> Result<()> {
        self.swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&self.topic)
            .map_err(|e| anyhow!("Failed to subscribe to topic: {}", e))?;
        Ok(())
    }

    /// Send a text message
    pub fn send_text_message(&mut self, content: String) -> Result<()> {
        let message = ChatMessage {
            id: Uuid::new_v4().to_string(),
            sender: self.nickname.clone(),
            message: Message::Text {
                content,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs(),
            },
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        };

        let data = serde_json::to_vec(&message)?;
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.topic.clone(), data)
            .map_err(|e| anyhow!("Failed to publish message: {}", e))?;

        Ok(())
    }

    /// Send an image message
    pub fn send_image_message(&mut self, data: Vec<u8>, filename: String) -> Result<()> {
        // Check if image is too large (1MB limit for JSON serialization overhead)
        if data.len() > 1024 * 1024 {
            return Err(anyhow!(
                "Image file is too large. Maximum size is 1MB, got {} bytes",
                data.len()
            ));
        }

        let message = ChatMessage {
            id: Uuid::new_v4().to_string(),
            sender: self.nickname.clone(),
            message: Message::Image {
                data,
                filename,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs(),
            },
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        };

        let msg_data = serde_json::to_vec(&message)?;

        // Check final message size
        if msg_data.len() > 2 * 1024 * 1024 {
            return Err(anyhow!(
                "Message is too large after serialization. Maximum size is 2MB, got {} bytes",
                msg_data.len()
            ));
        }

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.topic.clone(), msg_data)
            .map_err(|e| anyhow!("Failed to publish image: {}", e))?;

        Ok(())
    }
}
