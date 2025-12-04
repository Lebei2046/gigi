use clap::Parser;
use gigi_downloading::{FileTransferClient, ClientConfig, FileTransferEvent, ComposedEvent};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Server address for connecting to
    #[arg(short, long)]
    addr: String,

    /// Shared code for file downloading
    #[arg(short, long)]
    code: String,

    /// Directory for saving files
    #[arg(short, long, default_value = ".")]
    output: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let args = Args::parse();

    let server_addr: libp2p::Multiaddr = args.addr.parse()?;

    let config = ClientConfig { server_addr };

    let (mut client, mut event_receiver) = FileTransferClient::new(config).await?;
    println!("DEBUG: Client created successfully");

    println!("Connecting to server at {}", args.addr);

    // Start the client event loop
    let mut connected = false;
    let mut file_info_requested = false;
    let _download_started = false;

    use futures::StreamExt;
    
    loop {
        tokio::select! {
            // Poll swarm events - this will block until an event is available
            swarm_event = client.swarm.select_next_some() => {
                // Handle the swarm event directly here
                match swarm_event {
                    libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("Connected to server: {}", peer_id);
                        connected = true;
                        client.server_peer_id = Some(peer_id);
                    }
                    libp2p::swarm::SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        println!("Disconnected from server: {}", peer_id);
                        connected = false;
                        if client.server_peer_id == Some(peer_id) {
                            client.server_peer_id = None;
                        }
                    }
                    libp2p::swarm::SwarmEvent::Behaviour(ComposedEvent::RequestResponse(event)) => {
                        client.handle_request_response_event(event).await?;
                    }
                    _ => {}
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                println!("DEBUG: 5 second timeout - still waiting for events...");
                // Check if we have any connected peers
                println!("DEBUG: Connected peers: {:?}", client.swarm.connected_peers().collect::<Vec<_>>());
            }
            // Wait for events from the event receiver
            event = event_receiver.next() => {
                match event {
                    Some(event) => {
                        match event {
                            FileTransferEvent::Connected { peer_id } => {
                                println!("Connected to server: {}", peer_id);
                                connected = true;
                            }
                            FileTransferEvent::Disconnected { peer_id } => {
                                println!("Disconnected from server: {}", peer_id);
                                connected = false;
                            }
                            FileTransferEvent::ResponseReceived { response, .. } => {
                                match response {
                                    gigi_downloading::Response::FileInfo(Some(info)) => {
                                        println!("File info received:");
                                        println!("  Name: {}", info.name);
                                        println!("  Size: {} bytes", info.size);
                                        println!("  Chunks: {}", info.chunk_count);
                                        println!("  Hash: {}", info.hash);

                                        // Start download
                                        if let Err(e) = client.start_download(info.clone(), &args.output).await {
                                            eprintln!("Failed to start download: {}", e);
                                            return Ok(());
                                        }
                                        let _download_started = true;
                                    }
                                    gigi_downloading::Response::FileInfo(None) => {
                                        eprintln!("File not found with code: {}", args.code);
                                        return Ok(());
                                    }
                                    _ => {}
                                }
                            }
                            FileTransferEvent::DownloadProgress { file_id: _, downloaded_chunks, total_chunks } => {
                                let progress = (downloaded_chunks as f64 / total_chunks as f64) * 100.0;
                                println!("Download progress: {:.1}% ({}/{})", progress, downloaded_chunks, total_chunks);
                            }
                            FileTransferEvent::DownloadCompleted { file_id: _, path } => {
                                println!("Download completed!");
                                println!("File saved to: {:?}", path);
                                return Ok(());
                            }
                            FileTransferEvent::Error(error) => {
                                eprintln!("Error: {}", error);
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                    None => {
                        println!("Event receiver closed");
                        return Ok(());
                    }
                }
            }
        }

        // Request file info once connected
        if connected && !file_info_requested {
            println!("Requesting file info...");
            if let Err(e) = client.get_file_info(&args.code).await {
                eprintln!("Failed to request file info: {}", e);
                return Ok(());
            }
            file_info_requested = true;
        }
    }
}