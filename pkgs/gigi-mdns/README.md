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
use gigi_mdns::{Nickname, NicknameManager, NicknameBehaviourEvent};
use futures::StreamExt;
use libp2p::Transport;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create identity and behaviour
    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let peer_id = libp2p::PeerId::from(keypair.public());
    
    // Create behaviour with custom config
    let mdns_config = libp2p::mdns::Config {
        ttl: Duration::from_secs(60),
        query_interval: Duration::from_secs(10),
        ..libp2p::mdns::Config::default()
    };
    
    let behaviour = NicknameManager::create_behaviour(
        peer_id,
        mdns_config,
        libp2p::request_response::Config::default(),
    )?;
    
    // Create swarm
    let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_other_transport(|_keypair| {
            libp2p::tcp::tokio::Transport::default()
                .upgrade(libp2p::core::upgrade::Version::V1)
                .authenticate(
                    libp2p::noise::Config::new(&_keypair)
                        .expect("Signing libp2p-noise static DH keypair failed."),
                )
                .multiplex(libp2p::yamux::Config::default())
                .boxed()
        })
        .expect("Failed to create transport")
        .with_behaviour(|_keypair| behaviour)
        .expect("Failed to create behaviour")
        .with_swarm_config(|c| {
            c.with_idle_connection_timeout(Duration::from_secs(60))
        })
        .build();
    
    // Create nickname manager
    let mut manager = NicknameManager::with_swarm(swarm)?;
    
    // Set nickname
    let my_nickname = Nickname::new("awesome-device-123".to_string())?;
    manager.set_nickname(my_nickname);
    
    // Start listening
    let listen_addr: libp2p::Multiaddr = "/ip4/0.0.0.0/tcp/0".parse().unwrap();
    manager.start_listening(listen_addr)?;
    
    // Handle events
    loop {
        tokio::select! {
            event = manager.swarm.select_next_some() => {
                handle_swarm_event(&mut manager, event);
            }
        }
    }
}

fn handle_swarm_event(
    manager: &mut NicknameManager,
    event: libp2p::swarm::SwarmEvent<NicknameBehaviourEvent>,
) {
    match event {
        libp2p::swarm::SwarmEvent::Behaviour(NicknameBehaviourEvent::Mdns(mdns_event)) => {
            match mdns_event {
                libp2p::mdns::Event::Discovered(list) => {
                    for (peer_id, addr) in list {
                        let _ = manager.handle_mdns_discovered(peer_id, addr);
                        let _ = manager.request_nickname(peer_id);
                    }
                }
                libp2p::mdns::Event::Expired(list) => {
                    for (peer_id, _) in list {
                        if let Some(event) = manager.handle_mdns_expired(peer_id) {
                            // Handle peer expired event
                        }
                    }
                }
            }
        }
        libp2p::swarm::SwarmEvent::Behaviour(NicknameBehaviourEvent::RequestResponse(req_resp_event)) => {
            if let Ok(Some(event)) = manager.handle_request_response_event(req_resp_event) {
                // Handle request-response event
            }
        }
        _ => {}
    }
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
- `with_swarm()`: Create manager with existing swarm
- `create_behaviour()`: Create network behaviour for swarm construction
- `set_nickname()`: Set local nickname
- `start_listening()`: Start listening on network
- `handle_mdns_discovered()`: Handle mDNS peer discovery
- `handle_mdns_expired()`: Handle mDNS peer expiration
- `handle_request_response_event()`: Handle request-response events
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

## Migration Guide (v0.0.1)

The API has been refactored to provide more flexibility and better separation of concerns:

### Before
```rust
// Old API - created swarm internally
let (mut manager, mut event_rx) = NicknameManager::new_with_events().await?;
let _ = manager.run().await; // Problematic blocking method
```

### After
```rust
// New API - manual swarm creation and event handling
let keypair = libp2p::identity::Keypair::generate_ed25519();
let behaviour = NicknameManager::create_behaviour(peer_id, mdns_config, req_config)?;
let swarm = SwarmBuilder::with_existing_identity(keypair)
    .with_tokio()
    .with_other_transport(|keypair| { /* transport setup */ })
    .with_behaviour(|_| behaviour)
    .build();
let mut manager = NicknameManager::with_swarm(swarm)?;

// Manual event handling in your own loop
loop {
    tokio::select! {
        event = manager.swarm.select_next_some() => {
            handle_swarm_event(&mut manager, event);
        }
    }
}
```

### Benefits of New API
- **Better Control**: Full control over swarm construction and configuration
- **Flexibility**: Easier to test and use in different contexts
- **Separation of Concerns**: Protocol handling in library, application logic in your code
- **Cleaner Architecture**: No blocking methods that monopolize the instance

## Notes

- Requires Tokio async runtime
- mDNS only works on local networks
- It's recommended to add appropriate timeouts and retry mechanisms in production environments
- The `swarm` field is now public for direct access to libp2p functionality