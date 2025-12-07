use anyhow::Result;
use base64::{Engine as _, engine::general_purpose};
use bytes::Bytes;
use futures::StreamExt;
use libp2p::{
    core::Multiaddr,
    identity, noise,
    request_response::{self, ProtocolSupport},
    swarm::{Swarm, SwarmEvent},
    tcp, yamux, PeerId, StreamProtocol,
};
use serde::{Deserialize, Serialize};
use serde_bytes;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info, warn};

pub const MESSAGING_PROTOCOL: &str = "/messaging/1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Text(String),
    Image {
        name: String,
        mime_type: String,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Ack,
    Error(String),
}

#[derive(Debug)]
pub enum CustomMessagingEvent {
    Connected(PeerId),
    Disconnected(PeerId),
    MessageReceived {
        from: PeerId,
        message: Message,
    },
    MessageSent {
        to: PeerId,
        message: Message,
    },
    Error(String),
}

// Using JSON codec provided by libp2p
type MessagingBehaviour = request_response::json::Behaviour<Message, Response>;

pub struct DirectMessaging {
    pub swarm: Swarm<MessagingBehaviour>,
    event_sender: tokio::sync::mpsc::Sender<CustomMessagingEvent>,
    connected_peers: HashMap<PeerId, String>,
    local_peer_id: PeerId,
}

impl DirectMessaging {
    pub async fn new() -> Result<(Self, tokio::sync::mpsc::Receiver<CustomMessagingEvent>)> {
        let id_keys = identity::Keypair::generate_ed25519();

        let behaviour = MessagingBehaviour::new(
            [(StreamProtocol::new(MESSAGING_PROTOCOL), ProtocolSupport::Full)],
            request_response::Config::default()
                .with_request_timeout(Duration::from_secs(30)),
        );

        let local_peer_id = PeerId::from(id_keys.public());
        
        let swarm = libp2p::SwarmBuilder::with_existing_identity(id_keys)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|_| behaviour)?
            .with_swarm_config(|c| {
                c.with_idle_connection_timeout(Duration::from_secs(120))
                 .with_max_negotiating_inbound_streams(100)
            })
            .build();

        let (event_sender, event_receiver) = tokio::sync::mpsc::channel(1000);

        let messaging = Self {
            swarm,
            event_sender,
            connected_peers: HashMap::new(),
            local_peer_id,
        };

        Ok((messaging, event_receiver))
    }

    pub fn start_listening(&mut self, port: u16) -> Result<Multiaddr> {
        let listen_addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", port).parse()?;
        self.swarm.listen_on(listen_addr.clone())?;
        info!("Starting to listen on: {}", listen_addr);
        Ok(listen_addr)
    }

    pub fn dial_peer(&mut self, addr: &Multiaddr) -> Result<()> {
        info!("Dialing peer at: {}", addr);
        self.swarm.dial(addr.clone())?;
        Ok(())
    }

    pub async fn send_message(&mut self, peer_id: PeerId, message: Message) -> Result<()> {
        if !self.swarm.is_connected(&peer_id) {
            return Err(anyhow::anyhow!("Not connected to peer: {}", peer_id));
        }

        debug!("Sending message to {}: {:?}", peer_id, message);

        let _request_id = self
            .swarm
            .behaviour_mut()
            .send_request(&peer_id, message.clone());

        // Store pending request for response handling
        // In a more complete implementation, you might want to track this
        
        let _ = self.event_sender.send(CustomMessagingEvent::MessageSent {
            to: peer_id,
            message,
        });

        Ok(())
    }

    pub fn get_listening_address(&self) -> Vec<Multiaddr> {
        self.swarm.listeners().cloned().collect()
    }

    pub fn get_connected_peers(&self) -> Vec<PeerId> {
        self.swarm.connected_peers().cloned().collect()
    }

    pub fn local_peer_id(&self) -> PeerId {
        self.local_peer_id
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::Behaviour(event) => {
                    self.handle_request_response_event(event).await?;
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    info!("Connection established with: {}", peer_id);
                    self.connected_peers.insert(peer_id, "connected".to_string());
                    let _ = self.event_sender.send(CustomMessagingEvent::Connected(peer_id));
                }
                SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                    info!("Connection closed with {}: {:?}", peer_id, cause);
                    self.connected_peers.remove(&peer_id);
                    let _ = self.event_sender.send(CustomMessagingEvent::Disconnected(peer_id));
                }
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Listening on: {}", address);
                }
                SwarmEvent::IncomingConnection { local_addr, send_back_addr, connection_id: _ } => {
                    debug!("Incoming connection from {} to {}", send_back_addr, local_addr);
                }
                SwarmEvent::IncomingConnectionError { error, .. } => {
                    warn!("Incoming connection error: {:?}", error);
                }
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    warn!("Outbound connection error to {:?}: {:?}", peer_id, error);
                    let _ = self.event_sender.send(
                        CustomMessagingEvent::Error(format!("Outbound connection error: {:?}", error))
                    );
                }
                SwarmEvent::ListenerClosed { addresses, reason, .. } => {
                    warn!("Listener closed on {:?}: {:?}", addresses, reason);
                }
                SwarmEvent::ListenerError { error, .. } => {
                    error!("Listener error: {}", error);
                }
                _ => {}
            }
        }
    }

    async fn handle_request_response_event(&mut self, event: request_response::Event<Message, Response>) -> Result<()> {
        match event {
            request_response::Event::Message { message, peer, .. } => {
                match message {
                    request_response::Message::Request { 
                        request, 
                        channel, 
                        ..
                    } => {
                        debug!("Received message from {}: {:?}", peer, request);
                        
                        // Send acknowledgment
                        let _ = self
                            .swarm
                            .behaviour_mut()
                            .send_response(channel, Response::Ack);

                        // Notify about received message
                        let _ = self.event_sender.send(CustomMessagingEvent::MessageReceived {
                            from: peer,
                            message: request,
                        });
                    }
                    request_response::Message::Response { 
                        response, 
                        ..
                    } => {
                        match response {
                            Response::Ack => {
                                debug!("Message acknowledged by peer");
                            }
                            Response::Error(error) => {
                                warn!("Peer responded with error: {}", error);
                                let _ = self.event_sender.send(
                                    CustomMessagingEvent::Error(format!("Peer error: {}", error))
                                );
                            }
                        }
                    }
                }
            }
            request_response::Event::OutboundFailure { 
                peer, 
                error, 
                ..
            } => {
                warn!("Outbound request failure to {}: {:?}", peer, error);
                let _ = self.event_sender.send(
                    CustomMessagingEvent::Error(format!("Outbound request failure: {:?}", error))
                );
            }
            request_response::Event::InboundFailure { 
                error, 
                ..
            } => {
                warn!("Inbound request failure: {:?}", error);
            }
            request_response::Event::ResponseSent { .. } => {
                debug!("Response sent successfully");
            }
        }
        Ok(())
    }
}

// Utility functions for creating messages
impl Message {
    pub fn text(content: impl Into<String>) -> Self {
        Message::Text(content.into())
    }

    pub fn image(name: impl Into<String>, mime_type: impl Into<String>, data: impl Into<Bytes>) -> Result<Self> {
        Ok(Message::Image {
            name: name.into(),
            mime_type: mime_type.into(),
            data: data.into().to_vec(),
        })
    }

    pub fn from_base64_image(name: impl Into<String>, mime_type: impl Into<String>, base64_data: &str) -> Result<Self> {
        let data = general_purpose::STANDARD.decode(base64_data)?;
        Ok(Message::Image {
            name: name.into(),
            mime_type: mime_type.into(),
            data,
        })
    }

    pub fn to_base64(&self) -> Result<String> {
        match self {
            Message::Text(text) => Ok(text.clone()),
            Message::Image { data, .. } => Ok(general_purpose::STANDARD.encode(data)),
        }
    }
}