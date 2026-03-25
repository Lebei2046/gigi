# Gigi P2P TypeScript Client

The Gigi P2P TypeScript Client is a TypeScript implementation of the Gigi P2P client, providing a high-level API for P2P communication, group messaging, and file sharing. This guide provides detailed information about the client's functionality, configuration, and usage.

## Overview

The Gigi P2P TypeScript Client is designed for use in web browsers, Node.js, and TypeScript/JavaScript applications. It provides a TypeScript-friendly interface to the Gigi P2P network, making it easy to integrate P2P functionality into web applications and services.

### Key Features

- **Libp2p Integration**: Built on Libp2p for robust P2P networking
- **Direct Messaging**: Send messages to specific peers
- **Group Messaging**: Use GossipSub for group communication
- **File Sharing**: Share and download files between peers
- **Peer Management**: Discover and manage connected peers
- **Event System**: Listen for network events
- **Error Handling**: Comprehensive error handling and retry mechanisms

## Installation

### Prerequisites

- **Node.js**: v18 or later
- **Bun**: Latest version

### Installation Steps

1. **Clone the Gigi repository**:
   ```bash
   git clone https://github.com/gigi-project/gigi.git
   cd gigi
   ```

2. **Install dependencies**:
   ```bash
   bun install
   ```

3. **Build the TypeScript client**:
   ```bash
   cd typescript/gigi-p2p-ts
   bun run build
   ```

4. **Install the client in your project**:
   ```bash
   # From your project directory
   bun add ../gigi/typescript/gigi-p2p-ts
   ```

## Configuration

The Gigi P2P TypeScript Client can be configured with various options to customize its behavior:

### Basic Configuration

```typescript
const clientConfig = {
  nickname: 'My Node',
  config: {
    bootstrapNodes: ['/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer'],
    enableKademlia: true,
    enableRelay: true,
    enableMdns: true,
    listenAddrs: ['/ip4/0.0.0.0/tcp/0', '/ip4/0.0.0.0/ws/0'],
  },
};
```

### Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `nickname` | Human-readable name for your node | `"Gigi Node"` |
| `config.bootstrapNodes` | List of bootstrap nodes to connect to | `[]` |
| `config.enableKademlia` | Enable Kademlia DHT for peer discovery | `true` |
| `config.enableRelay` | Enable circuit relay for NAT traversal | `true` |
| `config.enableMdns` | Enable mDNS for local peer discovery | `true` |
| `config.listenAddrs` | List of addresses to listen on | `['/ip4/0.0.0.0/tcp/0', '/ip4/0.0.0.0/ws/0']` |
| `config.maxConnections` | Maximum number of connections | `100` |
| `config.connectionTimeout` | Connection timeout in milliseconds | `30000` |
| `config.relayHopLimit` | Maximum number of relay hops | `3` |

## Usage

### Basic Usage

```typescript
import { P2pClient } from '@gigi/p2p-ts';

// Create client
const client = new P2pClient({
  nickname: 'My Node',
  config: {
    bootstrapNodes: ['/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer'],
    enableKademlia: true,
    enableRelay: true,
    enableMdns: true,
    listenAddrs: ['/ip4/0.0.0.0/tcp/0', '/ip4/0.0.0.0/ws/0'],
  },
});

// Start client
await client.start();
console.log(`Started with peer ID: ${client.getPeerId()}`);

// Send direct message
await client.sendDirectMessage('peer-id', 'Hello!');

// Join group
await client.joinGroup('general');

// Send group message
await client.sendGroupMessage('general', 'Hello everyone!');

// Share file
const shareCode = await client.shareFile('/path/to/file.txt');
console.log(`File shared with code: ${shareCode}`);

// Download file
const downloadId = await client.downloadFile('peer-id', shareCode);
console.log(`Download started with ID: ${downloadId}`);

// Listen for events
client.onEvent((event) => {
  if (event.type === 'direct-message') {
    console.log(`Received message from ${event.from}: ${event.message}`);
  } else if (event.type === 'group-message') {
    console.log(`Received group message from ${event.from} in ${event.group}: ${event.message}`);
  }
});

// Stop client
await client.stop();
```

### Advanced Usage

#### Custom Event Handling

```typescript
// Listen for specific events
client.onEvent((event) => {
  switch (event.type) {
    case 'peer-discovered':
      console.log(`Discovered peer: ${event.peerId}`);
      break;
    case 'peer-connected':
      console.log(`Connected to peer: ${event.peerId}`);
      break;
    case 'peer-disconnected':
      console.log(`Disconnected from peer: ${event.peerId}`);
      break;
    case 'direct-message':
      console.log(`Received message from ${event.from}: ${event.message}`);
      break;
    case 'group-message':
      console.log(`Received group message from ${event.from} in ${event.group}: ${event.message}`);
      break;
    case 'file-shared':
      console.log(`File shared with code: ${event.shareCode}`);
      break;
    case 'file-download-started':
      console.log(`File download started with ID: ${event.downloadId}`);
      break;
    case 'file-download-progress':
      console.log(`Download ${event.downloadId} progress: ${event.progress}%`);
      break;
    case 'file-download-completed':
      console.log(`Download ${event.downloadId} completed: ${event.filePath}`);
      break;
    case 'file-download-failed':
      console.log(`Download ${event.downloadId} failed: ${event.error}`);
      break;
    case 'error':
      console.error(`Error: ${event.error}`);
      break;
  }
});
```

#### Peer Management

```typescript
// List all discovered peers
const peers = await client.listPeers();
console.log(`Discovered ${peers.length} peers:`);
peers.forEach(peer => {
  console.log(`- ${peer.id}: ${peer.nickname || 'Unknown'}`);
});

// Connect to a specific peer
await client.connectToPeer('peer-id');

// Disconnect from a peer
await client.disconnectFromPeer('peer-id');
```

#### Group Management

```typescript
// List all joined groups
const groups = await client.listGroups();
console.log(`Joined ${groups.length} groups:`);
groups.forEach(group => {
  console.log(`- ${group.name}`);
});

// Join a group
await client.joinGroup('general');

// Leave a group
await client.leaveGroup('general');
```

#### File Sharing

```typescript
// Share a file
const shareCode = await client.shareFile('/path/to/file.txt');
console.log(`File shared with code: ${shareCode}`);

// Download a file
const downloadId = await client.downloadFile('peer-id', shareCode);
console.log(`Download started with ID: ${downloadId}`);

// Cancel a download
await client.cancelDownload(downloadId);

// List active downloads
const downloads = await client.listActiveDownloads();
console.log(`Active downloads: ${downloads.length}`);
downloads.forEach(download => {
  console.log(`- ${download.id}: ${download.progress}%`);
});
```

## Architecture

### Client Structure

The Gigi P2P TypeScript Client consists of several key components:

1. **P2pClient**: Main client class that orchestrates all functionality
2. **Libp2pSetup**: Configures and sets up the Libp2p instance
3. **PeerManager**: Manages peer discovery and connections
4. **GroupManager**: Handles group messaging and subscriptions
5. **FileSharing**: Manages file sharing and downloads
6. **EventEmitter**: Emits events for network activity

### Data Flow

1. **Client Initialization**: Client is created and configured
2. **Network Connection**: Client connects to the P2P network
3. **Peer Discovery**: Client discovers other peers
4. **Message Processing**: Client sends and receives messages
5. **File Transfer**: Client shares and downloads files

## Security

### Authentication

- **Peer Verification**: Peers are verified by their public keys
- **Encryption**: All communications are encrypted using Libp2p's noise protocol
- **Access Control**: Implement application-level access control

### Best Practices

- **Use Secure Transport**: Always use encrypted transports
- **Validate Peers**: Verify peer identities before communicating
- **Limit Exposure**: Only share necessary information
- **Update Regularly**: Keep the client updated to the latest version

## Troubleshooting

### Common Issues

#### Connection Problems

- **Symptom**: Cannot connect to the network
- **Solution**: Check network connectivity, firewall settings, and bootstrap nodes

#### Peer Discovery

- **Symptom**: Cannot find other peers
- **Solution**: Ensure mDNS and DHT are enabled, check network multicast settings

#### File Transfer

- **Symptom**: File transfer fails
- **Solution**: Check file permissions, disk space, and network stability

#### Group Messaging

- **Symptom**: Group messages not received
- **Solution**: Ensure all peers are subscribed to the same group topic

### Debugging

Enable debug logging to troubleshoot issues:

```typescript
// Enable debug logging
client.enableDebugLogging();

// Check client status
const status = await client.getStatus();
console.log('Client status:', status);

// Check network stats
const stats = await client.getNetworkStats();
console.log('Network stats:', stats);
```

## Advanced Features

### Custom Libp2p Configuration

You can provide a custom Libp2p configuration to fine-tune the client's behavior:

```typescript
import { createLibp2p } from 'libp2p';
import { tcp } from '@libp2p/tcp';
import { noise } from '@libp2p/noise';
import { yamux } from '@libp2p/yamux';
import { kadDHT } from '@libp2p/kad-dht';
import { gossipsub } from '@libp2p/gossipsub';

// Create custom Libp2p instance
const libp2p = await createLibp2p({
  addresses: {
    listen: ['/ip4/0.0.0.0/tcp/0', '/ip4/0.0.0.0/ws/0'],
  },
  transports: [tcp()],
  connectionEncryption: [noise()],
  streamMuxers: [yamux()],
  services: {
    dht: kadDHT({
      kBucketSize: 20,
    }),
    pubsub: gossipsub({
      allowPublishToZeroPeers: true,
    }),
  },
});

// Create Gigi client with custom Libp2p
const client = new P2pClient({
  nickname: 'My Node',
  libp2p,
});
```

### NAT Traversal

The client automatically handles NAT traversal using Libp2p's circuit relay protocol. For better results, you can configure relay options:

```typescript
const client = new P2pClient({
  nickname: 'My Node',
  config: {
    enableRelay: true,
    relayHopLimit: 3,
    // Additional relay configuration
  },
});
```

### Performance Optimization

To optimize performance:

- **Limit Connections**: Configure `maxConnections` in client settings
- **Enable Relay**: Use relay nodes for better NAT traversal
- **Optimize DHT**: Adjust DHT parameters for your network size
- **Use Fast Transports**: Prioritize faster transports like QUIC

## API Reference

### P2pClient Class

#### Constructor

```typescript
const client = new P2pClient(options);
```

**Parameters**:
- `options`: Configuration options object

#### Methods

##### `start()`

Start the P2P client.

**Returns**:
- `Promise<void>`

##### `stop()`

Stop the P2P client.

**Returns**:
- `Promise<void>`

##### `getPeerId()`

Get the peer ID of the client.

**Returns**:
- `string` (peer ID)

##### `getMultiaddrs()`

Get the multiaddresses the client is listening on.

**Returns**:
- `Array<string>` (multiaddresses)

##### `isConnected()`

Check if the client is connected to the network.

**Returns**:
- `boolean`

##### `listPeers()`

List all discovered peers.

**Returns**:
- `Promise<Array<PeerInfo>>`

##### `connectToPeer(peerId)`

Connect to a specific peer.

**Parameters**:
- `peerId`: Peer ID to connect to

**Returns**:
- `Promise<void>`

##### `disconnectFromPeer(peerId)`

Disconnect from a peer.

**Parameters**:
- `peerId`: Peer ID to disconnect from

**Returns**:
- `Promise<void>`

##### `sendDirectMessage(peerId, message)`

Send a direct message to a peer.

**Parameters**:
- `peerId`: Target peer ID
- `message`: Message content

**Returns**:
- `Promise<void>`

##### `listGroups()`

List all joined groups.

**Returns**:
- `Promise<Array<GroupInfo>>`

##### `joinGroup(groupName)`

Join a group.

**Parameters**:
- `groupName`: Group name

**Returns**:
- `Promise<void>`

##### `leaveGroup(groupName)`

Leave a group.

**Parameters**:
- `groupName`: Group name

**Returns**:
- `Promise<void>`

##### `sendGroupMessage(groupName, message)`

Send a message to a group.

**Parameters**:
- `groupName`: Group name
- `message`: Message content

**Returns**:
- `Promise<void>`

##### `shareFile(filePath)`

Share a file.

**Parameters**:
- `filePath`: Path to file

**Returns**:
- `Promise<string>` (share code)

##### `downloadFile(peerId, shareCode)`

Download a file.

**Parameters**:
- `peerId`: Peer ID to download from
- `shareCode`: Share code

**Returns**:
- `Promise<string>` (download ID)

##### `cancelDownload(downloadId)`

Cancel a download.

**Parameters**:
- `downloadId`: Download ID to cancel

**Returns**:
- `Promise<void>`

##### `listActiveDownloads()`

List active downloads.

**Returns**:
- `Promise<Array<DownloadInfo>>`

##### `onEvent(handler)`

Register an event handler.

**Parameters**:
- `handler`: Event handler function

##### `enableDebugLogging()`

Enable debug logging.

##### `getStatus()`

Get the client status.

**Returns**:
- `Promise<ClientStatus>`

##### `getNetworkStats()`

Get network statistics.

**Returns**:
- `Promise<NetworkStats>`

## Examples

### Basic Messaging

```typescript
import { P2pClient } from '@gigi/p2p-ts';

// Create and start client
const client = new P2pClient({
  nickname: 'Alice',
  config: {
    bootstrapNodes: ['/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer'],
  },
});

await client.start();
console.log(`Alice started with peer ID: ${client.getPeerId()}`);

// Send message to Bob
await client.sendDirectMessage('bob-peer-id', 'Hello Bob!');

// Listen for messages
client.onEvent((event) => {
  if (event.type === 'direct-message') {
    console.log(`Alice received: ${event.message} from ${event.from}`);
  }
});

// Stop client when done
// await client.stop();
```

### Group Chat

```typescript
import { P2pClient } from '@gigi/p2p-ts';

// Create and start client
const client = new P2pClient({
  nickname: 'Alice',
  config: {
    bootstrapNodes: ['/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer'],
  },
});

await client.start();

// Join general group
await client.joinGroup('general');
console.log('Alice joined general group');

// Send group message
await client.sendGroupMessage('general', 'Hello everyone!');

// Listen for group messages
client.onEvent((event) => {
  if (event.type === 'group-message') {
    console.log(`Alice received group message in ${event.group}: ${event.message} from ${event.from}`);
  }
});
```

### File Sharing

```typescript
import { P2pClient } from '@gigi/p2p-ts';

// Create and start client
const client = new P2pClient({
  nickname: 'Alice',
  config: {
    bootstrapNodes: ['/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer'],
  },
});

await client.start();

// Share a file
const shareCode = await client.shareFile('/path/to/document.pdf');
console.log(`File shared with code: ${shareCode}`);

// Listen for download events
client.onEvent((event) => {
  if (event.type === 'file-download-progress') {
    console.log(`Download progress: ${event.progress}%`);
  } else if (event.type === 'file-download-completed') {
    console.log(`Download completed: ${event.filePath}`);
  }
});

// Download a file from Bob
const downloadId = await client.downloadFile('bob-peer-id', 'share-code-from-bob');
console.log(`Download started with ID: ${downloadId}`);
```

## Conclusion

The Gigi P2P TypeScript Client provides a powerful, TypeScript-friendly interface to the Gigi P2P network, enabling secure, decentralized communication and file sharing in web applications and services. By following this guide, you can integrate P2P functionality into your TypeScript/JavaScript applications, creating robust, distributed systems that work across networks and devices.

For more information, see the [API Reference](api/typescript-api.md) and [Troubleshooting Guide](guides/troubleshooting-guide.md).