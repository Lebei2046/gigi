use gigi_messaging_lib::{MessagingClient, MessagingEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create messaging client with auto-generated keypair
    let mut client = MessagingClient::new("TestUser".to_string()).await?;
    println!("Messaging client started!");
    println!("Peer ID: {}", client.get_peer_id());
    println!("Public Key: {}", client.get_public_key());
    
    // Listen for events
    println!("Listening for events...");
    while let Some(event) = client.next_event().await {
        match event {
            MessagingEvent::PeerJoined { peer_id, nickname } => {
                println!("New peer joined: {} ({})", nickname, peer_id);
            }
            MessagingEvent::MessageReceived { from, content } => {
                println!("Message from {}: {}", from, content);
            }
            MessagingEvent::GroupMessageReceived { from, group, content } => {
                println!("Group message from {} in {}: {}", from, group, content);
            }
            MessagingEvent::FileTransferStarted { file_id, filename, total_size } => {
                println!("File download started: {} ({} bytes)", filename, total_size);
            }
            MessagingEvent::FileTransferCompleted { file_id, filename, final_path } => {
                println!("File download completed: {} -> {}", filename, final_path.display());
            }
            MessagingEvent::Error { message } => {
                println!("Error: {}", message);
            }
            _ => {
                println!("Event: {:?}", event);
            }
        }
    }
    
    Ok(())
}