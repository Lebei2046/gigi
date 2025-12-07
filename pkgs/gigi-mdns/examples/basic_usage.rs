//! Basic usage example of the gigi-mdns library
//!
//! This example demonstrates how to create a nickname manager,
//! set a nickname, discover peers, and handle events.
//!
//! Usage:
//!   cargo run --example basic_usage -- --nickname <your-nickname>
//!
//! Examples:
//!   cargo run --example basic_usage -- --nickname device-1
//!   cargo run --example basic_usage -- --nickname device-2

use futures::StreamExt;
use gigi_mdns::{Nickname, NicknameBehaviourEvent, NicknameEvent, NicknameManager};
use libp2p::Transport;
use std::env;
use tokio::time::{sleep, Duration};

/// Handle swarm events and convert to nickname events
fn handle_swarm_event(
    manager: &mut NicknameManager,
    event: libp2p::swarm::SwarmEvent<NicknameBehaviourEvent>,
) -> Vec<NicknameEvent> {
    let mut events = Vec::new();

    match event {
        libp2p::swarm::SwarmEvent::Behaviour(NicknameBehaviourEvent::Mdns(mdns_event)) => {
            match mdns_event {
                libp2p::mdns::Event::Discovered(list) => {
                    for (peer_id, addr) in list {
                        if let Ok(_) = manager.handle_mdns_discovered(peer_id, addr) {
                            // Try to get the nickname right after discovery
                            let _ = manager.request_nickname(peer_id);
                        }
                    }
                }
                libp2p::mdns::Event::Expired(list) => {
                    for (peer_id, _) in list {
                        if let Some(event) = manager.handle_mdns_expired(peer_id) {
                            events.push(event);
                        }
                    }
                }
            }
        }
        libp2p::swarm::SwarmEvent::Behaviour(NicknameBehaviourEvent::RequestResponse(
            req_resp_event,
        )) => {
            if let Ok(Some(event)) = manager.handle_request_response_event(req_resp_event) {
                events.push(event);
            }
        }
        libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
            events.push(NicknameEvent::ListeningOn { address });
        }
        libp2p::swarm::SwarmEvent::ListenerClosed { addresses, .. } => {
            for addr in addresses {
                events.push(NicknameEvent::NetworkEvent {
                    message: format!("Listener closed: {}", addr),
                });
            }
        }
        _ => {
            // Handle other swarm events if needed
        }
    }

    events
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Parse command line arguments
    let nickname = if args.len() > 2 && args[1] == "--nickname" {
        args[2].clone()
    } else {
        // Generate a random nickname if not provided
        let random_nickname = NicknameManager::generate_random();
        random_nickname.as_str().to_string()
    };

    println!("🚀 Starting mDNS Nickname Example");
    println!("📝 My nickname: {}", nickname);

    // Create identity and behaviour
    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let peer_id = libp2p::PeerId::from(keypair.public());

    // Create behaviour with custom mDNS config
    let mdns_config = libp2p::mdns::Config {
        ttl: std::time::Duration::from_secs(60),
        query_interval: std::time::Duration::from_secs(10),
        ..libp2p::mdns::Config::default()
    };

    let behaviour = NicknameManager::create_behaviour(
        peer_id,
        mdns_config,
        libp2p::request_response::Config::default(),
    )?;

    // Create swarm
    let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_other_transport(|_keypair| {
            libp2p::tcp::tokio::Transport::default()
                .upgrade(libp2p::core::upgrade::Version::V1)
                .authenticate(
                    libp2p::noise::Config::new(&_keypair)
                        .expect("Signing libp2p-noise static DH keypair failed."),
                )
                .multiplex(libp2p::yamux::Config::default())
                .boxed()
        })
        .expect("Failed to create transport")
        .with_behaviour(|_keypair| behaviour)
        .expect("Failed to create behaviour")
        .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(60)))
        .build();

    // Create nickname manager
    let mut manager = NicknameManager::with_swarm(swarm)?;

    // Set our nickname
    let my_nickname = Nickname::new(nickname)?;
    manager.set_nickname(my_nickname.clone());
    println!(
        "✅ Set nickname to: {}",
        manager.get_nickname().unwrap().as_str()
    );

    // Start listening on a local address
    println!("🔍 Starting peer discovery...");
    let listen_addr: libp2p::Multiaddr = "/ip4/0.0.0.0/tcp/0".parse().unwrap();
    println!("📍 Attempting to listen on: {}", listen_addr);
    manager.start_listening(listen_addr)?;

    // Listen for events (run for 30 seconds to allow discovery)
    println!("📡 Listening for events for 30 seconds...");
    let mut event_count = 0;
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < Duration::from_secs(30) {
        tokio::select! {
            // Handle swarm events
            event = manager.swarm.select_next_some() => {
                let events = handle_swarm_event(&mut manager, event);
                for event in events {
                    match event {
                        NicknameEvent::PeerDiscovered { peer_id, nickname } => {
                            println!("🎉 Peer discovered: {} (nickname: {:?})", peer_id, nickname);
                        }
                        NicknameEvent::NicknameUpdated { peer_id, nickname } => {
                            println!("🔄 Nickname updated: {} -> {}", peer_id, nickname);
                        }
                        NicknameEvent::PeerExpired { peer_id } => {
                            println!("💀 Peer expired: {}", peer_id);
                        }
                        NicknameEvent::NetworkEvent { message } => {
                            println!("🌐 Network: {}", message);
                        }
                        NicknameEvent::RequestReceived { peer_id, request } => {
                            println!("📨 Request from {}: {:?}", peer_id, request);
                        }
                        NicknameEvent::ListeningOn { address } => {
                            println!("🎧 Actually listening on: {}", address);
                        }
                    }
                    event_count += 1;
                }
            }
            _ = sleep(Duration::from_millis(500)) => {
                // Periodically announce our nickname
                if event_count > 0 && event_count % 10 == 0 {
                    println!("📢 Announcing nickname to network...");
                    let _ = manager.announce_nickname();
                }
            }
        }
    }

    // Show discovered peers
    println!("\n📋 Discovered peers:");
    for (peer_id, peer_info) in manager.get_discovered_peers() {
        let nickname = peer_info
            .nickname
            .as_ref()
            .map(|n| n.as_str())
            .unwrap_or("unnamed");
        println!(
            "  {} -> {} ({} addresses)",
            peer_id,
            nickname,
            peer_info.addresses.len()
        );
    }

    // Announce our nickname to discovered peers one final time
    println!("\n📢 Final nickname broadcast...");
    manager.announce_nickname()?;

    println!("✨ Example completed!");
    Ok(())
}
