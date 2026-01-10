// Copyright 2024 Gigi Team.
//
// Simple example of using Gigi DNS with full swarm integration

use clap::Parser;
use futures::StreamExt;
use gigi_dns::{GigiDnsBehaviour, GigiDnsConfig};
use libp2p::swarm::SwarmEvent;
use libp2p::{identity, noise, tcp, yamux, SwarmBuilder};
use libp2p_identity::PeerId;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Nickname for this peer
    #[arg(short, long, default_value = "Anonymous")]
    nickname: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let args = Args::parse();

    // Generate local peer ID
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    println!("🚀 Gigi DNS started");
    println!("👤 Peer ID: {}", local_peer_id);
    println!("📝 Nickname: {}", args.nickname);

    // Create Gigi DNS config with nickname
    let config = GigiDnsConfig {
        nickname: args.nickname.clone(),
        capabilities: vec!["chat".to_string(), "file_sharing".to_string()],
        ttl: Duration::from_secs(6 * 60),
        query_interval: Duration::from_secs(10),
        use_broadcast: false, // Use multicast instead of broadcast
        enable_ipv6: false,   // Use IPv4 for simplicity
        ..Default::default()
    };

    println!("🔧 Capabilities: {:?}", config.capabilities);

    // Create Gigi DNS behavior
    let mdns = GigiDnsBehaviour::new(local_peer_id, config)?;

    // Build the swarm with tcp transport, noise, yamux, and gigi-dns
    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_| mdns)?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    // Listen on all interfaces
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("✅ Gigi DNS initialized successfully!");
    println!("ℹ️  Listening for peers on local network...");
    println!("ℹ️  Press Ctrl+C to stop");
    println!("");

    // Main event loop
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("📍 Listening on: {}", address);
            }
            SwarmEvent::Behaviour(gigi_dns::GigiDnsEvent::Discovered(peer_info)) => {
                println!("");
                println!("🎉 Discovered new peer:");
                println!("   👤 Nickname: {}", peer_info.nickname);
                println!("   🔑 Peer ID: {}", peer_info.peer_id);
                println!("   🌐 Multiaddr: {}", peer_info.multiaddr);
                println!("   🔧 Capabilities: {:?}", peer_info.capabilities);
                if !peer_info.metadata.is_empty() {
                    println!("   📋 Metadata: {:?}", peer_info.metadata);
                }
                println!("");
            }
            SwarmEvent::Behaviour(gigi_dns::GigiDnsEvent::Updated {
                peer_id,
                old_info,
                new_info,
            }) => {
                println!("");
                println!("🔄 Peer updated: {}", peer_id);
                println!("   Old nickname: {}", old_info.nickname);
                println!("   New nickname: {}", new_info.nickname);
                if old_info.multiaddr != new_info.multiaddr {
                    println!(
                        "   Address changed: {} -> {}",
                        old_info.multiaddr, new_info.multiaddr
                    );
                }
                println!("");
            }
            SwarmEvent::Behaviour(gigi_dns::GigiDnsEvent::Expired { peer_id, .. }) => {
                println!("");
                println!("⏰ Peer expired: {}", peer_id);
                println!("");
            }
            SwarmEvent::Behaviour(gigi_dns::GigiDnsEvent::Offline { peer_id, .. }) => {
                println!("");
                println!("🔌 Peer offline: {}", peer_id);
                println!("");
            }
            event => {
                tracing::debug!("Unhandled event: {:?}", event);
            }
        }
    }
}
