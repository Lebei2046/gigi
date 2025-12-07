//! mDNS Nickname Library with libp2p Integration
//!
//! This library provides functionality for managing device nicknames
//! in mDNS (Multicast DNS) networks with seamless peer discovery
//! using libp2p's mDNS and request-response protocols.

use futures::{AsyncReadExt, AsyncWriteExt};
use libp2p::{mdns, swarm::NetworkBehaviour, Multiaddr, PeerId, Swarm};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tokio::sync::mpsc;

// Import request-response components
use libp2p::request_response::{self, Behaviour, Config, OutboundRequestId, ProtocolSupport};

/// Errors that can occur during nickname operations
#[derive(Error, Debug)]
pub enum NicknameError {
    #[error("Invalid nickname format: {0}")]
    InvalidFormat(String),
    #[error("Nickname too long: maximum {max} characters, got {actual}")]
    TooLong { max: usize, actual: usize },
    #[error("Nickname contains invalid characters")]
    InvalidCharacters,
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Peer not found: {0}")]
    PeerNotFound(PeerId),
    #[error("Request timeout")]
    RequestTimeout,
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
}

/// Result type for nickname operations
pub type Result<T> = std::result::Result<T, NicknameError>;

/// A device nickname for mDNS discovery
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Nickname {
    name: String,
}

impl Nickname {
    /// Create a new nickname with validation
    pub fn new(name: String) -> Result<Self> {
        Self::validate(&name)?;
        Ok(Nickname { name })
    }

    /// Get nickname string
    pub fn as_str(&self) -> &str {
        &self.name
    }

    /// Validate a nickname string
    fn validate(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(NicknameError::InvalidFormat(
                "Nickname cannot be empty".to_string(),
            ));
        }

        if name.len() > 63 {
            return Err(NicknameError::TooLong {
                max: 63,
                actual: name.len(),
            });
        }

        // Check for valid characters (alphanumeric, hyphens, underscores)
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(NicknameError::InvalidCharacters);
        }

        // Cannot start or end with hyphen or underscore
        if name.starts_with('-')
            || name.ends_with('-')
            || name.starts_with('_')
            || name.ends_with('_')
        {
            return Err(NicknameError::InvalidFormat(
                "Nickname cannot start or end with hyphen or underscore".to_string(),
            ));
        }

        Ok(())
    }
}

impl From<Nickname> for String {
    fn from(nickname: Nickname) -> Self {
        nickname.name
    }
}

/// Request-response protocol messages for nickname exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NicknameRequest {
    /// Request peer's nickname
    GetNickname,
    /// Request all discovered nicknames
    GetDiscoveredPeers,
    /// Announce nickname to network
    AnnounceNickname { nickname: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NicknameResponse {
    /// Response with peer's nickname
    Nickname { peer_id: String, nickname: String },
    /// Response with discovered peers
    DiscoveredPeers(Vec<(String, String)>),
    /// Error response
    Error(String),
    /// Acknowledgment for nickname announcement
    Ack,
}

/// Custom codec for nickname protocol
#[derive(Debug, Clone, Default)]
pub struct NicknameCodec;

#[async_trait::async_trait]
impl request_response::Codec for NicknameCodec {
    type Protocol = std::borrow::Cow<'static, str>;
    type Request = NicknameRequest;
    type Response = NicknameResponse;

    async fn read_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> std::io::Result<Self::Request>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        let mut buf = vec![0; 1024];
        let n = io.read(&mut buf).await?;
        buf.truncate(n);
        serde_json::from_slice(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> std::io::Result<Self::Response>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        let mut buf = vec![0; 1024];
        let n = io.read(&mut buf).await?;
        buf.truncate(n);
        serde_json::from_slice(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> std::io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        let data = serde_json::to_vec(&req)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        io.write_all(&data).await?;
        io.close().await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> std::io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        let data = serde_json::to_vec(&res)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        io.write_all(&data).await?;
        io.close().await?;
        Ok(())
    }
}

/// Custom network behaviour combining mDNS and request-response
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "NicknameBehaviourEvent")]
pub struct NicknameBehaviour {
    mdns: mdns::tokio::Behaviour,
    request_response: Behaviour<NicknameCodec>,
}

/// Events generated by the network behaviour
#[derive(Debug)]
pub enum NicknameBehaviourEvent {
    Mdns(mdns::Event),
    RequestResponse(request_response::Event<NicknameRequest, NicknameResponse>),
}

impl From<mdns::Event> for NicknameBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        Self::Mdns(event)
    }
}

impl From<request_response::Event<NicknameRequest, NicknameResponse>> for NicknameBehaviourEvent {
    fn from(event: request_response::Event<NicknameRequest, NicknameResponse>) -> Self {
        Self::RequestResponse(event)
    }
}

/// Peer information with nickname
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub nickname: Option<Nickname>,
    pub addresses: Vec<Multiaddr>,
    pub last_seen: std::time::Instant,
}

/// mDNS nickname manager with real libp2p integration
pub struct NicknameManager {
    current_nickname: Option<Nickname>,
    pub swarm: Swarm<NicknameBehaviour>,
    discovered_peers: HashMap<PeerId, PeerInfo>,
    event_sender: Option<mpsc::UnboundedSender<NicknameEvent>>,
    pending_requests: HashMap<OutboundRequestId, PeerId>,
}

impl std::fmt::Debug for NicknameManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NicknameManager")
            .field("current_nickname", &self.current_nickname)
            .field("discovered_peers", &self.discovered_peers)
            .field("pending_requests", &self.pending_requests)
            .field("event_sender", &self.event_sender.is_some())
            .finish()
    }
}

/// Events from nickname manager
#[derive(Debug, Clone)]
pub enum NicknameEvent {
    PeerDiscovered {
        peer_id: PeerId,
        nickname: Option<String>,
    },
    PeerExpired {
        peer_id: PeerId,
    },
    NicknameUpdated {
        peer_id: PeerId,
        nickname: String,
    },
    RequestReceived {
        peer_id: PeerId,
        request: NicknameRequest,
    },
    NetworkEvent {
        message: String,
    },
    ListeningOn {
        address: Multiaddr,
    },
}

impl NicknameManager {
    /// Create a nickname manager with existing swarm
    pub fn with_swarm(swarm: Swarm<NicknameBehaviour>) -> Result<Self> {
        Ok(Self {
            current_nickname: None,
            swarm,
            discovered_peers: HashMap::new(),
            event_sender: None,
            pending_requests: HashMap::new(),
        })
    }

    /// Create a nickname behaviour for external swarm creation
    pub fn create_behaviour(
        peer_id: PeerId,
        mdns_config: mdns::Config,
        request_config: Config,
    ) -> Result<NicknameBehaviour> {
        let behaviour = NicknameBehaviour {
            mdns: mdns::tokio::Behaviour::new(mdns_config, peer_id)
                .map_err(|e| NicknameError::NetworkError(e.to_string()))?,
            request_response: Behaviour::new(
                [(
                    std::borrow::Cow::Borrowed("/nickname/1.0.0"),
                    ProtocolSupport::Full,
                )],
                request_config,
            ),
        };
        Ok(behaviour)
    }

    /// Set current nickname
    pub fn set_nickname(&mut self, nickname: Nickname) {
        self.current_nickname = Some(nickname.clone());

        if let Some(ref tx) = self.event_sender {
            let _ = tx.send(NicknameEvent::NetworkEvent {
                message: format!("Nickname set to: {}", nickname.as_str()),
            });
        }
    }

    /// Get current nickname
    pub fn get_nickname(&self) -> Option<&Nickname> {
        self.current_nickname.as_ref()
    }

    /// Get local peer ID
    pub fn local_peer_id(&self) -> PeerId {
        *self.swarm.local_peer_id()
    }

    /// Start listening on given address
    pub fn start_listening(&mut self, addr: Multiaddr) -> Result<()> {
        self.swarm
            .listen_on(addr)
            .map_err(|e| NicknameError::NetworkError(e.to_string()))?;
        Ok(())
    }

    /// Get discovered peers
    pub fn get_discovered_peers(&self) -> &HashMap<PeerId, PeerInfo> {
        &self.discovered_peers
    }

    /// Get peer by nickname
    pub fn get_peer_by_nickname(&self, nickname: &str) -> Option<&PeerInfo> {
        self.discovered_peers.values().find(|peer| {
            peer.nickname
                .as_ref()
                .map_or(false, |n| n.as_str() == nickname)
        })
    }

    /// Request nickname from a peer
    pub fn request_nickname(&mut self, peer_id: PeerId) -> Result<OutboundRequestId> {
        let request_id = self
            .swarm
            .behaviour_mut()
            .request_response
            .send_request(&peer_id, NicknameRequest::GetNickname);
        self.pending_requests.insert(request_id, peer_id);
        Ok(request_id)
    }

    /// Announce nickname to all discovered peers
    pub fn announce_nickname(&mut self) -> Result<()> {
        if let Some(ref nickname) = self.current_nickname {
            let peer_ids: Vec<PeerId> = self.discovered_peers.keys().copied().collect();
            for peer_id in peer_ids {
                self.swarm.behaviour_mut().request_response.send_request(
                    &peer_id,
                    NicknameRequest::AnnounceNickname {
                        nickname: nickname.as_str().to_string(),
                    },
                );
            }
        }
        Ok(())
    }

    /// Handle a single mDNS discovered event
    pub fn handle_mdns_discovered(&mut self, peer_id: PeerId, addr: Multiaddr) -> Result<()> {
        let peer_info = PeerInfo {
            peer_id,
            nickname: None,
            addresses: vec![addr.clone()],
            last_seen: std::time::Instant::now(),
        };
        self.discovered_peers.insert(peer_id, peer_info);

        // Try to connect to discovered peer
        if let Err(_) = self.swarm.dial(addr) {
            // Connection attempt will be handled by ConnectionEstablished/ConnectionClosed events
        }
        Ok(())
    }

    /// Handle a single mDNS expired event
    pub fn handle_mdns_expired(&mut self, peer_id: PeerId) -> Option<NicknameEvent> {
        self.discovered_peers.remove(&peer_id);
        Some(NicknameEvent::PeerExpired { peer_id })
    }

    /// Handle a single request-response event
    pub fn handle_request_response_event(
        &mut self,
        event: request_response::Event<NicknameRequest, NicknameResponse>,
    ) -> Result<Option<NicknameEvent>> {
        match event {
            request_response::Event::Message { message, peer, .. } => {
                match message {
                    request_response::Message::Request {
                        request, channel, ..
                    } => {
                        // Handle request and send response
                        let response = self.handle_incoming_request(request.clone(), peer);
                        let _ = self
                            .swarm
                            .behaviour_mut()
                            .request_response
                            .send_response(channel, response);

                        Ok(Some(NicknameEvent::RequestReceived {
                            peer_id: peer,
                            request,
                        }))
                    }
                    request_response::Message::Response {
                        request_id,
                        response,
                        ..
                    } => {
                        if let Some(peer_id) = self.pending_requests.remove(&request_id) {
                            Ok(self.handle_incoming_response(response, peer_id))
                        } else {
                            Ok(None)
                        }
                    }
                }
            }
            _ => Ok(None),
        }
    }

    /// Handle incoming request and generate response
    fn handle_incoming_request(
        &self,
        request: NicknameRequest,
        _peer_id: PeerId,
    ) -> NicknameResponse {
        match request {
            NicknameRequest::GetNickname => {
                let nickname = self
                    .current_nickname
                    .as_ref()
                    .map(|n| n.as_str().to_string())
                    .unwrap_or_else(|| "unnamed".to_string());
                NicknameResponse::Nickname {
                    peer_id: self.local_peer_id().to_string(),
                    nickname,
                }
            }
            NicknameRequest::GetDiscoveredPeers => {
                let peers: Vec<(String, String)> = self
                    .discovered_peers
                    .iter()
                    .filter_map(|(id, info)| {
                        info.nickname
                            .as_ref()
                            .map(|n| (id.to_string(), n.as_str().to_string()))
                    })
                    .collect();
                NicknameResponse::DiscoveredPeers(peers)
            }
            NicknameRequest::AnnounceNickname { nickname } => {
                if let Ok(_nick) = Nickname::new(nickname.clone()) {
                    // This would update the peer info in a real implementation
                    NicknameResponse::Ack
                } else {
                    NicknameResponse::Error("Invalid nickname format".to_string())
                }
            }
        }
    }

    /// Handle incoming response and return structured event
    fn handle_incoming_response(
        &mut self,
        response: NicknameResponse,
        peer_id: PeerId,
    ) -> Option<NicknameEvent> {
        match response {
            NicknameResponse::Nickname {
                peer_id: _,
                nickname,
            } => {
                if let Ok(nick) = Nickname::new(nickname.clone()) {
                    if let Some(peer_info) = self.discovered_peers.get_mut(&peer_id) {
                        let was_previously_unknown = peer_info.nickname.is_none();
                        let previous_nickname = peer_info.nickname.clone();
                        peer_info.nickname = Some(nick.clone());
                        peer_info.last_seen = std::time::Instant::now();

                        if was_previously_unknown {
                            // First time getting nickname - raise PeerDiscovered event
                            Some(NicknameEvent::PeerDiscovered {
                                peer_id,
                                nickname: Some(nickname.clone()),
                            })
                        } else if previous_nickname.as_ref().map(|n| n.as_str())
                            != Some(nickname.as_str())
                        {
                            // Nickname actually changed - raise NicknameUpdated event
                            Some(NicknameEvent::NicknameUpdated { peer_id, nickname })
                        } else {
                            // If nickname is the same as before, don't raise any event
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            NicknameResponse::DiscoveredPeers(peers) => {
                for (peer_id_str, nickname_str) in peers {
                    if let Ok(peer_id) = peer_id_str.parse::<PeerId>() {
                        if let Ok(nickname) = Nickname::new(nickname_str) {
                            let peer_info = PeerInfo {
                                peer_id,
                                nickname: Some(nickname),
                                addresses: Vec::new(),
                                last_seen: std::time::Instant::now(),
                            };
                            self.discovered_peers.insert(peer_id, peer_info);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Generate a random nickname
    pub fn generate_random() -> Nickname {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let adjectives = ["swift", "clever", "bright", "quick", "smart", "nimble"];
        let nouns = ["fox", "eagle", "wolf", "hawk", "lion", "tiger"];

        let adj = adjectives[rng.gen_range(0..adjectives.len())];
        let noun = nouns[rng.gen_range(0..nouns.len())];
        let number: u16 = rng.gen_range(1000..9999);

        Nickname::new(format!("{}-{}-{}", adj, noun, number)).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::{core::upgrade, identity::Keypair, noise, tcp, yamux, SwarmBuilder, Transport};
    use std::time::Duration;

    #[test]
    fn test_valid_nicknames() {
        assert!(Nickname::new("device-123".to_string()).is_ok());
        assert!(Nickname::new("my_device".to_string()).is_ok());
        assert!(Nickname::new("test-device-1".to_string()).is_ok());
    }

    #[test]
    fn test_invalid_nicknames() {
        assert!(Nickname::new("".to_string()).is_err());
        assert!(Nickname::new("-invalid".to_string()).is_err());
        assert!(Nickname::new("invalid-".to_string()).is_err());
        assert!(Nickname::new("invalid@name".to_string()).is_err());
        assert!(Nickname::new("a".repeat(64).to_string()).is_err());
    }

    #[tokio::test]
    async fn test_nickname_manager_creation() {
        // Create a test manager using the new API
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());

        let mdns_config = mdns::Config {
            ttl: Duration::from_secs(60),
            query_interval: Duration::from_secs(10),
            ..mdns::Config::default()
        };

        let behaviour = NicknameBehaviour {
            mdns: mdns::tokio::Behaviour::new(mdns_config, peer_id).unwrap(),
            request_response: Behaviour::new(
                [(
                    std::borrow::Cow::Borrowed("/nickname/1.0.0"),
                    ProtocolSupport::Full,
                )],
                Config::default(),
            ),
        };

        let swarm = SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_other_transport(|_keypair| {
                tcp::tokio::Transport::default()
                    .upgrade(upgrade::Version::V1)
                    .authenticate(noise::Config::new(&_keypair).unwrap())
                    .multiplex(yamux::Config::default())
                    .boxed()
            })
            .expect("Failed to create transport")
            .with_behaviour(|_keypair| behaviour)
            .expect("Failed to create behaviour")
            .build();

        let manager = NicknameManager::with_swarm(swarm).unwrap();
        assert!(manager.get_nickname().is_none());
    }

    #[test]
    fn test_peer_info() {
        let peer_id = PeerId::random();
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
        let nickname = Nickname::new("test-peer".to_string()).unwrap();

        let peer_info = PeerInfo {
            peer_id,
            nickname: Some(nickname.clone()),
            addresses: vec![addr.clone()],
            last_seen: std::time::Instant::now(),
        };

        assert_eq!(peer_info.peer_id, peer_id);
        assert_eq!(peer_info.nickname.as_ref().unwrap().as_str(), "test-peer");
        assert_eq!(peer_info.addresses.len(), 1);
        assert_eq!(peer_info.addresses[0], addr);
    }

    #[test]
    fn test_serialization() {
        let request = NicknameRequest::GetNickname;
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: NicknameRequest = serde_json::from_str(&serialized).unwrap();
        match deserialized {
            NicknameRequest::GetNickname => {}
            _ => panic!("Deserialization failed"),
        }

        let response = NicknameResponse::Nickname {
            peer_id: "test-peer".to_string(),
            nickname: "test-nickname".to_string(),
        };
        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: NicknameResponse = serde_json::from_str(&serialized).unwrap();
        match deserialized {
            NicknameResponse::Nickname {
                peer_id: _,
                nickname,
            } => {
                assert_eq!(nickname, "test-nickname");
            }
            _ => panic!("Deserialization failed"),
        }
    }

    #[tokio::test]
    async fn test_peer_lookup_by_nickname() {
        // Create a test manager using the new API
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());

        let mdns_config = mdns::Config {
            ttl: Duration::from_secs(60),
            query_interval: Duration::from_secs(10),
            ..mdns::Config::default()
        };

        let behaviour = NicknameBehaviour {
            mdns: mdns::tokio::Behaviour::new(mdns_config, peer_id).unwrap(),
            request_response: Behaviour::new(
                [(
                    std::borrow::Cow::Borrowed("/nickname/1.0.0"),
                    ProtocolSupport::Full,
                )],
                Config::default(),
            ),
        };

        let swarm = SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_other_transport(|_keypair| {
                tcp::tokio::Transport::default()
                    .upgrade(upgrade::Version::V1)
                    .authenticate(noise::Config::new(&_keypair).unwrap())
                    .multiplex(yamux::Config::default())
                    .boxed()
            })
            .expect("Failed to create transport")
            .with_behaviour(|_keypair| behaviour)
            .expect("Failed to create behaviour")
            .build();

        let mut manager = NicknameManager::with_swarm(swarm).unwrap();
        let peer_id1 = PeerId::random();
        let peer_id2 = PeerId::random();
        let nickname1 = Nickname::new("peer-one".to_string()).unwrap();
        let nickname2 = Nickname::new("peer-two".to_string()).unwrap();

        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();

        manager.discovered_peers.insert(
            peer_id1,
            PeerInfo {
                peer_id: peer_id1,
                nickname: Some(nickname1.clone()),
                addresses: vec![addr.clone()],
                last_seen: std::time::Instant::now(),
            },
        );

        manager.discovered_peers.insert(
            peer_id2,
            PeerInfo {
                peer_id: peer_id2,
                nickname: Some(nickname2.clone()),
                addresses: vec![addr],
                last_seen: std::time::Instant::now(),
            },
        );

        let found_peer = manager.get_peer_by_nickname("peer-one");
        assert!(found_peer.is_some());
        assert_eq!(found_peer.unwrap().peer_id, peer_id1);

        let not_found = manager.get_peer_by_nickname("nonexistent");
        assert!(not_found.is_none());
    }
}
