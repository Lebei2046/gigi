# Gigi: P2P Social Application

A decentralized peer-to-peer social application built with Rust and TypeScript/React, leveraging Tauri for cross-platform development.

## Project Structure

```
├── apps/
│   ├── gigi-app/          # Desktop application (Tauri + React)
│   └── gigi-mobile/       # Mobile application (Tauri + React)
├── pkgs/
│   ├── gigi-dm/          # Core P2P direct messaging library
│   ├── gigi-messaging/    # Tauri backend plugin for messaging
│   ├── gigi-downloading/  # File downloading functionality
│   └── gigi-mdns/         # mDNS nickname resolution service
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── package.json
└── README.md
```

## Core Components

### Direct Messaging Library (`pkgs/gigi-dm`)

A Rust library built on libp2p that provides:
- **Direct TCP connections** (no mDNS required)
- **Text and image message** support
- **Noise protocol encryption** for secure communication
- **Yamux multiplexing** for efficient connection usage
- **Request-Response protocol** for reliable message delivery
- **Async processing** using Tokio runtime

### Gigi Messaging Plugin (`pkgs/gigi-messaging`)

A Tauri backend plugin that integrates messaging into frontend apps:
- **Gossipsub protocol** for publish-subscribe messaging
- **mDNS for peer discovery** and automatic connection management
- **Event-based architecture** for message reception and peer discovery
- **Command API** for frontend integration (subscribe, unsubscribe, send messages)

### Frontend Applications (`apps/`)

React-based applications using Tauri for cross-platform deployment:
- **Mobile-first design** with responsive UI
- **Authentication system** with registration and login flows
- **Router-based navigation** using React Router
- **State management** architecture for application state
- **Tauri integration** for accessing native functionality

## Technology Stack

### Backend
- **Rust**: Core language for P2P libraries
- **libp2p**: P2P networking framework
- **Tokio**: Async runtime
- **Serde**: Serialization/deserialization
- **Tauri**: App framework bridge

### Frontend
- **TypeScript**: Type-safe JavaScript
- **React**: UI framework
- **Tauri**: Cross-platform app wrapper
- **React Router**: Navigation
- **Bun**: Package manager

## Key Features

### Decentralized Communication
- **Point-to-point messaging** without central servers
- **Publish-subscribe model** for topic-based communication
- **Automatic peer discovery** via mDNS
- **Encrypted connections** using Noise protocol

### Messaging Capabilities
- **Text messages**: Simple and reliable text communication
- **Image sharing**: Support for image file transmission
- **Message acknowledgment**: Reliable message delivery confirmation
- **Content addressing**: Unique identification for messages

### Cross-Platform Support
- **Desktop applications** for Windows, macOS, and Linux
- **Mobile applications** for iOS and Android
- **Native OS integration** via Tauri
- **Consistent UI/UX** across platforms

### Security
- **Ed25519 key pairs** for identity verification
- **End-to-end encryption** for all communications
- **Secure peer-to-peer connections**
- **Connection timeout protection**

## Getting Started

### Prerequisites
- **Rust**: Install via [rustup](https://rustup.rs/)
- **Bun**: Install via [bun.sh](https://bun.sh/)
- **Tauri CLI**: Install via `cargo install tauri-cli`

### Installation

```bash
# Install project dependencies
bun install

# Build Rust libraries
cargo build
```

### Running Applications

#### Desktop Application
```bash
bun run --cwd apps/gigi-app tauri dev
```

#### Mobile Application
```bash
bun run --cwd apps/gigi-mobile tauri dev
```

### Building Applications

#### Desktop Application
```bash
bun run --cwd apps/gigi-app tauri build
```

#### Mobile Application
```bash
bun run --cwd apps/gigi-mobile tauri build
```

## Development

### Direct Messaging Library

```bash
# Build the library
cargo build --package gigi-dm

# Run tests
cargo test --package gigi-dm

# Run chat example
cargo run --example chat --package gigi-dm -- --port 8080
```

### Gigi Messaging Plugin

```bash
# Build the plugin
cargo build --package gigi-messaging

# Run tests
cargo test --package gigi-messaging
```

## How to Contribute

1. Fork the repository
2. Create a new branch for your feature or bug fix
3. Make your changes with appropriate tests
4. Submit a pull request

Please follow the project's code style and ensure all tests pass before submitting.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.