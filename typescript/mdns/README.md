# Gigi DNS TypeScript Client

## Overview

Gigi DNS is the decentralized DNS service for the Gigi P2P network, allowing peers to resolve names to peer IDs. It provides a human-friendly way to address peers without needing to remember long peer IDs, using mDNS for local network discovery.

### Features

- **Local Peer Discovery**: Uses mDNS for peer discovery on the local network
- **Nickname Support**: Allows peers to advertise human-readable nicknames
- **Metadata Support**: Enables peers to advertise additional metadata and capabilities
- **Secure**: Includes packet validation and source verification
- **Extensible**: Easy to integrate with libp2p applications

## Installation/Test

### Installation

```bash
pnpm add @gigi/mdns
```

### Testing

```bash
pnpm test
```

## License

Apache 2.0
