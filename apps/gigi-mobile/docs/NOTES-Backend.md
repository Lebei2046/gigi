# Gigi Mobile Integration Specification

## Overview

This document describes the functions and architecture of the rust backend of `gigi-mobile`, which provides a high-level command and event interfacethe with integration of `gigi-p2p` as the network layer. The architecture separates concerns cleanly:

- **gigi-mobile**: Mobile application that consumes the rust backend
- **rust backend*: High-level command/event API layer over `gigi-p2p`
- **gigi-p2p**: Low-level P2P networking library (libp2p-based)

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   gigi-mobile   │───▶│   rust backend   │───▶│   gigi-p2p      │
│  (Mobile App)   │    │ (Command API)    │    │ (Network Layer) │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │                         │
                              ▼                         ▼
                       Commands/Events            Network Events
```

## Integration Requirements

### Configuration

Configuration for paths and settings:

```rust
  /// Path to share.json file for file sharing metadata
  pub share_json_path: PathBuf,
  
  /// Directory path for saving downloaded files
  pub downloads_dir: PathBuf,
  
  /// Directory path for temporary files during transfer
  pub temp_dir: PathBuf,
  
  /// Maximum file size for sharing (in bytes)
  pub max_file_size: u64,
  
  /// Chunk size for file transfers
  pub chunk_size: usize,
```

### Key Management

Enhanced key management with frontend-provided private keys for the creation of swarm in `gigi-p2p`.


## Implementation the counterpart of the Tauri commands and events

### P2pClient

The main client for P2P networking.

#### Methods -> Commands

- `start_listening(address)` - Start listening on an address
- `send_direct_message(nickname, message)` - Send a direct message
- `send_direct_image(nickname, image_path)` - Send an image
- `join_group(group_name)` - Join a group
- `leave_group(group_name)` - Leave a group
- `send_group_message(group_name, message)` - Send group message
- `send_group_image(group_name, image_path)` - Send image to group (async)
- `share_file(file_path)` - Share a file (returns share code)
- `unshare_file(share_code)` - Remove shared file record
- `download_file(nickname, share_code)` - Download a shared file
- `list_peers()` - Get list of discovered peers
- `list_shared_files()` - Get list of shared files

### Events -> Emitted Events in the swarm loop

All P2P events are emitted through the event receiver:

- `PeerDiscovered` - New peer discovered
- `PeerExpired` - Peer disconnected
- `DirectMessage` - Received direct message
- `DirectImageMessage` - Received direct image
- `GroupMessage` - Received group message
- `GroupImageMessage` - Received group image
- `FileShareRequest` - File share offer received
- `FileShared` - File successfully shared
- `FileRevoked` - File share revoked
- `FileInfoReceived` - File information received
- `ChunkReceived` - File chunk received
- `FileDownloadStarted` - Download started
- `FileDownloadProgress` - Download progress update
- `FileDownloadCompleted` - Download completed
- `FileDownloadFailed` - Download failed
- `ListeningOn` - Client started listening
- `Connected` - Connected to peer
- `Disconnected` - Disconnected from peer
- `Error` - Error occurred

