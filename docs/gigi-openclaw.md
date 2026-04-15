# Gigi OpenClaw Plugin

The Gigi OpenClaw Plugin integrates the Gigi P2P network with OpenClaw, enabling P2P messaging and file sharing through the OpenClaw interface. This guide provides detailed information about the plugin's functionality, configuration, and usage.

## Overview

The Gigi OpenClaw Plugin acts as a bridge between the OpenClaw interface and the Gigi P2P network, allowing users to send messages, join groups, and share files through a familiar interface without directly interacting with the underlying P2P stack. The plugin leverages the Agent Messaging Protocol (AMP) for standardized message formatting and routing.

### Key Features

- **P2P Messaging**: Direct peer-to-peer messaging with end-to-end encryption
- **Group Messaging**: Messages to groups using GossipSub protocol
- **File Sharing**: Share files between peers with share codes (auto-download on receive)
- **Peer Discovery**: Find peers using Kademlia DHT and mDNS
- **NAT Traversal**: Connect peers behind NAT using circuit relay
- **Status Monitoring**: Health checks and connection status
- **Message Queuing**: Reliable message delivery with retries
- **Mnemonic Support**: BIP-39 mnemonic for consistent peer ID derivation
- **Agent Messaging Protocol**: Standardized message format for agent communication
- **Gateway Architecture**: Per-account gateway instances for isolation
- **Security Policies**: Configurable DM and group policies

## Installation

### Prerequisites

- **OpenClaw**: 2026.3.22 or later
- **Node.js**: v18 or later
- **pnpm**: Latest version

### Installation Steps

1. **Clone the Gigi repository**:
   ```bash
   git clone https://github.com/Lebei2046/gigi.git
   cd gigi
   ```

2. **Install dependencies**:
   ```bash
   pnpm install
   ```

3. **Build the plugin**:
   ```bash
   cd apps/gigi-openclaw
   pnpm run build:bundle
   ```

4. **Install the plugin in OpenClaw**:
   ```bash
   openclaw plugins install /path/to/gigi/apps/gigi-openclaw
   ```

## Configuration

The Gigi OpenClaw Plugin can be configured through the OpenClaw configuration system.

### Basic Configuration

```json
{
  "channels": {
    "gigi-openclaw": {
      "mnemonic": "abandon amount liar amount expire adjust cage candy arch gather drum buyer",
      "multiaddrs": [
        "/ip4/0.0.0.0/tcp/0",
        "/ip4/0.0.0.0/tcp/0/ws"
      ],
      "displayName": "My Gigi Node",
      "nickname": "My Gigi Node",
      "enabled": true,
      "config": {
        "dmPolicy": "open",
        "allowFrom": ["*"],
        "groupPolicy": "open",
        "agents": {}
      }
    }
  }
}
```

### Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `mnemonic` | BIP-39 mnemonic for identity | Generated during setup |
| `multiaddrs` | Network addresses to listen on | `["/ip4/0.0.0.0/tcp/0", "/ip4/0.0.0.0/tcp/0/ws"]` |
| `displayName` | Human-readable display name | `"My Gigi Node"` |
| `nickname` | Network nickname | Same as displayName |
| `bootstrapPeers` | Bootstrap nodes for network discovery | `[]` |
| `enableMdns` | Enable mDNS for local discovery | `true` |
| `enableDht` | Enable Kademlia DHT | `true` |
| `enableRelay` | Enable circuit relay | `true` |
| `enabled` | Whether the channel is enabled | `true` |
| `config.allowFrom` | List of allowed peer IDs | `[]` |
| `config.dmPolicy` | Direct message policy (open/pairing) | `"open"` |
| `config.groupPolicy` | Group message policy (open/allowlist) | `"open"` |
| `config.groupAllowFrom` | List of allowed groups | `[]` |
| `config.agents` | Agent configurations | `{}` |

### Generating a Mnemonic

To generate a BIP-39 mnemonic phrase for consistent peer ID:

```bash
cd apps/gigi-openclaw
pnpm run generate-mnemonic
```

**Important**: Store your mnemonic securely. It is the root of all your Gigi P2P keys.

## Usage

### Command-Line Interface

The Gigi OpenClaw Plugin provides a command-line interface through OpenClaw:

#### Add Gigi Channel

```bash
openclaw channels add gigi-openclaw --name "My Gigi Node"
```

#### Start Gigi Channel

```bash
openclaw channels start gigi-openclaw
```

#### Stop Gigi Channel

```bash
openclaw channels stop gigi-openclaw
```

#### Send Direct Message

```bash
openclaw message send --channel gigi-openclaw --target <peerId> --message "Hello!"
```

#### Send Group Message

```bash
openclaw message send --channel gigi-openclaw --target "group:test-group" --message "Hello everyone!"
```

#### Check Status

```bash
openclaw channels status gigi-openclaw
```

### Programmatic Usage

You can also use the Gigi OpenClaw Plugin programmatically in your own applications:

```typescript
import { GigiClient } from "gigi-openclaw";

// Create a Gigi client
const client = new GigiClient({
  multiaddrs: ["/ip4/0.0.0.0/tcp/0", "/ip4/0.0.0.0/tcp/0/ws"],
  displayName: "My Gigi Node",
  nickname: "My Gigi Node",
  mnemonic: "your mnemonic here",
  bootstrapPeers: [],
  enableMdns: true,
  enableDht: true,
  enableRelay: true
});

// Start the client
await client.start();

// Send a direct message
await client.sendMessage("peer-id", "Hello!");

// Join a group
await client.joinGroup("my-group-topic");

// Send a group message
await client.sendGroupMessage("my-group-topic", "Hello everyone!");

// Share a file
const shareCode = await client.shareFile("/path/to/file.txt");

// Download a file
const downloadId = await client.downloadFile("peer-id", shareCode);

// List joined groups
const groups = client.listGroups();
console.log("Joined groups:", groups);

// Leave a group
await client.leaveGroup("my-group-topic");

// Stop the client
await client.stop();
```

## Architecture

### Plugin Structure

The Gigi OpenClaw Plugin consists of several key components:

1. **GigiClient** (`src/GigiClient.ts`): Wraps the P2P client and provides a high-level API
2. **OutboundManager** (`src/outbound.ts`): Handles message queuing and retries
3. **Channel Plugin** (`src/channel.ts`): Implements the OpenClaw ChannelPlugin interface
4. **Account Management** (`src/accounts.ts`): Handles account configuration and validation
5. **Status Monitoring** (`src/probe.ts`): Provides health checks and status information
6. **Type Definitions** (`src/types.ts`): TypeScript interfaces and types
7. **Configuration Schema** (`src/config-schema.ts`): Configuration validation schema

### Gateway Architecture

The plugin uses a gateway architecture where each account has its own gateway instance:

```mermaid
flowchart TD
    A[OpenClaw Core] -->|Message| B[Gigi Channel Plugin]
    B -->|Start Gateway| C[Gateway Context]
    C -->|Create| D[GigiClient]
    C -->|Create| E[OutboundManager]
    D -->|P2P Communication| F[Gigi Network]
    E -->|Queue Messages| D
    F -->|Incoming Message| D
    D -->|Process| G[Message Handler]
    G -->|Dispatch| H[OpenClaw Agents]
```

### Data Flow

1. **Message Creation**: User creates a message in OpenClaw
2. **Gateway Check**: Plugin checks if gateway is active and connected
3. **Message Formatting**: Message is formatted as AMP message
4. **P2P Delivery**: Message is sent through the P2P network via OutboundManager
5. **Message Reception**: Plugin receives message from P2P network
6. **Message Processing**: Message is parsed and converted to AMP format
7. **Agent Routing**: Message is routed to appropriate OpenClaw agents
8. **Message Delivery**: Message is delivered to OpenClaw

### File Sharing Flow

1. **Sharing**: User shares a file via `shareFile()`
2. **Share Code**: A unique share code is generated
3. **Message Creation**: File share message is created with share code
4. **P2P Delivery**: Message is sent to peers
5. **Auto-download**: Recipients automatically download the file
6. **Storage**: Files are saved to appropriate directory

## Security

### Authentication

- **Peer Verification**: Peers are verified by their public keys
- **Access Control**: Configure who can send you messages using `dmPolicy` and `allowFrom`
- **Encryption**: All communications are encrypted using Libp2p's noise protocol

### Best Practices

- **Use Mnemonic**: Generate and securely store a BIP-39 mnemonic for consistent identity
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

## API Reference

### GigiClient Methods

#### `start()`

Start the P2P client.

**Returns**: `Promise<void>`

#### `stop()`

Stop the P2P client.

**Returns**: `Promise<void>`

#### `sendMessage(targetPeerId, content)`

Send a direct message to a peer.

**Parameters**:
- `targetPeerId`: Target peer ID (string)
- `content`: Message content (string)

**Returns**: `Promise<void>`

#### `sendGroupMessage(groupName, content)`

Send a message to a group.

**Parameters**:
- `groupName`: Group name (string)
- `content`: Message content (string or file share object)

**Returns**: `Promise<void>`

#### `joinGroup(groupName)`

Join a group.

**Parameters**:
- `groupName`: Group name (string)

**Returns**: `Promise<void>`

#### `leaveGroup(groupName)`

Leave a group.

**Parameters**:
- `groupName`: Group name (string)

**Returns**: `Promise<void>`

#### `shareFile(filePath)`

Share a file.

**Parameters**:
- `filePath`: Path to file (string)

**Returns**: `Promise<string>` (share code)

#### `downloadFile(peerId, shareCode)`

Download a file.

**Parameters**:
- `peerId`: Peer ID to download from (string)
- `shareCode`: Share code (string)

**Returns**: `Promise<string>` (download ID)

#### `onMessage(handler)`

Register a message handler.

**Parameters**:
- `handler`: Message handler function `(msg: GigiMessage) => void`

#### `getPeerId()`

Get the peer ID of the client.

**Returns**: `string`

#### `getMultiaddrs()`

Get the multiaddresses the client is listening on.

**Returns**: `string[]`

#### `isConnected()`

Check if the client is connected to the network.

**Returns**: `boolean`

#### `listPeers()`

List all discovered peers.

**Returns**: `Array<{ peerId: string; nickname?: string }>`

#### `listGroups()`

List all joined groups.

**Returns**: `Array<{ name: string; members?: string[] }>`

#### `getFileByShareCode(shareCode)`

Get file information by share code.

**Parameters**:
- `shareCode`: Share code (string)

**Returns**: `any`

## Examples

### Basic Messaging

```bash
# Add Gigi channel
openclaw channels add gigi-p2p-bundled --name "My Gigi Node"

# Start channel
openclaw channels start gigi-p2p-bundled

# Send message
openclaw message send --channel gigi-p2p-bundled --target "friend-peer-id" --message "Hello from Gigi!"
```

### Group Chat

```bash
# Send group message
openclaw message send --channel gigi-p2p-bundled --target "test-group" --message "Hi everyone!"
```

### File Sharing

Files are automatically downloaded when a file share message is received. The downloaded files are saved to the `./downloads` directory relative to the OpenClaw gateway's working directory.

To share a file programmatically:

```typescript
const shareCode = await client.shareFile("/path/to/photo.jpg");
const fileShareMessage = {
  type: 'fileShare',
  shareCode,
  filename: 'photo.jpg',
  fileSize: 1024000,
  fileType: 'image/jpeg'
};
await client.sendGroupMessage("family", fileShareMessage);
```

## File Structure

```
apps/gigi-openclaw/
├── src/
│   ├── channel.ts      # Main plugin implementation
│   ├── GigiClient.ts   # High-level client wrapper
│   ├── outbound.ts     # Message queue management
│   ├── accounts.ts     # Account resolution
│   ├── probe.ts        # Status monitoring
│   ├── types.ts        # Type definitions
│   └── config-schema.ts # Configuration schema
├── scripts/
│   └── generate-mnemonic.ts # Mnemonic generation script
├── package.json
├── tsconfig.json
└── README.md
```

## License

MIT
