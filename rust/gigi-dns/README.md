# Gigi DNS

## Overview

Gigi DNS is a custom mDNS protocol that automatically discovers peers with nicknames, capabilities, and metadata in local area networks. It provides a libp2p-compatible NetworkBehaviour for peer discovery with human-readable nicknames.

### Features

- Auto-discovery peers with nickname + peer_id + multiaddr in ONE protocol
- Rich metadata including capabilities and key-value pairs
- Human-readable nicknames instead of cryptic peer IDs
- Simple API - easy to integrate
- libp2p compatible - implements NetworkBehaviour trait
- Zero configuration - works out of box
- Low bandwidth - efficient multicast-based discovery

## Installation/Test

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
gigi-dns = {
  path = "../gigi-dns",
  version = "0.1.0"
}
```

### Testing

Run the tests to verify functionality:

```bash
cargo test
```

## License

MIT OR Apache-2.0
