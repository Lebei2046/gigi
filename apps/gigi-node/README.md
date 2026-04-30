# Gigi Node

## Overview

A standalone P2P network node for the Gigi messaging system. Built on rust-libp2p, it supports bootstrap, relay, and full node modes for enabling cross-network mobile communication.

### Core Components

Gigi Node implements a unified libp2p stack with multiple network behaviors:

- **Kademlia DHT** - Distributed hash table for peer discovery and routing
- **GossipSub** - Pub/sub messaging for group communication
- **Identify Protocol** - Peer metadata exchange
- **Ping Protocol** - Connection health monitoring
- **Circuit Relay** - NAT traversal support

### Node Modes

- **Bootstrap Mode**: Provides DHT entry points for the network
- **Relay Mode**: Enables NAT traversal for mobile devices
- **Full Mode**: Combined bootstrap and relay capabilities

### Features

- **Kademlia DHT**: Distributed peer discovery with memory-based storage
- **GossipSub**: Pub/sub messaging with signed authentication
- **Circuit Relay**: NAT traversal for P2P connections
- **Identity Persistence**: Save/load cryptographic keys
- **Multi-Address Support**: Listen on multiple addresses simultaneously
- **Auto-Discovery**: Automatic DHT bootstrapping and peer routing table updates

## Installation/Test

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd gigi/apps/gigi-node

# Build the main binary
cargo build --release --bin gigi-node

# Or build with examples
cargo build --release --bin gigi-node --examples
```

### Testing

#### Quick Local Test (In-Process)

The fastest way to test group messaging without Docker:

```bash
RUST_LOG=info cargo run -p gigi-node --example group_messaging
```

This runs an in-process test with:
- 2 bootstrap nodes forming DHT backbone (node-1, node-2)
- Alice (sender) connecting to node-1
- Bob (receiver) connecting to node-2
- Messages propagated via GossipSub on `gigi-general` topic

#### Docker-Based Test (Distributed NAT Simulation)

For testing real NAT traversal with distributed nodes:

```bash
# From project root
docker build -t gigi-node:latest -f apps/gigi-node/Dockerfile .

# Clean up old containers and volumes
docker-compose -f apps/gigi-node/docker-compose.yml down -v --remove-orphans

# Start network
./apps/gigi-node/start-network.sh

# View logs
docker-compose -f apps/gigi-node/docker-compose.yml logs -f
```

To stop the network:

```bash
./apps/gigi-node/stop-network.sh
```

## License

Same as Gigi project
