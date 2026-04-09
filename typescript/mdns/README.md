# Gigi DNS TypeScript Client

Gigi DNS is the decentralized DNS service for the Gigi P2P network, allowing peers to resolve names to peer IDs. It provides a human-friendly way to address peers without needing to remember long peer IDs.

## Features

- **Local Peer Discovery**: Uses mDNS for peer discovery on the local network
- **Nickname Support**: Allows peers to advertise human-readable nicknames
- **Metadata Support**: Enables peers to advertise additional metadata and capabilities
- **Secure**: Includes packet validation and source verification
- **Extensible**: Easy to integrate with libp2p applications

## Installation

```bash
pnpm add @gigi/mdns
```

## Usage

### Basic Usage

```typescript
import {
  GigiDnsBehaviour,
  GigiDnsCommand,
  defaultGigiDnsConfig,
} from '@gigi/mdns';
import { createPeerId } from '@libp2p/peer-id';

// Create a peer ID
const peerId = await createPeerId();

// Create DNS behavior with default config
const dns = new GigiDnsBehaviour(peerId, defaultGigiDnsConfig);

// Update listen addresses when they change
dns.updateListenAddresses(['/ip4/192.168.1.100/tcp/8000', '/ip6/::1/tcp/8000']);

// Listen for discovery events
dns.on('Discovered', (event) => {
  console.log(
    'Discovered peer:',
    event.peerInfo.nickname,
    event.peerInfo.peerId.toString()
  );
  console.log('Peer address:', event.peerInfo.multiaddr.toString());
  console.log('Peer capabilities:', event.peerInfo.capabilities);
  console.log('Peer metadata:', event.peerInfo.metadata);
});

// Listen for error events
dns.on('Error', (event) => {
  console.error('DNS error:', event.error.message, 'in', event.context);
});

// Update nickname dynamically
dns.handleCommand({
  type: GigiDnsCommand.UpdateNickname,
  nickname: 'Alice',
});

// Update capabilities
dns.handleCommand({
  type: GigiDnsCommand.UpdateCapabilities,
  capabilities: ['chat', 'file_sharing'],
});

// Update metadata
dns.handleCommand({
  type: GigiDnsCommand.UpdateMetadata,
  key: 'version',
  value: '1.0.0',
});

// Get discovered peers
const peers = dns.getDiscoveredPeers();
console.log('Discovered peers:', peers.size);

// Find peer by nickname
const peer = dns.findPeerByNickname('Bob');
if (peer) {
  console.log('Found peer by nickname:', peer.peerId.toString());
}

// Stop DNS behavior when done
dns.stop();
```

### Custom Configuration

```typescript
import { GigiDnsBehaviour, GigiDnsConfig } from '@gigi/mdns';

const customConfig: GigiDnsConfig = {
  nickname: 'MyNode',
  ttl: 5 * 60 * 1000, // 5 minutes
  queryInterval: 3 * 60 * 1000, // 3 minutes
  announceInterval: 10 * 1000, // 10 seconds
  cleanupInterval: 20 * 1000, // 20 seconds
  enableIpv6: true,
  capabilities: ['chat', 'file_sharing'],
  metadata: {
    version: '1.0.0',
    os: 'Linux',
  },
  useLocalhost: false,
};

const dns = new GigiDnsBehaviour(peerId, customConfig);
```

## API Reference

### GigiDnsBehaviour

#### Constructor

```typescript
new GigiDnsBehaviour(peerId: PeerId, config: GigiDnsConfig)
```

- `peerId`: The local peer ID
- `config`: Configuration for DNS behavior

#### Methods

##### `updateListenAddresses(addresses: Multiaddr[])`

Updates the list of listen addresses to advertise in DNS responses.

- `addresses`: Array of libp2p multiaddresses

##### `handleCommand(command: GigiDnsCommandParams)`

Handles commands to update DNS configuration.

- `command`: Command to execute
  - `UpdateNickname`: Update the advertised nickname
  - `UpdateCapabilities`: Update the list of capabilities
  - `UpdateMetadata`: Update or add a metadata key-value pair

##### `getDiscoveredPeers(): Map<string, GigiPeerInfo>`

Returns a map of discovered peers, keyed by peer ID string.

##### `findPeerById(peerId: PeerId): GigiPeerInfo | undefined`

Finds a peer by its peer ID.

- `peerId`: Peer ID to search for
- Returns: Peer information if found, undefined otherwise

##### `findPeerByNickname(nickname: string): GigiPeerInfo | undefined`

Finds a peer by its nickname.

- `nickname`: Nickname to search for
- Returns: Peer information if found, undefined otherwise

##### `stop()`

Stops the DNS behavior and cleans up resources.

##### `on(event: string, listener: (event: GigiDnsEvent) => void)`

Adds an event listener.

- `event`: Event type to listen for
- `listener`: Listener function

##### `off(event: string, listener: (event: GigiDnsEvent) => void)`

Removes an event listener.

- `event`: Event type to remove listener from
- `listener`: Listener function to remove

### GigiDnsConfig

| Option             | Type                   | Description                                            | Default            |
| ------------------ | ---------------------- | ------------------------------------------------------ | ------------------ |
| `nickname`         | string                 | Human-readable nickname for this peer                  | 'Anonymous'        |
| `ttl`              | number                 | Time-to-live for DNS records (milliseconds)            | 360000 (6 minutes) |
| `queryInterval`    | number                 | Interval between discovery queries (milliseconds)      | 300000 (5 minutes) |
| `announceInterval` | number                 | Interval between announcements (milliseconds)          | 15000 (15 seconds) |
| `cleanupInterval`  | number                 | Interval for cleanup operations (milliseconds)         | 30000 (30 seconds) |
| `enableIpv6`       | boolean                | Enable IPv6 multicast                                  | false              |
| `capabilities`     | string[]               | List of capabilities this peer provides                | []                 |
| `metadata`         | Record<string, string> | Optional metadata key-value pairs                      | {}                 |
| `useLocalhost`     | boolean                | Use localhost unicast instead of multicast for testing | false              |

### GigiDnsEvent

#### Discovered

```typescript
{
  type: 'Discovered';
  peerInfo: GigiPeerInfo;
}
```

Emitted when a new peer is discovered.

#### Updated

```typescript
{
  type: 'Updated';
  peerId: PeerId;
  oldInfo: GigiPeerInfo;
  newInfo: GigiPeerInfo;
}
```

Emitted when an existing peer's information is updated.

#### Expired

```typescript
{
  type: 'Expired';
  peerId: PeerId;
  info: GigiPeerInfo;
}
```

Emitted when a peer's information expires.

#### Offline

```typescript
{
  type: 'Offline';
  peerId: PeerId;
  info: GigiPeerInfo;
  reason: OfflineReason;
}
```

Emitted when a peer goes offline.

#### Error

```typescript
{
  type: 'Error';
  error: Error;
  context: string;
}
```

Emitted when an error occurs.

## Events

- **Discovered**: A new peer was discovered
- **Updated**: An existing peer's information was updated
- **Expired**: A peer's information expired
- **Offline**: A peer went offline
- **Error**: An error occurred

## Troubleshooting

### Common Issues

#### No peers discovered

- Check network connectivity
- Ensure multicast is enabled on your network
- Verify firewall settings allow UDP traffic on port 7173

#### DNS errors

- Check the error event listener for details
- Verify network interface configuration
- Ensure you have permission to create UDP sockets

### Debugging

Enable debug logging to troubleshoot issues:

```typescript
// Enable debug logging
process.env.DEBUG = 'gigi-dns:*';

// Create DNS instance
const dns = new GigiDnsBehaviour(peerId, config);
```

## License

Apache 2.0
