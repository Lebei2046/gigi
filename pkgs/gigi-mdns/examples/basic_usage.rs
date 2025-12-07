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

use mdns_nickname::{Nickname, NicknameEvent, NicknameManager};
use std::env;
use tokio::time::{sleep, Duration};

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

    // Create a nickname manager
    let mut manager = NicknameManager::new().await?;

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
            swarm_events = manager.handle_swarm_events() => {
                if !swarm_events.is_empty() {
                    for event in swarm_events {
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
