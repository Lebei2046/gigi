# Gigi TUI

## Overview

Gigi TUI is a terminal-based P2P chat client for the Gigi P2P ecosystem. It provides an interactive command-line interface for communicating with AI agents, managing sessions, and interacting with the P2P network.

## Features

- **P2P Communication**: Direct peer-to-peer communication with AI agents
- **Interactive Terminal Interface**: User-friendly command-line interface
- **Session Management**: Create, switch, and delete chat sessions
- **Agent Management**: View and switch between different AI agents
- **Markdown Support**: Render Markdown-formatted messages in the terminal
- **Chat History**: Load and display previous chat history
- **Command-Line Commands**: Execute various commands for managing sessions and agents

## Installation

### Prerequisites

- Node.js 18+
- pnpm package manager

### Installation Steps

1. **Clone the repository**
   ```bash
   git clone https://github.com/Lebei2046/gigi.git
   cd gigi
   ```

2. **Install dependencies**
   ```bash
   pnpm install
   ```

3. **Build the project**
   ```bash
   pnpm run build
   ```

## Usage

### Starting Gigi TUI

```bash
# Run in development mode
pnpm run dev

# Run in production mode
pnpm run build
pnpm run start
```

### Command-Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `--nickname <name>` | Client node nickname | `gigi-tui` |
| `--session <key>` | Session key | `main` (or `global` in global mode) |
| `--deliver` | Deliver assistant replies | `false` |
| `--thinking <level>` | Thinking level override | - |
| `--message <text>` | Send initial message after connecting | - |
| `--timeout-ms <ms>` | Agent timeout in milliseconds | - |
| `--history-limit <n>` | Number of history entries to load | `200` |
| `--host <host>` | P2P bootstrap host | - |
| `--port <port>` | P2P bootstrap port | - |

### Example Usage

```bash
# Start with custom nickname
pnpm run dev -- --nickname "my-tui-client"

# Connect to specific bootstrap node
pnpm run dev -- --host 192.168.1.100 --port 8080

# Start and send initial message
pnpm run dev -- --message "Hello, Gigi!"
```

## TUI Commands

Within the Gigi TUI interface, you can use the following commands:

| Command | Description | Example |
|---------|-------------|---------|
| `/session list` | List all sessions | `/session list` |
| `/session create <name>` | Create a new session | `/session create my-new-session` |
| `/session delete <name>` | Delete a session | `/session delete old-session` |
| `/session <name>` | Switch to specified session | `/session work` |
| `/agent <id>` | Switch to specified agent | `/agent main` |
| `/clear` | Clear terminal screen | `/clear` |
| `/exit` or `/quit` | Exit TUI | `/exit` |

## Interaction

- **Send message**: Type text and press Enter
- **Execute command**: Type `/` followed by command name
- **Exit**: Press Ctrl+C twice, or use `/exit` command

## Message Format

- **User messages**: Displayed as entered text
- **Assistant messages**: Support Markdown format, rendered in the terminal

## Connection Status

Gigi TUI displays current connection and activity status:
- `connecting`: Connecting to P2P network
- `connected`: Successfully connected
- `disconnected`: Connection lost
- `sending`: Sending message
- `waiting`: Waiting for assistant response
- `idle`: Idle state

## Troubleshooting

### Connection Issues
- Ensure network connectivity
- Check firewall settings
- Try specifying bootstrap node (`--host` and `--port` options)

### Session Issues
- Use `/session list` to view all available sessions
- If session is lost, try reconnecting

### Message Sending Failures
- Check P2P connection status
- Ensure agent is online
- Check terminal logs for detailed error information

## Logs

Gigi TUI uses Pino for structured logging. Log files are located in the `logs` folder at the project root.

## Architecture

Gigi TUI is built using:

- **TypeScript**: Type-safe JavaScript superset
- **Node.js**: Runtime environment
- **@gigi/p2p**: Gigi P2P client library
- **@gigi/amp**: Agent Messaging Protocol implementation
- **@gigi/logging**: Structured logging
- **commander**: Command-line argument processing
- **marked**: Markdown parsing
- **marked-terminal**: Terminal Markdown rendering

## Project Structure

```
├── notes/              # Example chat logs
├── src/                # Source code
│   ├── tui/            # TUI-related code
│   │   ├── theme/      # Theme configuration
│   │   ├── p2p-chat-client.ts  # P2P chat client
│   │   ├── tui-formatters.ts   # Formatting utilities
│   │   ├── tui-types.ts        # Type definitions
│   │   └── tui.ts              # TUI main logic
│   └── index.ts        # Main entry file
├── package.json        # Project configuration and dependencies
└── tsconfig.json       # TypeScript configuration
```

## Related Components

- **@gigi/p2p**: Core P2P client library
- **@gigi/amp**: Agent Messaging Protocol implementation
- **@gigi/logging**: Structured logging library
