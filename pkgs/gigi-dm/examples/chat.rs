use clap::Parser;
use futures::StreamExt;
use gigi_dm::{save_received_image, DirectMessaging, Message, MessagingEvent};
use libp2p::{identity, noise, swarm::SwarmEvent, tcp, yamux, PeerId};
use std::io::{self, Write};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "0")]
    port: u16,

    /// Peer address to connect to (optional)
    #[arg(short = 'a', long)]
    addr: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let args = Args::parse();

    // Create swarm in chat.rs (moved from lib.rs)
    let id_keys = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(id_keys.public());

    let behaviour = DirectMessaging::create_behaviour(
        libp2p::request_response::Config::default()
            .with_request_timeout(std::time::Duration::from_secs(30)),
    )?;

    let swarm = libp2p::SwarmBuilder::with_existing_identity(id_keys)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_| behaviour)?
        .with_swarm_config(|c| {
            c.with_idle_connection_timeout(std::time::Duration::from_secs(120))
                .with_max_negotiating_inbound_streams(100)
        })
        .build();

    let mut messaging = DirectMessaging::with_swarm(swarm)?;
    println!("Local peer ID: {}", local_peer_id);

    // Start listening
    let listen_addr = messaging.start_listening(args.port)?;
    println!("Listening on: {}", listen_addr);

    // Connect to peer if address provided
    if let Some(addr_str) = args.addr {
        // Small delay to ensure the first peer is ready
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let addr: libp2p::Multiaddr = addr_str.parse()?;
        println!("Attempting to connect to: {}", addr);

        match messaging.dial_peer(&addr) {
            Ok(()) => {
                println!("✓ Connection attempt initiated to: {}", addr);
            }
            Err(e) => {
                eprintln!("✗ Failed to initiate connection: {}", e);
                eprintln!("Make sure the peer is running and accessible");
                eprintln!("Waiting for manual connections via /connect command...");
            }
        }
    }

    // Main event loop that handles both swarm events and user input
    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut line = String::new();

    loop {
        tokio::select! {
            // Handle swarm events
            event = messaging.swarm.select_next_some() => {
                match event {
                    SwarmEvent::Behaviour(req_resp_event) => {
                        if let Ok(Some(messaging_event)) = messaging.handle_request_response_event(req_resp_event).await {
                            handle_messaging_event(messaging_event).await;
                        }
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("\n✓ Connected to: {}", peer_id);
                        print!("> ");
                        io::stdout().flush().unwrap();
                    }
                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        println!("\n✗ Disconnected from: {}", peer_id);
                        print!("> ");
                        io::stdout().flush().unwrap();
                    }
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on: {}", address);
                    }
                    SwarmEvent::OutgoingConnectionError { error, .. } => {
                        eprintln!("Connection error: {}", error);
                    }
                    _ => {}
                }
            }

            // Handle user input
            _ = stdin.read_line(&mut line) => {
                let input = line.trim();
                if !input.is_empty() {
                    if let Some(command) = input.strip_prefix('/') {
                        handle_command(command, &mut messaging).await;
                    } else {
                        // Send as text message to all connected peers
                        let peers = messaging.get_connected_peers();
                        if peers.is_empty() {
                            println!("No connected peers. Waiting for connections...");
                        } else {
                            for peer_id in peers {
                                if let Err(e) = messaging.send_message(peer_id, Message::text(input)).await {
                                    eprintln!("Failed to send message to {}: {}", peer_id, e);
                                }
                            }
                        }
                    }
                }
                print!("> ");
                io::stdout().flush().unwrap();
                line.clear();
            }
        }
    }
}

async fn handle_messaging_event(event: MessagingEvent) {
    match event {
        MessagingEvent::MessageReceived { peer, message } => {
            match message {
                Message::Text(text) => {
                    println!("\n[{}] {}", peer, text);
                }
                Message::Image {
                    name,
                    mime_type,
                    data,
                } => {
                    println!(
                        "\n[{}] Image: {} ({} bytes, {})",
                        peer,
                        name,
                        data.len(),
                        mime_type
                    );
                    // Save the image to a file (moved to lib.rs)
                    if let Err(e) = save_received_image(&name, &data).await {
                        eprintln!("Failed to save image: {}", e);
                    } else {
                        println!("Image saved as: received_{}", name);
                    }
                }
            }
            print!("> ");
            io::stdout().flush().unwrap();
        }
        MessagingEvent::MessageAcknowledged { .. } => {
            // Message was acknowledged, no need to print
        }
        MessagingEvent::PeerError { error, .. } => {
            println!("\nPeer responded with error: {}", error);
            print!("> ");
            io::stdout().flush().unwrap();
        }
        MessagingEvent::OutboundFailure { peer, error } => {
            eprintln!("Outbound request failure to {}: {}", peer, error);
        }
        MessagingEvent::InboundFailure { error } => {
            eprintln!("Inbound request failure: {}", error);
        }
        MessagingEvent::ResponseSent => {
            // Response sent successfully
        }
    }
}

async fn handle_command(command: &str, messaging: &mut DirectMessaging) {
    let parts: Vec<&str> = command.split_whitespace().collect();
    let peers = messaging.get_connected_peers();

    match parts.get(0) {
        Some(&"connect") => {
            if parts.len() != 2 {
                println!("Usage: /connect <multiaddr>");
                return;
            }

            let addr: libp2p::Multiaddr = match parts[1].parse() {
                Ok(addr) => addr,
                Err(e) => {
                    println!("Invalid address: {}", e);
                    return;
                }
            };

            match messaging.dial_peer(&addr) {
                Ok(()) => {
                    println!("Connection attempt initiated to: {}", addr);
                    println!("Waiting for connection to be established...");
                }
                Err(e) => {
                    println!("Connection failed: {}", e);
                }
            }
        }
        Some(&"text") => {
            if parts.len() < 2 {
                println!("Usage: /text <message>");
                return;
            }

            if peers.is_empty() {
                println!("No connected peers to send message to");
                return;
            }

            let message_text = parts[1..].join(" ");
            for peer_id in &peers {
                if let Err(e) = messaging
                    .send_message(*peer_id, Message::text(&message_text))
                    .await
                {
                    eprintln!("Failed to send text message to {}: {}", peer_id, e);
                }
            }
            println!("Text message sent to {} peers", peers.len());
        }
        Some(&"image") => {
            if parts.len() != 2 {
                println!("Usage: /image <path_to_image>");
                return;
            }

            if peers.is_empty() {
                println!("No connected peers to send image to");
                return;
            }

            let path = PathBuf::from(parts[1]);
            let image_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown");

            match messaging.send_image_to_all(&peers, &path).await {
                Ok(()) => {
                    println!("Image '{}' sent to {} peers", image_name, peers.len());
                }
                Err(e) => {
                    println!("Failed to send image '{}': {}", image_name, e);
                }
            }
        }
        Some(&"peers") => {
            if peers.is_empty() {
                println!("No connected peers");
            } else {
                println!("Connected peers ({}):", peers.len());
                for peer in peers {
                    println!("  {}", peer);
                }
            }
        }
        Some(&"help") => {
            println!("Available commands:");
            println!("  /connect <multiaddr>  - Connect to a peer");
            println!("  /text <message>       - Send a text message to all connected peers");
            println!("  /image <path>         - Send an image file to all connected peers");
            println!("  /peers                - List connected peers");
            println!("  /help                 - Show this help");
            println!("\nYou can also type text directly (without any command) to send it to all connected peers.");
        }
        _ => {
            println!("Unknown command. Type /help for available commands.");
        }
    }
}

// Functions send_image_to_all and save_received_image moved to lib.rs
