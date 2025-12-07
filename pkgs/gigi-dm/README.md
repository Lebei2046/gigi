# Direct Messaging Library

A peer-to-peer direct messaging library based on Rust libp2p framework.

## Features

- 🔗 **Direct TCP Connections** - Direct dial connections without mDNS
- 💬 **Text Messaging** - Real-time text message transmission
- 🖼️ **Image Transfer** - Support for any image format with automatic MIME type detection
- 🔐 **End-to-End Encryption** - Secure communication using Noise protocol
- 🚀 **Asynchronous Processing** - High-performance async architecture based on Tokio
- 📡 **Multi-Connection Support** - Connect to multiple nodes simultaneously

## Architecture Overview

The library has been refactored to provide a cleaner separation of concerns:

- **Protocol Layer**: Handles all libp2p request-response protocol logic automatically
- **Application Layer**: Your application handles user interaction and display logic
- **Event System**: Structured events provide clear information about messaging operations

This design allows applications to have full control over their event loops while the library handles the complex P2P protocol details.

## Recent Refactoring Improvements

### What Changed

- **Removed `new()` method**: Applications now create their own swarm for maximum flexibility
- **Added `handle_request_response_event()` method**: Centralizes protocol handling in the library
- **Introduced `MessagingEvent` enum**: Provides structured, type-safe event information
- **Moved utility functions**: `send_image_to_all()` is now a method of `DirectMessaging`
- **Eliminated circular dependencies**: Cleaner dependency structure

### Benefits

- **Better Separation of Concerns**: Protocol logic is encapsulated, display logic stays in applications
- **Improved Flexibility**: Applications control their own event loops and swarm configuration
- **Reduced Boilerplate**: Common protocol handling is provided by the library
- **Cleaner API**: Structured events instead of raw protocol events
- **Better Testing**: Protocol logic can be tested independently

### Migration Guide

If you were using the old API with `DirectMessaging::new()`:

```rust
// Old approach (no longer supported)
let (mut messaging, mut event_receiver) = DirectMessaging::new().await?;

// New approach (recommended)
let id_keys = libp2p::identity::Keypair::generate_ed25519();
let behaviour = DirectMessaging::create_behaviour(libp2p::request_response::Config::default())?;
let swarm = libp2p::SwarmBuilder::with_existing_identity(id_keys)
    .with_tokio()
    .with_tcp(libp2p::tcp::Config::default(), libp2p::noise::Config::new, libp2p::yamux::Config::default)?
    .with_behaviour(|_| behaviour)?
    .build();
let mut messaging = DirectMessaging::with_swarm(swarm)?;
```

## Quick Start

### Installation

```toml
[dependencies]
gigi-dm = { path = "pkgs/gigi-dm" }
libp2p = { version = "0.56", features = ["tcp", "noise", "yamux", "request-response", "json"] }
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1.0"
```

### Basic Usage

```rust
use gigi_dm::{DirectMessaging, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create identity and behavior
    let id_keys = libp2p::identity::Keypair::generate_ed25519();
    let local_peer_id = libp2p::PeerId::from(id_keys.public());
    
    // Create messaging behavior
    let behaviour = DirectMessaging::create_behaviour(
        libp2p::request_response::Config::default()
            .with_request_timeout(std::time::Duration::from_secs(30))
    )?;
    
    // Build swarm
    let swarm = libp2p::SwarmBuilder::with_existing_identity(id_keys)
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_behaviour(|_| behaviour)?
        .with_swarm_config(|c| {
            c.with_idle_connection_timeout(std::time::Duration::from_secs(120))
                .with_max_negotiating_inbound_streams(100)
        })
        .build();
    
    // Create messaging instance
    let mut messaging = DirectMessaging::with_swarm(swarm)?;
    
    // Start listening for connections
    let listen_addr = messaging.start_listening(0)?;
    println!("Listening on: {}", listen_addr);
    
    // Connect to other nodes
    let addr: libp2p::Multiaddr = "/ip4/127.0.0.1/tcp/8080".parse()?;
    messaging.dial_peer(&addr)?;
    
    // Main event loop
    loop {
        tokio::select! {
            // Handle swarm events
            event = messaging.swarm.select_next_some() => {
                match event {
                    libp2p::swarm::SwarmEvent::Behaviour(req_resp_event) => {
                        if let Ok(Some(messaging_event)) = messaging.handle_request_response_event(req_resp_event).await {
                            handle_messaging_event(messaging_event).await;
                        }
                    }
                    libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("Connected to: {}", peer_id);
                    }
                    libp2p::swarm::SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        println!("Disconnected from: {}", peer_id);
                    }
                    _ => {}
                }
            }
            // Handle other logic...
        }
    }
}

async fn handle_messaging_event(event: gigi_dm::MessagingEvent) {
    match event {
        gigi_dm::MessagingEvent::MessageReceived { from, message } => {
            match message {
                gigi_dm::Message::Text(text) => {
                    println!("Received text from {}: {}", from, text);
                }
                gigi_dm::Message::Image { name, mime_type, data } => {
                    println!("Received image from {}: {} ({} bytes)", from, name, data.len());
                }
            }
        }
        gigi_dm::MessagingEvent::PeerError { error, .. } => {
            println!("Peer error: {}", error);
        }
        gigi_dm::MessagingEvent::OutboundFailure { peer, error } => {
            println!("Outbound failure to {}: {}", peer, error);
        }
        _ => {}
    }
}
```

## Chat Example

### Running the Example

```bash
# Start first node (listening mode)
cargo run --example chat -- --port 8080

# Start second node (connection mode)
cargo run --example chat -- --addr /ip4/127.0.0.1/tcp/8080
```

### Interactive Commands

- `Type text directly` - Send text messages
- `/text <message>` - Send text messages
- `/image <path>` - Send image files
- `/connect <multiaddr>` - Connect to nodes
- `/peers` - View connection status
- `/help` - Help information

### Example Conversation

```
Local peer ID: 12D3KooWQVtBYE7zasPLcpkTzs55uo7kDmq3c7EdrH48VxKy2JJG
Listening on: /ip4/0.0.0.0/tcp/8080

✓ Connected to: 12D3KooWJE9WyaRhqyWoDXnwehsgmvULRicLb8kkaxR4EhFKJviT
> hello world
[12D3KooWJE9WyaRhqyWoDXnwehsgmvULRicLb8kkaxR4EhFKJviT] Hello there!
> /image ~/screenshot.png
Image 'screenshot.png' sent to 1 peers
[12D3KooWJE9WyaRhqyWoDXnwehsgmvULRicLb8kkaxR4EhFKJviT] Image: screenshot.png (1024567 bytes, image/png)
> /peers
Connected peers (1):
  12D3KooWJE9WyaRhqyWoDXnwehsgmvULRicLb8kkaxR4EhFKJviT
```

## API Reference

### DirectMessaging

The main messaging structure.

#### Methods

- `with_swarm(swarm)` - Create a messaging instance with an existing swarm
- `create_behaviour(config)` - Create a request-response behavior for external swarm creation
- `start_listening(port)` - Start listening on the specified port
- `dial_peer(addr)` - Connect to a node at the specified address
- `send_message(peer_id, message)` - Send message to the specified node
- `send_image_to_all(peers, image_path)` - Send an image file to all connected peers
- `get_connected_peers()` - Get all connected nodes
- `get_listening_address()` - Get all listening addresses
- `local_peer_id()` - Get local node ID
- `handle_request_response_event(event)` - Handle request-response events and return structured event information

### Message

Message type enumeration.

```rust
pub enum Message {
    Text(String),
    Image {
        name: String,
        mime_type: String,
        data: Vec<u8>,
    },
}
```

#### Constructor Methods

- `Message::text(content)` - Create text message
- `Message::image(name, mime_type, data)` - Create image message

### MessagingEvent

Event type for receiving messaging events after handling request-response protocol events.

```rust
pub enum MessagingEvent {
    MessageReceived { peer: PeerId, message: Message },
    MessageAcknowledged { peer: PeerId },
    PeerError { peer: PeerId, error: String },
    OutboundFailure { peer: PeerId, error: String },
    InboundFailure { error: String },
    ResponseSent,
}
```

## Network Protocols

### Transport Layer

- **TCP**: Reliable transport protocol
- **Noise**: Encrypted handshake protocol
- **Yamux**: Connection multiplexing

### Application Layer

- **Request-Response**: Request-response pattern
- **JSON**: Message serialization format
- **Protocol ID**: `/messaging/1.0.0`

## Security Features

- **End-to-End Encryption**: Using Noise protocol
- **Authentication**: Ed25519 key pairs
- **Timeout Protection**: 30-second request timeout
- **Connection Management**: Automatic cleanup of disconnected connections

## Performance Optimization

- **Asynchronous I/O**: Tokio-based event loop
- **Stream Processing**: Support for large numbers of concurrent connections
- **Memory Optimization**: Efficient message buffering
- **Zero-Copy**: Reduce unnecessary data copying

## Troubleshooting

### Common Issues

**Q: What to do about connection timeout?**
A: Check if the target address is correct, ensure the target node is listening and network reachable.

**Q: Image sending failed?**
A: Ensure the image file exists and format is supported, check file permissions.

**Q: Messages not received?**
A: Check connection status, use `/peers` command to confirm nodes are connected.

### Debugging

Enable verbose logging:

```bash
RUST_LOG=debug cargo run --example chat
```

## License

MIT License - see LICENSE file in the project root directory.