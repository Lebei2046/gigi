# Gigi P2P

## Overview

A comprehensive peer-to-peer networking library built on libp2p, supporting direct messaging, group messaging, and unified file sharing for all file types.

### Features

- 🌐 **Peer Discovery**: Automatic peer discovery via mDNS
- 💬 **Direct Messaging**: Send messages and files directly to peers
- 📢 **Group Messaging**: Join groups and broadcast messages
- 📁 **Universal File Sharing**: Share any file type with unique share codes
- 🖼️ **Image Messaging**: Optimized image handling with preview capabilities
- 🔍 **Nicknames**: Human-readable peer identification
- ⬇️ **Chunked Downloads**: Large files downloaded with real-time progress tracking
- ✅ **Integrity Verification**: SHA256 hash verification for all file transfers
- 📊 **Progress Events**: Detailed download progress and completion events

### Architecture

The library uses a unified `NetworkBehaviour` that combines:

1. **mDNS** - For local peer discovery
2. **Request-Response** - For direct messaging and file transfers
3. **GossipSub** - For group messaging

### File Sharing

Files are shared using a unique share code system with features like chunked transfers, real-time progress, hash verification, and concurrent downloads.

## Installation/Test

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
gigi-p2p = { path = "rust/gigi-p2p" }
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Testing

Run the test suite:

```bash
cargo test --package gigi-p2p
```

### Examples

Run the included chat example:

```bash
# Terminal 1
cargo run --package gigi-p2p --example chat -- --nickname Alice

# Terminal 2  
cargo run --package gigi-p2p --example chat -- --nickname Bob
```

## License

This project is part of the Gigi P2P networking suite.