// Copyright 2024 Gigi Team.
//
// Comprehensive tests for MessageStore

use gigi_store::{
    message_store::MessageStore, MessageContent, MessageDirection, MessageType, PersistenceConfig,
    SyncStatus,
};
use tempfile::NamedTempFile;
use uuid::Uuid;

#[tokio::test]
async fn test_message_store_initialization() {
    let temp_file = NamedTempFile::new().unwrap();
    let store = MessageStore::new(temp_file.path().to_path_buf())
        .await
        .expect("Failed to create message store");
}

#[tokio::test]
async fn test_store_and_retrieve_message() {
    let temp_file = NamedTempFile::new().unwrap();
    let store = MessageStore::new(temp_file.path().to_path_buf())
        .await
        .expect("Failed to create message store");

    let msg_id = Uuid::new_v4().to_string();
    let msg = create_test_message(&msg_id, "Hello, World!");

    // Store the message
    store
        .store_message(msg.clone())
        .await
        .expect("Failed to store message");

    // Retrieve the message
    let retrieved = store
        .get_message(&msg_id)
        .await
        .expect("Failed to retrieve message");

    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, msg.id);
    assert_eq!(retrieved.sender_nickname, msg.sender_nickname);
    assert_eq!(retrieved.recipient_nickname, msg.recipient_nickname);
}

#[tokio::test]
async fn test_offline_queue_operations() {
    let temp_file = NamedTempFile::new().unwrap();
    let store = MessageStore::new(temp_file.path().to_path_buf())
        .await
        .expect("Failed to create message store");

    // Create and store a message
    let msg_id = Uuid::new_v4().to_string();
    let msg = create_test_message(&msg_id, "Hello, Bob!");
    store
        .store_message(msg)
        .await
        .expect("Failed to store message");

    // Enqueue for offline peer
    store
        .enqueue_offline(msg_id.clone(), "Bob".to_string())
        .await
        .expect("Failed to enqueue message");

    // Retrieve pending messages
    let pending = store
        .get_pending_messages("Bob", 10)
        .await
        .expect("Failed to get pending messages");

    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, msg_id);

    // Mark as sent
    store
        .mark_message_sent(&msg_id)
        .await
        .expect("Failed to mark message as sent");
}

#[tokio::test]
async fn test_conversation_history() {
    let temp_file = NamedTempFile::new().unwrap();
    let store = MessageStore::new(temp_file.path().to_path_buf())
        .await
        .expect("Failed to create message store");

    // Store multiple messages from same conversation
    for i in 0..5 {
        let msg_id = Uuid::new_v4().to_string();
        let mut msg = create_test_message(&msg_id, &format!("Message {}", i));
        msg.sender_nickname = "Alice".to_string();
        msg.recipient_nickname = Some("Bob".to_string());
        store.store_message(msg).await.unwrap();
    }

    // Retrieve conversation history
    let history = store
        .get_conversation("Bob", 10, 0)
        .await
        .expect("Failed to get conversation");

    assert_eq!(history.len(), 5);

    // Messages should be ordered by timestamp (descending)
    for i in 0..4 {
        assert!(
            history[i].timestamp >= history[i + 1].timestamp,
            "Messages should be ordered by timestamp"
        );
    }
}

#[tokio::test]
async fn test_mark_delivered() {
    let temp_file = NamedTempFile::new().unwrap();
    let store = MessageStore::new(temp_file.path().to_path_buf())
        .await
        .expect("Failed to create message store");

    let msg_id = Uuid::new_v4().to_string();
    let msg = create_test_message(&msg_id, "Test message");
    store.store_message(msg).await.unwrap();

    // Enqueue for offline peer (mark_delivered needs the queue item)
    store
        .enqueue_offline(msg_id.clone(), "Bob".to_string())
        .await
        .expect("Failed to enqueue message");

    // Mark as delivered
    store
        .mark_delivered(&msg_id)
        .await
        .expect("Failed to mark message as delivered");

    // Verify delivery status
    let retrieved = store.get_message(&msg_id).await.unwrap().unwrap();
    assert!(retrieved.delivered);
    assert!(retrieved.delivered_at.is_some());
}

#[tokio::test]
async fn test_mark_read() {
    let temp_file = NamedTempFile::new().unwrap();
    let store = MessageStore::new(temp_file.path().to_path_buf())
        .await
        .expect("Failed to create message store");

    let msg_id = Uuid::new_v4().to_string();
    let msg = create_test_message(&msg_id, "Test message");
    store.store_message(msg).await.unwrap();

    // Mark as read
    store
        .mark_read(&msg_id)
        .await
        .expect("Failed to mark message as read");

    // Verify read status
    let retrieved = store.get_message(&msg_id).await.unwrap().unwrap();
    assert!(retrieved.read);
    assert!(retrieved.read_at.is_some());
}

#[tokio::test]
async fn test_retry_with_exponential_backoff() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut config = PersistenceConfig::default();
    config.max_retry_attempts = 5; // Lower for faster testing
    config.db_path = temp_file.path().to_path_buf();

    let store = MessageStore::with_config(config)
        .await
        .expect("Failed to create message store");

    let msg_id = Uuid::new_v4().to_string();
    let msg = create_test_message(&msg_id, "Retry test");
    store.store_message(msg).await.unwrap();

    // Enqueue for offline peer
    store
        .enqueue_offline(msg_id.clone(), "Bob".to_string())
        .await
        .expect("Failed to enqueue message");

    // Simulate multiple failures
    for attempt in 1..=3 {
        store
            .update_retry(&msg_id, false)
            .await
            .expect("Failed to update retry");

        let pending = store
            .get_pending_messages("Bob", 10)
            .await
            .expect("Failed to get pending messages");

        // Should still be pending
        assert_eq!(
            pending.len(),
            1,
            "Message should still be pending after attempt {}",
            attempt
        );
    }

    // Simulate success
    store
        .update_retry(&msg_id, true)
        .await
        .expect("Failed to update retry on success");

    // Message should no longer be pending
    let pending = store.get_pending_messages("Bob", 10).await.unwrap();
    assert_eq!(
        pending.len(),
        0,
        "Message should not be pending after successful delivery"
    );
}

#[tokio::test]
async fn test_get_unread_count() {
    let temp_file = NamedTempFile::new().unwrap();
    let store = MessageStore::new(temp_file.path().to_path_buf())
        .await
        .expect("Failed to create message store");

    // Store unread messages
    for i in 0..3 {
        let msg_id = Uuid::new_v4().to_string();
        let mut msg = create_test_message(&msg_id, &format!("Unread {}", i));
        msg.read = false;
        msg.sender_nickname = "Alice".to_string();
        store.store_message(msg).await.unwrap();
    }

    // Store read messages
    for i in 0..2 {
        let msg_id = Uuid::new_v4().to_string();
        let mut msg = create_test_message(&msg_id, &format!("Read {}", i));
        msg.read = true;
        msg.sender_nickname = "Alice".to_string();
        store.store_message(msg).await.unwrap();
    }

    // Get unread count
    let count = store
        .get_unread_count("Alice")
        .await
        .expect("Failed to get unread count");

    assert_eq!(count, 3, "Should count only unread messages");
}

#[tokio::test]
async fn test_clear_conversation() {
    let temp_file = NamedTempFile::new().unwrap();
    let store = MessageStore::new(temp_file.path().to_path_buf())
        .await
        .expect("Failed to create message store");

    // Store messages in conversation
    for i in 0..5 {
        let msg_id = Uuid::new_v4().to_string();
        let mut msg = create_test_message(&msg_id, &format!("Message {}", i));
        msg.sender_nickname = "Alice".to_string();
        msg.recipient_nickname = Some("Bob".to_string());
        store.store_message(msg).await.unwrap();
    }

    // Clear conversation
    let count = store
        .clear_conversation("Bob")
        .await
        .expect("Failed to clear conversation");

    assert_eq!(count, 5, "Should have deleted 5 messages");

    // Verify messages are gone
    let history = store.get_conversation("Bob", 10, 0).await.unwrap();
    assert_eq!(
        history.len(),
        0,
        "Conversation should be empty after clearing"
    );
}

#[tokio::test]
async fn test_group_messages() {
    let temp_file = NamedTempFile::new().unwrap();
    let store = MessageStore::new(temp_file.path().to_path_buf())
        .await
        .expect("Failed to create message store");

    let group_name = "TestGroup";

    // Store group messages
    for i in 0..3 {
        let msg_id = Uuid::new_v4().to_string();
        let mut msg = create_test_message(&msg_id, &format!("Group message {}", i));
        msg.msg_type = MessageType::Group;
        msg.group_name = Some(group_name.to_string());
        store.store_message(msg).await.unwrap();
    }

    // Retrieve group messages
    let group_msgs = store
        .get_group_messages(group_name, 10, 0)
        .await
        .expect("Failed to get group messages");

    assert_eq!(group_msgs.len(), 3);
    for msg in &group_msgs {
        assert_eq!(msg.group_name.as_ref().unwrap(), group_name);
    }
}

#[tokio::test]
async fn test_message_content_types() {
    let temp_file = NamedTempFile::new().unwrap();
    let store = MessageStore::new(temp_file.path().to_path_buf())
        .await
        .expect("Failed to create message store");

    // Test text message
    let msg_id1 = Uuid::new_v4().to_string();
    let msg1 = create_test_message(&msg_id1, "Text message");
    store.store_message(msg1).await.unwrap();
    let retrieved1 = store.get_message(&msg_id1).await.unwrap().unwrap();
    assert!(matches!(retrieved1.content, MessageContent::Text { .. }));

    // Test file share message
    let msg_id2 = Uuid::new_v4().to_string();
    let mut msg2 = create_test_message(&msg_id2, "");
    msg2.content = MessageContent::FileShare {
        share_code: "code123".to_string(),
        filename: "test.jpg".to_string(),
        file_size: 1024,
        file_type: "image/jpeg".to_string(),
    };
    store.store_message(msg2).await.unwrap();
    let retrieved2 = store.get_message(&msg_id2).await.unwrap().unwrap();
    assert!(matches!(
        retrieved2.content,
        MessageContent::FileShare { .. }
    ));

    // Test group share message
    let msg_id3 = Uuid::new_v4().to_string();
    let mut msg3 = create_test_message(&msg_id3, "");
    msg3.content = MessageContent::ShareGroup {
        group_id: "group123".to_string(),
        group_name: "Test Group".to_string(),
        inviter_nickname: "Alice".to_string(),
    };
    store.store_message(msg3).await.unwrap();
    let retrieved3 = store.get_message(&msg_id3).await.unwrap().unwrap();
    assert!(matches!(
        retrieved3.content,
        MessageContent::ShareGroup { .. }
    ));
}

#[tokio::test]
async fn test_update_message_peer_id() {
    let temp_file = NamedTempFile::new().unwrap();
    let store = MessageStore::new(temp_file.path().to_path_buf())
        .await
        .expect("Failed to create message store");

    let msg_id = Uuid::new_v4().to_string();
    let mut msg = create_test_message(&msg_id, "Test message");
    msg.peer_id = "".to_string(); // Initially empty
    store.store_message(msg).await.unwrap();

    // Update peer_id
    let new_peer_id = "12D3KooW...".to_string();
    store
        .update_message_peer_id(&msg_id, new_peer_id.clone())
        .await
        .expect("Failed to update peer_id");

    // Verify update
    let retrieved = store.get_message(&msg_id).await.unwrap().unwrap();
    assert_eq!(retrieved.peer_id, new_peer_id);
}

#[tokio::test]
async fn test_cleanup_expired_messages() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut config = PersistenceConfig::default();
    config.message_ttl_seconds = 1; // 1 second TTL for testing
    config.db_path = temp_file.path().to_path_buf();

    let store = MessageStore::with_config(config)
        .await
        .expect("Failed to create message store");

    // Store a message
    let msg_id = Uuid::new_v4().to_string();
    let msg = create_test_message(&msg_id, "Expiring message");
    store.store_message(msg).await.unwrap();

    // Enqueue for offline peer
    store
        .enqueue_offline(msg_id.clone(), "Bob".to_string())
        .await
        .expect("Failed to enqueue message");

    // Wait for expiration
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Cleanup expired messages
    let count = store
        .cleanup_expired()
        .await
        .expect("Failed to cleanup expired messages");

    assert!(
        count >= 1,
        "Should have cleaned up at least 1 expired message"
    );
}

#[tokio::test]
async fn test_custom_config() {
    let temp_file = NamedTempFile::new().unwrap();

    let config = PersistenceConfig {
        db_path: temp_file.path().to_path_buf(),
        sync_interval_seconds: 60,
        retry_interval_seconds: 600,
        cleanup_interval_seconds: 1800,
        message_ttl_seconds: 14 * 24 * 3600,
        max_retry_attempts: 20,
        max_batch_size: 100,
    };

    let store = MessageStore::with_config(config)
        .await
        .expect("Failed to create message store with custom config");

    // Verify store works with custom config
    let msg_id = Uuid::new_v4().to_string();
    let msg = create_test_message(&msg_id, "Test message");
    store.store_message(msg).await.unwrap();

    let retrieved = store.get_message(&msg_id).await.unwrap();
    assert!(retrieved.is_some());
}

// Helper function to create test messages
fn create_test_message(id: &str, text: &str) -> gigi_store::StoredMessage {
    gigi_store::StoredMessage {
        id: id.to_string(),
        msg_type: MessageType::Direct,
        direction: MessageDirection::Sent,
        content: MessageContent::Text {
            text: text.to_string(),
        },
        sender_nickname: "Alice".to_string(),
        recipient_nickname: Some("Bob".to_string()),
        group_name: None,
        peer_id: "peer123".to_string(),
        timestamp: chrono::Utc::now(),
        created_at: chrono::Utc::now(),
        delivered: false,
        delivered_at: None,
        read: false,
        read_at: None,
        sync_status: SyncStatus::Pending,
        sync_attempts: 0,
        last_sync_attempt: None,
        expires_at: chrono::Utc::now() + chrono::Duration::days(7),
    }
}
