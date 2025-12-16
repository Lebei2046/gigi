# Gigi: P2P Social Application

A comprehensive decentralized peer-to-peer social application built with Rust and TypeScript/React, leveraging Tauri for cross-platform deployment.

## Project Structure

```
gigi/
├── apps/
│   ├── gigi-app/          # Desktop application (Tauri + React)
│   └── gigi-mobile/       # Mobile application (Tauri + React)
├── pkgs/
│   └── gigi-p2p/          # Comprehensive P2P networking library
├── Cargo.lock
├── Cargo.toml             # Rust workspace configuration
├── package.json           # Node.js workspace configuration
├── LICENSE
└── README.md
```

## Core Components

### P2P Networking Library (`pkgs/gigi-p2p`)

A comprehensive Rust library built on libp2p that provides the complete P2P functionality:

#### Core Networking
- **Peer Discovery**: Automatic mDNS-based peer discovery with nickname resolution
- **Multiple Protocols**: TCP, QUIC support with Noise protocol encryption
- **Multiplexing**: Yamux for efficient connection management
- **Request-Response**: Reliable message delivery with acknowledgments

#### Communication Features
- **Direct Messaging**: Text messages and image sharing between peers
- **Group Messaging**: Gossipsub-based pub/sub group communication
- **Nicknames**: Human-readable peer identification system

#### File Sharing System
- **Chunked Transfers**: Large files transferred in 256KB chunks
- **Progress Tracking**: Real-time download progress events
- **Hash Verification**: SHA256 integrity checking
- **Persistent Storage**: Shared files saved to `shared.json`
- **Share Codes**: Unique codes for file access
- **Duplicate Detection**: Same files share existing codes

#### Key APIs
- `send_direct_message()` - Direct peer messaging
- `send_group_message()` - Group messaging
- `share_file()` - Share files with unique codes
- `download_file()` - Download shared files
- `unshare_file()` - Remove file sharing
- `send_direct_image()` - Direct image sharing
- `send_group_image()` - Group image sharing

### Frontend Applications (`apps/`)

Modern React-based applications built with TypeScript and Tauri:

#### Desktop Application (`apps/gigi-app`)
- **Cross-platform desktop app** for Windows, macOS, and Linux
- **TypeScript + React**: Type-safe frontend development
- **Vite**: Fast development server and building
- **Tauri Integration**: Native OS access and performance

#### Mobile Application (`apps/gigi-mobile`)
- **Mobile-optimized** for iOS and Android deployment
- **Responsive Design**: Adaptive UI for mobile screens
- **Touch-optimized**: Gestures and mobile interactions
- **React Router**: Navigation between chat screens

Both applications feature:
- **Modern UI**: Clean, intuitive user interface
- **Real-time Updates**: Live P2P event handling
- **File Management**: Download and file sharing interfaces
- **Peer Management**: Visual peer discovery and connection status
- **Group Chat**: Multi-user communication interfaces

## Technology Stack

### Backend (Rust Workspace)
- **Rust**: Core language for P2P libraries
- **libp2p**: Comprehensive P2P networking framework
  - mDNS, Gossipsub, Request-Response protocols
  - TCP/QUIC transport with Noise encryption
  - Yamux multiplexing and Kademlia DHT
- **Tokio**: High-performance async runtime
- **Serde**: JSON/CBOR serialization
- **Tauri**: Native app framework and plugin system
- **Additional Libraries**: blake3, sha2, chrono, uuid, tracing

### Frontend (TypeScript/React)
- **TypeScript**: Type-safe JavaScript development
- **React**: Modern UI framework with hooks
- **Tauri**: Cross-platform app wrapper and native bridge
- **Vite**: Fast development server and build tool
- **React Router**: Client-side routing
- **Bun**: JavaScript runtime and package manager
- **ESLint**: Code quality and style enforcement

## Key Features

### P2P Network Infrastructure
- **Zero-Configuration**: Automatic peer discovery via mDNS
- **Multi-Protocol**: TCP and QUIC transport support
- **Encrypted Communication**: Noise protocol for all connections
- **Identity Management**: Ed25519 key pairs and nickname system
- **Connection Multiplexing**: Efficient resource usage

### Communication System
- **Direct Messaging**: Real-time text and image sharing between peers
- **Group Communication**: Gossipsub-based broadcast messaging with bidirectional communication
  - **Group Creation**: Users can create groups and invite other peers  
  - **Owner/Member Roles**: Distinguished access between group creators and invited members
    - `joined: false` = Group Creator/Owner (owns the group)
    - `joined: true` = Invited Member (joined the group)
  - **Topic Subscription**: Both owners and members automatically subscribe to group topics for messaging
- **File Sharing**: Comprehensive file transfer system
  - Share any file type with unique codes
  - Chunked downloads with progress tracking
  - Automatic integrity verification
  - Persistent sharing registry
- **Media Support**: Image sharing with metadata (PNG, JPEG, GIF, WebP)

### User Experience
- **Cross-Platform**: Desktop and mobile applications
- **Real-time UI**: Live updates for all P2P events
- **Intuitive Interface**: Clean, modern design
- **Mobile-Optimized**: Touch gestures and responsive layouts
- **Native Integration**: OS-level features via Tauri

### Developer Experience
- **Type Safety**: Full TypeScript support
- **Hot Reload**: Fast development cycles
- **Monorepo**: Shared dependencies and build system
- **Comprehensive APIs**: Well-documented P2P functionality
- **Example Applications**: Chat app with full feature demonstration

## Getting Started

### Prerequisites
- **Rust**: Install via [rustup](https://rustup.rs/) (latest stable)
- **Bun**: Install via [bun.sh](https://bun.sh/) for JavaScript tooling
- **Tauri CLI**: Install via `cargo install tauri-cli`
- **Node.js**: For frontend development (if not using Bun)

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd gigi

# Install JavaScript/TypeScript dependencies
bun install

# Build all Rust workspace packages
cargo build

# Run tests
cargo test
```

### Running Applications

#### Desktop Application
```bash
# Development mode with hot reload
bun run --cwd apps/gigi-app tauri dev

# Or using Tauri CLI directly
cd apps/gigi-app
cargo tauri dev
```

#### Mobile Application
```bash
# Development mode
$ANDROID_HOME/emulator/emulator -list-avds
$ANDROID_HOME/emulator/emulator -avd Medium_Phone_API_36.1
bun run --cwd apps/gigi-mobile tauri dev

# Note: Mobile development may require additional platform-specific setup
```

### Building Applications

#### Production Builds
```bash
# Desktop application
bun run --cwd apps/gigi-app tauri build

# Mobile application
bun run --cwd apps/gigi-mobile tauri build
```

## Development

### P2P Library Development

```bash
# Build the P2P library
cargo build --package gigi-p2p

# Run tests
cargo test --package gigi-p2p

# Run the interactive chat example
cargo run --package gigi-p2p --example chat -- --nickname Alice

# Run with custom port
cargo run --package gigi-p2p --example chat -- --nickname Bob --port 8080
```

### Messaging Plugin Development

```bash
# Build the Tauri plugin
cargo build --package gigi-messaging

# Run tests
cargo test --package gigi-messaging
```

### Chat Application Features

The included chat example demonstrates all features:
- Peer discovery and connection management
- Direct text and image messaging
- Group chat with multiple participants
- File sharing with progress tracking
- Command-line interface for all operations

Available commands in the chat example:
- `help` - Show all available commands
- `peers` - List connected peers
- `send <nick> <message>` - Send direct message
- `send-image <nick> <path>` - Send image
- `join <group>` - Join group chat
- `send-group <group> <message>` - Send group message
- `share <path>` - Share file with code
- `unshare <code>` - Remove file sharing
- `files` - List shared files
- `download <nick> <code>` - Download file

## Workspace Architecture

This project uses a monorepo structure with Rust and JavaScript workspaces:

### Rust Workspace (Cargo.toml)
- **Shared Dependencies**: Managed at workspace level
- **Version Sync**: All packages use the same version
- **Optimized Builds**: LTO and size optimization for release
- **Dev Features**: Test builds with additional debugging

### JavaScript Workspace (package.json)
- **Package Management**: Bun for fast operations
- **Shared Config**: ESLint, TypeScript configurations
- **Cross-Platform**: Consistent development environment

## Contributing

### Development Workflow
1. **Fork** the repository
2. **Create** a feature branch from `main`
3. **Make changes** with comprehensive tests
4. **Test** across both Rust and TypeScript
5. **Update documentation** as needed
6. **Submit** a pull request with clear description

## Troubleshooting

### Common Issues and Solutions

#### Group Messaging Issues
**Problem**: Group owner cannot send messages, group members don't receive messages

**Root Cause**: Group owners (`joined: false`) and members (`joined: true`) both need to subscribe to the gossipsub topic to participate in group messaging.

**Solution**: The application now automatically subscribes both group owners and members to group topics when opening group chats, regardless of the `joined` flag status.

**Debugging Steps**:
1. Check console logs for "✅ Successfully joined group" messages
2. Verify both instances show "📊 Total groups in local storage: 1" 
3. Ensure "📤 Sending group message" and "✅ Group message published successfully" appear
4. Check for "🔥 Raw gossipsub message received" on receiver side

#### Mobile Development Setup
**Android Emulator Setup**:
```bash
# List available emulators
$ANDROID_HOME/emulator/emulator -list-avds

# Start emulator
$ANDROID_HOME/emulator/emulator -avd Medium_Phone_API_36.1
```

**Common Build Issues**:
- Ensure Android Studio and Android SDK are properly installed
- Verify `$ANDROID_HOME` environment variable is set
- Run `bun run tauri android init` before first mobile build

### Code Standards
- **Rust**: Follow `rustfmt` and `clippy` recommendations
- **TypeScript**: ESLint configuration with strict rules
- **Commit Messages**: Conventional commit format
- **Documentation**: Keep README files current
- **Tests**: Ensure all tests pass before PR

### Areas for Contribution
- **UI/UX Improvements**: Enhanced frontend interfaces
- **Protocol Extensions**: Additional P2P protocols
- **Mobile Features**: Platform-specific optimizations
- **Performance**: Network efficiency and speed improvements
- **Security**: Enhanced encryption and verification

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **libp2p**: The foundational P2P networking library
- **Tauri**: Cross-platform application framework
- **Rust Community**: Tools and ecosystem support