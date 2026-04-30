// Copyright 2024 Gigi Team.
//
// Simple example of using Gigi DNS with full swarm integration

use clap::Parser;
use gigi_dns::{GigiDnsBehaviour, GigiDnsConfig};
use libp2p::futures::StreamExt;
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

    /// TTL for peer records in seconds
    #[arg(short = 't', long, default_value = "360")]
    ttl: u64,

    /// Query interval in seconds
    #[arg(short = 'q', long, default_value = "300")]
    query_interval: u64,

    /// Announce interval in seconds
    #[arg(short = 'a', long, default_value = "15")]
    announce_interval: u64,

    /// Cleanup interval in seconds
    #[arg(short = 'c', long, default_value = "30")]
    cleanup_interval: u64,

    /// Enable IPv6
    #[arg(long)]
    ipv6: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Respect RUST_LOG environment variable, default to INFO level
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let args = Args::parse();

    // Generate local peer ID
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    println!("🚀 Gigi DNS started");
    println!("👤 Peer ID: {}", local_peer_id);
    println!("📝 Nickname: {}", args.nickname);

    // Create Gigi DNS config with all timing parameters
    let config = GigiDnsConfig {
        nickname: args.nickname.clone(),
        capabilities: vec!["chat".to_string(), "file_sharing".to_string()],
        ttl: Duration::from_secs(args.ttl),
        query_interval: Duration::from_secs(args.query_interval),
        announce_interval: Duration::from_secs(args.announce_interval),
        cleanup_interval: Duration::from_secs(args.cleanup_interval),
        enable_ipv6: args.ipv6,
        ..Default::default()
    };

    println!("🔧 Capabilities: {:?}", config.capabilities);
    println!("⏱️  TTL: {}s", args.ttl);
    println!(
        "⏱️  Query interval: {}s (adaptive probing: 500ms → ... → {}s)",
        args.query_interval, args.query_interval
    );
    println!("⏱️  Announce interval: {}s", args.announce_interval);
    println!("⏱️  Cleanup interval: {}s", args.cleanup_interval);
    println!("📡 Mode: Multicast");
    println!("🌐 IP Version: {}", if args.ipv6 { "IPv6" } else { "IPv4" });

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
    let listen_addr = if args.ipv6 {
        "/ip6/::/tcp/0"
    } else {
        "/ip4/0.0.0.0/tcp/0"
    };
    swarm.listen_on(listen_addr.parse()?)?;

    println!("✅ Gigi DNS initialized successfully!");
    println!("ℹ️  Listening for peers on local network...");
    println!("ℹ️  Adaptive probing will discover peers quickly (starting at 500ms)");
    println!("ℹ️  Press Ctrl+C to stop");
    println!();

    // Main event loop
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("📍 Listening on: {}", address);
            }
            SwarmEvent::Behaviour(gigi_dns::GigiDnsEvent::Discovered(peer_info)) => {
                println!();
                println!("🎉 Discovered new peer:");
                println!("   👤 Nickname: {}", peer_info.nickname);
                println!("   🔑 Peer ID: {}", peer_info.peer_id);
                println!("   🌐 Multiaddr: {}", peer_info.multiaddr);
                println!("   🔧 Capabilities: {:?}", peer_info.capabilities);
                if !peer_info.metadata.is_empty() {
                    println!("   📋 Metadata: {:?}", peer_info.metadata);
                }
                println!();
            }
            SwarmEvent::Behaviour(gigi_dns::GigiDnsEvent::Updated {
                peer_id,
                old_info,
                new_info,
            }) => {
                println!();
                println!("🔄 Peer updated: {}", peer_id);
                println!("   Old nickname: {}", old_info.nickname);
                println!("   New nickname: {}", new_info.nickname);
                if old_info.multiaddr != new_info.multiaddr {
                    println!(
                        "   Address changed: {} -> {}",
                        old_info.multiaddr, new_info.multiaddr
                    );
                }
                println!();
            }
            SwarmEvent::Behaviour(gigi_dns::GigiDnsEvent::Expired { peer_id, .. }) => {
                println!();
                println!("⏰ Peer expired: {}", peer_id);
                println!();
            }
            SwarmEvent::Behaviour(gigi_dns::GigiDnsEvent::Offline {
                peer_id, reason, ..
            }) => {
                println!();
                println!("🔌 Peer offline: {} (reason: {:?})", peer_id, reason);
                println!();
            }
            event => {
                gigi_logging::debug!("Unhandled event: {:?}", event);
            }
        }
    }
}
