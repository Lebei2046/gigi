## Offline Message Processing Flow

### **Step 1: Message is Sent While Recipient is Offline**

When `send_direct_message()` is called in `p2p_client.rs`, the system checks if the recipient is online:

```rust
// p2p_client.rs:700-877
let peer_id = self.peer_manager.get_peer_id_by_nickname(nickname);

match peer_id {
    Some(peer_id) => {
        if let Some(peer) = self.peer_manager.get_peer(&peer_id) {
            if peer.connected {
                // Peer is online - send immediately
                self.swarm.behaviour_mut().direct_msg.send_request(...);
            } else {
                // Peer exists but is NOT connected → Store for later delivery
                self.store_and_enqueue_offline_message(...);
            }
        }
    }
    None => {
        // Peer not found at all → Store for later delivery
        self.store_and_enqueue_offline_message(...);
    }
}
```

### **Step 2: Message Storage & Queueing**

When the recipient is offline, the message is:

1. **Stored in the message store** with metadata:
   - `sync_status: Pending` - marked for future delivery
   - `delivered: false` - not yet delivered
   - `expires_at: 7 days from now` - auto-expires after 7 days
   - `sync_attempts: 0` - no delivery attempts yet

2. **Added to the offline queue** via `enqueue_offline(message_id, nickname)`:
   ```rust
   // p2p_client.rs:664, 756, 810, 865
   msg_clone.enqueue_offline(message_id, nickname.to_string()).await?;
   ```

### **Step 3: UI Feedback for Sender**

The sender sees immediate feedback:
- Message appears in the chat UI **instantly** (added synchronously to state)
- Message is marked as "pending" visually (though the current UI may not explicitly show this status)
- The `sending` flag is set to `false` after the async operation completes

### **Step 4: Automatic Delivery When Recipient Comes Online**

When the recipient reconnects, the system automatically delivers pending messages:

**Trigger**: `P2pEvent::Connected` event fires in `hooks.rs`:

```rust
// hooks.rs:495-510
gigi_p2p::P2pEvent::Connected { peer_id, nickname } => {
    // ... peer status updates ...
    
    spawn(async move {
        if let Err(e) = P2pService::deliver_pending_messages(&nickname_clone).await {
            println!("Failed to deliver pending messages to {}: {:?}", nickname_clone, e);
        } else {
            println!("Delivered pending messages to {}", nickname_clone);
        }
    });
}
```

**Delivery Process** (`send_pending_messages` in `p2p_client.rs:1520-1566`):

1. Retrieves all pending messages for the peer (up to 100 at a time)
2. Sends each message via P2P:
   ```rust
   self.swarm.behaviour_mut().direct_msg.send_request(
       &peer_id,
       crate::behaviour::DirectMessage::Text { message: text },
   );
   ```
3. Updates the message store:
   - Fills in `peer_id` if it was empty (for peers discovered after message was queued)
   - Marks messages as sent via `mark_message_sent(&msg.id)`

### **Step 5: Message Retention & Cleanup**

- **Expiration**: Messages expire after **7 days** (`expires_at` field)
- **Cleanup Task**: A periodic cleanup task (`start_cleanup_task`) removes expired messages
- **Sync Manager**: The `SyncManager` tracks sync states and handles retry logic for failed deliveries

---

## Key Technical Details

| Aspect | Implementation |
|--------|---------------|
| **Storage Layer** | `gigi_store::MessageStore` (SQLite via `gigi-store`) |
| **Queue Mechanism** | `enqueue_offline()` adds to offline queue table |
| **Delivery Trigger** | `Connected` event from P2P network |
| **Batch Size** | Up to 100 messages per delivery attempt |
| **Expiration** | 7 days from creation |
| **Retry Logic** | `SyncManager` with configurable retry intervals |

---

## Summary Diagram

```
Sender sends message
        ↓
Recipient online?
        ├─ Yes → Send immediately via P2P
        └─ No  → Store in DB + Add to offline queue
                       ↓
              [Message waits with status "Pending"]
                       ↓
         Recipient connects (Connected event)
                       ↓
              deliver_pending_messages() called
                       ↓
              Retrieve pending messages (≤100)
                       ↓
              Send each via P2P
                       ↓
              Mark as sent in DB
```