# Gigi OpenClaw Plugin

The Gigi OpenClaw Plugin integrates the Gigi P2P network with OpenClaw, enabling P2P messaging and file sharing through the OpenClaw interface. This guide provides detailed information about the plugin's functionality, configuration, and usage.

## Overview

The Gigi OpenClaw Plugin acts as a bridge between the OpenClaw interface and the Gigi P2P network, allowing users to send messages, join groups, and share files through a familiar interface without directly interacting with the underlying P2P stack.

### Key Features

- **P2P Messaging**: Direct peer-to-peer messaging with end-to-end encryption
- **Group Messaging**: Messages to groups using GossipSub protocol
- **File Sharing**: Share files between peers with share codes
- **Peer Discovery**: Find peers using Kademlia DHT and mDNS
- **NAT Traversal**: Connect peers behind NAT using circuit relay
- **Status Monitoring**: Health checks and connection status
- **Message Queuing**: Reliable message delivery with retries

## Installation

### Prerequisites

- **OpenClaw**: Latest version
- **Node.js**: v18 or later
- **pnpm**: Latest version

### Installation Steps

1. **Clone the Gigi repository**:
   ```bash
   git clone https://github.com/gigi-project/gigi.git
   cd gigi
   ```

2. **Install dependencies**:
   ```bash
   pnpm install
   ```

3. **Build the plugin**:
   ```bash
   cd pkgs/gigi-openclaw
   pnpm run build
   ```

4. **Add the plugin to OpenClaw**:
   ```bash
   openclaw plugins add --path ../pkgs/gigi-openclaw
   ```

## Configuration

The Gigi OpenClaw Plugin can be configured through the OpenClaw configuration system. Here's a comprehensive example:

### Basic Configuration

```json
{
  "channels": {
    "gigi": {
      "peerId": "your-peer-id",
      "multiaddrs": ["/ip4/0.0.0.0/tcp/0", "/ip4/0.0.0.0/ws/0"],
      "displayName": "Your Display Name",
      "bootstrapPeers": ["/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer"],
      "enableMdns": true,
      "enableDht": true,
      "enableRelay": true,
      "config": {
        "dmPolicy": "open",
        "allowFrom": ["*"]
      }
    }
  }
}
```

### Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `peerId` | Unique identifier for your node | Required |
| `multiaddrs` | List of addresses to listen on | Required |
| `displayName` | Human-readable name for your node | Same as peerId |
| `bootstrapPeers` | List of bootstrap nodes to connect to | Empty |
| `enableMdns` | Enable mDNS for local peer discovery | `true` |
| `enableDht` | Enable Kademlia DHT for peer discovery | `true` |
| `enableRelay` | Enable circuit relay for NAT traversal | `true` |
| `config.dmPolicy` | Direct message policy (`open`, `pairing`, `allowlist`) | `open` |
| `config.allowFrom` | List of allowed peer IDs or `["*"]` for all | `["*"]` |

## Usage

### Command-Line Interface

The Gigi OpenClaw Plugin provides a command-line interface through OpenClaw:

#### Add Gigi Channel

```bash
openclaw channels add gigi --peer-id <peerId> --multiaddrs <multiaddrs> --display-name "Your Name" --bootstrap-peers <bootstrapPeers>
```

#### Start Gigi Channel

```bash
openclaw channels start gigi
```

#### Stop Gigi Channel

```bash
openclaw channels stop gigi
```

#### Send Direct Message

```bash
openclaw send gigi --to <peerId> --message "Hello!"
```

#### Join Group

```bash
openclaw channels gigi join-group --group general
```

#### Send Group Message

```bash
openclaw send gigi --to group:general --message "Hello everyone!"
```

#### Share File

```bash
openclaw channels gigi share-file --path "/path/to/file.txt"
```

#### Download File

```bash
openclaw channels gigi download-file --from <peerId> --share-code <shareCode>
```

#### Check Status

```bash
openclaw channels status gigi
```

### Programmatic Usage

You can also use the Gigi OpenClaw Plugin programmatically in your own applications:

```typescript
import { gigiPlugin, GigiClient } from '@gigi/openclaw';

// Create a Gigi client
const client = new GigiClient({
  peerId: 'your-peer-id',
  multiaddrs: ['/ip4/0.0.0.0/tcp/0', '/ip4/0.0.0.0/ws/0'],
  displayName: 'Your Display Name',
  bootstrapPeers: ['/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer'],
});

// Start the client
await client.start();

// Send a message
await client.sendMessage('peer-id', 'Hello!');

// Join a group
await client.joinGroup('general');

// Send a group message
await client.sendGroupMessage('general', 'Hello everyone!');

// Share a file
const shareCode = await client.shareFile('/path/to/file.txt');

// Download a file
const downloadId = await client.downloadFile('peer-id', shareCode);

// Listen for messages
client.onMessage((message) => {
  console.log(`Received message from ${message.from}: ${message.content}`);
});

// Stop the client
await client.stop();
```

## Architecture

### Plugin Structure

The Gigi OpenClaw Plugin consists of several key components:

1. **GigiClient**: Wraps the P2P client and provides a high-level API
2. **OutboundManager**: Handles message queuing and retries
3. **Channel Plugin**: Implements the OpenClaw plugin interface
4. **Account Management**: Handles account configuration and validation
5. **Status Monitoring**: Provides health checks and status information

### Data Flow

1. **Message Creation**: User creates a message in OpenClaw
2. **Message Processing**: Plugin processes and validates the message
3. **P2P Delivery**: Message is sent through the P2P network
4. **Message Reception**: Plugin receives message from P2P network
5. **Message Delivery**: Message is delivered to OpenClaw

## Security

### Authentication

- **Peer Verification**: Peers are verified by their public keys
- **Access Control**: Configure who can send you messages using `dmPolicy` and `allowFrom`
- **Encryption**: All communications are encrypted using Libp2p's noise protocol

### Best Practices

- **Use Strong Peer IDs**: Generate secure peer IDs
- **Limit Allow List**: Restrict `allowFrom` to trusted peers
- **Monitor Connections**: Regularly check connected peers
- **Update Regularly**: Keep the plugin updated to the latest version

## Troubleshooting

### Common Issues

#### Connection Problems

- **Symptom**: Cannot connect to other peers
- **Solution**: Check network connectivity, firewall settings, and bootstrap peers

#### Peer Discovery

- **Symptom**: Cannot find other peers
- **Solution**: Ensure mDNS and DHT are enabled, check network multicast settings

#### File Transfer

- **Symptom**: File transfer fails
- **Solution**: Check file permissions, disk space, and network stability

#### Group Messaging

- **Symptom**: Group messages not received
- **Solution**: Ensure all peers are subscribed to the same group topic

### Logs

The Gigi OpenClaw Plugin logs are integrated with OpenClaw's logging system. To enable debug logging:

```bash
openclaw config set --key logging.level --value debug
```

## Advanced Features

### Custom Network

You can set up a custom Gigi network by running your own bootstrap nodes:

```bash
# Start a bootstrap node
cd apps/gigi-node
cargo run -- --mode bootstrap --listen /ip4/0.0.0.0/tcp/4001

# Configure the plugin to use your bootstrap node
openclaw channels add gigi --peer-id <peerId> --multiaddrs <multiaddrs> --bootstrap-peers "/ip4/your-server-ip/tcp/4001/p2p/<bootstrap-peer-id>"
```

### NAT Traversal

The plugin automatically handles NAT traversal using Libp2p's circuit relay protocol. For better results, you can run dedicated relay nodes:

```bash
# Start a relay node
cd apps/gigi-node
cargo run -- --mode relay --listen /ip4/0.0.0.0/tcp/4002 --bootstrap /ip4/127.0.0.1/tcp/4001/p2p/<bootstrap-peer-id>
```

### Performance Optimization

To optimize performance:

- **Limit Connections**: Configure max connections in P2P settings
- **Enable Relay**: Use relay nodes for better NAT traversal
- **Optimize DHT**: Adjust DHT parameters for your network size
- **Use Fast Transports**: Prioritize faster transports like QUIC

## API Reference

### Plugin Methods

#### `listAccountIds(cfg)`

List all configured Gigi account IDs.

**Parameters**:
- `cfg`: Configuration object

**Returns**:
- Array of account IDs

#### `resolveAccount({ cfg, accountId })`

Resolve a Gigi account by ID.

**Parameters**:
- `cfg`: Configuration object
- `accountId`: Account ID to resolve

**Returns**:
- GigiAccount object or null

#### `startAccount(ctx)`

Start a gateway for an account.

**Parameters**:
- `ctx`: Context object with account information

**Returns**:
- Promise<void>

#### `stopAccount(accountId)`

Stop a gateway for an account.

**Parameters**:
- `accountId`: Account ID to stop

**Returns**:
- Promise<void>

#### `sendMessage(ctx)`

Send a message through the gateway.

**Parameters**:
- `ctx`: Context object with message information

**Returns**:
- Promise<void>

### GigiClient Methods

#### `start()`

Start the P2P client.

**Returns**:
- Promise<void>

#### `stop()`

Stop the P2P client.

**Returns**:
- Promise<void>

#### `sendMessage(targetPeerId, content)`

Send a direct message to a peer.

**Parameters**:
- `targetPeerId`: Target peer ID
- `content`: Message content

**Returns**:
- Promise<void>

#### `sendGroupMessage(groupName, content)`

Send a message to a group.

**Parameters**:
- `groupName`: Group name
- `content`: Message content

**Returns**:
- Promise<void>

#### `joinGroup(groupName)`

Join a group.

**Parameters**:
- `groupName`: Group name

**Returns**:
- Promise<void>

#### `leaveGroup(groupName)`

Leave a group.

**Parameters**:
- `groupName`: Group name

**Returns**:
- Promise<void>

#### `shareFile(filePath)`

Share a file.

**Parameters**:
- `filePath`: Path to file

**Returns**:
- Promise<string> (share code)

#### `downloadFile(peerId, shareCode)`

Download a file.

**Parameters**:
- `peerId`: Peer ID to download from
- `shareCode`: Share code

**Returns**:
- Promise<string> (download ID)

#### `onMessage(handler)`

Register a message handler.

**Parameters**:
- `handler`: Message handler function

#### `getPeerId()`

Get the peer ID of the client.

**Returns**:
- string (peer ID)

#### `getMultiaddrs()`

Get the multiaddresses the client is listening on.

**Returns**:
- Array of strings (multiaddresses)

#### `isConnected()`

Check if the client is connected to the network.

**Returns**:
- boolean

#### `listPeers()`

List all discovered peers.

**Returns**:
- Array of peer objects

#### `listGroups()`

List all joined groups.

**Returns**:
- Array of group objects

## Examples

### Basic Messaging

```bash
# Add Gigi channel
openclaw channels add gigi --peer-id "my-peer-id" --multiaddrs "/ip4/0.0.0.0/tcp/0"

# Start channel
openclaw channels start gigi

# Send message
openclaw send gigi --to "friend-peer-id" --message "Hello from Gigi!"
```

### Group Chat

```bash
# Join group
openclaw channels gigi join-group --group "family"

# Send group message
openclaw send gigi --to group:family --message "Hi everyone!"
```

### File Sharing

```bash
# Share file
openclaw channels gigi share-file --path "/path/to/photo.jpg"
# Output: Share code: abc123

# Download file
openclaw channels gigi download-file --from "friend-peer-id" --share-code "abc123"
```

## Conclusion

The Gigi OpenClaw Plugin provides a powerful interface to the Gigi P2P network, enabling secure, decentralized communication and file sharing. By following this guide, you can configure and use the plugin to its full potential, creating a robust P2P communication system that works across networks and devices.

For more information, see the [API Reference](api/openclaw-plugin-api.md) and [Troubleshooting Guide](guides/troubleshooting-guide.md).