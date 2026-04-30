# Gigi TUI

Gigi TUI is a terminal-based P2P chat client for communicating with AI agents in the Gigi P2P ecosystem. It provides an interactive command-line interface that supports chat, session management, agent management, and other features.

## Features

- **P2P Communication**: Peer-to-peer communication based on Libp2p, no centralized servers required
- **Chat Functionality**: Real-time chat with AI agents
- **Session Management**: Create, switch, and delete sessions
- **Agent Management**: View and switch between different AI agents
- **Markdown Support**: Render Markdown-formatted messages
- **Command-Line Interface**: Interactive terminal interface supporting commands and message input
- **History**: Load and display chat history
- **Session Status**: Display session and agent status information

## Tech Stack

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
│   └── chat-with-charlie.md
├── src/                # Source code
│   ├── tui/            # TUI-related code
│   │   ├── theme/      # Theme configuration
│   │   │   └── theme.ts
│   │   ├── p2p-chat-client.ts  # P2P chat client
│   │   ├── tui-formatters.ts   # Formatting utilities
│   │   ├── tui-types.ts        # Type definitions
│   │   └── tui.ts              # TUI main logic
│   └── index.ts        # Main entry file
├── package.json        # Project configuration and dependencies
└── tsconfig.json       # TypeScript configuration
```

## Installation

### Prerequisites

- Node.js 18+
- pnpm package manager

### Installation Steps

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/gigi.git
   cd gigi
   ```

2. Install dependencies:
   ```bash
   pnpm install
   ```

3. Build the project:
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

### Examples

```bash
# Start with custom nickname
pnpm run dev -- --nickname "my-tui-client"

# Connect to specific bootstrap node
pnpm run dev -- --host 192.168.1.100 --port 8080

# Start and send initial message
pnpm run dev -- --message "Hello, Gigi!"
```

## Command-Line Commands

Within the TUI interface, you can use the following commands:

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

TUI displays current connection and activity status:
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

## Contribution

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch
3. Commit changes
4. Push to the branch
5. Open a Pull Request

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Related Projects

- [@gigi/p2p](https://github.com/your-username/gigi/tree/main/packages/p2p) - Gigi P2P client library
- [@gigi/amp](https://github.com/your-username/gigi/tree/main/packages/amp) - Agent Messaging Protocol implementation
- [@gigi/logging](https://github.com/your-username/gigi/tree/main/packages/logging) - Structured logging library
