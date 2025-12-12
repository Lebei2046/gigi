# Gigi Mobile Rust Backend

This directory contains the Rust backend for the Gigi mobile application, implemented directly in the Tauri app using the `gigi-p2p` library.

## Architecture

The backend integrates the `gigi-p2p` library which provides a complete peer-to-peer networking implementation with:
- Auto Discovery via mDNS
- Nickname Exchange via request-response
- Direct Messaging via request-response  
- Group Messaging via Gossipsub
- File Transfer via request-response
- Unified event system

## Data Persistence

### Shared Files
Shared files information is persisted to disk in JSON format:
- **Location**: `{download_folder}/shared_files.json`
- **Format**: JSON object mapping share codes to file metadata
- **Auto-loading**: Shared files are automatically loaded when the application starts
- **Auto-saving**: Shared files are automatically saved when new files are shared
- **Manual control**: Use `messaging_save_shared_files()` to manually save and `messaging_remove_shared_file()` to remove entries

The persistence ensures that shared files are remembered across application restarts, maintaining the list of files that were previously shared along with their metadata (name, size, mime type, etc.).

## Features

### Core Messaging Commands
- `messaging_initialize_with_key(private_key)` - Initialize with existing private key

### Peer Management
- `messaging_get_peers()` - Get discovered peers
- `messaging_set_nickname(nickname)` - Set user nickname

### Direct Messaging
- `messaging_send_message_to_nickname(nickname, message)` - Send direct message to peer by nickname
- `messaging_send_message(to_peer_id, message)` - Legacy command (use nickname version instead)

### Group Messaging
- `messaging_join_group(group_id)` - Join a group
- `messaging_send_group_message(group_id, message)` - Send group message

### File Sharing
- `messaging_share_file(file_path)` - Share a file (returns share code)
- `messaging_request_file_from_nickname(nickname, shareCode)` - Request file by nickname
- `messaging_request_file(file_id, from_peer_id)` - Legacy command (use nickname version)
- `messaging_cancel_download(download_id)` - Cancel active download
- `messaging_get_shared_files()` - Get locally shared files
- `messaging_remove_shared_file(shareCode)` - Remove file from shared files
- `messaging_save_shared_files()` - Manually save shared files to disk

### Configuration & State
- `messaging_get_public_key()` - Get public key
- `messaging_get_active_downloads()` - Get active downloads
- `messaging_update_config(config)` - Update configuration
- `messaging_get_config()` - Get current configuration

### Utility Commands
- `get_peer_id()` - Get current peer ID
- `try_get_peer_id(priv_key)` - Try to get peer ID from private key

## Events

The backend emits the following Tauri events that can be listened to from the frontend:

### Discovery Events
- `peer-discovered` - When a new peer is discovered
- `peer-expired` - When a peer expires from mDNS
- `peer-connected` - When connection to peer is established
- `peer-disconnected` - When connection to peer is lost
- `nickname-updated` - When a peer updates their nickname

### Messaging Events
- `direct-message` - When a direct message is received
- `group-message` - When a group message is received

### File Sharing Events
- `file-share-request` - When someone offers to share a file
- `download-progress` - Download progress updates
- `download-completed` - When a download completes
- `download-failed` - When a download fails

### System Events
- `peer-id-changed` - When peer ID changes
- `public-key-changed` - When public key changes
- `nickname-changed` - When local nickname changes
- `config-changed` - When configuration changes
- `p2p-error` - When a P2P error occurs

## Implementation Details

- Uses the `gigi-p2p` crate for all P2P functionality
- Maintains P2P client state in `Arc<Mutex<Option<P2pClient>>>`
- Handles events asynchronously with tokio tasks
- Automatically starts mDNS discovery and listening on random ports
- Supports chunked file transfers with resumable downloads
- Provides real-time event emission to frontend

## Network Protocols

- **mDNS**: Automatic peer discovery on local network
- **Request-Response**: Direct messaging and file transfers
- **Gossipsub**: Group messaging with pub/sub model
- **TCP/QUIC**: Transport layer with noise encryption

## File Sharing

- Files are shared using 8-character share codes (Blake3 hash)
- Large files are transferred in 256KB chunks
- Download progress is tracked and reported via events
- Files are automatically verified using SHA256 hashes
- Shared files list is persisted to `shared.json`