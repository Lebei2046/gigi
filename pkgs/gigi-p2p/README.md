# Gigi P2P

A comprehensive peer-to-peer networking library built on libp2p, supporting direct messaging, group messaging, file sharing, and image transfers.

## Features

- 🌐 **Peer Discovery**: Automatic peer discovery via mDNS
- 💬 **Direct Messaging**: Send messages and images directly to peers
- 📢 **Group Messaging**: Join groups and broadcast messages
- 📁 **File Sharing**: Share files with unique share codes
- 🖼️ **Image Transfer**: Send and receive images directly or in groups
- 🔍 **Nicknames**: Human-readable peer identification
- ⬇️ **Chunked Downloads**: Large files downloaded with progress tracking

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
gigi-p2p = { path = "pkgs/gigi-p2p" }
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Basic Usage

```rust
use gigi_p2p::P2pClient;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a P2P client
    let (mut client, mut event_receiver) = P2pClient::new(
        libp2p::identity::Keypair::generate_ed25519(),
        "my-nickname".to_string(),
        PathBuf::from("./downloads"),
    )?;

    // Start listening for connections
    client.start_listening("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Handle events
    while let Some(event) = event_receiver.recv().await {
        match event {
            P2pEvent::PeerDiscovered { peer_id, nickname, address } => {
                println!("{} ({}) joined from {}", nickname, peer_id, address);
            }
            P2pEvent::DirectMessage { from_nickname, message, .. } => {
                println!("{} says: {}", from_nickname, message);
            }
            // Handle other events...
            _ => {}
        }
    }

    Ok(())
}
```

## Examples

### Chat Application

Run the included chat example:

```bash
# Terminal 1
cargo run --package gigi-p2p --example chat -- --nickname Alice

# Terminal 2  
cargo run --package gigi-p2p --example chat -- --nickname Bob
```

Chat commands:
- `/peers` - List connected peers
- `/send <nickname> <message>` - Send direct message
- `/send-image <nickname> <path>` - Send image
- `/join <group>` - Join a group
- `/send-group <group> <message>` - Send group message
- `/share <file-path>` - Share a file
- `/download <nickname> <share-code>` - Download a file

## API Reference

### P2pClient

The main client for P2P networking.

#### Methods

- `new(keypair, nickname, download_dir)` - Create a new client
- `start_listening(address)` - Start listening on an address
- `send_direct_message(nickname, message)` - Send a direct message
- `send_direct_image(nickname, image_path)` - Send an image
- `join_group(group_name)` - Join a group
- `leave_group(group_name)` - Leave a group
- `send_group_message(group_name, message)` - Send group message
- `share_file(file_path)` - Share a file (returns share code)
- `download_file(nickname, share_code)` - Download a shared file
- `list_peers()` - Get list of discovered peers
- `list_shared_files()` - Get list of shared files

### Events

All P2P events are emitted through the event receiver:

- `PeerDiscovered` - New peer discovered
- `PeerExpired` - Peer disconnected
- `DirectMessage` - Received direct message
- `DirectImageMessage` - Received direct image
- `GroupMessage` - Received group message
- `GroupImageMessage` - Received group image
- `FileShareRequest` - File share offer received
- `FileDownloadStarted` - Download started
- `FileDownloadProgress` - Download progress update
- `FileDownloadCompleted` - Download completed
- `ListeningOn` - Client started listening
- `Connected` - Connected to peer
- `Disconnected` - Disconnected from peer
- `Error` - Error occurred

## Architecture

The library uses a unified `NetworkBehaviour` that combines:

1. **mDNS** - For local peer discovery
2. **Request-Response** - For direct messaging and file transfers
3. **GossipSub** - For group messaging

All P2P functionality is integrated into a single behavior, avoiding the complexity of multiple modular packages.

## File Sharing

Files are shared using a unique share code system:

1. Share a file: `client.share_file("path/to/file.txt")` → returns share code
2. Download a file: `client.download_file("peer-nickname", "share-code")`

Files are transferred in chunks with progress tracking and automatically saved to the configured download directory.

## Image Support

The library supports common image formats (PNG, JPEG, GIF, WebP) and includes metadata for size and dimensions.

## Testing

Run the test suite:

```bash
cargo test --package gigi-p2p
```

## Dependencies

- `libp2p` - Core P2P networking
- `tokio` - Async runtime
- `serde` - Serialization
- `tracing` - Logging
- `uuid` - Unique identifiers
- `image` - Image processing

## License

This project is part of the Gigi P2P networking suite.