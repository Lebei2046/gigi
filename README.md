# Gigi: P2P Social Application

## Project Overview

Gigi is an ambitious peer-to-peer (P2P) social application built for secure, decentralized communication across mobile and desktop platforms. It combines modern web technologies (React/TypeScript) with Rust-based P2P networking to create a privacy-focused messaging and file-sharing platform.

## Project Structure

```
├── apps/
│   ├── gigi-mobile/    # Mobile React/Tauri application
│   └── gigi-node/      # Standalone P2P node for cloud deployment
└── pkgs/               # Rust libraries
    ├── gigi-auth/      # Account management and key derivation
    ├── gigi-dns/       # Peer discovery with nicknames
    ├── gigi-file-sharing/ # Chunked file transfer
    ├── gigi-p2p/       # Core P2P networking library
    ├── gigi-store/     # Data persistence
    └── tauri-plugin-gigi/ # Tauri plugin for frontend integration
```

## Core Architecture

### P2P Network Layer

The **gigi-p2p** library provides unified P2P functionality through a modular architecture:

- **UnifiedBehaviour**: Combines multiple libp2p protocols into a single network behavior
  - **GigiDNS**: Custom DNS-like discovery with nicknames, capabilities, and metadata
  - **Kademlia DHT**: Distributed hash table for WAN peer discovery
  - **Circuit Relay**: NAT traversal for peers behind routers
  - **Direct Messaging**: Request-response protocol for 1:1 communication
  - **GossipSub**: Pub-sub protocol for group messaging
  - **File Sharing**: Chunked file transfer with integrity verification

- **Protocol Stack**:
  - TCP/QUIC transport with Noise encryption
  - Yamux multiplexing for efficient connection management
  - CBOR serialization for efficient data transfer

### Cloud Infrastructure (gigi-node)

**gigi-node** enables cross-network communication by deploying bootstrap and relay nodes on cloud hosts:

- **Bootstrap Nodes**: Well-known DHT entry points for peer discovery
- **Relay Nodes**: Enable NAT traversal for mobile devices behind routers
- **Full Nodes**: Combined bootstrap and relay capabilities

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              INTERNET (WAN)                                 │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                    CLOUD HOSTS (Bootstrap + Relay)                  │   │
│   │                                                                     │   │
│   │   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐            │   │
│   │   │ Bootstrap 1 │    │ Bootstrap 2 │    │ Relay Node  │            │   │
│   │   │ 203.0.113.10│    │ 203.0.113.11│    │ 203.0.113.12│            │   │
│   │   └─────────────┘    └─────────────┘    └─────────────┘            │   │
│   │                                                                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                    ▲                                        │
│                                    │                                        │
│              ┌─────────────────────┴─────────────────────┐                  │
│              │                                           │                  │
│              ▼                                           ▼                  │
│   ┌─────────────────────┐                     ┌─────────────────────┐       │
│   │     WIFI NETWORK 1  │                     │     WIFI NETWORK 2  │       │
│   │  ┌───────────────┐  │                     │  ┌───────────────┐  │       │
│   │  │  Gigi Mobile  │  │◄───────────────────►│  │  Gigi Mobile  │  │       │
│   │  │    App A      │  │   Via Cloud Relay   │  │    App B      │  │       │
│   │  └───────────────┘  │                     │  └───────────────┘  │       │
│   └─────────────────────┘                     └─────────────────────┘       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Peer Discovery

**Local Discovery (gigi-dns)**:
- Multicast-based peer announcement (mDNS)
- Nickname resolution and capability advertising
- TTL-based cache management
- Per-interface monitoring for network changes

**WAN Discovery (Kademlia DHT)**:
- Distributed hash table for internet-scale peer discovery
- O(log n) lookup efficiency
- Bootstrap nodes for network entry
- Works across NATs and firewalls

### File Sharing System

The file sharing system uses a unique **share code** mechanism:
1. Files split into 256KB chunks with BLAKE3 hashes
2. Share code generated for each file (unique per share instance)
3. Pull-based transfer: receivers request chunks on-demand
4. SHA256 verification for complete file integrity
5. Download tracking for UI integration

### Authentication & Identity

**gigi-auth** provides secure account management:
- BIP-39 mnemonic phrase generation
- ChaCha20-Poly1305 encryption for mnemonics
- BIP-32 key derivation for multiple identities:
  - EVM addresses for blockchain interactions
  - Peer IDs for libp2p identity
  - Group IDs for P2P groups

## Frontend-Backend Integration

### Tauri Plugin Architecture

The **tauri-plugin-gigi** exposes Rust P2P functionality to the React frontend:
- **Commands**: 80+ Tauri commands for all P2P operations
- **Events**: Real-time event system for:
  - Peer discovery/expiry
  - Message reception (direct/group)
  - File share events
  - Download progress updates

### State Management

- **PluginState**: Global state managing all P2P components
- **Managers**: Specialized managers for:
  - Authentication (AuthManager)
  - Contact management (ContactManager)
  - File sharing (FileSharingManager)
  - Group management (GroupManager)
  - Message persistence (MessageStore)

## Key Features

1. **Auto-discovery**: Peers find each other without centralized servers
2. **Nickname System**: Human-friendly identifiers instead of cryptic IDs
3. **Secure Communication**: End-to-end encryption for messages and files
4. **Cross-platform**: Mobile (iOS/Android) and desktop (Windows/macOS/Linux)
5. **Offline Support**: Message persistence for offline viewing
6. **File Sharing**: Secure, integrity-verified file transfer
7. **Group Messaging**: Pub-sub based group chats with nickname support
8. **Download Tracking**: Real-time progress updates for mobile UI

## Technology Stack

- **Frontend**: React 18, TypeScript, Tauri
- **Backend**: Rust, libp2p, Tokio, SeaORM (SQLite)
- **Networking**: TCP/QUIC, Noise encryption, Yamux, GossipSub, mDNS, Kademlia DHT, Circuit Relay
- **Security**: ChaCha20-Poly1305, BIP-32/BIP-39, BLAKE3, SHA256
- **Build Tools**: Cargo, Vite, Tauri CLI, Bun

## Development Workflow

### Getting Started

1. **Install dependencies**:
   ```bash
   # Rust dependencies
   cargo install tauri-cli
   
   # Frontend dependencies
   bun install
   cd apps/gigi-mobile
   bun install
   ```

2. **Run development server**:
   ```bash
   # Mobile app
   cd apps/gigi-mobile
   bun run tauri dev
   
   # Desktop app  
   cd apps/gigi-app
   bun run tauri dev
   
   # Cloud node (for testing)
   cd apps/gigi-node
   cargo run -- --mode bootstrap --listen /ip4/0.0.0.0/tcp/4001
   ```

3. **Build for production**:
   ```bash
   # Mobile app
   cd apps/gigi-mobile
   bun run tauri build
   
   # Cloud node
   cd apps/gigi-node
   cargo build --release
   ```

### Key Development Features

- **Hot reload** for React components
- **Type-safe API** between frontend and backend
- **Comprehensive logging** with tracing
- **Unit/integration tests** for all Rust libraries
- **Cross-compilation** support

## Security Considerations

- **No centralized servers** to compromise
- **End-to-end encryption** for all communication
- **Secure key storage** with platform-native mechanisms
- **File integrity verification** at every step
- **Defense-in-depth** with multiple security layers

## Cloud Deployment

### Deploy Bootstrap Node

```bash
cd apps/gigi-node
cargo build --release

./target/release/gigi-node \
  --mode bootstrap \
  --listen /ip4/0.0.0.0/tcp/4001 \
  --external /ip4/YOUR_IP/tcp/4001 \
  --identity /data/bootstrap.key
```

### Deploy Relay Node

```bash
./target/release/gigi-node \
  --mode relay \
  --listen /ip4/0.0.0.0/tcp/4003 \
  --external /ip4/YOUR_IP/tcp/4003 \
  --bootstrap /ip4/BOOTSTRAP_IP/tcp/4001/p2p/PEER_ID
```

See [gigi-node README](apps/gigi-node/README.md) and [Cloud Setup Guide](apps/gigi-mobile/CLOUD_SETUP.md) for detailed deployment instructions.

## Future Enhancements

- Enhanced NAT traversal for better connectivity
- Support for larger file transfers
- Audio/video calls over P2P
- Improved mobile performance optimizations
- Additional P2P discovery mechanisms
- Decentralized group management

## Conclusion

Gigi represents a modern approach to decentralized communication, leveraging Rust's memory safety and performance with React's developer experience. Its modular architecture allows for easy extension and maintenance, while its P2P design provides strong privacy guarantees. The project demonstrates a sophisticated understanding of distributed systems, cryptography, and cross-platform development.

This analysis provides a comprehensive overview of the Gigi P2P social application, highlighting its architecture, key features, and technical implementation details.