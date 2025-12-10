# Gigi Gossip

A Rust library for group messaging using `libp2p`'s `gossipsub` protocol.

## Features

- Peer discovery using refactored `gigi-mdns` (instead of raw libp2p::mdns)
- Pub/sub messaging with gossipsub
- Support for text and image messages
- Manual swarm management for flexible integration
- Event-driven architecture with tokio
- Consistent API pattern with other gigi packages

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
gigi-gossip = { path = "../gigi-gossip" }
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
libp2p = { version = "0.54", features = ["tcp", "noise", "yamux", "quic", "gossipsub", "mdns"] }
```

## Usage

### Basic Setup

```rust
use gigi_gossip::{GossipChat, GossipEvent, Message};
use libp2p::{identity::Keypair, SwarmBuilder};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create identity
    let keypair = Keypair::generate_ed25519();
    
    // Create behaviour
    let behaviour = GossipChat::create_behaviour(keypair.clone())?;
    
    // Build swarm
    let swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_behaviour(|_| behaviour)?
        .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(300)))
        .build();
    
    // Create gossip chat instance
    let (mut chat, mut event_receiver) = GossipChat::with_swarm(
        swarm,
        "my-nickname".to_string(),
        "my-topic".to_string(),
    )?;
    
    // Listen for incoming connections
    chat.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    
    // Main event loop
    loop {
        tokio::select! {
            // Handle swarm events
            swarm_event = chat.swarm.select_next_some() => {
                // Process swarm events and forward to chat
                match swarm_event {
                    libp2p::swarm::SwarmEvent::Behaviour(event) => {
                        if let Err(e) = chat.handle_event(event) {
                            eprintln!("Error handling event: {}", e);
                        }
                    }
                    _ => {}
                }
            }
            
            // Handle chat events
            Some(chat_event) = event_receiver.next() => {
                match chat_event {
                    GossipEvent::MessageReceived { from, sender, message } => {
                        match message {
                            Message::Text { content, .. } => {
                                println!("Message from {}: {}", sender, content);
                            }
                            Message::Image { data, filename, .. } => {
                                println!("Received image {} from {} ({} bytes)", 
                                    filename, sender, data.len());
                                // Save image...
                            }
                        }
                    }
                    GossipEvent::PeerJoined { peer_id, nickname } => {
                        println!("{} ({}) joined", nickname, peer_id);
                    }
                    GossipEvent::PeerLeft { peer_id, nickname } => {
                        println!("{} ({}) left", nickname, peer_id);
                    }
                    GossipEvent::Error(err) => {
                        eprintln!("Error: {}", err);
                    }
                }
            }
        }
    }
}
```

### Sending Messages

```rust
// Send text message
chat.send_text_message("Hello, world!".to_string())?;

// Send image message
let image_data = std::fs::read("image.jpg")?;
chat.send_image_message(image_data, "image.jpg".to_string())?;
```

## API Reference

### GossipChat

Main struct for managing gossip chat functionality.

#### Methods

- `with_swarm(swarm, nickname, topic)` - Create instance with existing swarm
- `create_behaviour(keypair)` - Create gossipsub+mdns behaviour for swarm
- `send_text_message(content)` - Send a text message
- `send_image_message(data, filename)` - Send an image message
- `handle_event(event)` - Process swarm events
- `get_peers()` - Get list of connected peers

#### Fields

- `swarm: Swarm<GossipBehaviour>` - The libp2p swarm (public for manual event handling)
- `event_sender: mpsc::UnboundedSender<GossipEvent>` - Event channel sender

### Events

#### GossipEvent

- `MessageReceived { from, sender, message }` - Received a message
- `PeerJoined { peer_id, nickname }` - New peer joined
- `PeerLeft { peer_id, nickname }` - Peer left
- `Error(String)` - Error occurred

#### Message Types

- `Text { content, timestamp }` - Text message
- `Image { data, filename, timestamp }` - Image message

## Example

Run the chat example:

```bash
# Terminal 1
cargo run --package gigi-gossip --example chat -- --nickname Alice --topic test-chat

# Terminal 2  
cargo run --package gigi-gossip --example chat -- --nickname Bob --topic test-chat
```

### Testing

Use the provided test script to start both instances:

```bash
# From the gigi-gossip directory
./test_chat.sh
```

Then you can:
1. Type messages in either terminal to send to all peers
2. Type `image <path>` to send image files
3. Type `peers` to see connected peers
4. Type `quit` to exit

The chat supports:
- ✅ Text messaging between peers
- ✅ Image file sharing (max 1MB per image)
- ✅ Peer discovery via mDNS with duplicate filtering
- ✅ Real-time message delivery
- ✅ Automatic peer connection management
- ✅ Error handling for oversized files

## Features

No optional features - all functionality is included by default.

## Dependencies

- `libp2p` - Peer-to-peer networking
- `gigi-mdns` - Refactored local peer discovery with nickname management
- `tokio` - Async runtime
- `serde` - Serialization
- `anyhow` - Error handling
- `uuid` - Message IDs
- `futures` - Async utilities

## License

Licensed under the same terms as the project.