# Gigi Downloading

A Rust library for peer-to-peer file transfers using `libp2p`'s `request-response` protocol.

The `gigi-downloading` is designed to be integrated into gigi-dm program for large-file sharing. 

## Features

- **File Transfer**: Transfer files between peers over the network
- **Chunked Transfer**: Large files are automatically split into chunks for efficient transfer
- **File Integrity**: SHA-256 hash verification ensures file integrity
- **Event-Driven**: Async event handling for transfer progress
- **Discovery**: Built-in file listing and metadata exchange
- **Secure**: Uses libp2p's noise protocol for encrypted communication

## Usage

### Server Example

```bash
# Start a server sharing files
cargo run --example server -- \
  --info-path ./shared_files.json \
  --files file1.txt file2.txt \
  --port 8080
```

### Client Example

```bash
# Download a file from server
cargo run --example client -- \
  --addr /ip4/127.0.0.1/tcp/8080 \
  --code <file-id-from-server> \
  --output ./downloads
```

## API

### Server

```rust
use gigi_downloading::{FileTransferServer, ServerConfig};
use std::path::PathBuf;

let config = ServerConfig {
    info_path: PathBuf::from("./shared_files.json"),
    listen_port: 8080,
};

let (mut server, mut event_receiver) = FileTransferServer::new(config).await?;

// Share a file
let file_id = server.share_file(&PathBuf::from("./my_file.txt"))?;

// List shared files
let files = server.list_files();

// Run the server
tokio::spawn(async move {
    server.run().await
});
```

### Client

```rust
use gigi_downloading::{FileTransferClient, ClientConfig};
use libp2p::Multiaddr;

let server_addr: Multiaddr = "/ip4/127.0.0.1/tcp/8080".parse()?;
let config = ClientConfig { server_addr };

let (mut client, mut event_receiver) = FileTransferClient::new(config).await?;

// Get file info
client.get_file_info("file-id").await?;

// Start download (after receiving file info)
// client.start_download(file_info, &PathBuf::from("./downloads")).await?;

// Run the client
tokio::spawn(async move {
    client.run().await
});
```

## Design

The library provides:

- **Client/Server Pattern**: Supports multiple clients connecting to a server
- **Recovery Support**: Both server and client support recovery of transfers
- **File Sharing**: Server shares files and records sharing info for recovery
- **File Management**: Server supports listing shared file info and revoking file sharing
- **Chunked Transfer**: Server responds with requested chunks of data
- **Download Codes**: Client uses sharing codes to retrieve file info
- **Progress Tracking**: Client displays download progress
- **Integrity Verification**: Client verifies file checksum and renames temporary files

## Dependencies

All dependencies are managed through the workspace to ensure compatibility.

## Protocol

The library uses the `/gigi/file-transfer/1.0.0` protocol for request-response communication over libp2p.