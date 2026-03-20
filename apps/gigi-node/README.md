# Gigi Node

A standalone P2P network node for the Gigi messaging system. Built on rust-libp2p, it supports bootstrap, relay, and full node modes for enabling cross-network mobile communication.

## Architecture

### Core Components

Gigi Node implements a unified libp2p stack with multiple network behaviors:

```
┌─────────────────────────────────────────────────────────────┐
│                        Gigi Node                            │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │ Kademlia │  │GossipSub │  │ Identify │  │  Relay   │    │
│  │   DHT    │  │ Pub/Sub  │  │ Protocol │  │ Circuit  │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
├─────────────────────────────────────────────────────────────┤
│              TCP + Noise + Yamux                         │
└─────────────────────────────────────────────────────────────┘
```

#### Network Behaviors

- **Kademlia DHT** - Distributed hash table for peer discovery and routing
  - Bootstrap nodes act as DHT servers, storing routing information
  - Relay/Full nodes query DHT to discover other peers
  - Memory-based storage for routing tables
  - Automatic peer discovery via `RoutingUpdated` and `UnroutablePeer` events

- **GossipSub** - Pub/sub messaging for group communication
  - Message validation with strict mode
  - Signed message authentication using Ed25519 keys
  - 10-second heartbeat for mesh maintenance
  - Topics: `gigi-general` (default, configurable)
  - Events: `Message`, `Subscribed`, `Unsubscribed`, `GossipsubNotSupported`

- **Identify Protocol** - Peer metadata exchange
  - Protocol version: `/gigi/1.0.0`
  - Automatically exchanges peer capabilities and observed addresses

- **Ping Protocol** - Connection health monitoring
  - Keeps connections alive through regular ping/pong
  - Debug-level logging for ping events
  - Default ping interval

- **Circuit Relay** - NAT traversal support
  - Enables peers behind NAT to connect
  - Relay nodes facilitate indirect connections
  - Logs relay events for debugging

### Node Modes

#### Bootstrap Mode
- Provides DHT entry points for the network
- Acts as a DHT server (stores and provides routing information)
- Does not subscribe to GossipSub topics by default
- Used as anchor nodes for network bootstrapping
- Logs: DHT server mode, bootstrap complete

#### Relay Mode
- Enables NAT traversal for mobile devices
- Queries DHT in client mode
- Subscribes to GossipSub topics for message relay
- Acts as a relay circuit for peer-to-peer connections
- Logs: DHT client mode, topic subscriptions

#### Full Mode
- Combines both bootstrap and relay capabilities
- Acts as DHT server and relay node
- Subscribes to GossipSub topics
- Ideal for stable servers in the network
- Logs: DHT server mode, topic subscriptions

### Transport Layer

- **Protocol Stack**: TCP → Noise (XX handshake) → Yamux (multiplexing)
- **Authentication**: Noise protocol with Ed25519 keys
- **Multiplexing**: Yamux for stream multiplexing over single connections
- **Connection Timeout**: 60 seconds idle connection timeout

### Identity Management

- **Ed25519 Key Pairs** - Cryptographic identity for each node
- **Persistent Storage**: Keys can be saved to and loaded from files via protobuf encoding
- **Ephemeral Mode**: Generate temporary keys for testing (default)
- **File Format**: Protobuf-encoded Keypair

## Features

- **Bootstrap Mode**: Provides DHT entry points for new peers
- **Relay Mode**: Enables NAT traversal for mobile devices behind routers
- **Full Mode**: Combined bootstrap and relay capabilities
- **Kademlia DHT**: Distributed peer discovery with memory-based storage
- **GossipSub**: Pub/sub messaging with signed authentication
- **Circuit Relay**: NAT traversal for P2P connections
- **Identity Persistence**: Save/load cryptographic keys
- **Multi-Address Support**: Listen on multiple addresses simultaneously
- **Auto-Discovery**: Automatic DHT bootstrapping and peer routing table updates

## Usage

### Build

Build the main binary:

```bash
cargo build --release --bin gigi-node
```

Or build with examples:

```bash
cargo build --release --bin gigi-node --examples
```

### Environment Variables

- `RUST_LOG` - Set logging level (default: based on environment)
  - `RUST_LOG=info` - Info level logging
  - `RUST_LOG=debug` - Debug level (includes ping events)
  - `RUST_LOG=gigi_node=debug` - Debug for gigi-node only

### Run as Bootstrap Node

```bash
./target/release/gigi-node \
  --mode bootstrap \
  --listen /ip4/0.0.0.0/tcp/4001 \
  --external /ip4/203.0.113.10/tcp/4001 \
  --identity /data/bootstrap.key
```

**Note**: Bootstrap nodes do not need the `--bootstrap` parameter as they provide DHT entry points.

### Run as Relay Node

```bash
./target/release/gigi-node \
  --mode relay \
  --listen /ip4/0.0.0.0/tcp/4003 \
  --external /ip4/203.0.113.12/tcp/4003 \
  --bootstrap /ip4/203.0.113.10/tcp/4001/p2p/12D3KooW... \
  --identity /data/relay.key \
  --topics gigi-general
```

### Run as Full Node

```bash
./target/release/gigi-node \
  --mode full \
  --listen /ip4/0.0.0.0/tcp/4002 \
  --external /ip4/203.0.113.11/tcp/4002 \
  --bootstrap /ip4/203.0.113.10/tcp/4001/p2p/12D3KooW... \
  --identity /data/fullnode.key \
  --topics gigi-general
```

## Command Line Options

| Option | Description | Example | Required |
|--------|-------------|---------|----------|
| `--mode` | Node mode: bootstrap, relay, or full | `--mode bootstrap` | Yes |
| `--listen` | Listen addresses (can specify multiple) | `--listen /ip4/0.0.0.0/tcp/4001` | Yes |
| `--external` | External addresses to advertise | `--external /ip4/203.0.113.10/tcp/4001` | No |
| `--bootstrap` | Bootstrap peer addresses (for non-bootstrap nodes) | `--bootstrap /ip4/203.0.113.10/tcp/4001/p2p/12D3KooW...` | No* |
| `--identity` | Path to identity key file (generated if not exists) | `--identity /data/node.key` | No |
| `--topics` | Topics to subscribe to (relay/full mode) | `--topics gigi-general` | No |

*Required for relay and full nodes, optional for bootstrap nodes

## Event Handling

### Swarm Events

The node handles the following events:

- **NewListenAddr** - When the node starts listening on an address
- **ConnectionEstablished** - When a connection is made (inbound or outbound)
- **ConnectionClosed** - When a connection is closed (with cause)

### Kademlia Events

- **Bootstrap** - DHT bootstrap completion or failure
- **GetClosestPeers** - Found closest peers query results
- **RoutingUpdated** - New peer added to routing table
- **UnroutablePeer** - Peer is no longer routable

### GossipSub Events

- **Message** - Received a message from the pub/sub mesh
- **Subscribed** - Peer subscribed to a topic
- **Unsubscribed** - Peer unsubscribed from a topic
- **GossipsubNotSupported** - Peer doesn't support GossipSub protocol

## Testing

### Quick Local Test (In-Process)

The fastest way to test group messaging without Docker:

```bash
cd apps/gigi-node
RUST_LOG=info cargo run -p gigi-node --example group_messaging
```

This runs an in-process test with:
- 2 bootstrap nodes forming DHT backbone (node-1, node-2)
- Alice (sender) connecting to node-1
- Bob (receiver) connecting to node-2
- Messages propagated via GossipSub on `gigi-general` topic

**Expected Behavior**:
1. Alice publishes messages to the group
2. Messages propagate through the GossipSub mesh
3. Bob receives messages and replies
4. Alice receives Bob's reply

### Docker-Based Test (Distributed NAT Simulation)

For testing real NAT traversal with distributed nodes:

#### 1. Build Docker Image

```bash
# From project root
docker build -t gigi-node:latest -f apps/gigi-node/Dockerfile .

# Or from within the gigi-node directory
cd apps/gigi-node
docker build -t gigi-node:latest .
```

#### 2. Clean Up Old Containers and Volumes

If you encounter errors like `KeyError: 'ContainerConfig'`, clean up old containers and volumes:

```bash
cd apps/gigi-node
docker-compose down -v --remove-orphans
```

This will:
- Remove all containers
- Remove all volumes
- Remove orphaned containers

#### 3. Start Network

First, start just the bootstrap node to get its Peer ID:

```bash
docker run -it --rm gigi-node:latest \
  --mode bootstrap \
  --listen /ip4/0.0.0.0/tcp/4001
```

Look for `Local peer ID: 12D3KooW...` in the output, then update `docker-compose.yml` replacing `QmPLACEHOLDER` with the actual Peer ID.

**Or use the automated startup script** (recommended):

```bash
./start-network.sh
```

This script will:
1. Start the bootstrap node
2. Automatically extract the peer ID from logs
3. Update `docker-compose.yml` with the correct peer ID
4. Start all remaining services

To stop the network:

```bash
./stop-network.sh
```

#### 4. Start Network

```bash
docker compose up -d
```

#### 5. View Logs

```bash
docker compose logs -f
```

### Docker Network Topology

```
┌─────────────────── public-net ────────────────────┐
│                                                  │
│   bootstrap:4001      relay:4002                │
│   (DHT entry)        (NAT traversal)             │
│                                                  │
└─────────────────────┬────────────────────────────┘
                      │
      ┌───────────────┼───────────────┐
      │               │               │
 nat-net-1       nat-net-2       nat-net-3
      │               │               │
   alice          bob           charlie
 (sender)      (receiver)    (receiver)
```

**Network Isolation**: Each client (alice, bob, charlie) is in a separate bridge network, simulating NATed environments. They can only reach the bootstrap node via `host.docker.internal`.

### Using docker_group_client Directly

The `docker_group_client` example can be run manually:

```bash
# Run as alice
docker run -it --rm gigi-node:latest \
  /usr/local/bin/docker_group_client \
  --username alice \
  --port 0 \
  --bootstrap /ip4/host.docker.internal/tcp/4001/p2p/12D3KooW... \
  --relay /ip4/host.docker.internal/tcp/4002/p2p/12D3KooW...
```

**CLI Arguments**:
- `--username` (required): Your display name in the chat
- `--port` (default: 0): Port to listen on (0 for auto-assign)
- `--bootstrap`: Bootstrap node address with peer ID
- `--relay`: Relay node address with peer ID

**Features**:
- Interactive chat - type messages and press Enter to send
- Receives messages from other participants via GossipSub
- Both send and receive capability

## Troubleshooting

### Common Issues

**Problem**: "Failed to trigger bootstrap: No known peers"
- **Cause**: No bootstrap peers configured
- **Solution**: Ensure `--bootstrap` parameter is provided for relay/full nodes

**Problem**: "Peer is unroutable"
- **Cause**: Peer disconnected or not reachable
- **Solution**: Check network connectivity and firewall settings

**Problem**: "GossipSub: Peer does not support GossipSub"
- **Cause**: Peer doesn't have GossipSub enabled
- **Solution**: Ensure both peers support GossipSub protocol

### Log Levels

```bash
# Info level (default)
RUST_LOG=info ./gigi-node --mode bootstrap --listen /ip4/0.0.0.0/tcp/4001

# Debug level (includes ping events)
RUST_LOG=debug ./gigi-node --mode bootstrap --listen /ip4/0.0.0.0/tcp/4001

# Only debug for gigi-node module
RUST_LOG=gigi_node=debug ./gigi-node --mode bootstrap --listen /ip4/0.0.0.0/tcp/4001
```

## Examples

### group_messaging.rs

In-process test demonstrating:
- Multi-node P2P network
- DHT peer discovery
- GossipSub pub/sub messaging
- Message propagation across the network

### docker_group_client.rs

Standalone client for Docker deployment:
- Uses clap for CLI argument parsing
- `--username`: Display name in the chat
- `--port`: Listen port (0 for auto-assign)
- `--bootstrap`: Bootstrap node address with peer ID
- `--relay`: Relay node address with peer ID
- Interactive stdin/stdout chat interface
- Both send and receive capability via GossipSub

## License

Same as Gigi project