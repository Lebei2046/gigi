# Direct Messaging Library

A peer-to-peer direct messaging library based on Rust libp2p framework.

## Features

- 🔗 **Direct TCP Connections** - Direct dial connections without mDNS
- 💬 **Text Messaging** - Real-time text message transmission
- 🖼️ **Image Transfer** - Support for any image format with automatic MIME type detection
- 🔐 **End-to-End Encryption** - Secure communication using Noise protocol
- 🚀 **Asynchronous Processing** - High-performance async architecture based on Tokio
- 📡 **Multi-Connection Support** - Connect to multiple nodes simultaneously

## Quick Start

### Installation

```toml
[dependencies]
gigi-dm = { path = "pkgs/gigi-dm" }
libp2p = { version = "0.56", features = ["json"] }
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage

```rust
use gigi_dm::{DirectMessaging, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create messaging instance
    let (mut messaging, mut event_receiver) = DirectMessaging::new().await?;
    
    // Start listening for connections
    let listen_addr = messaging.start_listening(0)?;
    println!("Listening on: {}", listen_addr);
    
    // Connect to other nodes
    let addr: libp2p::Multiaddr = "/ip4/127.0.0.1/tcp/8080".parse()?;
    messaging.dial_peer(&addr)?;
    
    // Send messages
    loop {
        tokio::select! {
            event = event_receiver.recv() => {
                if let Some(event) = event {
                    handle_event(event).await;
                }
            }
            // Handle other logic...
        }
    }
}

async fn handle_event(event: gigi_dm::CustomMessagingEvent) {
    match event {
        gigi_dm::CustomMessagingEvent::MessageReceived { from, message } => {
            match message {
                gigi_dm::Message::Text(text) => {
                    println!("Received text from {}: {}", from, text);
                }
                gigi_dm::Message::Image { name, mime_type, data } => {
                    println!("Received image from {}: {} ({} bytes)", from, name, data.len());
                }
            }
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

- `new()` - Create a new messaging instance
- `start_listening(port)` - Start listening on the specified port
- `dial_peer(addr)` - Connect to a node at the specified address
- `send_message(peer_id, message)` - Send message to the specified node
- `get_connected_peers()` - Get all connected nodes
- `local_peer_id()` - Get local node ID

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

### CustomMessagingEvent

Custom event type for receiving network events.

```rust
pub enum CustomMessagingEvent {
    Connected(PeerId),
    Disconnected(PeerId),
    MessageReceived { from: PeerId, message: Message },
    MessageSent { to: PeerId, message: Message },
    Error(String),
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