# Gigi OpenClaw Plugin

The Gigi OpenClaw Plugin is an integration plugin for OpenClaw that enables seamless communication with the Gigi P2P network. It allows OpenClaw to join Gigi P2P groups and chat with Gigi P2P clients.

## What is Gigi P2P?

Gigi P2P is a decentralized peer-to-peer network built on top of Libp2p, enabling secure, direct communication between peers without relying on centralized servers. It provides:
- Decentralized peer discovery via Kademlia DHT and mDNS
- Secure encrypted communication
- Group chat functionality
- File sharing capabilities

## Features

- **P2P Messaging**: Send and receive messages directly between peers
- **Group Chat**: Join and participate in Gigi P2P groups
- **Multiple Transports**: Support for TCP, WebSocket, and WebRTC
- **Seamless Integration**: Works directly with OpenClaw Gateway
- **Automatic Reconnection**: Handles network disruptions gracefully
- **Group Management**: Tools to join, leave, and list Gigi P2P groups

## Installation

### Prerequisites

- Node.js 18+ or compatible runtime
- OpenClaw 2026.3.22+
- pnpm (recommended for monorepo setup)

### From Source

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

## Configuration

### Plugin Configuration

The plugin can be configured through the OpenClaw dashboard or by editing the OpenClaw configuration file:

```json
{
  "plugins": {
    "gigi-p2p-bundled": {
      "enabled": true
    }
  }
}
```

### Configure Channel

To maintain a consistent peer ID across restarts, you can use a BIP-39 mnemonic phrase to derive your peer ID and private key. Here's how to configure your channel:

#### 1. Generate a Mnemonic Phrase

You can generate a new BIP-39 mnemonic phrase using one of the following methods:

**Method 1: Use the OpenClaw agent tool**
- The `gigi_generate_mnemonic` tool is available for OpenClaw agents to generate mnemonics

**Method 2: Use the provided script**
- Run the mnemonic generation script from the `gigi-openclaw` directory:

```bash
cd /path/to/gigi/apps/gigi-openclaw
pnpm run generate-mnemonic
```

**Method 3: Use a secure mnemonic generator**
- Use a trusted BIP-39 mnemonic generator tool to create a 12-word mnemonic phrase

#### 2. Update Channel Configuration

Add the mnemonic phrase to your OpenClaw channel configuration:

```json
{
  "channels": {
    "gigi-p2p-bundled": {
      "peerId": "12D3KooW...", // Will be generated from mnemonic
      "multiaddrs": [
        "/ip4/0.0.0.0/tcp/0",    // Listen on all TCP interfaces
        "/ip4/0.0.0.0/tcp/0/ws"   // Listen on all WebSocket interfaces
      ],
      "mnemonic": "abandon amount liar amount expire adjust cage candy arch gather drum buyer", // Your mnemonic phrase
      "displayName": "My Gigi Node",
      "enabled": true
    }
  }
}
```

**Important**: The `mnemonic` field contains your mnemonic phrase, which is the root of all your Gigi P2P keys. Make sure to keep your configuration file secure. This field is optional, but without it, a new peer ID will be generated each time the client starts.

When the Gigi client starts, it will use the mnemonic phrase to derive your peer ID and private key. This provides a more user-friendly way to manage your identity compared to manually specifying private keys.

### Multiple Groups Support

The Gigi P2P plugin allows agents to use a single Gigi P2P client to connect to the network and join multiple groups simultaneously. This extends the agent's capability to participate in different communities and conversations without needing multiple client instances.

The plugin manages group memberships through a single Gigi P2P client instance, optimizing resource usage and network connections while providing seamless access to multiple groups.

### Gateway Configuration

When starting the OpenClaw gateway, ensure the plugin is enabled:

```bash
openclaw gateway start
```

## Usage

### Group Management

The plugin provides several functions for managing Gigi P2P groups, which can be used through the GigiClient API:

#### Using the GigiClient API

```typescript
import { GigiClient } from "gigi-p2p-bundled";

// Create a Gigi client
const client = new GigiClient({
  peerId: "12D3KooW...",
  multiaddrs: ["/ip4/0.0.0.0/tcp/0", "/ip4/0.0.0.0/tcp/0/ws"],
  mnemonic: "your mnemonic here",
  displayName: "My Gigi Node"
});

// Start the client
await client.start();

// Join a group
await client.joinGroup("my-group-topic");

// List joined groups
const groups = client.listGroups();
console.log("Joined groups:", groups);

// Leave a group
await client.leaveGroup("my-group-topic");

// Stop the client
await client.stop();
```

### Sending Messages

#### Direct Message

To send a direct message using the OpenClaw API:

```typescript
await openclaw.message.send({
  channel: "gigi-p2p-bundled",
  accountId: "my-gigi-account",
  target: "12D3KooW...", // Recipient's peer ID
  message: "Hello from Gigi!",
});
```

#### Group Message

To send a group message using the OpenClaw API:

```typescript
await openclaw.message.send({
  channel: "gigi-p2p-bundled",
  accountId: "my-gigi-account",
  target: "my-group-topic", // Group topic
  message: "Hello everyone!",
});
```

### Checking Status

```typescript
const status = await gigiPlugin.status.checkStatus("my-gigi-account");
console.log(status);
```

## Testing

### Unit Tests

Run the unit tests to verify the plugin's functionality:

```bash
pnpm test
```

### Integration Tests

To test the plugin with OpenClaw:

1. Start the OpenClaw gateway:

```bash
openclaw gateway start
```

2. Use the GigiClient API in your code to test the plugin's functionality, or use the OpenClaw dashboard to interact with the plugin.

### Manual Testing

1. **Start multiple OpenClaw instances** with the Gigi plugin enabled
2. **Create a Gigi account** on each instance
3. **Join the same group** on all instances
4. **Send messages** between instances to verify communication
5. **Test file sharing** by sending files between instances

### Test Plan

For a comprehensive test plan, see the [TEST_PLAN.md](./docs/TEST_PLAN.md) file in the docs directory.

## Troubleshooting

### Common Issues

1. **Plugin not loading**
   - Check if the plugin is enabled in the OpenClaw configuration
   - Verify that the bundled file exists at `dist/bundle.js`
   - Check the OpenClaw logs for error messages

2. **Cannot join groups**
   - Ensure your peer ID is correctly configured
   - Check if you're connected to the Gigi P2P network
   - Verify that the group topic is valid

3. **Messages not being delivered**
   - Check network connectivity
   - Verify that both peers are online
   - Ensure both peers are using the same Gigi P2P protocol version

4. **Performance issues**
   - Reduce the number of active groups
   - Limit the size of messages
   - Ensure your system has sufficient resources

### Logs

Check the OpenClaw logs for detailed error information:

```bash
openclaw logs
```

## Development

### Building

```bash
# Build the plugin
pnpm run build

# Build and bundle with dependencies
pnpm run build:bundle

# Watch mode
pnpm run dev
```

### Code Structure

- `src/channel.ts`: Implements the ChannelPlugin interface
- `src/GigiClient.ts`: Manages the P2P client instance
- `src/accounts.ts`: Account resolution and management
- `src/outbound.ts`: Outbound message handling
- `src/probe.ts`: Status checking and probing
- `src/types.ts`: Type definitions
- `index.ts`: Plugin entry point

### Documentation

- [README.md](./README.md): This file
- [TEST_PLAN.md](./docs/TEST_PLAN.md): Comprehensive test plan
- [Gigi P2P Documentation](/docs/gigi-p2p.md): Core Gigi P2P documentation

## Protocol

Gigi uses the following Libp2p protocols:

- **Direct Messaging**: `/gigi/direct/1.0.0`
- **Group Messaging**: `/gigi/group/1.0.0`
- **File Sharing**: `/gigi/file/1.0.0`

### Message Format

```typescript
{
  from: string;    // Sender's peer ID
  to: string;      // Recipient's peer ID or group topic
  content: string; // Message content
  timestamp: number; // Unix timestamp
  type: "direct" | "group" | "file"; // Message type
}
```

## License

MIT