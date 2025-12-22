# Gigi P2P

A comprehensive peer-to-peer networking library built on libp2p, supporting direct messaging, group messaging, file sharing, and image transfers.

## Features

- 🌐 **Peer Discovery**: Automatic peer discovery via mDNS
- 💬 **Direct Messaging**: Send messages and images directly to peers
- 📢 **Group Messaging**: Join groups and broadcast messages
- 📁 **File Sharing**: Share files with unique share codes
- 🖼️ **Image Transfer**: Send and receive images directly or in groups
- 🔍 **Nicknames**: Human-readable peer identification
- ⬇️ **Chunked Downloads**: Large files downloaded with real-time progress tracking
- ✅ **Integrity Verification**: SHA256 hash verification for all file transfers
- 📊 **Progress Events**: Detailed download progress and completion events

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
            P2pEvent::GroupMessage { from_nickname, group, message } => {
                println!("[{}] {}: {}", group, from_nickname, message);
            }
            P2pEvent::GroupImageMessage { from_nickname, group, filename, .. } => {
                println!("[{}] {}: Shared image: {}", group, from_nickname, filename);
            }
            P2pEvent::FileDownloadCompleted { file_id, path } => {
                println!("Download complete: {} -> {}", file_id, path.display());
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
- `help` or `?` - Show available commands
- `peers` - List connected peers
- `send <nickname> <message>` - Send direct message
- `send-image <nickname> <path>` - Send image
- `join <group>` - Join a group
- `leave <group>` - Leave a group
- `send-group <group> <message>` - Send group message
- `send-group-image <group> <path>` - Send image to group
- `share <file-path>` - Share a file (generates unique share code)
- `unshare <share-code>` - Remove shared file record
- `files` - List shared files
- `download <nickname> <share-code>` - Download a shared file with progress tracking
- `quit` or `exit` - Exit the chat

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
- `send_group_image(group_name, image_path)` - Send image to group (async)
- `share_file(file_path)` - Share a file (returns share code)
- `unshare_file(share_code)` - Remove shared file record
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
- `FileShared` - File successfully shared
- `FileRevoked` - File share revoked
- `FileInfoReceived` - File information received
- `ChunkReceived` - File chunk received
- `FileDownloadStarted` - Download started (with filename and peer info)
- `FileDownloadProgress` - Download progress update (percentage and chunk counts)
- `FileDownloadCompleted` - Download completed (with final file path)
- `FileDownloadFailed` - Download failed (with detailed error message)
- `ListeningOn` - Client started listening
- `Connected` - Connected to peer
- `Disconnected` - Disconnected from peer
- `Error` - Error occurred

## Architecture

The library uses a unified `NetworkBehaviour` that combines:

1. **mDNS** - For local peer discovery
2. **Request-Response** - For direct messaging and file transfers
3. **GossipSub** - For group messaging

### Protocols:
- `/nickname/1.0.0` - Nickname exchange
- `/direct/1.0.0` - Direct messaging
- `/file/1.0.0` - File transfer operations
- GossipSub topics for group messaging

All P2P functionality is integrated into a single behavior, avoiding the complexity of multiple modular packages.

## File Sharing

Files are shared using a unique share code system:

1. Share a file: `client.share_file("path/to/file.txt")` → returns share code
2. Download a file: `client.download_file("peer-nickname", "share-code")`
3. Unshare a file: `client.unshare_file("share-code")` → removes file record

### Features:
- **Chunked transfers** - Large files transferred in 256KB chunks with sliding window
- **Real-time progress** - Detailed download progress events with percentage completion
- **Hash verification** - SHA256 integrity checking for both chunks and complete files
- **Concurrent downloads** - Up to 5 concurrent chunk requests for optimal performance
- **Persistent storage** - Shared files saved to `shared.json`
- **Automatic cleanup** - Invalid files removed from registry
- **Duplicate detection** - Same files share existing codes
- **Error handling** - Comprehensive error reporting for download failures

### Download Process:
1. **Initiation** - `FileDownloadStarted` event sent immediately when download begins
2. **Chunk Requests** - Initial 5 chunks requested automatically with sliding window
3. **Progress Tracking** - `FileDownloadProgress` events sent as chunks arrive
4. **Verification** - File hash validated against expected SHA256
5. **Completion** - `FileDownloadCompleted` event with final file path

Files are automatically saved to the configured download directory and verified for integrity.

## Image Support

The library supports seamless image sharing:
- **Direct images** - Send/receive images directly to peers
- **Group images** - Share images in group chats via download links
- **Automatic conversion** - Images converted to shareable formats
- **Size limits** - Group images use file sharing to avoid message size limits

Supported formats: PNG, JPEG, GIF, WebP, BMP, ICO, TIFF

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
- `blake3` - Fast hashing
- `sha2` - SHA256 hashing
- `chrono` - Date/time handling
- `anyhow` - Error handling
- `thiserror` - Error types
- `clap` - Command line parsing (examples)

## Recent Improvements

### File Download Enhancements
- ✅ **Fixed Download Initiation** - Downloads now start immediately when requested
- ✅ **Added Progress Events** - Real-time download progress with percentage indicators
- ✅ **Improved Error Handling** - Clear error messages for failed downloads
- ✅ **Concurrent Chunk Requests** - Optimized performance with sliding window approach
- ✅ **Hash Verification** - Both chunk-level and file-level integrity checking

### Chat Application
- ✅ **Enhanced Command Interface** - Better user feedback and command handling
- ✅ **Real-time Progress** - Visual progress indicators during file transfers
- ✅ **Improved Error Messages** - Clear feedback for all operations

The download functionality now provides immediate feedback and detailed progress tracking, making file transfers more reliable and user-friendly.

## License

This project is part of the Gigi P2P networking suite.