# Gigi Mobile - Cloud Infrastructure Setup

This document describes how to configure the Gigi mobile app to connect to cloud-hosted bootstrap and relay nodes for WAN P2P communication.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              INTERNET (WAN)                                 │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                    CLOUD HOSTS (Bootstrap + Relay)                  │   │
│   │                                                                     │   │
│   │   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐            │   │
│   │   │ Bootstrap 1 │    │ Bootstrap 2 │    │ Relay Node  │            │   │
│   │   │ 203.0.113.10│    │ 203.0.113.11│    │ 203.0.113.12│            │   │
│   │   └─────────────┘    └─────────────┘    └─────────────┘            │   │
│   │                                                                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                    ▲                                        │
│                                    │                                        │
│              ┌─────────────────────┴─────────────────────┐                  │
│              │                                           │                  │
│              ▼                                           ▼                  │
│   ┌─────────────────────┐                     ┌─────────────────────┐       │
│   │     WIFI NETWORK 1  │                     │     WIFI NETWORK 2  │       │
│   │  ┌───────────────┐  │                     │  ┌───────────────┐  │       │
│   │  │  Gigi Mobile  │  │◄───────────────────►│  │  Gigi Mobile  │  │       │
│   │  │    App A      │  │   Via Cloud Relay   │  │    App B      │  │       │
│   │  └───────────────┘  │                     │  └───────────────┘  │       │
│   └─────────────────────┘                     └─────────────────────┘       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Configuration

### 1. Update Bootstrap Node Addresses

Edit `src/config/p2p.ts` and replace the example addresses with your actual cloud node addresses:

```typescript
export const PRODUCTION_BOOTSTRAP_NODES = [
  // Replace with your actual deployed bootstrap node addresses
  '/dns4/bootstrap1.yourdomain.com/tcp/4001/p2p/12D3KooWABC...',
  '/dns4/bootstrap2.yourdomain.com/tcp/4002/p2p/12D3KooWDEF...',
  '/dns4/relay.yourdomain.com/tcp/4003/p2p/12D3KooWGHI...',
];
```

### 2. Get Bootstrap Node Peer IDs

After deploying your gigi-node instances, get their peer IDs from the logs:

```bash
# On each cloud host, check the logs
docker logs gigi-node-bootstrap-1

# Look for:
# INFO gigi_node: Local peer ID: 12D3KooWxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

### 3. Update Configuration

Use the `messaging_set_bootstrap_nodes` command to update bootstrap nodes at runtime:

```typescript
import { invoke } from '@tauri-apps/api/core'

// Set bootstrap nodes
await invoke('messaging_set_bootstrap_nodes', {
  bootstrapNodes: [
    '/ip4/203.0.113.10/tcp/4001/p2p/12D3KooWABC...',
    '/ip4/203.0.113.11/tcp/4002/p2p/12D3KooWDEF...',
  ]
})
```

## Deployment Steps

### 1. Deploy Cloud Nodes

Follow the deployment guide in `apps/gigi-node/README.md` to deploy bootstrap and relay nodes.

### 2. Configure DNS (Optional but Recommended)

Set up DNS records for your bootstrap nodes:

```
bootstrap1.yourdomain.com  A  203.0.113.10
bootstrap2.yourdomain.com  A  203.0.113.11
relay.yourdomain.com       A  203.0.113.12
```

### 3. Update Mobile App Config

Update `src/config/p2p.ts` with the deployed node addresses.

### 4. Build and Deploy Mobile App

```bash
# Build the app
cd apps/gigi-mobile
npm run build:android
# or
npm run build:ios
```

## How It Works

### Connection Flow

1. **App Starts**: Mobile app initializes with bootstrap node configuration
2. **DHT Bootstrap**: App connects to bootstrap nodes and joins the Kademlia DHT
3. **Peer Discovery**: App discovers other peers via DHT lookups
4. **NAT Traversal**: If peers are behind NAT, the relay node facilitates connections
5. **Direct Communication**: Once connected, peers communicate directly (P2P)

### Protocol Stack

| Protocol | Purpose |
|----------|---------|
| **Kademlia DHT** | WAN peer discovery and routing |
| **Circuit Relay** | NAT traversal for behind-router peers |
| **GossipSub** | Group messaging (pub/sub) |
| **Direct Messaging** | 1-to-1 messages (request/response) |
| **File Sharing** | Chunked file transfers |

## Troubleshooting

### Cannot Connect to Bootstrap Nodes

1. Check network connectivity:
   ```bash
   telnet bootstrap1.yourdomain.com 4001
   ```

2. Verify bootstrap nodes are running:
   ```bash
   docker ps | grep gigi-node
   ```

3. Check firewall rules on cloud hosts

### High Latency

1. Deploy bootstrap nodes geographically close to users
2. Use DNS-based bootstrap addresses for better load balancing
3. Enable QUIC transport for better performance

### Connection Drops

1. Check relay node capacity
2. Increase idle connection timeout
3. Verify mobile app background execution permissions

## Security Considerations

1. **Bootstrap Node Security**: Keep bootstrap nodes secure - they are entry points to the network
2. **Relay Abuse**: Monitor relay node bandwidth usage to prevent abuse
3. **Peer Verification**: Implement peer verification for sensitive communications

## Related Documentation

- [Gigi Node README](../../apps/gigi-node/README.md)
- [Gigi P2P Documentation](../../pkgs/gigi-p2p/README.md)
- [Deployment Script](../../apps/gigi-node/deploy.sh)
