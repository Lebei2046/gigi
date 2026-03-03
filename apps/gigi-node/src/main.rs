use clap::Parser;
use futures::StreamExt;
use libp2p::{
    core::{muxing::StreamMuxerBox, transport::Boxed},
    gossipsub, identify, identity, kad, noise, ping, relay,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm, Transport,
};
use std::time::Duration;
use tracing::{info, warn};

/// Unified network behaviour for Gigi nodes
#[derive(NetworkBehaviour)]
struct NodeBehaviour {
    /// Kademlia DHT for peer discovery
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    /// GossipSub for pub/sub messaging
    gossipsub: gossipsub::Behaviour,
    /// Identify protocol for peer metadata
    identify: identify::Behaviour,
    /// Ping for keepalive
    ping: ping::Behaviour,
    /// Circuit relay for NAT traversal
    relay: relay::Behaviour,
}

#[derive(Parser, Debug, Clone)]
#[command(name = "gigi-node")]
#[command(about = "Gigi P2P Network Node")]
struct Args {
    /// Node mode: bootstrap, relay, or full
    #[arg(long, value_enum)]
    mode: NodeMode,

    /// Listen addresses (e.g., /ip4/0.0.0.0/tcp/4001)
    #[arg(long)]
    listen: Vec<String>,

    /// External addresses advertised to the network
    #[arg(long)]
    external: Vec<String>,

    /// Bootstrap peer addresses (for non-bootstrap nodes)
    #[arg(long)]
    bootstrap: Vec<String>,

    /// Path to identity key file (generated if not exists)
    #[arg(long)]
    identity: Option<String>,

    /// Topics to subscribe to (for relay/full nodes)
    #[arg(long, default_value = "gigi-general")]
    topics: Vec<String>,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum NodeMode {
    /// Bootstrap node - provides DHT entry points
    Bootstrap,
    /// Relay node - helps NATed peers connect
    Relay,
    /// Full node - both bootstrap and relay capabilities
    Full,
}

struct GigiNode {
    swarm: Swarm<NodeBehaviour>,
    mode: NodeMode,
}

impl GigiNode {
    async fn new(args: &Args) -> anyhow::Result<Self> {
        // Load or create identity
        let local_key = load_or_create_identity(args.identity.as_deref()).await?;
        let local_peer_id = PeerId::from(local_key.public());
        info!("Local peer ID: {}", local_peer_id);

        // Create transport
        let transport = create_transport(&local_key).await?;

        // Create behaviour
        let behaviour = create_behaviour(&local_key, args).await?;

        // Build swarm
        let mut swarm = Swarm::new(
            transport,
            behaviour,
            local_peer_id,
            libp2p::swarm::Config::with_tokio_executor()
                .with_idle_connection_timeout(Duration::from_secs(60)),
        );

        // Listen on configured addresses
        for addr_str in &args.listen {
            let addr: Multiaddr = addr_str.parse()?;
            info!("Listening on: {}", addr);
            swarm.listen_on(addr)?;
        }

        // Add external addresses
        for addr_str in &args.external {
            let addr: Multiaddr = addr_str.parse()?;
            info!("External address: {}", addr);
            swarm.add_external_address(addr);
        }

        Ok(Self {
            swarm,
            mode: args.mode,
        })
    }

    async fn run(&mut self, args: &Args) -> anyhow::Result<()> {
        info!("Starting Gigi node in {:?} mode", self.mode);

        // Bootstrap into DHT
        for peer_str in &args.bootstrap {
            if let Some((peer_id, addr)) = parse_peer_addr(peer_str) {
                info!("Adding bootstrap peer: {} at {}", peer_id, addr);
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, addr);
            }
        }

        if !args.bootstrap.is_empty() {
            info!("Starting DHT bootstrap...");
            self.swarm.behaviour_mut().kademlia.bootstrap()?;
        }

        // Subscribe to topics if relay/full mode
        if matches!(self.mode, NodeMode::Relay | NodeMode::Full) {
            for topic_str in &args.topics {
                let topic = gossipsub::IdentTopic::new(topic_str);
                match self.swarm.behaviour_mut().gossipsub.subscribe(&topic) {
                    Ok(_) => info!("Subscribed to topic: {}", topic_str),
                    Err(e) => warn!("Failed to subscribe to topic {}: {:?}", topic_str, e),
                }
            }
        }

        // Event loop
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => {
                    self.handle_event(event).await?;
                }
            }
        }
    }

    async fn handle_event(&mut self, event: SwarmEvent<NodeBehaviourEvent>) -> anyhow::Result<()> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on: {}", address);
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                info!(
                    "Connected to: {} (via {})",
                    peer_id,
                    if endpoint.is_dialer() {
                        "outbound"
                    } else {
                        "inbound"
                    }
                );
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                info!("Disconnected from: {} (cause: {:?})", peer_id, cause);
            }
            SwarmEvent::Behaviour(NodeBehaviourEvent::Kademlia(event)) => {
                self.handle_kademlia_event(event).await?;
            }
            SwarmEvent::Behaviour(NodeBehaviourEvent::Gossipsub(event)) => {
                self.handle_gossip_event(event).await?;
            }
            SwarmEvent::Behaviour(NodeBehaviourEvent::Identify(event)) => {
                info!("Identify: {:?}", event);
            }
            SwarmEvent::Behaviour(NodeBehaviourEvent::Ping(event)) => {
                tracing::debug!("Ping: {:?}", event);
            }
            SwarmEvent::Behaviour(NodeBehaviourEvent::Relay(event)) => {
                info!("Relay: {:?}", event);
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_kademlia_event(&mut self, event: kad::Event) -> anyhow::Result<()> {
        match event {
            kad::Event::OutboundQueryProgressed { result, .. } => match result {
                kad::QueryResult::Bootstrap(Ok(result)) => {
                    info!("DHT bootstrap complete, peer: {:?}", result.peer);
                }
                kad::QueryResult::Bootstrap(Err(e)) => {
                    warn!("DHT bootstrap failed: {:?}", e);
                }
                kad::QueryResult::GetClosestPeers(Ok(peers)) => {
                    info!("Found {} closest peers", peers.peers.len());
                }
                _ => {}
            },
            kad::Event::RoutingUpdated {
                peer, is_new_peer, ..
            } => {
                if is_new_peer {
                    info!("New peer added to routing table: {}", peer);
                }
            }
            kad::Event::UnroutablePeer { peer, .. } => {
                warn!("Peer {} is unroutable", peer);
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_gossip_event(&mut self, event: gossipsub::Event) -> anyhow::Result<()> {
        match event {
            gossipsub::Event::Message {
                propagation_source,
                message,
                message_id,
            } => {
                let data = String::from_utf8_lossy(&message.data);
                info!(
                    "Message {} from {} on topic {}: {}",
                    message_id, propagation_source, message.topic, data
                );
            }
            gossipsub::Event::Subscribed { peer_id, topic } => {
                info!("Peer {} subscribed to topic {}", peer_id, topic);
            }
            gossipsub::Event::Unsubscribed { peer_id, topic } => {
                info!("Peer {} unsubscribed from topic {}", peer_id, topic);
            }
            gossipsub::Event::GossipsubNotSupported { peer_id } => {
                warn!("Peer {} does not support GossipSub", peer_id);
            }
            _ => {}
        }
        Ok(())
    }
}

async fn create_transport(
    local_key: &identity::Keypair,
) -> anyhow::Result<Boxed<(PeerId, StreamMuxerBox)>> {
    let noise_config = noise::Config::new(local_key)?;

    // TCP + Noise + Yamux
    let tcp_transport = tcp::tokio::Transport::new(tcp::Config::default())
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(noise_config.clone())
        .multiplex(yamux::Config::default())
        .boxed();

    Ok(tcp_transport)
}

async fn create_behaviour(
    local_key: &identity::Keypair,
    args: &Args,
) -> anyhow::Result<NodeBehaviour> {
    let local_peer_id = PeerId::from(local_key.public());

    // Kademlia config based on mode
    let kademlia_config = if matches!(args.mode, NodeMode::Bootstrap | NodeMode::Full) {
        info!("Kademlia mode: Server (storing DHT records)");
        kad::Config::default()
    } else {
        info!("Kademlia mode: Client (querying only)");
        kad::Config::default()
    };

    let store = kad::store::MemoryStore::new(local_peer_id);
    let kademlia = kad::Behaviour::with_config(local_peer_id, store, kademlia_config);

    // GossipSub
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .build()?;

    let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(local_key.clone()),
        gossipsub_config,
    )
    .map_err(|e| anyhow::anyhow!("Failed to create gossipsub: {}", e))?;

    // Identify
    let identify = identify::Behaviour::new(identify::Config::new(
        "/gigi/1.0.0".to_string(),
        local_key.public(),
    ));

    // Ping
    let ping = ping::Behaviour::new(ping::Config::new());

    // Relay
    let relay = relay::Behaviour::new(local_peer_id, Default::default());

    Ok(NodeBehaviour {
        kademlia,
        gossipsub,
        identify,
        ping,
        relay,
    })
}

async fn load_or_create_identity(path: Option<&str>) -> anyhow::Result<identity::Keypair> {
    if let Some(path) = path {
        if std::path::Path::new(path).exists() {
            info!("Loading identity from {}", path);
            let bytes = tokio::fs::read(path).await?;
            Ok(identity::Keypair::from_protobuf_encoding(&bytes)?)
        } else {
            info!("Generating new identity and saving to {}", path);
            let keypair = identity::Keypair::generate_ed25519();
            let bytes = keypair.to_protobuf_encoding()?;
            tokio::fs::write(path, bytes).await?;
            Ok(keypair)
        }
    } else {
        info!("Generating ephemeral identity (not persisted)");
        Ok(identity::Keypair::generate_ed25519())
    }
}

fn parse_peer_addr(s: &str) -> Option<(PeerId, Multiaddr)> {
    let addr: Multiaddr = s.parse().ok()?;
    let iter = addr.iter();
    let mut last_protocol = None;
    let mut addr_parts = Vec::new();

    for protocol in iter {
        if let libp2p::multiaddr::Protocol::P2p(peer_id) = protocol {
            last_protocol = Some(peer_id);
        } else {
            addr_parts.push(protocol);
        }
    }

    if let Some(peer_id) = last_protocol {
        // Reconstruct address without P2p protocol
        let addr_without_peer: Multiaddr = addr_parts.into_iter().collect();
        Some((peer_id, addr_without_peer))
    } else {
        None
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let mut node = GigiNode::new(&args).await?;
    node.run(&args).await?;

    Ok(())
}
