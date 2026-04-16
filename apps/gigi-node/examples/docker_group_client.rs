use anyhow::Result;
use clap::Parser;
use futures::StreamExt;
use libp2p::{
    gossipsub, identify, identity, kad, ping, relay,
    swarm::{NetworkBehaviour, SwarmEvent},
    Multiaddr, PeerId, Swarm, Transport,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tracing::{debug, info, warn};

#[derive(NetworkBehaviour)]
struct ClientBehaviour {
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    gossipsub: gossipsub::Behaviour,
    identify: identify::Behaviour,
    ping: ping::Behaviour,
    relay: relay::Behaviour,
}

const GROUP: &str = "gigi-general";

#[derive(Default)]
struct Stats {
    messages_received: usize,
    messages_sent: usize,
}

struct PeerRegistry {
    #[allow(dead_code)]
    peer_addresses: HashMap<PeerId, Multiaddr>,
    connected_peers: Vec<PeerId>,
}

impl PeerRegistry {
    fn new() -> Self {
        Self {
            peer_addresses: HashMap::new(),
            connected_peers: Vec::new(),
        }
    }
}

#[derive(Parser, Debug, Clone)]
#[command(name = "docker_group_client")]
#[command(about = "Gigi P2P Group Chat Client")]
struct Args {
    #[arg(long, help = "Your display name in the chat")]
    username: String,

    #[arg(
        long,
        default_value = "0",
        help = "Port to listen on (0 for auto-assign)"
    )]
    port: u16,

    #[arg(
        long,
        help = "Bootstrap node address (e.g., /ip4/host.docker.internal/tcp/4001/p2p/Qm...)"
    )]
    bootstrap: Option<String>,

    #[arg(
        long,
        help = "Relay node address (e.g., /ip4/host.docker.internal/tcp/4002/p2p/Qm...)"
    )]
    relay: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    let username = &args.username;
    let listen_port = args.port;
    let bootstrap_addr = args.bootstrap.as_deref().unwrap_or("");

    info!(
        "Username: {}, Listen port: {}, Bootstrap: {}",
        username, listen_port, bootstrap_addr
    );

    let key = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(key.public());
    info!("Starting {} client, peer_id = {}", username, peer_id);

    let transport = libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::default())
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(libp2p::noise::Config::new(&key)?)
        .multiplex(libp2p::yamux::Config::default())
        .boxed();

    let identify = identify::Behaviour::new(identify::Config::new(
        "/gigi-client/1.0.0".to_string(),
        key.public(),
    ));

    let ping = ping::Behaviour::new(ping::Config::new());

    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(1))
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build gossipsub config: {}", e))?;

    let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(key.clone()),
        gossipsub_config,
    )
    .map_err(|e| anyhow::anyhow!("Failed to create gossipsub: {}", e))?;

    let mut swarm = Swarm::new(
        transport,
        ClientBehaviour {
            kademlia: kad::Behaviour::new(
                key.public().to_peer_id(),
                kad::store::MemoryStore::new(key.public().to_peer_id()),
            ),
            gossipsub,
            identify,
            ping,
            relay: relay::Behaviour::new(key.public().to_peer_id(), Default::default()),
        },
        peer_id,
        libp2p::swarm::Config::with_tokio_executor(),
    );

    let listen_addr = if listen_port > 0 {
        format!("/ip4/0.0.0.0/tcp/{}", listen_port)
    } else {
        "/ip4/0.0.0.0/tcp/0".to_string()
    };
    info!("Listening on: {}", listen_addr);
    swarm.listen_on(listen_addr.parse()?)?;

    let topic = gossipsub::IdentTopic::new(GROUP);
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
    info!("Subscribed to topic: {}", GROUP);

    // Add ourselves as a peer in the topic mesh
    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
    info!("Added explicit peer to mesh: {}", peer_id);

    let peer_registry = Arc::new(Mutex::new(PeerRegistry::new()));

    // Connect to bootstrap node
    if !bootstrap_addr.is_empty() {
        if let Some((bootstrap_peer_id, bootstrap_base_addr)) = parse_peer_addr(bootstrap_addr) {
            // Extract the base address from the bootstrap_addr (remove /p2p/peer_id)
            let base_addr = bootstrap_base_addr.to_string();
            info!(
                "Bootstrap peer: {} at base address {}",
                bootstrap_peer_id, base_addr
            );

            // Add bootstrap to explicit peers for GossipSub
            swarm
                .behaviour_mut()
                .gossipsub
                .add_explicit_peer(&bootstrap_peer_id);
            info!("Added bootstrap as explicit peer for GossipSub");

            // Add to Kademlia and dial
            swarm
                .behaviour_mut()
                .kademlia
                .add_address(&bootstrap_peer_id, base_addr.parse()?);

            let full_addr: libp2p::Multiaddr =
                format!("{}/p2p/{}", base_addr, bootstrap_peer_id).parse()?;
            match swarm.dial(full_addr.clone()) {
                Ok(_) => {
                    info!("Dialed bootstrap at {}", full_addr);
                }
                Err(e) => {
                    warn!("Failed to dial {}: {}", full_addr, e);
                }
            }

            match swarm.behaviour_mut().kademlia.bootstrap() {
                Ok(_) => info!("DHT bootstrap initiated"),
                Err(e) => warn!("DHT bootstrap failed: {:?}", e),
            }
        }
    }

    // Connect to relay node
    if let Some(relay_addr) = &args.relay {
        if let Some((relay_peer_id, relay_base_addr)) = parse_peer_addr(relay_addr) {
            let base_addr = relay_base_addr.to_string();
            info!(
                "Relay peer: {} at base address {}",
                relay_peer_id, base_addr
            );

            // Add relay to explicit peers for GossipSub
            swarm
                .behaviour_mut()
                .gossipsub
                .add_explicit_peer(&relay_peer_id);
            info!("Added relay as explicit peer for GossipSub");

            let full_relay_addr: libp2p::Multiaddr =
                format!("{}/p2p/{}", base_addr, relay_peer_id).parse()?;
            match swarm.dial(full_relay_addr.clone()) {
                Ok(_) => {
                    info!("Dialed relay at {}", full_relay_addr);
                }
                Err(e) => {
                    warn!("Failed to dial relay {}: {}", full_relay_addr, e);
                }
            }
        }
    }

    // Wait for connections and GossipSub mesh to establish
    tokio::time::sleep(Duration::from_secs(10)).await;

    let stats = Arc::new(Mutex::new(Stats::default()));

    let (tx, mut msg_tx) = tokio::sync::mpsc::channel::<String>(100);

    let username_clone = username.to_string();
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let mut reader = tokio::io::BufReader::new(stdin).lines();

        println!("\n=== Group Chat Started ===");
        println!("You are: {}", username_clone);
        println!("Type your messages and press Enter to send.");
        println!("==========================\n");

        while let Ok(Some(line)) = reader.next_line().await {
            if !line.trim().is_empty() {
                let _ = tx_clone.send(line).await;
            }
        }
    });

    info!("Starting chat loop for {}...", username);

    loop {
        tokio::select! {
            Some(msg) = msg_tx.recv() => {
                let full_msg = format!("{}: {}", username, msg);
                info!("Publishing: {}", full_msg);
                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(
                    topic.clone(),
                    full_msg.as_bytes().to_vec(),
                ) {
                    warn!("Publish error: {:?}", e);
                } else {
                    stats.lock().unwrap().messages_sent += 1;
                }
            }

            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::Behaviour(ClientBehaviourEvent::Gossipsub(
                        gossipsub::Event::Message { message, .. }
                    )) => {
                        let text = String::from_utf8_lossy(&message.data);

                        if !text.starts_with(&format!("{}:", username)) {
                            let mut s = stats.lock().unwrap();
                            s.messages_received += 1;
                            println!("\n[{}] {}", s.messages_received, text);
                            print!("{}> ", username);
                            let _ = std::io::Write::flush(&mut std::io::stdout());
                        }
                    }
                    SwarmEvent::Behaviour(ClientBehaviourEvent::Gossipsub(
                        gossipsub::Event::Subscribed { peer_id, topic }
                    )) => {
                        info!("Peer {} subscribed to topic {}", peer_id, topic);
                    }
                    SwarmEvent::Behaviour(ClientBehaviourEvent::Gossipsub(
                        gossipsub::Event::Unsubscribed { peer_id, topic }
                    )) => {
                        info!("Peer {} unsubscribed from topic {}", peer_id, topic);
                    }
                    SwarmEvent::Behaviour(ClientBehaviourEvent::Gossipsub(
                        gossipsub::Event::GossipsubNotSupported { peer_id }
                    )) => {
                        warn!("Peer {} does not support GossipSub!", peer_id);
                    }
                    SwarmEvent::Behaviour(ClientBehaviourEvent::Identify(identify::Event::Received { peer_id, info, .. })) => {
                        debug!("Identify: received info from peer {}", peer_id);
                        for addr in &info.listen_addrs {
                            debug!("Peer {} listening on: {}", peer_id, addr);
                            if !swarm.is_connected(&peer_id) {
                                let dial_addr = format!("{}/p2p/{}", addr, peer_id);
                                if let Ok(multiaddr) = dial_addr.parse::<Multiaddr>() {
                                    debug!("Dialing discovered peer at: {}", multiaddr);
                                    let _ = swarm.dial(multiaddr);
                                }
                            }
                        }
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, connection_id, .. } => {
                        info!("Connected to peer {} (connection: {:?})", peer_id, connection_id);
                        // Add to GossipSub mesh so we receive messages from this peer
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        peer_registry.lock().unwrap().connected_peers.push(peer_id);
                    }
                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        info!("Disconnected from peer {}", peer_id);
                    }
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("Listening on: {}", address);
                    }
                    SwarmEvent::Behaviour(ClientBehaviourEvent::Relay(e)) => {
                        debug!("Relay event: {:?}", e);
                    }
                    SwarmEvent::Behaviour(ClientBehaviourEvent::Kademlia(e)) => {
                        debug!("Kademlia event: {:?}", e);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn parse_peer_addr(s: &str) -> Option<(PeerId, libp2p::Multiaddr)> {
    let addr: libp2p::Multiaddr = s.parse().ok()?;
    let mut iter = addr.iter();
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
        let addr_without_peer: libp2p::Multiaddr = addr_parts.into_iter().collect();
        Some((peer_id, addr_without_peer))
    } else {
        None
    }
}
