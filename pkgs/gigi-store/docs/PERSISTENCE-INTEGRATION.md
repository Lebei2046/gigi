# Message Persistence Integration Guide

This document explains how to integrate the message persistence feature into the P2pClient.

## Overview

The message persistence system provides:
- **Offline message queuing**: Messages are stored when peers are offline
- **Automatic retry**: Messages are re-delivered when peers come back online
- **Message history**: Full conversation history is maintained
- **Read receipts**: Track when messages are delivered and read

## Architecture

### Components

1. **MessageStore**: SQLite-based persistent storage
   - Stores all messages with metadata
   - Manages offline queue
   - Tracks delivery status

2. **SyncManager**: Coordinates message synchronization
   - Handles peer online/offline events
   - Manages retry logic with exponential backoff
   - Periodic cleanup of expired messages

3. **SyncProtocol**: P2P protocol for message sync
   - SyncRequest/Response messages
   - Message acknowledgments
   - History queries

### Integration Points

1. **behaviour.rs**: Add `SyncMessage` protocol to `UnifiedBehaviour`
   - Already implemented with `/sync/1.0.0` protocol

2. **p2p_client.rs**: Add persistence API methods
   - `send_persistent_message()`: Send with offline queue
   - `get_conversation_history()`: Retrieve history
   - `get_unread_count()`: Count unread messages

3. **event_handler.rs**: Handle sync events
   - `SyncMessageEventHandler`: Process sync protocol messages

## Usage Example

### Basic Setup

```rust
use gigi_p2p::{P2pClient, PersistenceConfig};
use libp2p::{identity, Multiaddr};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create keypair
    let keypair = identity::Keypair::generate_ed25519();

    // Configure persistence
    let persistence_config = PersistenceConfig {
        db_path: PathBuf::from("gigi-messages.db"),
        sync_interval_seconds: 30,
        retry_interval_seconds: 300,
        cleanup_interval_seconds: 3600,
        message_ttl_seconds: 7 * 24 * 3600, // 7 days
        max_retry_attempts: 10,
        max_batch_size: 50,
    };

    // Create client with persistence
    let (mut client, mut event_rx) = P2pClient::new_with_config(
        keypair,
        "alice".to_string(),
        PathBuf::from("./downloads"),
        PathBuf::from("./shared.json"),
        Some(persistence_config),
    ).await?;

    // Start listening
    client.start_listening("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Handle events
    while let Some(event) = event_rx.next().await {
        match event {
            P2pEvent::Connected { peer_id, nickname } => {
                println!("Peer {} ({}) connected", nickname, peer_id);
            }
            P2pEvent::DirectMessage { from_nickname, message, .. } => {
                println!("Message from {}: {}", from_nickname, message);
            }
            P2pEvent::PendingMessagesAvailable { nickname, .. } => {
                println!("Offline messages available for {}", nickname);
            }
            _ => {}
        }
    }

    Ok(())
}
```

### Sending Persistent Messages

```rust
// Send message - automatically handles offline queue
async fn send_message(
    client: &mut P2pClient,
    nickname: &str,
    message: String,
) -> anyhow::Result<String> {
    let message_id = client.send_persistent_message(nickname, message).await?;
    println!("Message sent with ID: {}", message_id);
    Ok(message_id)
}
```

### Retrieving Conversation History

```rust
// Get conversation history
async fn get_history(
    client: &P2pClient,
    nickname: &str,
) -> anyhow::Result<Vec<StoredMessage>> {
    let messages = client
        .get_conversation_history(nickname, 50, 0)
        .await?;
    Ok(messages)
}
```

### Marking Messages as Read

```rust
// Mark message as read
async fn mark_read(client: &P2pClient, message_id: &str) -> anyhow::Result<()> {
    client.mark_message_as_read(message_id).await?;
    Ok(())
}
```

## Sync Protocol Flow

### When Peer Comes Online

```
1. P2pClient detects peer connection
2. SyncManager.on_peer_online() is called
3. Retrieve pending messages from queue
4. Send SyncRequest to peer
5. Peer responds with SyncResponse containing messages
6. Update message status to 'Delivered'
```

### When Peer is Offline

```
1. Send message via send_persistent_message()
2. Check if peer is connected
3. If connected: Send immediately, mark as 'Delivered'
4. If offline: Store message, add to offline queue
5. Queue for retry with exponential backoff
```

### Retry Mechanism

Messages are retried with exponential backoff:
- Attempt 1: 5 minutes
- Attempt 2: 10 minutes
- Attempt 3: 20 minutes
- Attempt 4: 40 minutes
- ...
- Attempt 10: 2560 minutes

After max attempts, message is marked as 'Expired'.

## Database Schema

### Messages Table
```sql
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    msg_type TEXT NOT NULL,
    direction TEXT NOT NULL,
    content_type TEXT NOT NULL,
    content_json TEXT NOT NULL,
    sender_nickname TEXT NOT NULL,
    recipient_nickname TEXT,
    group_name TEXT,
    peer_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    delivered BOOLEAN DEFAULT FALSE,
    delivered_at INTEGER,
    read BOOLEAN DEFAULT FALSE,
    read_at INTEGER,
    sync_status TEXT DEFAULT 'Pending',
    sync_attempts INTEGER DEFAULT 0,
    last_sync_attempt INTEGER,
    expires_at INTEGER NOT NULL
);
```

### Offline Queue Table
```sql
CREATE TABLE offline_queue (
    message_id TEXT PRIMARY KEY,
    target_nickname TEXT NOT NULL,
    target_peer_id TEXT,
    queued_at INTEGER NOT NULL,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 10,
    last_retry_at INTEGER,
    next_retry_at INTEGER,
    expires_at INTEGER NOT NULL,
    status TEXT DEFAULT 'Pending',
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);
```

### Message Acknowledgments Table
```sql
CREATE TABLE message_acknowledgments (
    id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL,
    acknowledged_by_nickname TEXT NOT NULL,
    acknowledged_by_peer_id TEXT NOT NULL,
    acknowledged_at INTEGER NOT NULL,
    ack_type TEXT NOT NULL,
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
);
```

## Current Implementation Status

### Completed
- ✅ MessageStore with SQLite
- ✅ SyncManager with retry logic
- ✅ SyncProtocol message types
- ✅ PersistenceConfig
- ✅ Helper functions for creating messages
- ✅ Unit tests for message storage

### Integration Work Needed
- ⚠️ P2pClient async initialization with persistence
- ⚠️ Sync message handling in event loop
- ⚠️ Connection with tokio-rusqlite for thread safety
- ⚠️ Periodic sync task management
- ⚠️ Integration tests

### Known Issues
1. `tokio-rusqlite` requires different async patterns than `rusqlite`
2. Need to restructure MessageStore initialization to be fully async
3. P2pClient::new_with_config needs to handle async persistence setup

## Testing

Run unit tests:
```bash
cd pkgs/gigi-p2p
cargo test persistence
```

Run the persistence demo:
```bash
cd pkgs/gigi-p2p
cargo run --example persistence_demo -- Alice
```

## Next Steps

1. **Fix async initialization**: Make `P2pClient::new_with_config` properly async
2. **Thread-safe database**: Complete migration to `tokio-rusqlite`
3. **Event loop integration**: Connect sync events to main event loop
4. **Integration tests**: Add end-to-end tests
5. **Documentation**: Complete API documentation

## References

- Design document: `docs/NOTES-Persistence.md`
- Demo: `pkgs/gigi-p2p/examples/persistence_demo.rs`
- Source: `pkgs/gigi-p2p/src/persistence/`
