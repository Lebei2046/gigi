# Gigi P2P - TypeScript Port

TypeScript implementation of the Gigi P2P networking library, ported from the Rust [gigi-p2p](https://github.com/gigi/rust/gigi-p2p) crate.

## Features

- **Auto Discovery**: Automatic peer discovery via mDNS (local network) and Kademlia DHT (WAN)
- **NAT Traversal**: Circuit relay for connecting peers behind routers
- **Direct Messaging**: 1-to-1 peer communication via request-response protocol
- **Group Messaging**: Publish-subscribe model using GossipSub for group chats
- **File Transfer**: Request-response protocol for file sharing with integrity verification
- **Unified Event System**: All P2P activities emitted as typed events

## Protocol Stack

| Protocol      | Purpose               | Type             |
| ------------- | --------------------- | ---------------- |
| Gigi Direct   | 1-to-1 communication  | Request-Response |
| Gigi File     | Chunked file transfer | Request-Response |
| Gigi Group    | Group chat            | GossipSub        |
| mDNS          | Local peer discovery  | Multicast DNS    |
| Kademlia      | WAN peer discovery    | DHT              |
| Circuit Relay | NAT traversal         | Relay            |

## Installation

```bash
cd typescript/gigi-p2p-ts
npm install
```

## Usage

```typescript
import { P2pClient } from '@gigi/p2p';

async function main() {
  const client = new P2pClient({
    nickname: 'Alice',
    outputDirectory: './downloads',
    // You can provide a mnemonic to derive the peer ID and private key
    mnemonic:
      'abandon amount liar amount expire adjust cage candy arch gather drum buyer',
  });

  await client.start();

  console.log(`Peer ID: ${client.getPeerId()}`);
  console.log(`Listening on: ${client.getMultiaddrs().join(', ')}`);

  client.onEvent(async (event) => {
    console.log('Event:', event.type);
  });

  await client.sendDirectMessage('12D3KooW...', 'Hello!');

  await client.joinGroup('chat-room');
  await client.sendGroupMessage('chat-room', {
    type: 'text',
    text: 'Hello everyone!',
  });

  const shareCode = await client.shareFile('./document.pdf');
  console.log(`Share code: ${shareCode}`);

  const downloadId = await client.downloadFile('Bob', shareCode);

  await client.stop();
}

main().catch(console.error);
```

## API

### P2pClient

Main client for P2P networking.

#### Constructor

```typescript
new P2pClient(options: P2pClientOptions)
```

Options:

- `nickname: string` - Display name for this peer
- `outputDirectory?: string` - Directory for downloaded files (default: './downloads')
- `config?: P2pConfig` - P2P configuration

#### Methods

| Method                                           | Description              |
| ------------------------------------------------ | ------------------------ |
| `start()`                                        | Start the P2P client     |
| `stop()`                                         | Stop the P2P client      |
| `getPeerId()`                                    | Get local peer ID        |
| `getMultiaddrs()`                                | Get listening addresses  |
| `sendDirectMessage(peerId, message)`             | Send direct message      |
| `sendDirectMessageToNickname(nickname, message)` | Send to peer by nickname |
| `joinGroup(name)`                                | Join a group             |
| `leaveGroup(name)`                               | Leave a group            |
| `sendGroupMessage(group, message)`               | Send to group            |
| `shareFile(path)`                                | Share a file             |
| `downloadFile(nickname, shareCode)`              | Download a file          |
| `revokeFile(shareCode)`                          | Revoke a shared file     |
| `listSharedFiles()`                              | List shared files        |
| `getActiveDownloads()`                           | Get active downloads     |
| `onEvent(listener)`                              | Register event listener  |

## Events

### Discovery Events

- `peer-discovered` - New peer discovered
- `peer-expired` - Peer expired
- `nickname-updated` - Peer nickname updated

### Messaging Events

- `direct-message` - Direct message received
- `group-message` - Group message received

### File Events

- `file-shared` - File shared successfully
- `file-download-started` - Download started
- `file-download-progress` - Download progress
- `file-download-completed` - Download completed
- `file-download-failed` - Download failed

### Connection Events

- `connected` - Peer connected
- `disconnected` - Peer disconnected
- `listening-on` - Listening on address

## File Sharing

Files are shared using a unique **share code** system:

1. **Share**: `shareFile()` generates a unique share code
2. **Announce**: Share code sent to peer(s) via direct/group message
3. **Request**: Receiver uses `downloadFile()` with the share code
4. **Transfer**: File split into 256KB chunks, transferred on-demand
5. **Verify**: SHA256 verification for integrity

## Development

```bash
npm install
npm run build
npm run dev
```

## Key Derivation

The package provides functions for deriving peer IDs, group IDs, and private keys from BIP-39 mnemonic phrases:

```typescript
import {
  generateMnemonic,
  derivePeerId,
  deriveGroupId,
  derivePeerPrivateKey,
} from '@gigi/p2p';

// Generate a new mnemonic phrase
const mnemonic = generateMnemonic();
console.log('Mnemonic:', mnemonic);

// Derive peer ID from mnemonic
const peerId = await derivePeerId(mnemonic);
console.log('Peer ID:', peerId);

// Derive group ID from mnemonic
const groupId = await deriveGroupId(mnemonic, 'my-group');
console.log('Group ID:', groupId);

// Derive private key from mnemonic
const privateKey = derivePeerPrivateKey(mnemonic);
console.log('Private key:', privateKey);
```

## License

MIT
