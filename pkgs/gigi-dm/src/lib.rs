use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use bytes::Bytes;
use libp2p::{
    core::Multiaddr,
    request_response::{self, ProtocolSupport},
    swarm::Swarm,
    PeerId, StreamProtocol,
};
use serde::{Deserialize, Serialize};
use serde_bytes;
use tracing::{debug, info};

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

// Using JSON codec provided by libp2p
type MessagingBehaviour = request_response::json::Behaviour<Message, Response>;

#[derive(Debug, Clone)]
pub enum MessagingEvent {
    MessageReceived { peer: PeerId, message: Message },
    MessageAcknowledged { peer: PeerId },
    PeerError { peer: PeerId, error: String },
    OutboundFailure { peer: PeerId, error: String },
    InboundFailure { error: String },
    ResponseSent,
}

pub struct DirectMessaging {
    pub swarm: Swarm<MessagingBehaviour>,
    local_peer_id: PeerId,
}

impl DirectMessaging {
    /// Create DirectMessaging with existing swarm
    pub fn with_swarm(swarm: Swarm<MessagingBehaviour>) -> Result<Self> {
        let local_peer_id = *swarm.local_peer_id();

        let messaging = Self {
            swarm,
            local_peer_id,
        };

        Ok(messaging)
    }

    /// Create a messaging behaviour for external swarm creation
    pub fn create_behaviour(config: request_response::Config) -> Result<MessagingBehaviour> {
        let behaviour = MessagingBehaviour::new(
            [(
                StreamProtocol::new(MESSAGING_PROTOCOL),
                ProtocolSupport::Full,
            )],
            config,
        );
        Ok(behaviour)
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

    /// Handle a request-response event from the swarm and return display information
    pub async fn handle_request_response_event(
        &mut self,
        event: request_response::Event<Message, Response>,
    ) -> Result<Option<MessagingEvent>> {
        match event {
            request_response::Event::Message { message, peer, .. } => {
                match message {
                    request_response::Message::Request {
                        request, channel, ..
                    } => {
                        debug!("Received message from {}: {:?}", peer, request);

                        // Send acknowledgment
                        let _ = self
                            .swarm
                            .behaviour_mut()
                            .send_response(channel, Response::Ack);

                        // Return the received message for display
                        Ok(Some(MessagingEvent::MessageReceived {
                            peer,
                            message: request,
                        }))
                    }
                    request_response::Message::Response { response, .. } => match response {
                        Response::Ack => Ok(Some(MessagingEvent::MessageAcknowledged { peer })),
                        Response::Error(error) => {
                            Ok(Some(MessagingEvent::PeerError { peer, error }))
                        }
                    },
                }
            }
            request_response::Event::OutboundFailure { peer, error, .. } => {
                Ok(Some(MessagingEvent::OutboundFailure {
                    peer,
                    error: format!("{:?}", error),
                }))
            }
            request_response::Event::InboundFailure { error, .. } => {
                Ok(Some(MessagingEvent::InboundFailure {
                    error: format!("{:?}", error),
                }))
            }
            request_response::Event::ResponseSent { .. } => Ok(Some(MessagingEvent::ResponseSent)),
        }
    }

    /// Send an image to all connected peers
    pub async fn send_image_to_all(
        &mut self,
        peers: &[PeerId],
        image_path: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Read image file
        let image_data = tokio::fs::read(image_path).await?;

        // Determine MIME type
        let mime_type = mime_guess::from_path(image_path)
            .first_or_octet_stream()
            .to_string();

        let image_name = image_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        // Send image message to all peers
        let message = Message::image(image_name, mime_type, image_data)?;
        for peer_id in peers {
            if let Err(e) = self.send_message(*peer_id, message.clone()).await {
                eprintln!("Failed to send image to {}: {}", peer_id, e);
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

    pub fn image(
        name: impl Into<String>,
        mime_type: impl Into<String>,
        data: impl Into<Bytes>,
    ) -> Result<Self> {
        Ok(Message::Image {
            name: name.into(),
            mime_type: mime_type.into(),
            data: data.into().to_vec(),
        })
    }

    pub fn from_base64_image(
        name: impl Into<String>,
        mime_type: impl Into<String>,
        base64_data: &str,
    ) -> Result<Self> {
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

/// Save a received image to disk
pub async fn save_received_image(
    name: &str,
    data: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let filename = format!("received_{}", name);
    tokio::fs::write(&filename, data).await?;
    Ok(())
}
