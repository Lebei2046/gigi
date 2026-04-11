# Gigi P2P - TypeScript Port

## Overview

TypeScript implementation of the Gigi P2P networking library, ported from the Rust gigi-p2p crate. Provides a comprehensive P2P networking solution with peer discovery, messaging, and file sharing capabilities.

### Features

- **Auto Discovery**: Automatic peer discovery via mDNS (local network) and Kademlia DHT (WAN)
- **NAT Traversal**: Circuit relay for connecting peers behind routers
- **Direct Messaging**: 1-to-1 peer communication via request-response protocol
- **Group Messaging**: Publish-subscribe model using GossipSub for group chats
- **File Transfer**: Request-response protocol for file sharing with integrity verification
- **Unified Event System**: All P2P activities emitted as typed events

### Protocol Stack

| Protocol      | Purpose               | Type             |
| ------------- | --------------------- | ---------------- |
| Gigi Direct   | 1-to-1 communication  | Request-Response |
| Gigi File     | Chunked file transfer | Request-Response |
| Gigi Group    | Group chat            | GossipSub        |
| mDNS          | Local peer discovery  | Multicast DNS    |
| Kademlia      | WAN peer discovery    | DHT              |
| Circuit Relay | NAT traversal         | Relay            |

## Installation/Test

### Installation

```bash
cd typescript/p2p
pnpm install
```

### Testing

```bash
pnpm test
```

### Development

```bash
pnpm install
pnpm run build
pnpm run dev
```

## License

MIT
