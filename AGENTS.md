# Gigi P2P Ecosystem Guidelines

This document serves as a comprehensive guideline for the Gigi P2P ecosystem, describing its components, architecture, and usage. It provides an overview of the entire ecosystem and references detailed documentation for each component.

## 1. Components Overview

### Ecosystem Overview

The Gigi P2P ecosystem is a decentralized network of components designed to enable secure, direct communication between peers without relying on centralized servers. It is built on top of Libp2p, a modular network stack for peer-to-peer applications.

### Applications & Plugins

| Name | Description | Type | Documentation |
|------|-------------|------|---------------|
| Gigi Mobile | Mobile application for Gigi P2P network | Application | [apps/gigi-mobile/README.md](apps/gigi-mobile/README.md) |
| Gigi Network Node | Standalone network node for bootstrap/relay | Application | [docs/gigi-node.md](docs/gigi-node.md) |
| Gigi OpenClaw Plugin | Integration plugin for OpenClaw chat application | Plugin | [apps/gigi-openclaw/README.md](apps/gigi-openclaw/README.md) |

### Core Components

| Component | Description | Documentation |
|-----------|-------------|---------------|
| Gigi P2P Rust Client | Core P2P functionality with high performance | [docs/gigi-p2p.md](docs/gigi-p2p.md) |
| Gigi P2P TypeScript Client | High-level API for TypeScript applications | [docs/gigi-p2p-ts.md](docs/gigi-p2p-ts.md) |
| Gigi Request-Response TS | Request-response protocol for TypeScript | [pkgs/gigi-request-response-ts/README.md](pkgs/gigi-request-response-ts/README.md) |
| Gigi Auth | Authentication and key management | [docs/gigi-auth.md](docs/gigi-auth.md) |
| Gigi DNS | Decentralized name resolution | [docs/gigi-dns.md](docs/gigi-dns.md) |
| Gigi File Sharing | File sharing utilities | [docs/gigi-file-sharing.md](docs/gigi-file-sharing.md) |
| Gigi Store | Persistence layer for data storage | [docs/gigi-store.md](docs/gigi-store.md) |
| Tauri Plugin Gigi | Integration with Tauri desktop/mobile apps | [docs/tauri-plugin-gigi.md](docs/tauri-plugin-gigi.md) |

### Integration Flow

1. **User Setup**: User configures Gigi channel in OpenClaw or integrates Gigi P2P into their application
2. **Connection**: Gigi client starts and connects to the P2P network
3. **Discovery**: Peer discovery via Kademlia DHT and mDNS
4. **Communication**: Send/receive messages and files
5. **Group Interaction**: Join and participate in groups
6. **Data Persistence**: Store messages, contacts, and other data

## 2. Architecture

### 2.1 Layers

The Gigi P2P ecosystem follows a layered architecture:

1. **Network Layer**: Provides the foundation for peer-to-peer communication
2. **Protocol Layer**: Defines the rules for communication between peers
3. **Application Layer**: Provides higher-level functionality for end users

For detailed information about the architecture, see [docs/architecture.md](docs/architecture.md).

### 2.2 Network Topology

The Gigi P2P network uses a combination of several peer discovery and routing mechanisms:

- **Kademlia DHT**: Distributed hash table for peer discovery and content routing
- **mDNS**: Local peer discovery on the same network
- **GossipSub**: Topic-based publish/subscribe messaging for groups
- **Circuit Relay**: NAT traversal for peers behind firewalls

### 2.3 Technology Stacks

- **Libp2p**: Core P2P networking library
- **TypeScript**: For TypeScript client and OpenClaw plugin
- **Rust**: For Rust client and network node
- **Tauri**: For desktop and mobile integration
- **SQLite**: For data persistence in Gigi Store

### 2.4 Scalability Considerations

The Gigi P2P ecosystem is designed to scale efficiently:

- **Horizontal Scaling**: Decentralized design with no single point of failure
- **Performance Optimization**: Connection pooling, parallel processing, and caching
- **Network Resilience**: Redundancy, fault tolerance, and self-healing

### 2.5 Security Architecture

The Gigi P2P ecosystem implements multiple layers of security:

- **Transport Security**: Noise protocol for encrypted communication
- **Application Security**: Access control, message verification, and rate limiting
- **Data Security**: Encryption at rest, key management, and data integrity

For detailed information about the security architecture, see [docs/architecture.md](docs/architecture.md).

### 2.6 Security Considerations

- **Encryption**: All communications are encrypted
- **Peer Verification**: Peers are verified by their public keys
- **Access Control**: Configure who can send you messages
- **File Safety**: Be cautious when downloading files from unknown peers
- **Privacy**: Protect user privacy through anonymous communication

## 3. Best Practices

### 3.1 Testing Best Practices for Node.js/TypeScript

Testing is crucial for maintaining code quality, catching bugs early, and ensuring your application works as expected. Follow these guidelines for testing Node.js/TypeScript projects in the Gigi P2P ecosystem:

#### Key Testing Principles

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

## 4. Contribution & Support

### 4.1 Contributing

Contributions to the Gigi P2P ecosystem are welcome! Please see the project's GitHub repository for guidelines.

### 4.2 Troubleshooting

#### Common Issues
- **Connection Problems**: Check network connectivity and firewall settings
- **Peer Discovery**: Ensure mDNS and DHT are enabled
- **File Transfer**: Check file permissions and network stability
- **Group Messaging**: Ensure all peers are subscribed to the same topic
- **NAT Traversal**: Ensure relay nodes are available

#### Logs
Check the application logs and Gigi client logs for detailed error information.

### 4.3 Getting Started

For a step-by-step guide to getting started with the Gigi P2P ecosystem, see [docs/quick-start.md](docs/quick-start.md).

## 5. Additional Resources

### API Reference

For detailed API documentation for all components, see [docs/api-reference.md](docs/api-reference.md).

### Examples

For practical examples of how to use the Gigi P2P ecosystem, see [docs/examples.md](docs/examples.md).