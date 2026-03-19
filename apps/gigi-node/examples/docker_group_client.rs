use anyhow::Result;
use futures::StreamExt;
use libp2p::{
    gossipsub, identity, kad,
    swarm::{NetworkBehaviour, SwarmEvent},
    PeerId, Swarm, Transport,
};
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

#[derive(NetworkBehaviour)]
struct ClientBehaviour {
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    gossipsub: gossipsub::Behaviour,
}

const GROUP: &str = "gigi-general";

#[derive(Default)]
struct Stats {
    messages_received: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: docker_client <role> <bootstrap_addr>");
        println!("  role: 'sender' or 'receiver'");
        println!("  bootstrap_addr: e.g., /ip4/host.docker.internal/tcp/4001/p2p/Qm...");
        std::process::exit(1);
    }

    let role = &args[1];
    let bootstrap_addr = &args[2];

    let key = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(key.public());
    info!("Starting {} client, peer_id = {}", role, peer_id);

    let transport = libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::default())
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(libp2p::noise::Config::new(&key)?)
        .multiplex(libp2p::yamux::Config::default())
        .boxed();

    let mut swarm = Swarm::new(
        transport,
        ClientBehaviour {
            kademlia: kad::Behaviour::new(
                key.public().to_peer_id(),
                kad::store::MemoryStore::new(key.public().to_peer_id()),
            ),
            gossipsub: gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub::Config::default(),
            )
            .map_err(|e| anyhow::anyhow!("Failed to create gossipsub: {}", e))?,
        },
        peer_id,
        libp2p::swarm::Config::with_tokio_executor(),
    );

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let topic = gossipsub::IdentTopic::new(GROUP);
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
    info!("Subscribed to topic: {}", GROUP);

    if let Some((peer_id, addr)) = parse_peer_addr(bootstrap_addr) {
        info!("Adding bootstrap peer: {} at {}", peer_id, addr);
        swarm.behaviour_mut().kademlia.add_address(&peer_id, addr.clone());
        swarm.behaviour_mut().kademlia.bootstrap()?;
        swarm.dial(addr)?;
    }

    let stats = Arc::new(Mutex::new(Stats::default()));

    if role == "sender" {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        
        let messages = vec![
            "alice: hello group!",
            "alice: anyone here?",
            "alice: this is a test from Docker!",
        ];
        
        for msg in messages {
            info!("Publishing: {}", msg);
            if let Err(e) = swarm.behaviour_mut().gossipsub.publish(
                topic.clone(),
                msg.as_bytes().to_vec(),
            ) {
                warn!("Publish error: {:?}", e);
            }
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
        
        info!("Sender finished publishing, listening for 60s...");
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    } else {
        info!("Receiver mode: waiting for messages...");
        loop {
            tokio::select! {
                event = swarm.select_next_some() => {
                    if let SwarmEvent::Behaviour(ClientBehaviourEvent::Gossipsub(
                        gossipsub::Event::Message { message, .. }
                    )) = event {
                        let text = String::from_utf8_lossy(&message.data);
                        info!("Received: \"{}\" from {:?}", text, message.source);
                        
                        let mut s = stats.lock().unwrap();
                        s.messages_received += 1;
                        println!("[RECEIVED {}] {}", s.messages_received, text);
                    }
                }
            }
        }
    }

    Ok(())
}

fn parse_peer_addr(s: &str) -> Option<(PeerId, libp2p::Multiaddr)> {
    let parts: Vec<&str> = s.splitn(2, "/p2p/").collect();
    if parts.len() != 2 {
        return None;
    }
    let addr: libp2p::Multiaddr = parts[0].parse().ok()?;
    let peer_id: PeerId = parts[1].parse().ok()?;
    Some((peer_id, addr))
}