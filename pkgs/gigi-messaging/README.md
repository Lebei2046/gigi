# Gigi Messaging Library

A Rust library that provides a high-level API for peer-to-peer messaging and file sharing built on top of gigi-p2p.

## Features

- **Auto Discovery**: Automatically discover peers on the same network
- **Direct Messaging**: Send and receive messages with connected peers  
- **Group Messaging**: Join groups and send/receive group messages
- **File Sharing**: Share files with peers using share codes
- **File Transfer**: Download files with progress tracking
- **Event System**: Unified event handling for all P2P activities
- **Key Management**: Generate and manage cryptographic keypairs

## Quick Start

```rust
use gigi_messaging_lib::{MessagingClient, MessagingEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with auto-generated keypair
    let mut client = MessagingClient::new("nickname".to_string()).await?;
    
    println!("Peer ID: {}", client.get_peer_id());
    
    // Listen for events
    while let Some(event) = client.next_event().await {
        match event {
            MessagingEvent::PeerJoined { peer_id, nickname } => {
                println!("New peer: {} ({})", nickname, peer_id);
            }
            MessagingEvent::MessageReceived { from, content } => {
                println!("Message from {}: {}", from, content);
            }
            _ => println!("Event: {:?}", event),
        }
    }
    
    Ok(())
}
```

## API Reference

### Client Creation

```rust
// Create client with auto-generated keypair
let mut client = MessagingClient::new("nickname".to_string()).await?;

// Generate new keypair
let keypair = MessagingClient::generate_keypair()?;
```

### Peer Management

```rust
// Get your peer ID
let peer_id = client.get_peer_id();

// Get public key
let public_key = client.get_public_key();

// List connected peers
let peers = client.get_connected_peers().await?;
```

### Messaging

```rust
// Send direct message
client.send_message("peer_id".to_string(), "Hello!".to_string()).await?;

// Send image
let image_data = std::fs::read("image.jpg")?;
client.send_image("peer_id".to_string(), image_data, "image.jpg".to_string()).await?;

// Join group
client.join_group("group_name".to_string()).await?;

// Leave group
client.leave_group("group_name".to_string()).await?;
```

### File Sharing

```rust
// Share a file
let share_code = client.share_file("path/to/file.txt".to_string()).await?;

// Download file using share code
let download_id = client.request_file(share_code).await?;

// Get shared files list
let shared_files = client.get_shared_files().await?;

// Cancel active download
client.cancel_download(download_id).await?;

// Stop sharing a file
client.unshare_file(share_code).await?;
```

## Events

The library emits various events that you can listen to:

- `PeerJoined`: A new peer connected
- `PeerLeft`: A peer disconnected
- `MessageReceived`: Received a direct message
- `ImageReceived`: Received an image
- `GroupMessageReceived`: Received a group message
- `FileShared`: A file is now available for download
- `FileTransferStarted`: File download started
- `FileTransferProgress`: File download progress update
- `FileTransferCompleted`: File download completed
- `FileTransferFailed`: File download failed
- `Error`: An error occurred

## Configuration

The library uses default configuration but can be customized (implementation pending).

## Examples

See the `examples/` directory for complete working examples:

- `basic.rs`: Basic client setup and event handling
- `basic_client.rs`: More comprehensive client example
- `file_sharing.rs`: File sharing and download example

## Dependencies

- `gigi-p2p`: Core P2P functionality
- `libp2p`: Peer-to-peer networking
- `tokio`: Async runtime
- `serde`: Serialization
- `uuid`: Unique identifiers

## License

This project is part of the Gigi ecosystem.