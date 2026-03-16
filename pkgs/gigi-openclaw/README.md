# Gigi OpenClaw Plugin

OpenClaw plugin for Gigi P2P network integration.

## Features

- P2P messaging via libp2p
- Support for TCP, WebSocket, and WebRTC transports
- mDNS and Kademlia DHT for peer discovery
- Seamless integration with OpenClaw Gateway
- Automatic reconnection and retry logic

## Installation

```bash
cd pkgs/gigi-openclaw
npm install
```

## Configuration

Add Gigi account to your OpenClaw config:

```json
{
  "accounts": {
    "my-gigi-account": {
      "peerId": "12D3KooW...",
      "multiaddrs": [
        "/ip4/0.0.0.0/tcp/0",
        "/ip4/0.0.0.0/tcp/0/ws"
      ],
      "displayName": "My Gigi Node"
    }
  }
}
```

## Usage

### Starting a Gateway

```typescript
import { gigiPlugin } from "@gigi/openclaw";

await gigiPlugin.gateway.startAccount({
  accountId: "my-gigi-account",
  account: {
    accountId: "my-gigi-account",
    displayName: "My Gigi Node",
    peerId: "12D3KooW...",
    multiaddrs: ["/ip4/0.0.0.0/tcp/0"],
  },
  config: {
    gatewayUrl: "ws://127.0.0.1:18789",
    autoConnect: true,
  },
  onMessage: (msg) => {
    console.log("Received:", msg);
  },
});
```

### Sending Messages

```typescript
await gigiPlugin.gateway.sendMessage({
  accountId: "my-gigi-account",
  to: "12D3KooW...",
  content: "Hello from Gigi!",
});
```

### Checking Status

```typescript
const status = await gigiPlugin.status.checkStatus("my-gigi-account");
console.log(status);
```

## Development

```bash
# Build
npm run build

# Watch mode
npm run dev
```

## Protocol

Gigi uses the libp2p protocol `/gigi/direct/1.0.0` for direct P2P messaging.

Message format:
```typescript
{
  from: string;    // peerId
  to: string;      // peerId
  content: string;
  timestamp: number;
  type: "direct" | "broadcast";
}
```

## License

MIT
