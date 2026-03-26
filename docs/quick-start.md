# Quick Start Guide

This guide will help you get up and running with Gigi P2P quickly. By the end of this guide, you'll have a basic Gigi P2P setup with messaging and file sharing capabilities.

## Prerequisites

Before you begin, make sure you have the following installed:

- **Node.js** (v18+): For TypeScript components
- **Rust** (v1.60+): For Rust components
- **pnpm** (latest): For package management
- **OpenClaw** (latest): For the plugin interface

## Step 1: Clone the Repository

First, clone the Gigi P2P repository:

```bash
git clone https://github.com/gigi-project/gigi.git
cd gigi
```

## Step 2: Install Dependencies

Install dependencies using pnpm:

```bash
pnpm install
```

For Rust components, build them:

```bash
cargo build
```

## Step 3: Start a Network Node

Start a Gigi network node in bootstrap mode:

```bash
# From the gigi directory
cd apps/gigi-node
cargo run -- --mode bootstrap --listen /ip4/0.0.0.0/tcp/4001
```

This will start a bootstrap node that other peers can use to join the network.

## Step 4: Configure OpenClaw Plugin

Add the Gigi channel to OpenClaw:

```bash
# From the gigi directory
openclaw channels add gigi --peer-id "your-peer-id" --multiaddrs "/ip4/0.0.0.0/tcp/0,/ip4/0.0.0.0/ws/0" --bootstrap-peers "/ip4/127.0.0.1/tcp/4001/p2p/$(cat ~/.gigi/node-id)"
```

Replace `"your-peer-id"` with a unique peer ID for your node.

## Step 5: Start the Gigi Channel

Start the Gigi channel in OpenClaw:

```bash
openclaw channels start gigi
```

## Step 6: Send a Test Message

Send a test message to another peer:

```bash
openclaw send gigi --to "peer-id" --message "Hello from Gigi P2P!"
```

Replace `"peer-id"` with the peer ID of another Gigi node on the network.

## Step 7: Join a Group

Join a group and send a group message:

```bash
# Join a group
openclaw channels gigi join-group --group general

# Send a group message
openclaw send gigi --to group:general --message "Hello everyone!"
```

## Step 8: Share a File

Share a file with another peer:

```bash
# Share a file
openclaw channels gigi share-file --path "/path/to/file.txt"

# The command will return a share code
# Use this code to download the file from another peer
openclaw channels gigi download-file --from "peer-id" --share-code "share-code"
```

## Step 9: Check Status

Check the status of your Gigi channel:

```bash
openclaw channels status gigi
```

## Step 10: Explore More Features

Now that you have the basic setup working, you can explore more features:

- **Mobile App**: Run the Gigi mobile app on your phone
- **Desktop App**: Build a desktop app using the Tauri plugin
- **Web App**: Create a web app using the TypeScript client
- **Custom Network**: Set up your own private Gigi network

## Troubleshooting

If you encounter any issues:

1. **Connection Problems**: Check that your firewall allows connections on the required ports
2. **Peer Discovery**: Ensure mDNS and DHT are enabled in your configuration
3. **File Transfer**: Check file permissions and disk space
4. **Group Messaging**: Ensure all peers are subscribed to the same group topic

For more detailed troubleshooting, see the [Troubleshooting Guide](guides/troubleshooting-guide.md).

## Next Steps

- Read the [Installation Guide](installation.md) for more detailed installation instructions
- Explore the [Core Components](gigi-openclaw.md) documentation
- Check out the [API Reference](api/typescript-api.md) for more detailed API documentation
- Try the [Examples](examples/basic-messaging.md) to learn how to use Gigi P2P in your own applications