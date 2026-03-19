**Assuming we have two wifi network which can access to the internet, we should build a node software and rent several hosts which run the node software on cloud. Give a solution how to build the node software and run it on the hosts and how to make the mobiles running gigi-mobile in the two wifi network connected, and also support a single mobile running gigi-mobile can join the p2p network?**

Based on the Gigi architecture and your requirements, here's a complete solution for building node software and connecting mobile devices across two WiFi networks:

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              INTERNET (WAN)                                 │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                    CLOUD HOSTS (Bootstrap + Relay)                  │   │
│   │                                                                     │   │
│   │   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐            │   │
│   │   │ Bootstrap 1 │    │ Bootstrap 2 │    │ Relay Node  │            │   │
│   │   │ 203.0.113.10│    │ 203.0.113.11│    │ 203.0.113.12│            │   │
│   │   │ :4001/tcp   │    │ :4002/tcp   │    │ :4003/tcp   │            │   │
│   │   │ :4001/udp   │    │ :4002/udp   │    │ :4003/udp   │            │   │
│   │   │             │    │             │    │             │            │   │
│   │   │ Kademlia    │    │ Kademlia    │    │ Circuit     │            │   │
│   │   │ DHT Node    │    │ DHT Node    │    │ Relay v2    │            │   │
│   │   │ Bootstrap   │    │ Bootstrap   │    │ TURN Server │            │   │
│   │   │ Service     │    │ Service     │    │             │            │   │
│   │   └──────┬──────┘    └──────┬──────┘    └──────┬──────┘            │   │
│   │          └───────────────────┴──────────────────┘                   │   │
│   │                                                                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                    ▲                                        │
│                                    │                                        │
│              ┌─────────────────────┴─────────────────────┐                  │
│              │                                           │                  │
│              ▼                                           ▼                  │
│   ┌─────────────────────┐                     ┌─────────────────────┐       │
│   │     WIFI NETWORK 1  │                     │     WIFI NETWORK 2  │       │
│   │   (192.168.1.0/24)  │                     │   (192.168.2.0/24)  │       │
│   │                     │                     │                     │       │
│   │  ┌───────────────┐  │                     │  ┌───────────────┐  │       │
│   │  │  Gigi Mobile  │  │◄───────────────────►│  │  Gigi Mobile  │  │       │
│   │  │    App A1     │  │   Via Cloud Relay   │  │    App B1     │  │       │
│   │  │ 192.168.1.10  │  │   (NAT traversal)   │  │ 192.168.2.10  │  │       │
│   │  │ (NATed)       │  │                     │  │ (NATed)       │  │       │
│   │  └───────────────┘  │                     │  └───────────────┘  │       │
│   │                     │                     │                     │       │
│   │  ┌───────────────┐  │                     │  ┌───────────────┐  │       │
│   │  │  Gigi Mobile  │  │◄───────────────────►│  │  Gigi Mobile  │  │       │
│   │  │    App A2     │  │   P2P through DHT   │  │    App B2     │  │       │
│   │  │ 192.168.1.11  │  │                     │  │ 192.168.2.11  │  │       │
│   │  │ (NATed)       │  │                     │  │ (NATed)       │  │       │
│   │  └───────────────┘  │                     │  └───────────────┘  │       │
│   │                     │                     │                     │       │
│   └─────────────────────┘                     └─────────────────────┘       │
│                                                                             │
│   SINGLE MOBILE SCENARIO:                                                   │
│   ┌─────────────────────┐                                                   │
│   │  Any WiFi/4G/5G     │                                                   │
│   │  ┌───────────────┐  │                                                   │
│   │  │  Gigi Mobile  │  │◄──── Connects to cloud bootstrap/relay           │
│   │  │  Single App   │  │      Discovers peers via DHT                     │
│   │  │ (Any network) │  │      Can chat with anyone in network             │
│   │  └───────────────┘  │                                                   │
│   └─────────────────────┘                                                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Solution Components

### 1. Node Software (Rust + libp2p)

Create a new Rust project for the cloud nodes:

```rust
// Cargo.toml
[package]
name = "gigi-node"
version = "0.1.0"
edition = "2021"

[dependencies]
libp2p = { version = "0.54", features = [
    "tcp", "quic", "noise", "yamux",
    "kad", "gossipsub", "identify",
    "relay", "ping", "macros"
]}
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
config = "0.14"
```

```rust
// src/main.rs
use libp2p::{
    gossipsub, identify, identity, kad, noise, ping, relay,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm,
};
use std::time::Duration;
use tokio::select;
use tracing::{info, warn};
use clap::Parser;

/// Unified network behaviour
#[derive(NetworkBehaviour)]
struct NodeBehaviour {
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    gossipsub: gossipsub::Behaviour,
    identify: identify::Behaviour,
    ping: ping::Behaviour,
    relay: relay::Behaviour,
}

#[derive(Parser, Debug, Clone)]
#[command(name = "gigi-node")]
struct Args {
    #[arg(long, value_enum)]
    mode: NodeMode,

    #[arg(long)]
    listen: Vec<String>,

    #[arg(long)]
    external: Vec<String>,

    #[arg(long)]
    bootstrap: Vec<String>,

    #[arg(long)]
    identity: Option<String>,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum NodeMode {
    Bootstrap,
    Relay,
    Full,
}

struct GigiNode {
    swarm: Swarm<NodeBehaviour>,
    mode: NodeMode,
}

impl GigiNode {
    async fn new(args: &Args) -> anyhow::Result<Self> {
        // Load or create identity
        let local_key = load_identity(args.identity.as_deref()).await?;
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
            swarm.listen_on(addr)?;
            info!("Listening on: {}", addr);
        }

        // Add external addresses
        for addr_str in &args.external {
            let addr: Multiaddr = addr_str.parse()?;
            swarm.add_external_address(addr);
            info!("External address: {}", addr);
        }

        Ok(Self { swarm, mode: args.mode })
    }

    async fn run(&mut self, args: &Args) -> anyhow::Result<()> {
        info!("Starting Gigi node in {:?} mode", self.mode);

        // Bootstrap into DHT
        for peer_str in &args.bootstrap {
            if let Some((peer_id, addr)) = parse_peer_addr(peer_str) {
                info!("Adding bootstrap peer: {} at {}", peer_id, addr);
                self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
            }
        }

        if !args.bootstrap.is_empty() {
            self.swarm.behaviour_mut().kademlia.bootstrap()?;
        }

        // Subscribe to topics if relay/full mode
        if matches!(self.mode, NodeMode::Relay | NodeMode::Full) {
            let topic = gossipsub::IdentTopic::new("gigi-general");
            self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
        }

        // Event loop
        loop {
            select! {
                event = self.swarm.select_next_some() => {
                    self.handle_event(event).await?;
                }
            }
        }
    }

    async fn handle_event(
        &mut self,
        event: SwarmEvent<NodeBehaviourEvent>,
    ) -> anyhow::Result<()> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on: {}", address);
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("Connected to: {}", peer_id);
            }
            SwarmEvent::Behaviour(NodeBehaviourEvent::Kademlia(event)) => {
                handle_kad_event(event).await?;
            }
            SwarmEvent::Behaviour(NodeBehaviourEvent::Gossipsub(event)) => {
                handle_gossip_event(event).await?;
            }
            SwarmEvent::Behaviour(NodeBehaviourEvent::Identify(event)) => {
                info!("Identify: {:?}", event);
            }
            _ => {}
        }
        Ok(())
    }
}

async fn create_transport(
    local_key: &identity::Keypair,
) -> anyhow::Result<impl Transport> {
    let noise_config = noise::Config::new(local_key)?;

    let tcp_transport = tcp::tokio::Transport::new(tcp::Config::default())
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(noise_config.clone())
        .multiplex(yamux::Config::default());

    let quic_transport = libp2p::quic::tokio::Transport::new(
        libp2p::quic::Config::default(),
    );

    Ok(libp2p::core::transport::OrTransport::new(quic_transport, tcp_transport)
        .map(|either, _| match either {
            libp2p::core::either::Either::Left((peer_id, muxer)) => (peer_id, muxer),
            libp2p::core::either::Either::Right((peer_id, muxer)) => (peer_id, muxer),
        }))
}

async fn create_behaviour(
    local_key: &identity::Keypair,
    args: &Args,
) -> anyhow::Result<NodeBehaviour> {
    let local_peer_id = PeerId::from(local_key.public());

    // Kademlia config based on mode
    let mut kademlia_config = kad::Config::default();
    if matches!(args.mode, NodeMode::Bootstrap | NodeMode::Full) {
        kademlia_config.set_mode(Some(kad::Mode::Server));
    }

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
    )?;

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

async fn load_identity(path: Option<&str>) -> anyhow::Result<identity::Keypair> {
    if let Some(path) = path {
        // Load from file
        let bytes = tokio::fs::read(path).await?;
        Ok(identity::Keypair::from_protobuf_encoding(&bytes)?)
    } else {
        // Generate new (for testing)
        Ok(identity::Keypair::generate_ed25519())
    }
}

fn parse_peer_addr(s: &str) -> Option<(PeerId, Multiaddr)> {
    // Parse format: /ip4/.../tcp/.../p2p/12D3KooW...
    s.parse::<Multiaddr>()
        .ok()
        .and_then(|addr| {
            let mut iter = addr.iter();
            let last = iter.next_back()?;
            if let libp2p::multiaddr::Protocol::P2p(peer_id) = last {
                Some((peer_id, addr))
            } else {
                None
            }
        })
}

async fn handle_kad_event(event: kad::Event) -> anyhow::Result<()> {
    match event {
        kad::Event::OutboundQueryProgressed { result, .. } => {
            match result {
                kad::QueryResult::Bootstrap(Ok(_)) => {
                    info!("DHT bootstrap complete");
                }
                kad::QueryResult::GetClosestPeers(Ok(peers)) => {
                    info!("Found {} closest peers", peers.peers.len());
                }
                _ => {}
            }
        }
        kad::Event::RoutingUpdated { peer, .. } => {
            info!("Routing table updated: {}", peer);
        }
        _ => {}
    }
    Ok(())
}

async fn handle_gossip_event(event: gossipsub::Event) -> anyhow::Result<()> {
    match event {
        gossipsub::Event::Message { propagation_source, message, .. } => {
            info!(
                "Message from {} on {}: {:?}",
                propagation_source,
                message.topic,
                String::from_utf8_lossy(&message.data)
            );
        }
        gossipsub::Event::Subscribed { peer_id, topic } => {
            info!("Peer {} subscribed to {}", peer_id, topic);
        }
        _ => {}
    }
    Ok(())
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
```

### 2. Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/gigi-node /usr/local/bin/
EXPOSE 4001/tcp 4001/udp 4002/tcp 4002/udp 4003/tcp 4003/udp
ENTRYPOINT ["gigi-node"]
```

```yaml
# docker-compose.yml
version: '3.8'

services:
  bootstrap-1:
    build: .
    container_name: gigi-bootstrap-1
    ports:
      - "4001:4001/tcp"
      - "4001:4001/udp"
    volumes:
      - ./data/bootstrap1:/data
    command: >
      --mode bootstrap
      --listen /ip4/0.0.0.0/tcp/4001
      --listen /ip4/0.0.0.0/udp/4001/quic-v1
      --external /ip4/203.0.113.10/tcp/4001
      --external /ip4/203.0.113.10/udp/4001/quic-v1
      --identity /data/identity.key
    restart: unless-stopped

  bootstrap-2:
    build: .
    container_name: gigi-bootstrap-2
    ports:
      - "4002:4002/tcp"
      - "4002:4002/udp"
    volumes:
      - ./data/bootstrap2:/data
    command: >
      --mode bootstrap
      --listen /ip4/0.0.0.0/tcp/4002
      --listen /ip4/0.0.0.0/udp/4002/quic-v1
      --external /ip4/203.0.113.11/tcp/4002
      --external /ip4/203.0.113.11/udp/4002/quic-v1
      --bootstrap /ip4/203.0.113.10/tcp/4001/p2p/12D3KooW...
      --identity /data/identity.key
    restart: unless-stopped

  relay:
    build: .
    container_name: gigi-relay
    ports:
      - "4003:4003/tcp"
      - "4003:4003/udp"
    volumes:
      - ./data/relay:/data
    command: >
      --mode relay
      --listen /ip4/0.0.0.0/tcp/4003
      --listen /ip4/0.0.0.0/udp/4003/quic-v1
      --external /ip4/203.0.113.12/tcp/4003
      --external /ip4/203.0.113.12/udp/4003/quic-v1
      --bootstrap /ip4/203.0.113.10/tcp/4001/p2p/12D3KooW...
      --bootstrap /ip4/203.0.113.11/tcp/4002/p2p/12D3KooW...
      --identity /data/identity.key
    restart: unless-stopped
```

### 3. Gigi-Mobile Configuration

Update your mobile app to connect to the cloud infrastructure:

```rust
// gigi-mobile/src/p2p/bootstrap.rs

/// Production bootstrap nodes (cloud-hosted)
pub const BOOTSTRAP_NODES: &[&str] = &[
    // Primary bootstrap nodes
    "/dns4/bootstrap1.gigi.network/tcp/4001/p2p/12D3KooWABC...",
    "/dns4/bootstrap2.gigi.network/tcp/4002/p2p/12D3KooWDEF...",
    
    // Fallback IPs
    "/ip4/203.0.113.10/tcp/4001/p2p/12D3KooWABC...",
    "/ip4/203.0.113.11/tcp/4002/p2p/12D3KooWDEF...",
    
    // Relay node
    "/dns4/relay.gigi.network/tcp/4003/p2p/12D3KooWGHI...",
    "/ip4/203.0.113.12/tcp/4003/p2p/12D3KooWGHI...",
];

/// Mobile P2P configuration
pub struct MobileP2PConfig;

impl MobileP2PConfig {
    pub fn create_swarm(keypair: &identity::Keypair) -> anyhow::Result<Swarm<MobileBehaviour>> {
        let local_peer_id = PeerId::from(keypair.public());
        
        // Transport with relay support
        let transport = OrTransport::new(
            libp2p::quic::tokio::Transport::new(libp2p::quic::Config::default()),
            tcp::tokio::Transport::new(tcp::Config::default())
                .upgrade(upgrade::Version::V1)
                .authenticate(noise::Config::new(keypair)?)
                .multiplex(yamux::Config::default()),
        ).map(|either, _| match either {
            Either::Left((peer_id, muxer)) => (peer_id, muxer),
            Either::Right((peer_id, muxer)) => (peer_id, muxer),
        });

        // Behaviours
        let kad_config = kad::Config::default()
            .set_mode(Some(kad::Mode::Client)); // Mobile is client mode
            
        let kademlia = kad::Behaviour::with_config(
            local_peer_id,
            kad::store::MemoryStore::new(local_peer_id),
            kad_config,
        );

        let relay_client = relay::client::Behaviour::new(
            local_peer_id,
            Default::default(),
        );

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(keypair.clone()),
            gossipsub::Config::default(),
        )?;

        Ok(Swarm::new(
            transport,
            MobileBehaviour {
                kademlia,
                relay_client,
                gossipsub,
            },
            local_peer_id,
            Config::with_tokio_executor()
                .with_idle_connection_timeout(Duration::from_secs(60)),
        ))
    }
}
```

### 4. Connection Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    MOBILE APP CONNECTION (Single or Multiple)               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  SINGLE MOBILE SCENARIO:                                                    │
│  ┌─────────────┐                                                            │
│  │ 1. App Start│  - Generate/load peer ID                                   │
│  │             │  - No existing connections                                 │
│  └──────┬──────┘                                                            │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐  2. Connect to Bootstrap                                   │
│  │  Bootstrap  │     - Dial bootstrap1.gigi.network:4001                    │
│  │  Connection │     - Or fallback to bootstrap2                            │
│  │             │     - Exchange peer info via Identify                        │
│  └──────┬──────┘                                                            │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐  3. DHT Bootstrap                                           │
│  │  Kademlia   │     - Query: "Find peers near me"                          │
│  │  Discovery  │     - Receive ~20 peer addresses                            │
│  │             │     - Populate routing table                                 │
│  └──────┬──────┘                                                            │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐  4. NAT Traversal (if needed)                               │
│  │  Relay      │     - Reserve slot on relay node                            │
│  │  Setup      │     - Get relayed address: /p2p-circuit/p2p/<id>            │
│  │             │     - Advertise to network                                   │
│  └──────┬──────┘                                                            │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐  5. Join Topics                                             │
│  │  GossipSub  │     - Subscribe to "general-chat"                           │
│  │  Subscribe  │     - GossipSub discovers mesh peers                        │
│  │             │     - Build 3-6 direct connections                          │
│  └──────┬──────┘                                                            │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐  6. READY                                                   │
│  │  Connected  │     - Can send/receive messages                             │
│  │             │     - Discover other peers via DHT                          │
│  │             │     - Chat with anyone in network                           │
│  └─────────────┘                                                            │
│                                                                             │
│  MULTIPLE MOBILES (Two WiFi Networks):                                      │
│                                                                             │
│  Mobile A (WiFi 1)        Cloud Relay         Mobile B (WiFi 2)             │
│  ┌─────────────┐          ┌─────────┐         ┌─────────────┐              │
│  │ NATed       │◄────────►│ Relay   │◄───────►│ NATed       │              │
│  │ 192.168.1.x │  Relayed │ Node    │  Relayed│ 192.168.2.x │              │
│  │             │  Conn    │         │  Conn   │             │              │
│  └─────────────┘          └─────────┘         └─────────────┘              │
│                                                                             │
│  OR (if hole punching succeeds):                                            │
│                                                                             │
│  Mobile A ◄──────────────────────────────────► Mobile B                     │
│         Direct P2P (after relay discovery)                                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5. Deployment Steps

```bash
# 1. Build node software
cd gigi-node
cargo build --release

# 2. Create Docker image
docker build -t gigi-node:latest .

# 3. Deploy to cloud hosts
# Host 1 (Bootstrap 1)
scp -r docker-compose.yml Dockerfile target/release/gigi-node user@203.0.113.10:/opt/gigi/
ssh user@203.0.113.10 "cd /opt/gigi && docker-compose up -d bootstrap-1"

# Host 2 (Bootstrap 2)
scp -r docker-compose.yml Dockerfile target/release/gigi-node user@203.0.113.11:/opt/gigi/
ssh user@203.0.113.11 "cd /opt/gigi && docker-compose up -d bootstrap-2"

# Host 3 (Relay)
scp -r docker-compose.yml Dockerfile target/release/gigi-node user@203.0.113.12:/opt/gigi/
ssh user@203.0.113.12 "cd /opt/gigi && docker-compose up -d relay"

# 4. Configure DNS
# bootstrap1.gigi.network -> 203.0.113.10
# bootstrap2.gigi.network -> 203.0.113.11
# relay.gigi.network -> 203.0.113.12

# 5. Update mobile app config with actual peer IDs
# (Get peer IDs from node logs after first run)
```

### 6. Key Features Summary

| Feature | Implementation |
|---------|---------------|
| **Cross-WiFi Connectivity** | Cloud bootstrap + relay nodes enable NAT traversal |
| **Single Mobile Support** | Connects to bootstrap, discovers peers via DHT |
| **Scalable Discovery** | Kademlia DHT with O(log n) lookup |
| **Reliable Messaging** | GossipSub for group chat, request-response for 1:1 |
| **NAT Traversal** | Circuit relay v2 for P2P behind routers |
| **Flexible Deployment** | Docker containers on any cloud provider |

This solution supports both scenarios: multiple mobiles across different WiFi networks, and a single mobile joining the P2P network from anywhere.