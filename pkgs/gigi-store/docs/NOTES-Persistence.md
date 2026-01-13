# Message Persistence and Offline Delivery - Plan B

## Overview

This document describes the implementation of Plan B for message persistence in Gigi P2P: **Message Queue + Periodic Sync**.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│              Message Persistence Layer                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │  SQLite DB   │  │  Offline     │  │  Message     │         │
│  │  (Messages)  │  │  Queue      │  │  Indexing    │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│              Sync Manager                                        │
│  - Detect peers online                                          │
│  - Push pending messages                                        │
│  - Retry failed delivery                                        │
│  - Cleanup expired messages                                    │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│              P2P Network Layer (gigi-p2p)                        │
└─────────────────────────────────────────────────────────────────┘
```

## Implementation Status

### ✅ Completed Modules

1. **Message Store** (`src/persistence/message_store.rs`)
   - SQLite-based persistent storage
   - CRUD operations for messages
   - Offline queue management
   - Retry mechanism with exponential backoff
   - Automatic cleanup of expired messages
   - Conversation history retrieval
   - Unread message counting

2. **Sync Manager** (`src/persistence/sync_manager.rs`)
   - Peer online/offline detection
   - Pending message synchronization
   - Retry logic for failed delivery
   - Periodic cleanup tasks
   - Sync state tracking

3. **Sync Protocol** (`src/persistence/sync_protocol.rs`)
   - Message types for sync communication
   - Request/response patterns
   - Acknowledgments (delivered/read)
   - History synchronization

4. **Data Structures** (`src/events.rs`)
   - `StoredMessage`: Persistent message representation
   - `OfflineQueueItem`: Offline delivery queue
   - `MessageContent`: Text, FileShare, ShareGroup
   - `SyncStatus`: Pending, Synced, Delivered, Acknowledged
   - `QueueStatus`: Pending, InProgress, Delivered, Expired

5. **Configuration** (`src/persistence/mod.rs`)
   - `PersistenceConfig`: Database path, intervals, TTL, retry limits
   - Helper functions: `create_outbound_message()`, `create_inbound_message()`

### 📝 Usage Example

```rust
use gigi_p2p::{
    events::{MessageContent, MessageDirection},
    persistence::{PersistenceConfig, SyncManager},
    P2pClient,
};
use libp2p::identity::Keypair;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize message store
    let config = PersistenceConfig {
        db_path: "messages.db".into(),
        sync_interval_seconds: 30,
        retry_interval_seconds: 300,
        cleanup_interval_seconds: 3600,
        message_ttl_seconds: 7 * 24 * 3600, // 7 days
        max_retry_attempts: 10,
        max_batch_size: 50,
    };

    let message_store = gigi_p2p::MessageStore::with_config(config).await?;
    let sync_manager = SyncManager::new(
        std::sync::Arc::new(message_store.clone()),
        "Alice".to_string(),
    );

    // Create and store a message
    let msg = StoredMessage {
        id: uuid::Uuid::new_v4().to_string(),
        msg_type: MessageType::Direct,
        direction: MessageDirection::Sent,
        content: MessageContent::Text {
            text: "Hello, World!".to_string(),
        },
        sender_nickname: "Alice".to_string(),
        recipient_nickname: Some("Bob".to_string()),
        group_name: None,
        peer_id: peer_id.to_string(),
        timestamp: Utc::now(),
        created_at: Utc::now(),
        delivered: false,
        delivered_at: None,
        read: false,
        read_at: None,
        sync_status: SyncStatus::Pending,
        sync_attempts: 0,
        last_sync_attempt: None,
        expires_at: Utc::now() + chrono::Duration::days(1),
    };

    message_store.store_message(msg.clone()).await?;

    // If Bob is offline, enqueue for later delivery
    message_store.enqueue_offline(msg.id, "Bob".to_string()).await?;

    // When Bob comes online
    let pending = message_store.get_pending_messages("Bob", 50).await?;

    // Send messages to Bob via P2P
    for msg in pending {
        // ... send via P2P ...
        message_store.mark_delivered(&msg.id).await?;
    }

    // Run periodic sync tasks
    tokio::spawn(async move {
        sync_manager.run_sync_tasks(|peer_id, sync_msg| async move {
            // Send sync message via P2P
            Ok(())
        }).await;
    });

    Ok(())
}
```

### 🧪 Demo

Run the persistence demo:

```bash
cd /home/lebei/dev/gigi/pkgs/gigi-p2p
cargo run --example persistence_demo -- Alice
```

### 📊 Database Schema

```sql
-- Messages table
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    msg_type TEXT NOT NULL,         -- 'Direct' | 'Group'
    direction TEXT NOT NULL,         -- 'Sent' | 'Received'
    content_type TEXT NOT NULL,
    content_json TEXT NOT NULL,
    sender_nickname TEXT NOT NULL,
    recipient_nickname TEXT,         -- NULL for group messages
    group_name TEXT,                -- NULL for direct messages
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

-- Offline queue table
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

-- Message acknowledgments table
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

## Next Steps

### 🚧 Integration with P2pClient

To integrate persistence with existing `P2pClient`:

1. Add `MessageStore` and `SyncManager` to `P2pClient` struct
2. Implement persistent message sending:
   - Store message in DB
   - If peer online, send immediately
   - If offline, enqueue for later
3. Handle peer online events to trigger sync
4. Add sync protocol to `UnifiedBehaviour`

### 🔧 Required Changes

1. **behaviour.rs** - Add sync protocol:
   ```rust
   pub struct UnifiedBehaviour {
       pub gigi_dns: GigiDnsBehaviour,
       pub direct_msg: request_response::cbor::Behaviour<DirectMessage, DirectResponse>,
       pub gossipsub: gossipsub::Behaviour,
       pub file_sharing: request_response::cbor::Behaviour<FileSharingRequest, FileSharingResponse>,
       pub sync: request_response::cbor::Behaviour<SyncMessage, SyncMessage>, // NEW
   }
   ```

2. **client/p2p_client.rs** - Add persistence:
   ```rust
   pub struct P2pClient {
       // ... existing fields ...
       pub message_store: Option<Arc<MessageStore>>,
       pub sync_manager: Option<Arc<SyncManager>>,
   }
   ```

3. **client/event_handler.rs** - Add sync event handler:
   ```rust
   pub struct SyncEventHandler {
       pub client: P2pClient,
   }
   ```

### 📋 TODO

- [ ] Integrate SyncProtocol into UnifiedBehaviour
- [ ] Add SyncEventHandler to handle sync events
- [ ] Update P2pClient to use MessageStore
- [ ] Implement persistent `send_message()` method
- [ ] Add `get_history()` method to P2pClient
- [ ] Add `mark_as_read()` method to P2pClient
- [ ] Write integration tests
- [ ] Update chat example to use persistence
- [ ] Add CloudBase backup option (optional)

## Configuration

```toml
[dependencies]
# Added for persistence
rusqlite = { version = "0.30", features = ["bundled", "chrono"] }
tokio-rusqlite = "0.5"
```

## Technical Details

### Retry Strategy

- **Exponential backoff**: 5, 10, 20, 40, 80, 160, 320, 640, 1280, 2560 minutes
- **Max retries**: 10 attempts
- **Initial interval**: 5 minutes
- **Cleanup interval**: 1 hour

### Message TTL

- **Default**: 7 days (604800 seconds)
- **Configurable** via `PersistenceConfig`

### Sync Interval

- **Default**: 30 seconds
- **Configurable** via `PersistenceConfig`

## Advantages of Plan B

✅ **Simple and reliable**: SQLite is mature and stable
✅ **No external dependencies**: Uses well-tested Rust crates
✅ **Offline support**: Messages stored locally and synced when peers come online
✅ **Retry mechanism**: Failed delivery is automatically retried with exponential backoff
✅ **Cleanup**: Expired messages are automatically removed
✅ **History**: Full conversation history is stored and can be retrieved
✅ **Read receipts**: Message read status is tracked
✅ **Scalable**: Handles thousands of messages efficiently

## Comparison with Plan A (CRDT)

| Feature | Plan B (SQLite) | Plan A (OctoBase CRDT) |
|---------|----------------|------------------------|
| Complexity | Low | High |
| Dependencies | Stable (rusqlite) | Unstable (OctoBase in development) |
| Conflict Resolution | Not needed (single writer per message) | Automatic (CRDT) |
| Offline Sync | Manual implementation | Built-in |
| Convergence | N/A | Automatic |
| Learning Curve | Low | High |
| Performance | Good | Good |
| Storage Size | Small (compressed) | Larger (CRDT metadata) |

## Conclusion

Plan B provides a practical, reliable solution for message persistence without depending on unstable libraries. The implementation is complete and ready for integration with the existing P2P client.
