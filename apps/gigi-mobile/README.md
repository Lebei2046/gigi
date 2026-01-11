# Gigi Mobile

A decentralized peer-to-peer (P2P) mobile messaging application built with React, TypeScript, and Tauri. Gigi Mobile enables secure, private communication without relying on centralized servers.

## 🚀 Features

### Core Messaging
- **Direct Messaging**: P2P chat between connected peers
- **Group Messaging**: Create and join chat groups for multi-user conversations
- **Real-time Communication**: Instant message delivery using WebRTC and libp2p
- **Message History**: Local storage of chat history using IndexedDB

### User Experience
- **Intuitive Onboarding**: Simple mnemonic-based wallet creation
- **Contact Management**: Add friends via QR code scanning
- **Profile Customization**: Personal avatars and nicknames
- **Responsive Design**: Mobile-optimized interface with Tailwind CSS

### Security & Privacy
- **End-to-End Encryption**: Messages secured with cryptography
- **Decentralized Identity**: Peer-to-peer identity management
- **Local Data Storage**: All data stored locally on device
- **No Server Dependencies**: True P2P architecture

### File Sharing
- **File Transfer**: Share files directly with peers
- **Progress Tracking**: Real-time download progress
- **Thumbnail Support**: Image previews and file metadata

## 🏗️ Architecture

### Frontend Stack
- **React 19**: Modern reactive UI framework
- **TypeScript**: Type-safe development
- **Tauri 2**: Cross-platform desktop/mobile framework
- **Tailwind CSS**: Utility-first styling
- **Radix UI**: Accessible component library
- **Redux Toolkit**: State management
- **React Router**: Navigation and routing
- **Dexie**: IndexedDB wrapper for local storage

### Backend Stack
- **Rust**: High-performance backend
- **libp2p**: P2P networking protocol
- **WebRTC**: Real-time communication
- **Tokio**: Async runtime
- **Serde**: Serialization framework

### Key Libraries
- `@noble/ciphers`, `@noble/hashes`, `@noble/secp256k1`: Cryptographic operations
- `@scure/bip32`, `@scure/bip39`: Mnemonic and key derivation
- `react-qr-code`, `@yudiel/react-qr-scanner`: QR code functionality
- `dexie`, `dexie-react-hooks`: Local database management

## 📱 App Structure

```
src/
├── components/          # Reusable UI components
├── features/           # Feature-based modules
│   ├── chat/          # Chat interface and messaging
│   ├── home/          # Main dashboard
│   ├── me/            # Profile and settings
│   ├── signin/        # Authentication flow
│   └── signup/       # Onboarding flow
├── hooks/             # Custom React hooks
├── lib/               # Utility libraries
├── models/            # Data models and schemas
├── store/             # Redux state management
└── utils/             # Utility functions
    ├── peerUtils.ts   # Peer/Group ID formatting
    ├── messaging.ts    # P2P messaging utilities
    └── storage.ts     # Local storage helpers
```

## 🛠️ Development

### Prerequisites
- Node.js 18+ 
- Bun (package manager)
- Rust (for Tauri backend)

### Installation
```bash
# Clone the repository
git clone <repository-url>
cd gigi/apps/gigi-mobile

# Install dependencies
bun install

# Set up Rust toolchain
rustup target add x86_64-pc-windows-msvc  # Windows
rustup target add aarch64-apple-darwin     # macOS ARM
rustup target add x86_64-apple-darwin      # macOS Intel
```

### Development Commands
```bash
# Start development server
bun run dev

# Start Tauri development with different configs
bun run tauri          # Default config
bun run tauri:i2       # Instance 2
bun run tauri:i3       # Instance 3

RUST_LOG=gigi_dns=warn,gigi_p2p=info bun tauri dev # Logging

# Build for production
bun run build

# Testing
bun run test

# Code quality
bun run lint          # ESLint
bun run lint:fix      # Fix linting issues
bun run format        # Prettier formatting
bun run format:check  # Check formatting
bun run format-all    # Format everything
```

### Development for Mobile
```bash
# Development mode with Android emulator
$ANDROID_HOME/emulator/emulator -list-avds
$ANDROID_HOME/emulator/emulator -avd Medium_Phone_API_36.1
bun run tauri android dev

# Note: Mobile development may require additional platform-specific setup
```

### Building for Different Platforms
```bash
# Build for all platforms
bun run tauri build

# Specific platform builds
bun run tauri build --target x86_64-pc-windows-msvc  # Windows
bun run tauri build --target aarch64-apple-darwin     # macOS ARM
bun run tauri build --target x86_64-apple-darwin      # macOS Intel
```

## 🔧 Configuration

### Environment Variables
Create a `.env` file in the root directory:
```env
# Development settings
VITE_DEBUG_MODE=true
VITE_LOG_LEVEL=debug

# Network settings
VITE_DEFAULT_PORT=0  # 0 = random port
VITE_BOOTSTRAP_NODES=[]
```

### Tauri Configuration
Configuration files are located in `src-tauri/`:
- `tauri.conf.json` - Main configuration
- `tauri-i2.conf.json` - Instance 2 configuration  
- `tauri-i3.conf.json` - Instance 3 configuration

## 🔑 Peer & Group ID Formatting

Gigi Mobile uses shortened IDs for better UX:
- **Format**: First 6 characters + "..." + last 6 characters
- **Example**: `12ab34cd56ef78gh90ij34kl56mn78op90qr` → `12ab34...op90qr`
- **Short IDs**: 12 characters or less are shown in full

### Usage
```typescript
import { formatShortPeerId, formatShortGroupId } from '@/utils/peerUtils'

// Format peer IDs
const shortPeerId = formatShortPeerId('12ab34cd56ef78gh90ij34kl56mn78op90qr')
// Returns: "12ab34...op90qr"

// Format group IDs (alias function)
const shortGroupId = formatShortGroupId('group1234567890abcdef1234567890')
// Returns: "group1...567890"
```

## 📱 User Guide

### Getting Started
1. **Create Account**: Generate a mnemonic phrase during signup
2. **Set Profile**: Choose a nickname and avatar
3. **Create Groups**: Optionally create your first chat group
4. **Connect**: Discover and connect with nearby peers

### Adding Friends
- **QR Code**: Share your QR code with friends
- **Scan Code**: Use the built-in scanner to add contacts
- **Direct Connection**: Connect directly via peer discovery

### Group Chat
- **Create Groups**: Generate groups from your mnemonic
- **Share Groups**: Invite friends to join your groups
- **Join Groups**: Accept group invitations from others

## 🧪 Testing

### Running Tests
```bash
# Run all tests
bun run test

# Run specific test files
bun run test src/utils/__tests__/peerUtils.test.ts

# Watch mode
bun run test --watch
```

### Test Coverage
- Unit tests for utility functions
- Component tests for UI components
- Integration tests for P2P functionality

## 🐛 Debugging

### Debug Mode
Enable debug mode in development:
- **Debug Panel**: Press `Ctrl+Shift+D` to clear IndexedDB
- **Console Logs**: Detailed logging in browser console
- **P2P Logs**: Real-time P2P event monitoring

### Common Issues
1. **P2P Connection**: Check network settings and firewall
2. **Mnemonic Issues**: Verify phrase is correctly entered
3. **Storage**: Clear IndexedDB if data corruption occurs

## 🤝 Contributing

### Development Workflow
1. Fork the repository
2. Create a feature branch
3. Make changes with proper testing
4. Ensure code passes linting and formatting
5. Submit a pull request

### Code Style
- Use TypeScript for type safety
- Follow ESLint configuration
- Use Prettier for formatting
- Write meaningful commit messages

## 📄 License

This project is licensed under the MIT License. See the LICENSE file for details.

## 🔗 Related Projects

- **gigi-p2p**: Core P2P networking library
- **gigi-desktop**: Desktop version of the application
- **libp2p**: Decentralized networking protocol

## 📞 Support

For support and questions:
- Check the GitHub Issues section
- Review the documentation in `/NOTES-*.md` files
- Join the development community discussions

---

**Built with ❤️ for decentralized communication**