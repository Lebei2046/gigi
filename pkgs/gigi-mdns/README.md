# mDNS Nickname Library

A Rust library based on libp2p that implements a combination of mDNS + request-response protocols for seamless peer discovery and nickname management.

## Features

- **Real libp2p Integration**: Uses libp2p's mDNS and request-response protocols
- **Nickname Management**: Validation, storage, and lookup of peer nicknames
- **Auto Discovery**: Automatically discover peers on the network via mDNS
- **Protocol Communication**: Use request-response protocol for peer-to-peer communication
- **Event-Driven**: Asynchronous event handling mechanism
- **Type Safe**: Complete Rust type system and error handling

## Core Components

### Nickname
- Representation and validation of device nicknames
- Supports alphanumeric characters, hyphens, and underscores
- Maximum length 63 characters

### NicknameManager
- Main manager integrating mDNS and request-response
- Handles peer discovery and nickname exchange
- Provides event-driven API

### Protocol Messages
- `NicknameRequest`: GetNickname, GetDiscoveredPeers, AnnounceNickname
- `NicknameResponse`: Nickname, DiscoveredPeers, Error, Ack

## Usage Examples

### Basic Code Example

```rust
use mdns_nickname::{Nickname, NicknameManager, NicknameEvent};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create nickname manager
    let (mut manager, mut event_rx) = NicknameManager::new_with_events().await?;
    
    // Set nickname
    let my_nickname = Nickname::new("awesome-device-123".to_string())?;
    manager.set_nickname(my_nickname);
    
    // Start listening
    let listen_addr: libp2p::Multiaddr = "/ip4/0.0.0.0/tcp/0".parse().unwrap();
    manager.start_listening(listen_addr)?;
    
    // Handle events
    while let Some(event) = event_rx.recv().await {
        match event {
            NicknameEvent::PeerDiscovered { peer_id, nickname } => {
                println!("Peer discovered: {} (nickname: {:?})", peer_id, nickname);
            }
            NicknameEvent::NicknameUpdated { peer_id, nickname } => {
                println!("Nickname updated: {} -> {}", peer_id, nickname);
            }
            // ... other events
        }
    }
    
    Ok(())
}
```

### Command Line Example

Run the example program and specify a nickname:
```bash
# Start first instance
cargo run --example basic_usage -- --nickname device-1

# Start second instance (in another terminal)
cargo run --example basic_usage -- --nickname device-2
```

If no nickname is specified, a random nickname will be generated automatically:
```bash
cargo run --example basic_usage
```

## API Overview

### Main Methods
- `new()` / `new_with_events()`: Create manager
- `set_nickname()`: Set local nickname
- `start_listening()`: Start listening on network
- `request_nickname()`: Request peer's nickname
- `announce_nickname()`: Broadcast nickname to all discovered peers
- `get_discovered_peers()`: Get discovered peers
- `get_peer_by_nickname()`: Find peer by nickname

### Event Types
- `PeerDiscovered`: New peer discovered
- `PeerExpired`: Peer expired
- `NicknameUpdated`: Nickname updated
- `RequestReceived`: Request received
- `NetworkEvent`: Network event
- `ListeningOn`: Started listening

## Technical Implementation

### libp2p Protocol Combination
- **mDNS**: For local network peer discovery
- **Request-Response**: For reliable peer-to-peer communication
- **TCP + Noise + Yamux**: Transport layer secure connections

### Custom Protocol
- Protocol name: `/nickname/1.0.0`
- JSON serialized message format
- Asynchronous request-response pattern

## Testing

Run tests:
```bash
cargo test
```

Run examples:
```bash
cargo run --example basic_usage
```

## Dependencies

- `libp2p`: 0.56.0 (mdns, request-response, tokio, noise, yamux, tcp)
- `tokio`: Async runtime
- `serde`: Serialization/deserialization
- `thiserror`: Error handling
- `futures`: Async utilities
- `async-trait`: Async trait support
- `rand`: Random nickname generation

## Error Handling

The library provides a complete error type `NicknameError`, including:
- Nickname format errors
- Network errors
- Timeout errors
- Connection failures

## Notes

- Requires Tokio async runtime
- mDNS only works on local networks
- It's recommended to add appropriate timeouts and retry mechanisms in production environments