use clap::Parser;
use futures::StreamExt;
use gigi_downloading::{ComposedEvent, FileTransferServer};
use libp2p::SwarmBuilder;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory for recording shared file info
    #[arg(short, long)]
    info_path: PathBuf,

    /// Space-separated file paths to share
    #[arg(short, long)]
    files: Vec<PathBuf>,

    /// Port for listening
    #[arg(short, long, default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let args = Args::parse();

    // Create identity and behaviour
    let id_keys = libp2p::identity::Keypair::generate_ed25519();

    let behaviour = FileTransferServer::create_behaviour()?;

    let mut swarm = SwarmBuilder::with_existing_identity(id_keys)
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_behaviour(|_| behaviour)?
        .with_swarm_config(|c| {
            c.with_idle_connection_timeout(std::time::Duration::from_secs(300))
            // 5 minutes for large files
        })
        .build();

    let listen_addr: libp2p::Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", args.port).parse()?;
    swarm.listen_on(listen_addr)?;

    let (mut server, mut event_receiver) = FileTransferServer::with_swarm(swarm, args.info_path)?;

    // Share the specified files
    for file_path in args.files {
        match server.share_file(&file_path) {
            Ok(file_id) => {
                println!("Shared file: {:?} with ID: {}", file_path, file_id);
            }
            Err(e) => {
                eprintln!("Failed to share file {:?}: {}", file_path, e);
            }
        }
    }

    // Print initial file list
    let files = server.list_files();
    println!("Shared files:");
    for file in files {
        println!("  {} - {} bytes - ID: {}", file.name, file.size, file.id);
    }

    // Start event handling in a separate task
    tokio::spawn(async move {
        use futures::StreamExt;
        while let Some(event) = event_receiver.next().await {
            match event {
                gigi_downloading::FileTransferEvent::FileShared { file_id, info } => {
                    println!("File shared: {} (ID: {})", info.name, file_id);
                }
                gigi_downloading::FileTransferEvent::FileRevoked { file_id } => {
                    println!("File revoked: {}", file_id);
                }
                gigi_downloading::FileTransferEvent::Error(error) => {
                    eprintln!("Error: {}", error);
                }
                _ => {}
            }
        }
    });

    // Start the server
    println!("Server listening on port {}", args.port);
    println!("Press Ctrl+C to stop the server");

    // Run the server event loop in a separate task
    let server_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                swarm_event = server.swarm.select_next_some() => {
                    match swarm_event {
                        libp2p::swarm::SwarmEvent::Behaviour(ComposedEvent::RequestResponse(event)) => {
                            if let Err(e) = server.handle_request_response_event(event) {
                                eprintln!("Server error handling event: {}", e);
                            }
                        }
                        libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            println!("Connection established with {}", peer_id);
                        }
                        libp2p::swarm::SwarmEvent::IncomingConnectionError { local_addr, send_back_addr, error, .. } => {
                            eprintln!("Incoming connection error from {} to {}: {}", send_back_addr, local_addr, error);
                        }
                        libp2p::swarm::SwarmEvent::ListenerClosed { addresses, reason, .. } => {
                            eprintln!("Listener closed on {:?}: {:?}", addresses, reason);
                        }
                        libp2p::swarm::SwarmEvent::ListenerError { error, .. } => {
                            eprintln!("Listener error: {}", error);
                        }
                        _ => {}
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    println!("Ctrl+C received, shutting down...");
                    break;
                }
            }
        }
    });

    server_handle.await?;
    Ok(())
}
