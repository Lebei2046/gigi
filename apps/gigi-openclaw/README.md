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

### Account Configuration

Add a Gigi account to your OpenClaw configuration:

```json
{
  "accounts": {
    "my-gigi-account": {
      "type": "gigi-p2p-bundled",
      "peerId": "12D3KooW...", // Your Gigi peer ID
      "multiaddrs": [
        "/ip4/0.0.0.0/tcp/0",    // Listen on all TCP interfaces
        "/ip4/0.0.0.0/tcp/0/ws"   // Listen on all WebSocket interfaces
      ],
      "displayName": "My Gigi Node",
      "enabled": true
    }
  }
}
```

### Gateway Configuration

When starting the OpenClaw gateway, ensure the plugin is enabled:

```bash
openclaw gateway start
```

## Usage

### Group Management

The plugin provides several tools for managing Gigi P2P groups:

#### Join a Group

```bash
openclaw tools call gigi_join_group --accountId my-gigi-account --groupId "my-group-topic"
```

#### Leave a Group

```bash
openclaw tools call gigi_leave_group --accountId my-gigi-account --groupId "my-group-topic"
```

#### List Groups

```bash
openclaw tools call gigi_list_groups --accountId my-gigi-account
```

### Sending Messages

#### Direct Message

```typescript
await gigiPlugin.gateway.sendMessage({
  accountId: "my-gigi-account",
  to: "12D3KooW...", // Recipient's peer ID
  content: "Hello from Gigi!",
});
```

#### Group Message

```typescript
await gigiPlugin.gateway.sendMessage({
  accountId: "my-gigi-account",
  to: "my-group-topic", // Group topic
  content: "Hello everyone!",
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

2. Use the OpenClaw CLI to test the plugin's tools:

```bash
# Test group management
openclaw tools call gigi_list_groups --accountId my-gigi-account

# Test sending a message
export MESSAGE_ID=$(openclaw tools call gigi_send_message --accountId my-gigi-account --to "12D3KooW..." --content "Test message")

# Check if the message was sent successfully
openclaw tools call gigi_get_message --accountId my-gigi-account --messageId $MESSAGE_ID
```

### Manual Testing

1. **Start multiple OpenClaw instances** with the Gigi plugin enabled
2. **Create a Gigi account** on each instance
3. **Join the same group** on all instances
4. **Send messages** between instances to verify communication
5. **Test file sharing** by sending files between instances

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
- `src/types.ts`: Type definitions
- `index.ts`: Plugin entry point

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