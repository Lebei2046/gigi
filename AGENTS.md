# Gigi P2P Ecosystem Guidelines

This document serves as a comprehensive guideline for the Gigi P2P ecosystem, describing its components, architecture, and usage. It provides an overview of the entire ecosystem and references detailed documentation for each component.

## 1. Gigi P2P Ecosystem Overview

The Gigi P2P ecosystem is a decentralized network of components designed to enable secure, direct communication between peers without relying on centralized servers. It is built on top of Libp2p, a modular network stack for peer-to-peer applications.

### Core Components

| Component | Description | Documentation |
|-----------|-------------|---------------|
| Gigi P2P Rust Client | Core P2P functionality with high performance | [docs/gigi-p2p.md](docs/gigi-p2p.md) |
| Gigi P2P TypeScript Client | High-level API for TypeScript applications | [docs/gigi-p2p-ts.md](docs/gigi-p2p-ts.md) |
| Gigi OpenClaw Plugin | Integration with OpenClaw chat application | [docs/gigi-openclaw.md](docs/gigi-openclaw.md) |
| Gigi Network Node | Standalone network node for bootstrap/relay | [docs/gigi-node.md](docs/gigi-node.md) |
| Gigi Auth | Authentication and key management | [docs/gigi-auth.md](docs/gigi-auth.md) |
| Gigi DNS | Decentralized name resolution | [docs/gigi-dns.md](docs/gigi-dns.md) |
| Gigi File Sharing | File sharing utilities | [docs/gigi-file-sharing.md](docs/gigi-file-sharing.md) |
| Gigi Store | Persistence layer for data storage | [docs/gigi-store.md](docs/gigi-store.md) |
| Tauri Plugin Gigi | Integration with Tauri desktop/mobile apps | [docs/tauri-plugin-gigi.md](docs/tauri-plugin-gigi.md) |

### Architecture

The Gigi P2P ecosystem follows a layered architecture:

1. **Network Layer**: Provides the foundation for peer-to-peer communication
2. **Protocol Layer**: Defines the rules for communication between peers
3. **Application Layer**: Provides higher-level functionality for end users

For detailed information about the architecture, see [docs/architecture.md](docs/architecture.md).

## 2. Gigi OpenClaw Plugin (`pkgs/gigi-openclaw`)

### Description
The Gigi OpenClaw Plugin integrates the Gigi P2P network with OpenClaw, enabling P2P messaging and file sharing through the OpenClaw interface.

### Key Features
- **P2P Messaging**: Direct peer-to-peer messaging
- **Group Messaging**: Messages to groups using GossipSub
- **File Sharing**: Share files between peers with share codes
- **Peer Discovery**: Find peers using Kademlia DHT and mDNS
- **NAT Traversal**: Connect peers behind NAT using circuit relay
- **Status Monitoring**: Health checks and connection status

### Configuration
```json
{
  "channels": {
    "gigi": {
      "peerId": "your-peer-id",
      "multiaddrs": ["/ip4/0.0.0.0/tcp/0", "/ip4/0.0.0.0/ws/0"],
      "displayName": "Your Display Name",
      "bootstrapPeers": ["/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer"],
      "enableMdns": true,
      "enableDht": true,
      "enableRelay": true,
      "config": {
        "dmPolicy": "open",
        "allowFrom": ["*"]
      }
    }
  }
}
```

### Usage
```bash
# Add Gigi channel to OpenClaw
openclaw channels add gigi --peer-id <peerId> --multiaddrs <multiaddrs>

# Start the Gigi channel
openclaw channels start gigi
```

For detailed documentation, see [docs/gigi-openclaw.md](docs/gigi-openclaw.md).

## 3. Gigi P2P TypeScript Client (`pkgs/gigi-p2p-ts`)

### Description
A TypeScript implementation of the Gigi P2P client, providing a high-level API for P2P communication, group messaging, and file sharing.

### Key Features
- **Libp2p Integration**: Built on Libp2p for robust P2P networking
- **Direct Messaging**: Send messages to specific peers
- **Group Messaging**: Use GossipSub for group communication
- **File Sharing**: Share and download files between peers
- **Peer Management**: Discover and manage connected peers
- **Event System**: Listen for network events

### Usage
```typescript
import { P2pClient } from '@gigi/p2p-ts';

const client = new P2pClient({
  nickname: 'My Node',
  config: {
    bootstrapNodes: ['/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer'],
    enableKademlia: true,
    enableRelay: true,
    enableMdns: true,
    listenAddrs: ['/ip4/0.0.0.0/tcp/0', '/ip4/0.0.0.0/ws/0'],
  },
});

await client.start();
await client.sendDirectMessage('peer-id', 'Hello!');
await client.joinGroup('general');
await client.sendGroupMessage('general', 'Hello everyone!');
```

For detailed documentation, see [docs/gigi-p2p-ts.md](docs/gigi-p2p-ts.md).

## 4. Gigi P2P Rust Client (`pkgs/gigi-p2p`)

### Description
A Rust implementation of the Gigi P2P client, providing core P2P functionality with high performance and reliability.

### Key Features
- **Rust Performance**: Fast and memory-safe implementation
- **Full P2P Stack**: Complete P2P networking stack
- **File Sharing**: Efficient file transfer between peers
- **Group Management**: Create and manage groups
- **Message Reliability**: Ensure message delivery
- **Custom Protocols**: Extend with custom protocols

For detailed documentation, see [docs/gigi-p2p.md](docs/gigi-p2p.md).

## 5. Gigi Network Node (`apps/gigi-node`)

### Description
A standalone Gigi P2P network node that can operate as a bootstrap node, relay node, or full node.

### Modes
- **Bootstrap**: Provides DHT entry points for new nodes
- **Relay**: Helps NATed peers connect to the network
- **Full**: Combines bootstrap and relay capabilities

### Usage
```bash
# Start a bootstrap node
./gigi-node --mode bootstrap --listen /ip4/0.0.0.0/tcp/4001

# Start a relay node
./gigi-node --mode relay --listen /ip4/0.0.0.0/tcp/4002 --bootstrap /ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer

# Start a full node
./gigi-node --mode full --listen /ip4/0.0.0.0/tcp/4003 --bootstrap /ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer
```

For detailed documentation, see [docs/gigi-node.md](docs/gigi-node.md).

## 6. Gigi Auth (`pkgs/gigi-auth`)

### Description
Authentication and key management for Gigi P2P network participants.

### Features
- **Account Management**: Create and manage user accounts
- **Key Derivation**: Secure key derivation from passwords
- **Encryption**: Encrypt sensitive data
- **Settings Management**: Store and retrieve user settings
- **Mnemonic Recovery**: Recover accounts from mnemonic phrases

For detailed documentation, see [docs/gigi-auth.md](docs/gigi-auth.md).

## 7. Gigi DNS (`pkgs/gigi-dns`)

### Description
Decentralized DNS service for the Gigi P2P network, allowing peers to resolve names to peer IDs.

### Features
- **Name Resolution**: Resolve human-readable names to peer IDs
- **Decentralized**: No central authority
- **Secure**: Cryptographically verified records
- **Efficient**: Cached for performance
- **Name Registration**: Register and manage names

For detailed documentation, see [docs/gigi-dns.md](docs/gigi-dns.md).

## 8. Gigi File Sharing (`pkgs/gigi-file-sharing`)

### Description
File sharing utilities for the Gigi P2P network.

### Features
- **File Transfer**: Efficient file transfer between peers
- **Chunking**: Split large files for easier transfer
- **Progress Tracking**: Monitor download/upload progress
- **Error Handling**: Handle network errors gracefully
- **Resumable Transfers**: Resume interrupted transfers
- **Multi-peer Downloads**: Download from multiple peers for faster transfer

For detailed documentation, see [docs/gigi-file-sharing.md](docs/gigi-file-sharing.md).

## 9. Gigi Store (`pkgs/gigi-store`)

### Description
Persistence layer for the Gigi P2P network, storing messages, contacts, and other data.

### Features
- **Message Storage**: Store and retrieve messages
- **Contact Management**: Manage peer contacts
- **Group Storage**: Store group information
- **File Metadata**: Track shared files
- **Settings Persistence**: Save user settings
- **Offline Queue**: Queue messages for delivery when offline
- **Synchronization**: Sync data between devices
- **Transaction Support**: Ensure data consistency

For detailed documentation, see [docs/gigi-store.md](docs/gigi-store.md).

## 10. Tauri Plugin Gigi (`pkgs/tauri-plugin-gigi`)

### Description
Tauri plugin for integrating Gigi P2P functionality into desktop and mobile applications.

### Features
- **Cross-Platform**: Works on desktop (Windows, macOS, Linux) and mobile (iOS, Android)
- **Native Performance**: Rust backend for high performance and reliability
- **Easy Integration**: Simple API for Tauri apps
- **Comprehensive Commands**: Full P2P functionality exposed
- **Event System**: Real-time event handling

For detailed documentation, see [docs/tauri-plugin-gigi.md](docs/tauri-plugin-gigi.md).

## Integration Flow

1. **User Setup**: User configures Gigi channel in OpenClaw or integrates Gigi P2P into their application
2. **Connection**: Gigi client starts and connects to the P2P network
3. **Discovery**: Peer discovery via Kademlia DHT and mDNS
4. **Communication**: Send/receive messages and files
5. **Group Interaction**: Join and participate in groups
6. **Data Persistence**: Store messages, contacts, and other data

## Network Topology

The Gigi P2P network uses a combination of several peer discovery and routing mechanisms:

- **Kademlia DHT**: Distributed hash table for peer discovery and content routing
- **mDNS**: Local peer discovery on the same network
- **GossipSub**: Topic-based publish/subscribe messaging for groups
- **Circuit Relay**: NAT traversal for peers behind firewalls

## Security Architecture

The Gigi P2P ecosystem implements multiple layers of security:

- **Transport Security**: Noise protocol for encrypted communication
- **Application Security**: Access control, message verification, and rate limiting
- **Data Security**: Encryption at rest, key management, and data integrity

For detailed information about the security architecture, see [docs/architecture.md](docs/architecture.md).

## Scalability Considerations

The Gigi P2P ecosystem is designed to scale efficiently:

- **Horizontal Scaling**: Decentralized design with no single point of failure
- **Performance Optimization**: Connection pooling, parallel processing, and caching
- **Network Resilience**: Redundancy, fault tolerance, and self-healing

## API Reference

For detailed API documentation for all components, see [docs/api-reference.md](docs/api-reference.md).

## Examples

For practical examples of how to use the Gigi P2P ecosystem, see [docs/examples.md](docs/examples.md).

## Dependencies

- **Libp2p**: Core P2P networking library
- **TypeScript**: For TypeScript client and OpenClaw plugin
- **Rust**: For Rust client and network node
- **Tauri**: For desktop and mobile integration
- **SQLite**: For data persistence in Gigi Store

## Security Considerations

- **Encryption**: All communications are encrypted
- **Peer Verification**: Peers are verified by their public keys
- **Access Control**: Configure who can send you messages
- **File Safety**: Be cautious when downloading files from unknown peers
- **Privacy**: Protect user privacy through anonymous communication

## Troubleshooting

### Common Issues
- **Connection Problems**: Check network connectivity and firewall settings
- **Peer Discovery**: Ensure mDNS and DHT are enabled
- **File Transfer**: Check file permissions and network stability
- **Group Messaging**: Ensure all peers are subscribed to the same topic
- **NAT Traversal**: Ensure relay nodes are available

### Logs
Check the application logs and Gigi client logs for detailed error information.

## Contributing

Contributions to the Gigi P2P ecosystem are welcome! Please see the project's GitHub repository for guidelines.

## Getting Started

For a step-by-step guide to getting started with the Gigi P2P ecosystem, see [docs/quick-start.md](docs/quick-start.md).

## Testing Best Practices for Node.js/TypeScript

Testing is crucial for maintaining code quality, catching bugs early, and ensuring your application works as expected. Follow these guidelines for testing Node.js/TypeScript projects in the Gigi P2P ecosystem:

### Key Testing Principles

- **Framework Selection**: Use Vitest for faster testing with built-in TypeScript support, or Jest for broader compatibility.
- **Test Organization**: Place tests in `__tests__` directories or use `.test.ts` naming convention.
- **Type Safety**: Leverage TypeScript's type system in tests to catch errors early.
- **Mocking**: Properly mock external dependencies like libp2p to isolate tests.
- **Coverage Targets**: Aim for 80%+ test coverage, focusing on critical paths.
- **CI/CD Integration**: Automate tests in GitHub Actions for every push and pull request.
- **Test Quality**: Write descriptive, isolated tests that cover edge cases and error scenarios.
- **P2P-Specific Testing**: Mock libp2p, test network simulations, and verify file sharing functionality.
- **Continuous Improvement**: Regularly review coverage reports and update tests as code changes.

For detailed documentation and examples, see [docs/testing-best-practices.md](docs/testing-best-practices.md).