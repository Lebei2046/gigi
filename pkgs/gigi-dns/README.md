# Gigi DNS

Gigi DNS is a custom mDNS protocol that automatically discovers peers with nicknames, capabilities, and metadata in local area networks.

## Features

- Auto-discovery peers with nickname + peer_id + multiaddr in ONE protocol
- Rich metadata including capabilities and key-value pairs
- Human-readable nicknames instead of cryptic peer IDs
- Simple API - easy to integrate
- libp2p compatible - implements NetworkBehaviour trait
- Zero configuration - works out of box
- Low bandwidth - efficient multicast-based discovery

## Quick Start

```rust
use gigi_dns::{GigiDnsConfig, GigiDnsBehaviour};

let config = GigiDnsConfig {
    nickname: "Alice".to_string(),
    capabilities: vec!["chat".to_string()],
    ..Default::default()
};

let mdns = GigiDnsBehaviour::new(local_peer_id, config)?;
```

## License

MIT OR Apache-2.0
