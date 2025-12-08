# Gigi Downloading

A Rust library for peer-to-peer file transfers using `libp2p`'s `request-response` protocol.

The `gigi-downloading` is designed to be integrated into gigi-dm program for large-file sharing. 

## Features

- **File Transfer**: Transfer files between peers over the network
- **Chunked Transfer**: Large files are automatically split into chunks for efficient transfer
- **File Integrity**: SHA-256 hash verification ensures file integrity
- **Event-Driven**: Async event handling for transfer progress
- **Flexible Swarm Management**: Manual swarm creation for custom configurations
- **Non-blocking Design**: All operations are non-blocking for better async integration
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
use gigi_downloading::{FileTransferServer};
use libp2p::{SwarmBuilder, identity::Keypair};
use std::path::PathBuf;

// Create identity and behaviour
let id_keys = Keypair::generate_ed25519();
let behaviour = FileTransferServer::create_behaviour()?;

let mut swarm = SwarmBuilder::with_existing_identity(id_keys)
    .with_tokio()
    .with_tcp(
        libp2p::tcp::Config::default(),
        libp2p::noise::Config::new,
        libp2p::yamux::Config::default,
    )?
    .with_behaviour(|_| behaviour)?
    .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(300)))
    .build();

let (mut server, mut event_receiver) = FileTransferServer::with_swarm(swarm, PathBuf::from("./shared_files.json"))?;

// Share a file
let file_id = server.share_file(&PathBuf::from("./my_file.txt"))?;

// List shared files
let files = server.list_files();

// Handle events manually
tokio::spawn(async move {
    loop {
        tokio::select! {
            swarm_event = server.swarm.select_next_some() => {
                // Handle swarm events...
                match swarm_event {
                    libp2p::swarm::SwarmEvent::Behaviour(ComposedEvent::RequestResponse(event)) => {
                        server.handle_request_response_event(event)?;
                    }
                    _ => {}
                }
            }
            event = event_receiver.next() => {
                // Handle server events...
            }
        }
    }
});
```

### Client

```rust
use gigi_downloading::{FileTransferClient, ComposedEvent};
use libp2p::{SwarmBuilder, identity::Keypair};
use std::path::PathBuf;

// Create identity and behaviour
let id_keys = Keypair::generate_ed25519();
let behaviour = FileTransferClient::create_behaviour()?;

let mut swarm = SwarmBuilder::with_existing_identity(id_keys)
    .with_tokio()
    .with_tcp(
        libp2p::tcp::Config::default(),
        libp2p::noise::Config::new,
        libp2p::yamux::Config::default,
    )?
    .with_behaviour(|_| behaviour)?
    .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(300)))
    .build();

let (mut client, mut event_receiver) = FileTransferClient::with_swarm(swarm)?;

// Connect to server
client.swarm.dial("/ip4/127.0.0.1/tcp/8080".parse()?)?;

// Get file info
client.get_file_info("file-id").await?;

// Start download (after receiving file info)
// client.start_download(file_info, &PathBuf::from("./downloads")).await?;

// Handle events manually
tokio::spawn(async move {
    loop {
        tokio::select! {
            swarm_event = client.swarm.select_next_some() => {
                match swarm_event {
                    libp2p::swarm::SwarmEvent::Behaviour(ComposedEvent::RequestResponse(event)) => {
                        let events = client.handle_request_response_event(event)?;
                        for event in events {
                            let _ = client.event_sender.unbounded_send(event);
                        }
                    }
                    _ => {}
                }
            }
            event = event_receiver.next() => {
                // Handle client events...
                match event {
                    Some(FileTransferEvent::DownloadCompleted { .. }) => break,
                    _ => {}
                }
            }
        }
    }
});
```

## Design

The library provides:

- **Client/Server Pattern**: Supports multiple clients connecting to a server
- **Manual Swarm Management**: Users create and configure swarms manually for maximum flexibility
- **Non-blocking Architecture**: All operations are non-blocking and event-driven
- **Separation of Concerns**: Network events are separated from application logic
- **Recovery Support**: Both server and client support recovery of transfers
- **File Sharing**: Server shares files and records sharing info for recovery
- **File Management**: Server supports listing shared file info and revoking file sharing
- **Chunked Transfer**: Server responds with requested chunks of data
- **Download Codes**: Client uses sharing codes to retrieve file info
- **Progress Tracking**: Client displays download progress
- **Integrity Verification**: Client verifies file checksum and renames temporary files

## Migration Guide

The API has been refactored to provide more flexibility and better async integration:

### Before (Old API)
```rust
// Blocking initialization
let (mut server, mut event_receiver) = FileTransferServer::new(config).await?;

// Blocking event loop
server.run().await?;
```

### After (New API)
```rust
// Manual swarm creation
let behaviour = FileTransferServer::create_behaviour()?;
let swarm = SwarmBuilder::with_existing_identity(keys)
    .with_behaviour(|_| behaviour)?
    .build();

let (mut server, mut event_receiver) = FileTransferServer::with_swarm(swarm, info_path)?;

// Manual event handling
tokio::select! {
    swarm_event = server.swarm.select_next_some() => { /* handle */ }
    event = event_receiver.next() => { /* handle */ }
}
```

## Dependencies

All dependencies are managed through the workspace to ensure compatibility.

## Protocol

The library uses the `/gigi/file-transfer/1.0.0` protocol for request-response communication over libp2p.