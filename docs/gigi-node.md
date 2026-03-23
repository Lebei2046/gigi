# Gigi Network Node

The Gigi Network Node is a standalone Gigi P2P network node that can operate as a bootstrap node, relay node, or full node. These nodes form the backbone of the Gigi P2P network, providing infrastructure services to other peers. This guide provides detailed information about the network node's functionality, configuration, and usage.

## Overview

The Gigi Network Node is designed to provide network infrastructure for the Gigi P2P ecosystem, including bootstrap services for new nodes, relay services for NATed peers, and general network stability. It can be deployed in various modes to serve different purposes within the network.

### Key Features

- **Bootstrap Mode**: Provide DHT entry points for new nodes
- **Relay Mode**: Help NATed peers connect to the network
- **Full Mode**: Combine bootstrap and relay capabilities
- **Docker Support**: Easy deployment with Docker
- **Network Monitoring**: Track network health and performance
- **Scalable**: Handle multiple connections efficiently
- **Secure**: Built on Libp2p's security features

## Installation

### Prerequisites

- **Rust**: v1.60 or later
- **Cargo**: Latest version
- **Docker** (optional): For containerized deployment

### Installation Steps

1. **Clone the Gigi repository**:
   ```bash
   git clone https://github.com/gigi-project/gigi.git
   cd gigi
   ```

2. **Build the network node**:
   ```bash
   cd apps/gigi-node
   cargo build --release
   ```

3. **Run the node**:
   ```bash
   ./target/release/gigi-node --mode bootstrap --listen /ip4/0.0.0.0/tcp/4001
   ```

## Configuration

The Gigi Network Node can be configured through command-line arguments and environment variables:

### Command-Line Arguments

| Argument | Description | Default |
|----------|-------------|---------|
| `--mode` | Node mode (`bootstrap`, `relay`, `full`) | `full` |
| `--listen` | Address to listen on | `/ip4/0.0.0.0/tcp/4001` |
| `--bootstrap` | Bootstrap nodes to connect to | None |
| `--relay-hop-limit` | Maximum number of relay hops | `3` |
| `--max-connections` | Maximum number of connections | `100` |
| `--log-level` | Log level (`debug`, `info`, `warn`, `error`) | `info` |
| `--data-dir` | Directory to store data | `~/.gigi/node` |

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `GIGI_NODE_MODE` | Node mode | `full` |
| `GIGI_NODE_LISTEN` | Address to listen on | `/ip4/0.0.0.0/tcp/4001` |
| `GIGI_NODE_BOOTSTRAP` | Bootstrap nodes | None |
| `GIGI_NODE_RELAY_HOP_LIMIT` | Maximum relay hops | `3` |
| `GIGI_NODE_MAX_CONNECTIONS` | Maximum connections | `100` |
| `GIGI_NODE_LOG_LEVEL` | Log level | `info` |
| `GIGI_NODE_DATA_DIR` | Data directory | `~/.gigi/node` |

## Usage

### Basic Usage

#### Start a Bootstrap Node

```bash
# Start a bootstrap node
./gigi-node --mode bootstrap --listen /ip4/0.0.0.0/tcp/4001
```

#### Start a Relay Node

```bash
# Start a relay node
./gigi-node --mode relay --listen /ip4/0.0.0.0/tcp/4002 --bootstrap /ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer
```

#### Start a Full Node

```bash
# Start a full node
./gigi-node --mode full --listen /ip4/0.0.0.0/tcp/4003 --bootstrap /ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer
```

### Docker Usage

#### Build Docker Image

```bash
# From the gigi-node directory
docker build -t gigi-node .
```

#### Run Docker Container

```bash
# Run a bootstrap node
docker run -p 4001:4001 gigi-node --mode bootstrap --listen /ip4/0.0.0.0/tcp/4001

# Run a relay node
docker run -p 4002:4002 gigi-node --mode relay --listen /ip4/0.0.0.0/tcp/4002 --bootstrap /ip4/host.docker.internal/tcp/4001/p2p/QmBootstrapPeer
```

### Network Setup Scripts

The Gigi Network Node comes with helper scripts to set up a network:

#### Start Network

```bash
# Start a network with bootstrap, relay, and full nodes
./start-network.sh

# Start specific node types
./start-network.sh bootstrap
./start-network.sh relay
./start-network.sh full
```

#### Stop Network

```bash
# Stop all nodes
./stop-network.sh
```

## Node Modes

### Bootstrap Mode

**Purpose**: Provide DHT entry points for new nodes joining the network.

**Key Features**:
- Maintains a DHT (Distributed Hash Table) for peer discovery
- Provides initial peer connections for new nodes
- Helps new nodes find other peers in the network

**Use Case**: Deploy as a public bootstrap node for the Gigi network, or as the first node in a private network.

### Relay Mode

**Purpose**: Help NATed peers connect to the network by acting as a relay.

**Key Features**:
- Provides circuit relay functionality
- Helps peers behind NAT/firewall connect to the network
- Improves network connectivity for all peers

**Use Case**: Deploy in cloud environments to improve connectivity for peers with restricted network access.

### Full Mode

**Purpose**: Combine both bootstrap and relay capabilities.

**Key Features**:
- Provides DHT entry points for new nodes
- Offers relay services for NATed peers
- Acts as a comprehensive network infrastructure node

**Use Case**: Deploy as a multi-purpose network node in smaller networks or as a backup for larger networks.

## Architecture

### Node Structure

The Gigi Network Node consists of several key components:

1. **Node Core**: Main node logic and configuration
2. **Libp2p Integration**: Handles P2P networking
3. **DHT Service**: Provides distributed hash table functionality
4. **Relay Service**: Provides circuit relay functionality
5. **Monitoring**: Tracks network health and performance
6. **Logging**: Records node activity and events

### Data Flow

1. **Node Initialization**: Node is started with specified mode and configuration
2. **Network Connection**: Node connects to existing network or starts a new one
3. **Service Activation**: Relevant services (DHT, relay) are activated based on mode
4. **Peer Interaction**: Node handles peer discovery, connection requests, and relay traffic
5. **Monitoring**: Node tracks network health and performance metrics

## Security

### Authentication

- **Peer Verification**: Peers are verified by their public keys
- **Encryption**: All communications are encrypted using Libp2p's noise protocol
- **Access Control**: Implement firewall rules to restrict access if needed

### Best Practices

- **Use Secure Transport**: Always use encrypted transports
- **Limit Exposure**: Only expose necessary ports
- **Monitor Connections**: Regularly check connected peers
- **Update Regularly**: Keep the node updated to the latest version
- **Use Firewalls**: Configure firewalls to restrict access to trusted sources

## Troubleshooting

### Common Issues

#### Connection Problems

- **Symptom**: Node cannot connect to the network
- **Solution**: Check network connectivity, firewall settings, and bootstrap nodes

#### High CPU Usage

- **Symptom**: Node is using high CPU
- **Solution**: Reduce max connections, check for network issues, or upgrade hardware

#### Memory Leaks

- **Symptom**: Node memory usage keeps increasing
- **Solution**: Update to the latest version, check for memory leaks in custom configurations

#### Relay Failures

- **Symptom**: Relay functionality not working
- **Solution**: Check network configuration, ensure relay mode is enabled, verify firewall settings

### Debugging

Enable debug logging to troubleshoot issues:

```bash
# Start node with debug logging
./gigi-node --mode bootstrap --listen /ip4/0.0.0.0/tcp/4001 --log-level debug
```

### Monitoring

Monitor node performance and network health:

```bash
# Check node status
curl http://localhost:4001/status

# Check connected peers
curl http://localhost:4001/peers

# Check DHT status
curl http://localhost:4001/dht
```

## Advanced Features

### Custom Network

Set up a private Gigi network with multiple nodes:

```bash
# Start first bootstrap node
./gigi-node --mode bootstrap --listen /ip4/0.0.0.0/tcp/4001

# Start relay node
./gigi-node --mode relay --listen /ip4/0.0.0.0/tcp/4002 --bootstrap /ip4/127.0.0.1/tcp/4001/p2p/$(cat ~/.gigi/node/id)

# Start additional full nodes
./gigi-node --mode full --listen /ip4/0.0.0.0/tcp/4003 --bootstrap /ip4/127.0.0.1/tcp/4001/p2p/$(cat ~/.gigi/node/id)
```

### Load Balancing

Deploy multiple relay nodes for load balancing:

```bash
# Start multiple relay nodes
./gigi-node --mode relay --listen /ip4/0.0.0.0/tcp/4002 --bootstrap /ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer
./gigi-node --mode relay --listen /ip4/0.0.0.0/tcp/4003 --bootstrap /ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer
./gigi-node --mode relay --listen /ip4/0.0.0.0/tcp/4004 --bootstrap /ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer
```

### High Availability

Set up a high-availability network with multiple bootstrap nodes:

```bash
# Start first bootstrap node
./gigi-node --mode bootstrap --listen /ip4/0.0.0.0/tcp/4001

# Start second bootstrap node connected to the first
./gigi-node --mode bootstrap --listen /ip4/0.0.0.0/tcp/4002 --bootstrap /ip4/127.0.0.1/tcp/4001/p2p/$(cat ~/.gigi/node1/id)

# Configure clients to use both bootstrap nodes
# bootstrapPeers: ["/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrap1", "/ip4/127.0.0.1/tcp/4002/p2p/QmBootstrap2"]
```

## API Reference

### Command-Line Interface

#### `gigi-node --mode <mode> --listen <address>`

Start a Gigi network node.

**Parameters**:
- `--mode`: Node mode (`bootstrap`, `relay`, `full`)
- `--listen`: Address to listen on
- `--bootstrap`: Bootstrap nodes to connect to (optional)
- `--relay-hop-limit`: Maximum number of relay hops (optional)
- `--max-connections`: Maximum number of connections (optional)
- `--log-level`: Log level (optional)
- `--data-dir`: Directory to store data (optional)

### HTTP API

The Gigi Network Node provides an HTTP API for monitoring and management:

#### `GET /status`

Get node status.

**Response**:
```json
{
  "mode": "bootstrap",
  "peerId": "QmPeerId",
  "multiaddrs": ["/ip4/127.0.0.1/tcp/4001/p2p/QmPeerId"],
  "connections": 10,
  "uptime": 3600
}
```

#### `GET /peers`

List connected peers.

**Response**:
```json
{
  "peers": [
    {
      "id": "QmPeerId1",
      "multiaddrs": ["/ip4/192.168.1.100/tcp/5001/p2p/QmPeerId1"],
      "connectedAt": "2023-01-01T00:00:00Z"
    }
  ]
}
```

#### `GET /dht`

Get DHT status.

**Response**:
```json
{
  "size": 100,
  "closestPeers": ["QmPeerId1", "QmPeerId2"],
  "isBootstrap": true
}
```

#### `GET /relay`

Get relay status.

**Response**:
```json
{
  "enabled": true,
  "relayHops": 3,
  "activeRelays": 5
}
```

## Examples

### Basic Network Setup

```bash
# Start a bootstrap node
./gigi-node --mode bootstrap --listen /ip4/0.0.0.0/tcp/4001

# Start a relay node
./gigi-node --mode relay --listen /ip4/0.0.0.0/tcp/4002 --bootstrap /ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer

# Start a full node
./gigi-node --mode full --listen /ip4/0.0.0.0/tcp/4003 --bootstrap /ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer
```

### Docker Compose Setup

```yaml
# docker-compose.yml
version: '3'
services:
  bootstrap:
    build: .
    ports:
      - "4001:4001"
    command: --mode bootstrap --listen /ip4/0.0.0.0/tcp/4001
  relay:
    build: .
    ports:
      - "4002:4002"
    command: --mode relay --listen /ip4/0.0.0.0/tcp/4002 --bootstrap /ip4/bootstrap/tcp/4001/p2p/QmBootstrapPeer
    depends_on:
      - bootstrap
  full:
    build: .
    ports:
      - "4003:4003"
    command: --mode full --listen /ip4/0.0.0.0/tcp/4003 --bootstrap /ip4/bootstrap/tcp/4001/p2p/QmBootstrapPeer
    depends_on:
      - bootstrap
```

### Cloud Deployment

Deploy a Gigi network node on a cloud server:

```bash
# SSH to cloud server
ssh user@cloud-server

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/gigi-project/gigi.git
cd gigi/apps/gigi-node

# Build node
cargo build --release

# Start bootstrap node
./target/release/gigi-node --mode bootstrap --listen /ip4/0.0.0.0/tcp/4001 &

# Configure firewall
ufw allow 4001/tcp

# Set up systemd service
cat > /etc/systemd/system/gigi-node.service << EOF
[Unit]
Description=Gigi Network Node
After=network.target

[Service]
User=user
WorkingDirectory=/home/user/gigi/apps/gigi-node
ExecStart=/home/user/gigi/apps/gigi-node/target/release/gigi-node --mode bootstrap --listen /ip4/0.0.0.0/tcp/4001
Restart=always

[Install]
WantedBy=multi-user.target
EOF

systemctl enable gigi-node
systemctl start gigi-node
```

## Conclusion

The Gigi Network Node provides essential infrastructure for the Gigi P2P ecosystem, enabling reliable peer discovery and NAT traversal. By following this guide, you can deploy and configure network nodes to support your Gigi P2P network, whether for personal use, small teams, or large-scale deployments.

For more information, see the [API Reference](api/node-api.md) and [Troubleshooting Guide](guides/troubleshooting-guide.md).