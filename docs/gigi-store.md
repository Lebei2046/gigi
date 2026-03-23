# Gigi Store

Gigi Store is the persistence layer for the Gigi P2P network, storing messages, contacts, and other data. It provides a reliable way to store and retrieve data across sessions, ensuring that messages, contacts, and other data are preserved across application restarts and device reboots. This guide provides detailed information about Gigi Store's functionality, configuration, and usage.

## Overview

Gigi Store is designed to provide persistent storage for the Gigi P2P network, ensuring that messages, contacts, and other data are preserved across application restarts and device reboots. It uses a SQLite database for storage and provides a comprehensive set of APIs for managing various types of data, including messages, contacts, groups, and file metadata.

### Key Features

- **Message Storage**: Store and retrieve messages
- **Contact Management**: Manage peer contacts
- **Group Storage**: Store group information
- **File Metadata**: Track shared files
- **Settings Persistence**: Save user settings
- **Offline Queue**: Queue messages for delivery when offline
- **Synchronization**: Sync data between devices
- **Transaction Support**: Ensure data consistency
- **Migration Support**: Handle database schema changes

## Installation

### Prerequisites

- **Rust**: v1.60 or later
- **Cargo**: Latest version
- **SQLite**: For database storage

### Installation Steps

1. **Clone the Gigi repository**:
   ```bash
   git clone https://github.com/gigi-project/gigi.git
   cd gigi
   ```

2. **Build Gigi Store**:
   ```bash
   cd pkgs/gigi-store
   cargo build
   ```

3. **Add Gigi Store to your project**:
   ```toml
   # In your Cargo.toml
   [dependencies]
   gigi-store = {
     path = "../gigi/pkgs/gigi-store",
     version = "0.1.0"
   }
   ```

## Configuration

Gigi Store can be configured with various options to customize its behavior:

### Basic Configuration

```rust
use gigi_store::GigiStore;

// Create store with default settings
let store = GigiStore::new("/path/to/database").await?;
```

### Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `database_path` | Path to SQLite database file | `~/.gigi/store.db` |
| `max_connections` | Maximum number of database connections | `5` |
| `cache_size` | Size of in-memory cache | `1000` |
| `enable_wal` | Enable Write-Ahead Logging | `true` |
| `foreign_keys` | Enable foreign key constraints | `true` |
| `journal_mode` | SQLite journal mode | `WAL` |
| `synchronous` | SQLite synchronous mode | `NORMAL` |

## Usage

### Basic Usage

```rust
use gigi_store::GigiStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create store
    let store = GigiStore::new("/path/to/database").await?;
    
    // Save message
    store.save_message("peer-id", "Hello!").await?;
    
    // Get messages
    let messages = store.get_messages("peer-id").await?;
    println!("Found {} messages", messages.len());
    
    // Add contact
    store.add_contact("peer-id", "Alice").await?;
    
    // Get contacts
    let contacts = store.get_contacts().await?;
    println!("Found {} contacts", contacts.len());
    
    // Save settings
    store.save_settings("theme", "dark").await?;
    
    // Get settings
    let theme = store.get_settings("theme").await?;
    println!("Theme: {:?}", theme);
    
    Ok(())
}
```

### Advanced Usage

#### Custom Configuration

```rust
use gigi_store::GigiStoreConfig;

let config = GigiStoreConfig {
    database_path: "/path/to/database.db".to_string(),
    max_connections: 10,
    cache_size: 5000,
    enable_wal: true,
    foreign_keys: true,
    journal_mode: "WAL".to_string(),
    synchronous: "NORMAL".to_string(),
};

let store = GigiStore::new_with_config(config).await?;
```

#### Transaction Support

```rust
// Start a transaction
let transaction = store.begin_transaction().await?;

// Perform multiple operations
transaction.save_message("peer-id", "Hello!").await?;
transaction.add_contact("peer-id", "Alice").await?;

// Commit the transaction
transaction.commit().await?;
```

#### Batch Operations

```rust
// Save multiple messages
let messages = vec![
    ("peer-id-1", "Hello!"),
    ("peer-id-1", "How are you?"),
    ("peer-id-2", "Hi there!"),
];

for (peer_id, content) in messages {
    store.save_message(peer_id, content).await?;
}

// Get messages for a peer
let messages = store.get_messages("peer-id-1").await?;
println!("Found {} messages from peer-id-1", messages.len());
```

## Architecture

### Store Structure

The Gigi Store system consists of several key components:

1. **GigiStore**: Main store manager
2. **MessageStore**: Manages message storage
3. **ContactManager**: Manages contact information
4. **GroupManager**: Manages group information
5. **FileSharingStore**: Manages shared file metadata
6. **SettingsManager**: Manages user settings
7. **SyncManager**: Handles data synchronization
8. **ThumbnailStore**: Manages file thumbnails
9. **MigrationManager**: Handles database schema migrations

### Data Flow

1. **Data Creation**: User creates data (message, contact, etc.)
2. **Validation**: Data is validated for integrity
3. **Storage**: Data is stored in the database
4. **Caching**: Data is cached for faster access
5. **Retrieval**: Data is retrieved from storage or cache
6. **Synchronization**: Data is synchronized between devices

## Security

### Data Protection

- **Encryption**: Sensitive data can be encrypted
- **Access Control**: Implement application-level access control
- **Backup**: Regularly backup the database
- **Validation**: Validate data before storage
- **Sanitization**: Sanitize user input to prevent SQL injection

### Best Practices

- **Use Transactions**: Wrap multiple operations in transactions for consistency
- **Backup Regularly**: Regularly backup the database file
- **Encrypt Sensitive Data**: Encrypt sensitive data before storage
- **Limit Connections**: Use appropriate connection pool size
- **Update Regularly**: Keep Gigi Store updated to the latest version

## Troubleshooting

### Common Issues

#### Database Corruption

- **Symptom**: Database is corrupted or inaccessible
- **Solution**: Restore from backup, run database repair tools

#### Slow Performance

- **Symptom**: Store operations are slow
- **Solution**: Optimize queries, increase cache size, use appropriate indexes

#### Data Loss

- **Symptom**: Data is lost or missing
- **Solution**: Restore from backup, check transaction handling

#### Migration Failures

- **Symptom**: Database migration fails
- **Solution**: Check database compatibility, backup before migration

### Debugging

Enable debug logging to troubleshoot issues:

```rust
// Enable debug logging
env::set_var("RUST_LOG", "gigi_store=debug");

// Create store with debug logging
let store = GigiStore::new("/path/to/database").await?;
```

## Advanced Features

### Offline Queue

Queue messages for delivery when offline:

```rust
// Queue a message for delivery
store.queue_offline_message("peer-id", "Hello!").await?;

// Get offline messages
let offline_messages = store.get_offline_messages().await?;
println!("Found {} offline messages", offline_messages.len());

// Mark message as delivered
store.mark_message_as_delivered("message-id").await?;
```

### Synchronization

Sync data between devices:

```rust
// Get sync data
let sync_data = store.get_sync_data().await?;

// Apply sync data from another device
store.apply_sync_data(&sync_data).await?;
```

### Thumbnail Management

Manage file thumbnails:

```rust
// Save thumbnail
store.save_thumbnail("file-id", &thumbnail_data).await?;

// Get thumbnail
let thumbnail = store.get_thumbnail("file-id").await?;
```

### Integration with Other Components

Integrate Gigi Store with other Gigi components:

```rust
use gigi_store::GigiStore;
use gigi_p2p::P2pClient;

// Create store
let store = GigiStore::new("/path/to/database").await?;

// Create P2P client
let mut client = P2pClient::new("My Node", config).await?;

// Save message and send it
store.save_message("peer-id", "Hello!").await?;
client.send_direct_message("peer-id", "Hello!").await?;

// Handle incoming message
client.on_event(|event| {
    match event {
        Event::DirectMessage { from, message } => {
            // Save incoming message
            store.save_message(&from, &message).await?;
        }
        _ => {}
    }
});
```

## API Reference

### GigiStore Struct

#### Constructor

```rust
let store = GigiStore::new(database_path).await?;
```

**Parameters**:
- `database_path`: Path to SQLite database file

#### Methods

##### `save_message(peer_id, content)`

Save a message.

**Parameters**:
- `peer_id`: Peer ID who sent the message
- `content`: Message content

**Returns**:
- `Result<(), Error>`

##### `get_messages(peer_id)`

Get messages from a peer.

**Parameters**:
- `peer_id`: Peer ID to get messages from

**Returns**:
- `Result<Vec<Message>, Error>`

##### `add_contact(peer_id, nickname)`

Add a contact.

**Parameters**:
- `peer_id`: Peer ID of the contact
- `nickname`: Nickname for the contact

**Returns**:
- `Result<(), Error>`

##### `get_contacts()`

Get all contacts.

**Returns**:
- `Result<Vec<Contact>, Error>`

##### `update_contact(peer_id, nickname)`

Update a contact's nickname.

**Parameters**:
- `peer_id`: Peer ID of the contact
- `nickname`: New nickname

**Returns**:
- `Result<(), Error>`

##### `remove_contact(peer_id)`

Remove a contact.

**Parameters**:
- `peer_id`: Peer ID of the contact to remove

**Returns**:
- `Result<(), Error>`

##### `create_group(name)`

Create a group.

**Parameters**:
- `name`: Group name

**Returns**:
- `Result<String, Error>` (group ID)

##### `get_groups()`

Get all groups.

**Returns**:
- `Result<Vec<Group>, Error>`

##### `add_peer_to_group(group_id, peer_id)`

Add a peer to a group.

**Parameters**:
- `group_id`: Group ID
- `peer_id`: Peer ID to add

**Returns**:
- `Result<(), Error>`

##### `remove_peer_from_group(group_id, peer_id)`

Remove a peer from a group.

**Parameters**:
- `group_id`: Group ID
- `peer_id`: Peer ID to remove

**Returns**:
- `Result<(), Error>`

##### `save_settings(key, value)`

Save a setting.

**Parameters**:
- `key`: Setting key
- `value`: Setting value

**Returns**:
- `Result<(), Error>`

##### `get_settings(key)`

Get a setting.

**Parameters**:
- `key`: Setting key

**Returns**:
- `Result<Option<String>, Error>`

##### `queue_offline_message(peer_id, content)`

Queue a message for delivery when offline.

**Parameters**:
- `peer_id`: Peer ID to send the message to
- `content`: Message content

**Returns**:
- `Result<(), Error>`

##### `get_offline_messages()`

Get offline messages.

**Returns**:
- `Result<Vec<OfflineMessage>, Error>`

##### `mark_message_as_delivered(message_id)`

Mark a message as delivered.

**Parameters**:
- `message_id`: Message ID to mark as delivered

**Returns**:
- `Result<(), Error>`

##### `save_thumbnail(file_id, data)`

Save a thumbnail for a file.

**Parameters**:
- `file_id`: File ID
- `data`: Thumbnail data

**Returns**:
- `Result<(), Error>`

##### `get_thumbnail(file_id)`

Get a thumbnail for a file.

**Parameters**:
- `file_id`: File ID

**Returns**:
- `Result<Option<Vec<u8>>, Error>`

##### `begin_transaction()`

Begin a transaction.

**Returns**:
- `Result<Transaction, Error>`

## Examples

### Basic Message Storage

```rust
use gigi_store::GigiStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create store
    let store = GigiStore::new("/path/to/database").await?;
    
    // Save messages
    store.save_message("peer-id-1", "Hello!").await?;
    store.save_message("peer-id-1", "How are you?").await?;
    store.save_message("peer-id-2", "Hi there!").await?;
    
    println!("Saved messages");
    
    // Get messages for a peer
    let messages = store.get_messages("peer-id-1").await?;
    println!("Found {} messages from peer-id-1:", messages.len());
    
    for message in messages {
        println!("- {}", message.content);
    }
    
    // Get all messages
    let all_messages = store.get_all_messages().await?;
    println!("Found {} total messages", all_messages.len());
    
    Ok(())
}
```

### Contact Management

```rust
use gigi_store::GigiStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create store
    let store = GigiStore::new("/path/to/database").await?;
    
    // Add contacts
    store.add_contact("peer-id-1", "Alice").await?;
    store.add_contact("peer-id-2", "Bob").await?;
    store.add_contact("peer-id-3", "Charlie").await?;
    
    println!("Added contacts");
    
    // Get all contacts
    let contacts = store.get_contacts().await?;
    println!("Found {} contacts:", contacts.len());
    
    for contact in contacts {
        println!("- {}: {}", contact.peer_id, contact.nickname);
    }
    
    // Update a contact
    store.update_contact("peer-id-1", "Alice Smith").await?;
    println!("Updated Alice's nickname");
    
    // Remove a contact
    store.remove_contact("peer-id-3").await?;
    println!("Removed Charlie");
    
    // Get updated contacts
    let updated_contacts = store.get_contacts().await?;
    println!("Updated contacts: {}", updated_contacts.len());
    
    for contact in updated_contacts {
        println!("- {}: {}", contact.peer_id, contact.nickname);
    }
    
    Ok(())
}
```

### Group Management

```rust
use gigi_store::GigiStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create store
    let store = GigiStore::new("/path/to/database").await?;
    
    // Create groups
    let family_group = store.create_group("Family").await?;
    let work_group = store.create_group("Work").await?;
    
    println!("Created groups: Family ({}) and Work ({})", family_group, work_group);
    
    // Add peers to groups
    store.add_peer_to_group(&family_group, "peer-id-1").await?;
    store.add_peer_to_group(&family_group, "peer-id-2").await?;
    store.add_peer_to_group(&work_group, "peer-id-2").await?;
    store.add_peer_to_group(&work_group, "peer-id-3").await?;
    
    println!("Added peers to groups");
    
    // Get all groups
    let groups = store.get_groups().await?;
    println!("Found {} groups:", groups.len());
    
    for group in groups {
        println!("- {} ({})");
    }
    
    // Get members of a group
    let family_members = store.get_group_members(&family_group).await?;
    println!("Family group members: {}", family_members.len());
    
    for member in family_members {
        println!("- {}", member);
    }
    
    // Remove a peer from a group
    store.remove_peer_from_group(&work_group, "peer-id-3").await?;
    println!("Removed peer-id-3 from Work group");
    
    Ok(())
}
```

### Settings Management

```rust
use gigi_store::GigiStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create store
    let store = GigiStore::new("/path/to/database").await?;
    
    // Save settings
    store.save_settings("theme", "dark").await?;
    store.save_settings("notifications", "enabled").await?;
    store.save_settings("language", "en").await?;
    
    println!("Saved settings");
    
    // Get settings
    let theme = store.get_settings("theme").await?;
    let notifications = store.get_settings("notifications").await?;
    let language = store.get_settings("language").await?;
    
    println!("Theme: {:?}", theme);
    println!("Notifications: {:?}", notifications);
    println!("Language: {:?}", language);
    
    // Update a setting
    store.save_settings("theme", "light").await?;
    println!("Updated theme to light");
    
    // Get updated setting
    let updated_theme = store.get_settings("theme").await?;
    println!("Updated theme: {:?}", updated_theme);
    
    Ok(())
}
```

## Conclusion

Gigi Store provides a robust persistence layer for the Gigi P2P network, ensuring that messages, contacts, and other data are preserved across sessions. With its comprehensive set of APIs and features, Gigi Store enables applications to store and retrieve data reliably, even in offline scenarios.

By following the guidelines and examples in this guide, you can effectively integrate Gigi Store into your application and take advantage of its powerful features for data persistence. Whether you're building a messaging application, a file sharing tool, or any other P2P application, Gigi Store provides the reliable storage foundation you need.