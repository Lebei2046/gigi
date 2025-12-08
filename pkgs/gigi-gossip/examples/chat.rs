use clap::Parser;
use gigi_gossip::{ComposedEvent, GossipChat, GossipEvent, Message};
use libp2p::{futures::StreamExt, identity::Keypair, SwarmBuilder};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Topic for group messaging
    #[arg(short, long, default_value = "gigi-chat")]
    topic: String,

    /// Nickname for the peer
    #[arg(short, long, default_value = "anonymous")]
    nickname: String,

    /// Listen port (optional, will use random port if not specified)
    #[arg(short, long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let args = Args::parse();

    // Create identity
    let keypair = Keypair::generate_ed25519();

    // Create behaviour
    let behaviour = GossipChat::create_behaviour(keypair.clone())?;

    // Build swarm with custom configuration
    let mut swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|_| behaviour)?
        .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(300)))
        .build();

    // Listen on specified or random ports
    let listen_addr_tcp = if let Some(port) = args.port {
        format!("/ip4/0.0.0.0/tcp/{}", port).parse()?
    } else {
        "/ip4/0.0.0.0/tcp/0".parse()?
    };

    let listen_addr_quic = "/ip4/0.0.0.0/udp/0/quic-v1".parse()?;

    swarm.listen_on(listen_addr_tcp)?;
    swarm.listen_on(listen_addr_quic)?;

    // Create gossip chat instance
    let (mut chat, event_receiver) =
        GossipChat::with_swarm(swarm, args.nickname.clone(), args.topic.clone())?;

    println!("🎯 Gigi Gossip Chat started!");
    println!("📝 Topic: {}", args.topic);
    println!("👤 Nickname: {}", args.nickname);
    println!("📡 Listening on ports (check swarm listeners for actual ports)");
    println!("💬 Type messages and press Enter to send");
    println!("🖼️  Type 'image <path>' to send an image");
    println!("📊 Type 'peers' to see connected peers");
    println!("❌ Type 'quit' to exit");
    println!("---");

    // Spawn event handling task
    let event_handle = tokio::spawn({
        let mut event_receiver = event_receiver;
        async move {
            while let Some(event) = event_receiver.next().await {
                match event {
                    GossipEvent::MessageReceived {
                        from: _,
                        sender,
                        message,
                    } => {
                        match message {
                            Message::Text {
                                content,
                                timestamp: _,
                            } => {
                                println!("💬 {}: {}", sender, content);
                            }
                            Message::Image {
                                data,
                                filename,
                                timestamp: _,
                            } => {
                                println!(
                                    "🖼️  {} sent image: {} ({} bytes)",
                                    sender,
                                    filename,
                                    data.len()
                                );

                                // Save image to disk
                                let output_path = format!("received_{}", filename);
                                if let Err(e) = tokio::fs::write(&output_path, data).await {
                                    eprintln!("Failed to save image {}: {}", filename, e);
                                } else {
                                    println!("💾 Saved to: {}", output_path);
                                }
                            }
                        }
                    }
                    GossipEvent::PeerJoined { peer_id, nickname } => {
                        println!("👋 {} ({}) joined the chat", nickname, peer_id);
                    }
                    GossipEvent::PeerLeft { peer_id, nickname } => {
                        println!("👋 {} ({}) left the chat", nickname, peer_id);
                    }
                    GossipEvent::Error(error) => {
                        eprintln!("❌ Error: {}", error);
                    }
                }
            }
        }
    });

    // Spawn a simple stdin reader that sends messages via a channel
    let (stdin_sender, mut stdin_receiver) = tokio::sync::mpsc::unbounded_channel::<String>();
    let stdin_handle = tokio::spawn(async move {
        use tokio::io::{AsyncBufReadExt, BufReader};
        let mut stdin = BufReader::new(tokio::io::stdin());
        let mut line = String::new();

        loop {
            line.clear();
            match stdin.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let line = line.trim().to_string();
                    if !line.is_empty() {
                        if let Err(_) = stdin_sender.send(line) {
                            break; // Channel closed
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Main loop for handling swarm events and user input
    let mut running = true;
    while running {
        tokio::select! {
            // Handle swarm events
            swarm_event = chat.swarm.select_next_some() => {
                match swarm_event {
                    libp2p::swarm::SwarmEvent::Behaviour(ComposedEvent::Gossipsub(event)) => {
                        if let Err(e) = chat.handle_event(ComposedEvent::Gossipsub(event)) {
                            eprintln!("Error handling gossipsub event: {}", e);
                        }
                    }
                    libp2p::swarm::SwarmEvent::Behaviour(ComposedEvent::Mdns(event)) => {
                        if let Err(e) = chat.handle_event(ComposedEvent::Mdns(event)) {
                            eprintln!("Error handling mDNS event: {}", e);
                        }
                    }
                    libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                        println!("📡 Listening on: {}", address);
                    }
                    libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("🔗 Connected to: {}", peer_id);
                    }
                    libp2p::swarm::SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        println!("🔌 Disconnected from: {}", peer_id);
                    }
                    _ => {}
                }
            }

            // Handle stdin input
            Some(input) = stdin_receiver.recv() => {
                match input.trim() {
                    "quit" | "exit" => {
                        println!("👋 Goodbye!");
                        running = false;
                    }
                    "peers" => {
                        let peers = chat.get_peers();
                        println!("📊 Connected peers:");
                        for (peer_id, nickname) in peers {
                            println!("  - {} ({})", nickname, peer_id);
                        }
                    }
                    line if line.starts_with("image ") => {
                        let path = line.strip_prefix("image ").unwrap().trim();
                        match tokio::fs::read(path).await {
                            Ok(data) => {
                                let filename = PathBuf::from(path)
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                
                                let size_mb = data.len() as f64 / (1024.0 * 1024.0);
                                println!("📊 Image size: {:.2} MB", size_mb);

                                if let Err(e) = chat.send_image_message(data, filename) {
                                    eprintln!("❌ Failed to send image: {}", e);
                                    if e.to_string().contains("too large") {
                                        println!("💡 Try using a smaller image file (< 1MB)");
                                    }
                                } else {
                                    println!("🖼️  Sent image: {}", path);
                                }
                            }
                            Err(e) => {
                                eprintln!("❌ Failed to read image {}: {}", path, e);
                            }
                        }
                    }
                    _ => {
                        // Regular text message
                        if let Err(e) = chat.send_text_message(input) {
                            eprintln!("Failed to send message: {}", e);
                        }
                    }
                }
            }

            // Handle Ctrl+C
            _ = tokio::signal::ctrl_c() => {
                println!("\n👋 Goodbye!");
                running = false;
            }
        }
    }

    // Clean up
    event_handle.abort();
    stdin_handle.abort();
    Ok(())
}
