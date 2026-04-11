# Gigi Mobile

## Overview

A decentralized peer-to-peer (P2P) mobile messaging application built with React, TypeScript, and Tauri. Gigi Mobile enables secure, private communication without relying on centralized servers.

### Core Features
- **Direct Messaging**: P2P chat between connected peers
- **Group Messaging**: Create and join chat groups for multi-user conversations
- **Real-time Communication**: Instant message delivery using WebRTC and libp2p
- **Message History**: Local storage of chat history using IndexedDB
- **End-to-End Encryption**: Messages secured with cryptography
- **Decentralized Identity**: Peer-to-peer identity management
- **File Transfer**: Share files directly with peers

### Technology Stack
- **Frontend**: React 19, TypeScript, Tauri 2, Tailwind CSS, Radix UI, Redux Toolkit
- **Backend**: Rust, libp2p, WebRTC, Tokio, Serde
- **Key Libraries**: @noble/ciphers, @scure/bip32, @scure/bip39, react-qr-code, dexie

## Installation/Test

### Prerequisites
- Node.js 18+ 
- pnpm (package manager)
- Rust (for Tauri backend)

### Installation
```bash
# Clone the repository
git clone <repository-url>
cd gigi/apps/gigi-mobile

# Install dependencies
pnpm install

# Set up Rust toolchain
rustup target add x86_64-pc-windows-msvc  # Windows
rustup target add aarch64-apple-darwin     # macOS ARM
rustup target add x86_64-apple-darwin      # macOS Intel
```

### Testing
```bash
# Run all tests
pnpm run test

# Run specific test files
pnpm run test src/utils/__tests__/peerUtils.test.ts

# Watch mode
pnpm run test --watch
```

## License

This project is licensed under the MIT License. See the LICENSE file for details.