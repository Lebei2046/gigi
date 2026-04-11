# Gigi OpenClaw Plugin

## Overview

The Gigi OpenClaw Plugin is an integration plugin for OpenClaw that enables seamless communication with the Gigi P2P network. It allows OpenClaw to join Gigi P2P groups and chat with Gigi P2P clients.

### What is Gigi P2P?

Gigi P2P is a decentralized peer-to-peer network built on top of Libp2p, enabling secure, direct communication between peers without relying on centralized servers. It provides:

- Decentralized peer discovery via Kademlia DHT and mDNS
- Secure encrypted communication
- Group chat functionality
- File sharing capabilities

### Features

- **P2P Messaging**: Send and receive messages directly between peers
- **Group Chat**: Join and participate in Gigi P2P groups
- **Multiple Transports**: Support for TCP, WebSocket, and WebRTC
- **Seamless Integration**: Works directly with OpenClaw Gateway
- **Automatic Reconnection**: Handles network disruptions gracefully
- **Group Management**: Tools to join, leave, and list Gigi P2P groups

## Installation/Test

### Prerequisites

- Node.js 18+ or compatible runtime
- OpenClaw 2026.3.22+
- pnpm (recommended for monorepo setup)

### Installation

1. Clone the Gigi repository:

```bash
git clone <repository-url>
cd gigi
```

2. Install dependencies and build the project:

```bash
pnpm install
cd apps/gigi-openclaw
pnpm run build:bundle
```

3. Install the plugin in OpenClaw:

```bash
openclaw plugins install /path/to/gigi/apps/gigi-openclaw
```

### Testing

#### Unit Tests

Run the unit tests to verify the plugin's functionality:

```bash
pnpm test
```

#### Integration Tests

To test the plugin with OpenClaw:

1. Start the OpenClaw gateway:

```bash
openclaw gateway start
```

2. Use the GigiClient API in your code to test the plugin's functionality, or use the OpenClaw dashboard to interact with the plugin.

#### Manual Testing

1. **Start multiple OpenClaw instances** with the Gigi plugin enabled
2. **Create a Gigi account** on each instance
3. **Join the same group** on all instances
4. **Send messages** between instances to verify communication
5. **Test file sharing** by sending files between instances

## License

MIT
