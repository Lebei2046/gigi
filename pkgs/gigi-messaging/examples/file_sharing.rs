use gigi_messaging_lib::{MessagingClient, MessagingConfig, MessagingEvent};
use std::path::PathBuf;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let mut client = MessagingClient::new("file_sharer".to_string()).await?;
    
    println!("File sharing client initialized!");
    println!("Peer ID: {}", client.get_peer_id());
    
    // Create a test file to share
    let test_file_path = PathBuf::from("./examples/test_file.txt");
    std::fs::write(&test_file_path, "This is a test file for sharing!\nIt contains multiple lines.\nYou can download this file using the share code.")?;
    println!("Created test file: {}", test_file_path.display());
    
    // Share the file
    match client.share_file(test_file_path.to_string_lossy().to_string()).await {
        Ok(share_code) => {
            println!("File shared successfully! Share code: {}", share_code);
            println!("Other users can download this file using this share code.");
            
            // List shared files
            match client.get_shared_files().await {
                Ok(files) => {
                    println!("\nCurrently shared files:");
                    for file in files {
                        println!("  - {} ({} bytes, share code: {})", file.name, file.size, file.share_code);
                    }
                }
                Err(e) => eprintln!("Failed to list shared files: {}", e),
            }
        }
        Err(e) => eprintln!("Failed to share file: {}", e),
    }
    
    // Listen for events
    println!("\nListening for file sharing events...");
    loop {
        match client.next_event().await {
            Some(event) => {
                match event {
                    MessagingEvent::PeerJoined { peer_id, nickname } => {
                        println!("Peer joined: {} ({})", nickname, peer_id);
                    }
                    MessagingEvent::FileShared { file_id, filename, share_code } => {
                        println!("File available for download: {} (share code: {})", filename, share_code);
                    }
                    MessagingEvent::MessageReceived { from, content } => {
                        println!("Message from {}: {}", from, content);
                        
                        // If someone requests a file download
                        if content.starts_with("!download ") {
                            let share_code = content.trim_start_matches("!download ");
                            match client.request_file(share_code.to_string()).await {
                                Ok(download_id) => {
                                    println!("Started download with ID: {}", download_id);
                                }
                                Err(e) => {
                                    println!("Failed to start download: {}", e);
                                }
                            }
                        }
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
                        println!("Progress: {} - {:.1}% ({}/{} bytes, {:.2} KB/s)", 
                                file_id, progress, downloaded_size, total_size, speed / 1024.0);
                    }
                    MessagingEvent::FileTransferCompleted { file_id, filename, final_path } => {
                        println!("Download completed: {} -> {}", filename, final_path.display());
                    }
                    MessagingEvent::Error { message } => {
                        eprintln!("Error: {}", message);
                    }
                    _ => {}
                }
            }
            None => break,
        }
    }
    
    Ok(())
}