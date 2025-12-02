# Gigi Downloading

A Rust library for peer-to-peer file transfers using libp2p's request-response protocol.

## Features

- **File Transfer**: Transfer files between peers over the network
- **Chunked Transfer**: Large files are automatically split into chunks for efficient transfer
- **File Integrity**: SHA-256 hash verification ensures file integrity
- **Event-Driven**: Async event handling for transfer progress
- **Discovery**: Built-in file listing and metadata exchange
- **Secure**: Uses libp2p's noise protocol for encrypted communication

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
gigi-downloading = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
libp2p = { version = "0.56.0", features = ["request-response", "tokio", "noise", "yamux", "tcp"] }
```

## Usage

### Basic Example

```rust
use gigi_downloading::{TransferManager, TransferEvent};
use libp2p::Multiaddr;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a transfer manager
    let (mut manager, mut receiver) = TransferManager::new_with_events().await?;
    
    // Start listening
    let addr: Multiaddr = "/ip4/127.0.0.1/tcp/8080".parse()?;
    manager.start_listening(addr)?;
    
    // Add a file for sharing
    let file_data = b"Hello, World!".to_vec();
    let metadata = manager.add_file("hello.txt".to_string(), file_data)?;
    
    // Handle events
    while let Ok(event) = receiver.recv().await {
        match event {
            TransferEvent::PeerConnected { peer_id } => {
                println!("Peer connected: {}", peer_id);
            }
            TransferEvent::FileReceived { filename, metadata } => {
                println!("File received: {} ({} bytes)", filename, metadata.size);
            }
            TransferEvent::TransferCompleted { filename, peer_id } => {
                println!("Transfer completed: {} from {}", filename, peer_id);
            }
            _ => {}
        }
    }
    
    Ok(())
}
```

### File Transfer Between Two Peers

```rust
use gigi_downloading::TransferManager;
use libp2p::{Multiaddr, PeerId};

// Peer 1 (Sender)
let mut sender = TransferManager::new().await?;
let addr: Multiaddr = "/ip4/127.0.0.1/tcp/8080".parse()?;
sender.start_listening(addr)?;

// Add file to share
let file_data = std::fs::read("large_file.zip")?;
sender.add_file("large_file.zip".to_string(), file_data)?;

// Peer 2 (Receiver)
let mut receiver = TransferManager::new().await?;
let sender_addr: Multiaddr = "/ip4/127.0.0.1/tcp/8080".parse()?;
receiver.connect(sender_addr)?;

// Request file list
let peer_id = sender.local_peer_id();
let _request_id = receiver.request_file_list(peer_id)?;

// Request specific file
let _request_id = receiver.request_file(peer_id, "large_file.zip".to_string())?;
```

## API Reference

### TransferManager

The main struct for managing file transfers.

#### Methods

- `new()` - Create a new transfer manager
- `new_with_events()` - Create manager with event channel
- `local_peer_id()` - Get the local peer ID
- `start_listening(addr)` - Start listening on an address
- `connect(addr)` - Connect to a peer
- `add_file(filename, data)` - Add a file for sharing
- `remove_file(filename)` - Remove a file from sharing
- `get_available_files()` - Get list of available files
- `request_file_list(peer_id)` - Request file list from peer
- `request_file(peer_id, filename)` - Request a file from peer
- `send_file(peer_id, filename, data, chunk_size)` - Send a file to peer
- `handle_swarm_events()` - Process network events

### Events

The library emits various events through the event channel:

- `PeerConnected` - A peer connected
- `TransferStarted` - File transfer started
- `ChunkReceived` - File chunk received
- `TransferCompleted` - File transfer completed
- `TransferFailed` - File transfer failed
- `FileReceived` - Complete file received
- `NetworkEvent` - General network event

## File Transfer Protocol

The library uses a custom protocol built on top of libp2p's request-response:

### Request Types

- `ListFiles` - Request list of available files
- `RequestFile` - Request a specific file
- `RequestChunk` - Request a specific chunk
- `SendFile` - Send a file chunk

### Response Types

- `FileList` - List of available files with metadata
- `FileMetadata` - File metadata (size, hash, etc.)
- `FileChunk` - File chunk data
- `Error` - Error response
- `Success` - Success acknowledgment

## File Integrity

All transferred files are verified using SHA-256 hashes:

1. Sender calculates hash of original file
2. File is split into chunks and transferred
3. Receiver reassembles chunks
4. Receiver calculates hash of reassembled file
5. Transfer is successful only if hashes match

## Examples

See the `examples/` directory for complete working examples:

- `basic_usage.rs` - Basic file transfer example
- `multi_peer.rs` - Multi-peer file sharing

## Dependencies

- `libp2p` - Peer-to-peer networking
- `tokio` - Async runtime
- `serde` - Serialization
- `sha2` - SHA-256 hashing
- `thiserror` - Error handling

## License

This project is licensed under the MIT License.