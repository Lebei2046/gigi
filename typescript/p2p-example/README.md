# Gigi P2P Example

## Overview

This is a comprehensive example application demonstrating how to use the Gigi P2P TypeScript client for various P2P functionalities including group chat, file sharing, and more. It shows how multiple peers can interact with each other using the Gigi P2P network.

### Features

- **Group Chat**: Real-time messaging between multiple peers
- **Direct Messaging**: 1-to-1 communication between peers
- **File Sharing**: Share and download files with share codes
- **Peer Discovery**: Automatic discovery of peers on the network
- **Interactive CLI**: Command-line interface for testing P2P functionality

## Installation/Test

### Prerequisites

- Node.js 18 or later
- pnpm (as per project requirements)
- TypeScript

### Installation

1. **Install dependencies in the root directory**:

   ```bash
   cd /home/lebei/dev/gigi
   pnpm install
   ```

2. **Build the Gigi P2P TypeScript client**:

   ```bash
   cd typescript/p2p
   pnpm run build
   ```

3. **Install dependencies for the example**:
   ```bash
   cd ../p2p-example
   pnpm install
   ```

### Testing

You'll need to open multiple terminal windows to simulate different peers:

```bash
# Terminal 1 (Alice)
pnpm dev init --nickname Alice

# Terminal 2 (Bob)
pnpm dev init --nickname Bob

# Terminal 3 (Charlie)
pnpm dev init --nickname Charlie
```

## License

MIT
