# Gigi P2P Example

This is a comprehensive example application demonstrating how to use the Gigi P2P TypeScript client for various P2P functionalities including group chat, file sharing, and more. It shows how Alice, Bob, and Charlie can open the program and interact with each other using the Gigi P2P network.

## Prerequisites

- Node.js 18 or later
- pnpm (as per project requirements)
- TypeScript

## Setup

1. **Install dependencies in the root directory**:

   ```bash
   cd /home/lebei/dev/gigi
   pnpm install
   ```

2. **Build the Gigi P2P TypeScript client**:

   ```bash
   cd typescript/gigi-p2p-ts
   pnpm run build
   ```

3. **Install dependencies for the example**:
   ```bash
   cd ../gigi-p2p-example
   pnpm install
   ```

## Usage

You'll need to open three separate terminal windows to simulate Alice, Bob, and Charlie.

### Terminal 1 (Alice):

```bash
pnpm dev init --nickname Alice
```

### Terminal 2 (Bob):

```bash
pnpm dev init --nickname Bob
```

### Terminal 3 (Charlie):

```bash
pnpm dev init --nickname Charlie
```

## Available Commands

Once the clients are running, you can use the following commands:

- `/join <group>` - Join a group chat
- `/leave` - Leave current group
- `/msg <peer> <msg>` - Send direct message
- `/share <file>` - Share a file
- `/download <peer> <shareCode>` - Download a file
- `/peers` - List connected peers
- `/agents` - List registered agents
- `/settings` - Query agent settings
- `/quit` - Exit the program
- `<message>` - Send to current group

## CLI Commands

The example also provides the following CLI commands:

- `init` - Initialize P2P client with optional mnemonic and nickname
- `chat` - Start interactive chat (requires init first)
- `generate-mnemonic` - Generate a new BIP-39 mnemonic
- `derive-peer-id` - Derive peer ID from mnemonic

## Example Workflow

1. **All users join the same group**:
   - Alice: `/join general`
   - Bob: `/join general`
   - Charlie: `/join general`

2. **Start chatting**:
   - Alice: `Hello everyone!` (sends to current group)
   - Bob: `Hi Alice!` (sends to current group)
   - Charlie: `Hey everyone!` (sends to current group)

3. **Send direct messages**:
   - Alice: `/msg Bob Can you send me that file?`
   - Bob: `/msg Alice Sure, I'll share it with you.`

4. **Share and download files**:
   - Bob: `/share path/to/file.txt` (generates share code)
   - Bob: `/msg Alice Share code: ABC123`
   - Alice: `/download Bob ABC123` (downloads the file)

## How It Works

- **Peer Discovery**: The clients automatically discover each other using mDNS (local network) and Kademlia DHT (WAN).
- **Group Messaging**: Uses GossipSub protocol for publish-subscribe messaging.
- **Direct Messaging**: Uses request-response protocol for 1-to-1 communication.
- **Event System**: All P2P activities are emitted as typed events.

## Troubleshooting

- **Peers not discovering each other**: Make sure all clients are on the same network and that mDNS is enabled.
- **Messages not being received**: Ensure all users have joined the same group with the exact same name.
- **Connection issues**: Check network connectivity and firewall settings.

## License

MIT
