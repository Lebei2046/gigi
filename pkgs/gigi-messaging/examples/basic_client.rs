use gigi_messaging_lib::{MessagingClient, MessagingConfig, MessagingEvent};
use std::path::PathBuf;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Create messaging configuration
    let config = MessagingConfig {
        share_json_path: PathBuf::from("./share.json"),
        downloads_dir: PathBuf::from("./downloads"),
        temp_dir: PathBuf::from("./temp"),
        max_file_size: 100 * 1024 * 1024, // 100MB
        chunk_size: 64 * 1024, // 64KB
    };
    
    // Create messaging client
    let mut client = MessagingClient::new("test_user".to_string()).await?;
    
    println!("Client initialized!");
    println!("Peer ID: {}", client.get_peer_id());
    println!("Public Key: {}", client.get_public_key());
    
    // Generate a new keypair
    let keypair = MessagingClient::generate_keypair()?;
    println!("Generated new keypair:");
    println!("Private Key: base64 encoded");
    println!("Public Key: base64 encoded");
    
    // Start listening for events
    println!("Listening for events...");
    loop {
        match client.next_event().await {
            Some(event) => {
                println!("Received event: {:?}", event);
                
                match event {
                    MessagingEvent::PeerJoined { peer_id, nickname } => {
                        println!("New peer joined: {} ({})", nickname, peer_id);
                        
                        // Send a welcome message
                        if let Err(e) = client.send_message(peer_id, "Hello! Welcome to the network!".to_string()).await {
                            eprintln!("Failed to send message: {}", e);
                        }
                    }
                    MessagingEvent::MessageReceived { from, content } => {
                        println!("Message from {}: {}", from, content);
                    }
                    MessagingEvent::PeerLeft { peer_id, nickname } => {
                        println!("Peer left: {}", peer_id);
                    }
                    MessagingEvent::FileShared { file_id, filename, share_code } => {
                        println!("File shared: {} (ID: {}, Share Code: {})", filename, file_id, share_code);
                    }
                    MessagingEvent::FileTransferStarted { file_id, filename, total_size } => {
                        println!("Download started: {} (ID: {}, Size: {} bytes)", filename, file_id, total_size);
                    }
                    MessagingEvent::FileTransferProgress { file_id, downloaded_size, total_size, speed } => {
                        let progress = if total_size > 0 {
                            (downloaded_size as f64 / total_size as f64) * 100.0
                        } else {
                            0.0
                        };
                        println!("Download progress: {} - {:.1}% ({}/{} bytes, {:.2} KB/s)", 
                                file_id, progress, downloaded_size, total_size, speed / 1024.0);
                    }
                    MessagingEvent::FileTransferCompleted { file_id, filename, final_path } => {
                        println!("Download completed: {} -> {}", filename, final_path.display());
                    }
                    MessagingEvent::FileTransferFailed { file_id, error } => {
                        println!("Download failed: {} - {}", file_id, error);
                    }
                    MessagingEvent::Error { message } => {
                        eprintln!("Error: {}", message);
                    }
                    _ => {}
                }
            }
            None => {
                println!("Event stream ended");
                break;
            }
        }
    }
    
    Ok(())
}