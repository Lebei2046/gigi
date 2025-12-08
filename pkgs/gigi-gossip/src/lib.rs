use anyhow::{anyhow, Result};
use futures::channel::mpsc;
use gigi_mdns::{NicknameManager, NicknameBehaviour, NicknameBehaviourEvent};
use libp2p::{
    gossipsub::{self, Event as GossipsubEvent, IdentTopic, MessageAuthenticity, MessageId},
    identity::Keypair,
    swarm::{NetworkBehaviour, Swarm},
    PeerId,
};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

pub const PROTOCOL_NAME: &str = "/gigi/gossip/1.0.0";

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

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "ComposedEvent")]
pub struct GossipBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: NicknameBehaviour,
}

#[derive(Debug)]
pub enum ComposedEvent {
    Gossipsub(GossipsubEvent),
    Mdns(NicknameBehaviourEvent),
}

impl From<GossipsubEvent> for ComposedEvent {
    fn from(event: GossipsubEvent) -> Self {
        ComposedEvent::Gossipsub(event)
    }
}

impl From<NicknameBehaviourEvent> for ComposedEvent {
    fn from(event: NicknameBehaviourEvent) -> Self {
        ComposedEvent::Mdns(event)
    }
}

pub struct GossipChat {
    pub swarm: Swarm<GossipBehaviour>,
    nickname: String,
    topic: IdentTopic,
    pub event_sender: mpsc::UnboundedSender<GossipEvent>,
    known_peers: std::collections::HashSet<PeerId>,
}

impl GossipChat {
    /// Create a gossip chat instance with existing swarm
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
        };

        // Subscribe to the topic
        chat.subscribe_to_topic()?;

        Ok((chat, event_receiver))
    }

    /// Create a gossip behaviour for external swarm creation
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
            return Err(anyhow!("Image file is too large. Maximum size is 1MB, got {} bytes", data.len()));
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
            return Err(anyhow!("Message is too large after serialization. Maximum size is 2MB, got {} bytes", msg_data.len()));
        }

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.topic.clone(), msg_data)
            .map_err(|e| anyhow!("Failed to publish image: {}", e))?;

        Ok(())
    }

    /// Handle a single event and send to event receiver
    pub fn handle_event(&mut self, event: ComposedEvent) -> Result<()> {
        let events = match event {
            ComposedEvent::Gossipsub(gossipsub_event) => {
                self.handle_gossipsub_event(gossipsub_event)?
            }
            ComposedEvent::Mdns(mdns_event) => {
                self.handle_mdns_event(mdns_event)?
            }
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

                                events.push(GossipEvent::PeerJoined {
                                    peer_id,
                                    nickname: peer_id.to_string(), // Use peer_id as nickname for now
                                });
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

                                events.push(GossipEvent::PeerLeft {
                                    peer_id,
                                    nickname: peer_id.to_string(), // Use peer_id as nickname for now
                                });
                            }
                        }
                    }
                }
            }
            NicknameBehaviourEvent::RequestResponse(req_resp_event) => {
                // Handle request-response events from gigi-mdns
                tracing::debug!("Request-response event: {:?}", req_resp_event);
            }
        }

        Ok(events)
    }

    fn subscribe_to_topic(&mut self) -> Result<()> {
        self.swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&self.topic)
            .map_err(|e| anyhow!("Failed to subscribe to topic: {}", e))?;
        Ok(())
    }

    /// Get current peers in the gossipsub network
    pub fn get_peers(&self) -> Vec<(PeerId, String)> {
        // Get connected peers from gossipsub
        self.swarm
            .behaviour()
            .gossipsub
            .all_peers()
            .into_iter()
            .map(|(&peer_id, _topics)| (peer_id, peer_id.to_string()))
            .collect()
    }
}
