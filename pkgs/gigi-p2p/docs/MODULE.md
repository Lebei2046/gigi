# Gigi P2P Module Structure

This document describes the modular architecture of the gigi-p2p library, which has been refactored from a single large file into focused, behavior-specific modules.

## Module Overview

### `lib.rs` - Main Library Interface
- Re-exports all public API components
- Provides a clean and unified interface for users
- Re-exports commonly used libp2p types for convenience

### `error.rs` - Error Types
- Defines all P2P-specific error types
- Implements `Debug` and `Display` traits
- Provides detailed error context for troubleshooting

### `events.rs` - Events and Public Data Structures
- Defines the unified `P2pEvent` enum covering all P2P activities
- Contains public data structures used in the API:
  - `FileInfo` - File metadata and chunking information
  - `ChunkInfo` - Individual chunk data and verification
  - `SharedFile` - Shared file state and metadata
  - `PeerInfo` - Peer connection and discovery state
  - `GroupInfo` - Group subscription state
  - `GroupMessage` - Message format for group communications

### `behaviour.rs` - Network Protocols and Behaviors
- Defines all protocol message types:
  - `NicknameRequest/Response` - Peer nickname exchange
  - `DirectMessage/Response` - Direct communication
  - `FileTransferRequest/Response` - File sharing operations
- Implements `UnifiedBehaviour` combining all protocols
- Defines `UnifiedEvent` for internal event handling
- Provides helper functions for gossipsub configuration and behavior creation

### `file_transfer.rs` - File Management and Transfer
- Handles file chunking (256KB chunks by default)
- Manages shared file state and persistence
- Provides download management with hash verification
- Implements chunk-based file transfer with sliding window
- Handles share code generation and validation

### `client.rs` - Main P2P Client Implementation
- Implements the primary `P2pClient` struct
- Handles event routing and processing
- Provides all public API methods:
  - Peer discovery and management
  - Direct messaging and file sharing
  - Group operations
  - File download and upload operations
- Manages internal state and event dispatching

## Benefits of This Structure

1. **Separation of Concerns**: Each module has a single, well-defined responsibility
2. **Maintainability**: Smaller, focused files are easier to understand and modify
3. **Testability**: Individual modules can be tested in isolation
4. **Reusability**: Components can be reused or replaced as needed
5. **Clear API Surface**: The lib.rs re-exports provide a clean public interface
6. **Reduced Compilation Time**: Changes to one module don't require recompiling everything

## Public API

The public API remains unchanged and includes:
- `P2pClient` - Main client for P2P operations
- `P2pEvent` - All possible P2P events
- `P2pError` - Error types for P2P operations
- Data structures: `FileInfo`, `ChunkInfo`, `SharedFile`, `PeerInfo`, `GroupInfo`, `GroupMessage`
- Constants: `CHUNK_SIZE`

All internal implementation details are properly encapsulated and hidden from the public API.