# Gigi P2P Ecosystem API Reference

This document provides a comprehensive API reference for the Gigi P2P ecosystem, including APIs for the Rust client, TypeScript client, OpenClaw plugin, and Tauri plugin. It serves as a reference guide for developers working with the Gigi P2P ecosystem.

## Table of Contents

1. [Gigi P2P Rust Client API](#gigi-p2p-rust-client-api)
2. [Gigi P2P TypeScript Client API](#gigi-p2p-typescript-client-api)
3. [Gigi OpenClaw Plugin API](#gigi-openclaw-plugin-api)
4. [Tauri Plugin Gigi API](#tauri-plugin-gigi-api)
5. [Gigi Auth API](#gigi-auth-api)
6. [Gigi DNS API](#gigi-dns-api)
7. [Gigi File Sharing API](#gigi-file-sharing-api)
8. [Gigi Store API](#gigi-store-api)

## Gigi P2P Rust Client API

The Gigi P2P Rust Client provides a low-level API for P2P communication, built on top of Libp2p.

### P2pClient Struct

#### Constructor

```rust
pub async fn new(nickname: &str, config: P2pConfig) -> Result<Self, Error>
```

**Parameters**:
- `nickname`: Display name for the node
- `config`: Configuration for the P2P client

**Returns**:
- `Result<P2pClient, Error>`: The P2P client instance or an error

#### Methods

##### `start() -> Result<(), Error>`

Start the P2P client and connect to the network.

**Returns**:
- `Result<(), Error>`: Success or error

##### `stop() -> Result<(), Error>`

Stop the P2P client and disconnect from the network.

**Returns**:
- `Result<(), Error>`: Success or error

##### `send_direct_message(peer_id: &str, message: &str) -> Result<(), Error>`

Send a direct message to a peer.

**Parameters**:
- `peer_id`: Peer ID to send the message to
- `message`: Message content

**Returns**:
- `Result<(), Error>`: Success or error

##### `join_group(group: &str) -> Result<(), Error>`

Join a group.

**Parameters**:
- `group`: Group name

**Returns**:
- `Result<(), Error>`: Success or error

##### `leave_group(group: &str) -> Result<(), Error>`

Leave a group.

**Parameters**:
- `group`: Group name

**Returns**:
- `Result<(), Error>`: Success or error

##### `send_group_message(group: &str, message: &str) -> Result<(), Error>`

Send a message to a group.

**Parameters**:
- `group`: Group name
- `message`: Message content

**Returns**:
- `Result<(), Error>`: Success or error

##### `share_file(path: &str) -> Result<String, Error>`

Share a file with the network.

**Parameters**:
- `path`: Path to the file to share

**Returns**:
- `Result<String, Error>`: File ID or error

##### `download_file(file_id: &str, path: &str) -> Result<(), Error>`

Download a file from the network.

**Parameters**:
- `file_id`: File ID to download
- `path`: Path to save the downloaded file

**Returns**:
- `Result<(), Error>`: Success or error

##### `get_peers() -> Result<Vec<PeerInfo>, Error>`

Get a list of connected peers.

**Returns**:
- `Result<Vec<PeerInfo>, Error>`: List of peers or error

##### `get_joined_groups() -> Result<Vec<String>, Error>`

Get a list of joined groups.

**Returns**:
- `Result<Vec<String>, Error>`: List of groups or error

##### `get_peer_id() -> String`

Get the current peer ID.

**Returns**:
- `String`: Peer ID

##### `get_nickname() -> String`

Get the current nickname.

**Returns**:
- `String`: Nickname

##### `set_nickname(nickname: &str) -> Result<(), Error>`

Set the current nickname.

**Parameters**:
- `nickname`: New nickname

**Returns**:
- `Result<(), Error>`: Success or error

### Event Handling

```rust
pub fn on_event<F>(&mut self, handler: F) where F: Fn(Event) + Send + Sync + 'static
```

**Parameters**:
- `handler`: Event handler function

**Event Types**:
- `Event::PeerConnected(PeerInfo)`: Emitted when a peer connects
- `Event::PeerDisconnected(String)`: Emitted when a peer disconnects
- `Event::DirectMessage { from: String, content: String, timestamp: u64 }`: Emitted when a direct message is received
- `Event::GroupMessage { from: String, group: String, content: String, timestamp: u64 }`: Emitted when a group message is received
- `Event::FileShared { id: String, name: String, size: u64 }`: Emitted when a file is shared
- `Event::FileDownloaded { id: String, name: String, path: String, size: u64 }`: Emitted when a file is downloaded
- `Event::Error { code: String, message: String }`: Emitted when an error occurs

## Gigi P2P TypeScript Client API

The Gigi P2P TypeScript Client provides a high-level API for P2P communication, built on top of the Rust client.

### P2pClient Class

#### Constructor

```typescript
constructor(nickname: string, config: P2pConfig)
```

**Parameters**:
- `nickname`: Display name for the node
- `config`: Configuration for the P2P client

#### Methods

##### `async start(): Promise<void>`

Start the P2P client and connect to the network.

**Returns**:
- `Promise<void>`: Success or error

##### `async stop(): Promise<void>`

Stop the P2P client and disconnect from the network.

**Returns**:
- `Promise<void>`: Success or error

##### `async sendDirectMessage(peerId: string, content: string): Promise<void>`

Send a direct message to a peer.

**Parameters**:
- `peerId`: Peer ID to send the message to
- `content`: Message content

**Returns**:
- `Promise<void>`: Success or error

##### `async joinGroup(group: string): Promise<void>`

Join a group.

**Parameters**:
- `group`: Group name

**Returns**:
- `Promise<void>`: Success or error

##### `async leaveGroup(group: string): Promise<void>`

Leave a group.

**Parameters**:
- `group`: Group name

**Returns**:
- `Promise<void>`: Success or error

##### `async sendGroupMessage(group: string, content: string): Promise<void>`

Send a message to a group.

**Parameters**:
- `group`: Group name
- `content`: Message content

**Returns**:
- `Promise<void>`: Success or error

##### `async shareFile(path: string): Promise<string>`

Share a file with the network.

**Parameters**:
- `path`: Path to the file to share

**Returns**:
- `Promise<string>`: File ID

##### `async downloadFile(fileId: string, path: string): Promise<void>`

Download a file from the network.

**Parameters**:
- `fileId`: File ID to download
- `path`: Path to save the downloaded file

**Returns**:
- `Promise<void>`: Success or error

##### `async getPeers(): Promise<PeerInfo[]>`

Get a list of connected peers.

**Returns**:
- `Promise<PeerInfo[]>`: List of peers

##### `async getJoinedGroups(): Promise<string[]>`

Get a list of joined groups.

**Returns**:
- `Promise<string[]>`: List of groups

##### `async getPeerId(): Promise<string>`

Get the current peer ID.

**Returns**:
- `Promise<string>`: Peer ID

##### `async getNickname(): Promise<string>`

Get the current nickname.

**Returns**:
- `Promise<string>`: Nickname

##### `async setNickname(nickname: string): Promise<void>`

Set the current nickname.

**Parameters**:
- `nickname`: New nickname

**Returns**:
- `Promise<void>`: Success or error

### Event Handling

```typescript
on(event: string, callback: Function): void
```

**Parameters**:
- `event`: Event name
- `callback`: Event callback function

**Event Types**:
- `peer:connected`: Emitted when a peer connects
- `peer:disconnected`: Emitted when a peer disconnects
- `message:direct`: Emitted when a direct message is received
- `message:group`: Emitted when a group message is received
- `file:shared`: Emitted when a file is shared
- `file:downloaded`: Emitted when a file is downloaded
- `file:upload:progress`: Emitted during file upload progress
- `file:download:progress`: Emitted during file download progress
- `error`: Emitted when an error occurs

## Gigi OpenClaw Plugin API

The Gigi OpenClaw Plugin provides an API for integrating Gigi P2P functionality into OpenClaw.

### Commands

#### `gigi:init`

Initialize the Gigi P2P client.

**Parameters**:
- `config`: Configuration object

**Returns**:
- `Promise<void>`: Success or error

#### `gigi:start`

Start the Gigi P2P client.

**Returns**:
- `Promise<void>`: Success or error

#### `gigi:stop`

Stop the Gigi P2P client.

**Returns**:
- `Promise<void>`: Success or error

#### `gigi:sendDirectMessage`

Send a direct message to a peer.

**Parameters**:
- `peerId`: Peer ID
- `content`: Message content

**Returns**:
- `Promise<void>`: Success or error

#### `gigi:joinGroup`

Join a group.

**Parameters**:
- `group`: Group name

**Returns**:
- `Promise<void>`: Success or error

#### `gigi:leaveGroup`

Leave a group.

**Parameters**:
- `group`: Group name

**Returns**:
- `Promise<void>`: Success or error

#### `gigi:sendGroupMessage`

Send a message to a group.

**Parameters**:
- `group`: Group name
- `content`: Message content

**Returns**:
- `Promise<void>`: Success or error

#### `gigi:shareFile`

Share a file.

**Parameters**:
- `path`: File path

**Returns**:
- `Promise<string>`: File ID

#### `gigi:downloadFile`

Download a file.

**Parameters**:
- `fileId`: File ID
- `path`: Save path

**Returns**:
- `Promise<void>`: Success or error

#### `gigi:getPeers`

Get connected peers.

**Returns**:
- `Promise<PeerInfo[]>`: List of peers

#### `gigi:getJoinedGroups`

Get joined groups.

**Returns**:
- `Promise<string[]>`: List of groups

#### `gigi:getPeerId`

Get the current peer ID.

**Returns**:
- `Promise<string>`: Peer ID

#### `gigi:getNickname`

Get the current nickname.

**Returns**:
- `Promise<string>`: Nickname

#### `gigi:setNickname`

Set the current nickname.

**Parameters**:
- `nickname`: New nickname

**Returns**:
- `Promise<void>`: Success or error

### Events

- `gigi:peer:connected`: Emitted when a peer connects
- `gigi:peer:disconnected`: Emitted when a peer disconnects
- `gigi:message:direct`: Emitted when a direct message is received
- `gigi:message:group`: Emitted when a group message is received
- `gigi:file:shared`: Emitted when a file is shared
- `gigi:file:downloaded`: Emitted when a file is downloaded
- `gigi:file:upload:progress`: Emitted during file upload progress
- `gigi:file:download:progress`: Emitted during file download progress
- `gigi:error`: Emitted when an error occurs

## Tauri Plugin Gigi API

The Tauri Plugin Gigi provides an API for integrating Gigi P2P functionality into Tauri applications.

### Core Methods

#### `init(config?: GigiConfig): Promise<void>`

Initialize the Gigi P2P client.

**Parameters**:
- `config`: Optional configuration object

**Returns**:
- `Promise<void>`: Success or error

#### `start(): Promise<void>`

Start the Gigi P2P client and connect to the network.

**Returns**:
- `Promise<void>`: Success or error

#### `stop(): Promise<void>`

Stop the Gigi P2P client and disconnect from the network.

**Returns**:
- `Promise<void>`: Success or error

#### `sendDirectMessage(peerId: string, content: string): Promise<void>`

Send a direct message to a peer.

**Parameters**:
- `peerId`: Peer ID to send the message to
- `content`: Message content

**Returns**:
- `Promise<void>`: Success or error

#### `joinGroup(group: string): Promise<void>`

Join a group.

**Parameters**:
- `group`: Group name

**Returns**:
- `Promise<void>`: Success or error

#### `leaveGroup(group: string): Promise<void>`

Leave a group.

**Parameters**:
- `group`: Group name

**Returns**:
- `Promise<void>`: Success or error

#### `sendGroupMessage(group: string, content: string): Promise<void>`

Send a message to a group.

**Parameters**:
- `group`: Group name
- `content`: Message content

**Returns**:
- `Promise<void>`: Success or error

#### `shareFile(path: string): Promise<string>`

Share a file with the network.

**Parameters**:
- `path`: Path to the file to share

**Returns**:
- `Promise<string>`: File ID

#### `downloadFile(fileId: string, path: string): Promise<void>`

Download a file from the network.

**Parameters**:
- `fileId`: File ID to download
- `path`: Path to save the downloaded file

**Returns**:
- `Promise<void>`: Success or error

#### `getPeers(): Promise<Peer[]>`

Get a list of connected peers.

**Returns**:
- `Promise<Peer[]>`: List of peers

#### `getJoinedGroups(): Promise<string[]>`

Get a list of joined groups.

**Returns**:
- `Promise<string[]>`: List of groups

#### `getPeerId(): Promise<string>`

Get the current peer ID.

**Returns**:
- `Promise<string>`: Peer ID

#### `getNickname(): Promise<string>`

Get the current nickname.

**Returns**:
- `Promise<string>`: Nickname

#### `setNickname(nickname: string): Promise<void>`

Set the current nickname.

**Parameters**:
- `nickname`: New nickname

**Returns**:
- `Promise<void>`: Success or error

### Event Types

- `peer:connected`: Emitted when a peer connects
- `peer:disconnected`: Emitted when a peer disconnects
- `message:direct`: Emitted when a direct message is received
- `message:group`: Emitted when a group message is received
- `file:shared`: Emitted when a file is shared
- `file:downloaded`: Emitted when a file is downloaded
- `file:upload:progress`: Emitted during file upload progress
- `file:download:progress`: Emitted during file download progress
- `error`: Emitted when an error occurs

## Gigi Auth API

The Gigi Auth API provides authentication and key management functionality.

### AuthManager Struct

#### Constructor

```rust
pub fn new(storage_path: &str) -> Result<Self, Error>
```

**Parameters**:
- `storage_path`: Path to store authentication data

**Returns**:
- `Result<AuthManager, Error>`: Auth manager instance or error

#### Methods

##### `create_account(username: &str, password: &str) -> Result<Account, Error>`

Create a new account.

**Parameters**:
- `username`: Username
- `password`: Password

**Returns**:
- `Result<Account, Error>`: Account or error

##### `login(username: &str, password: &str) -> Result<Account, Error>`

Login to an existing account.

**Parameters**:
- `username`: Username
- `password`: Password

**Returns**:
- `Result<Account, Error>`: Account or error

##### `get_account(username: &str) -> Result<Option<Account>, Error>`

Get an account by username.

**Parameters**:
- `username`: Username

**Returns**:
- `Result<Option<Account>, Error>`: Account or None

##### `update_password(username: &str, old_password: &str, new_password: &str) -> Result<(), Error>`

Update an account password.

**Parameters**:
- `username`: Username
- `old_password`: Old password
- `new_password`: New password

**Returns**:
- `Result<(), Error>`: Success or error

##### `delete_account(username: &str, password: &str) -> Result<(), Error>`

Delete an account.

**Parameters**:
- `username`: Username
- `password`: Password

**Returns**:
- `Result<(), Error>`: Success or error

##### `get_mnemonic(username: &str, password: &str) -> Result<String, Error>`

Get the mnemonic phrase for account recovery.

**Parameters**:
- `username`: Username
- `password`: Password

**Returns**:
- `Result<String, Error>`: Mnemonic phrase or error

##### `recover_from_mnemonic(mnemonic: &str, password: &str) -> Result<Account, Error>`

Recover an account from a mnemonic phrase.

**Parameters**:
- `mnemonic`: Mnemonic phrase
- `password`: New password

**Returns**:
- `Result<Account, Error>`: Account or error

## Gigi DNS API

The Gigi DNS API provides decentralized name resolution functionality.

### DnsClient Struct

#### Constructor

```rust
pub async fn new(p2p_client: &P2pClient) -> Result<Self, Error>
```

**Parameters**:
- `p2p_client`: P2P client instance

**Returns**:
- `Result<DnsClient, Error>`: DNS client instance or error

#### Methods

##### `register_name(name: &str, peer_id: &str) -> Result<(), Error>`

Register a name to peer ID mapping.

**Parameters**:
- `name`: Name to register
- `peer_id`: Peer ID to map to

**Returns**:
- `Result<(), Error>`: Success or error

##### `resolve_name(name: &str) -> Result<Option<String>, Error>`

Resolve a name to a peer ID.

**Parameters**:
- `name`: Name to resolve

**Returns**:
- `Result<Option<String>, Error>`: Peer ID or None

##### `unregister_name(name: &str) -> Result<(), Error>`

Unregister a name.

**Parameters**:
- `name`: Name to unregister

**Returns**:
- `Result<(), Error>`: Success or error

##### `get_registered_names() -> Result<Vec<(String, String)>, Error>`

Get all registered names and their peer IDs.

**Returns**:
- `Result<Vec<(String, String)>, Error>`: List of (name, peer ID) pairs

## Gigi File Sharing API

The Gigi File Sharing API provides file sharing functionality.

### FileSharingClient Struct

#### Constructor

```rust
pub fn new(p2p_client: &P2pClient) -> Result<Self, Error>
```

**Parameters**:
- `p2p_client`: P2P client instance

**Returns**:
- `Result<FileSharingClient, Error>`: File sharing client instance or error

#### Methods

##### `share_file(path: &str) -> Result<String, Error>`

Share a file with the network.

**Parameters**:
- `path`: Path to the file to share

**Returns**:
- `Result<String, Error>`: File ID or error

##### `download_file(file_id: &str, path: &str) -> Result<(), Error>`

Download a file from the network.

**Parameters**:
- `file_id`: File ID to download
- `path`: Path to save the downloaded file

**Returns**:
- `Result<(), Error>`: Success or error

##### `get_file_info(file_id: &str) -> Result<Option<FileInfo>, Error>`

Get information about a shared file.

**Parameters**:
- `file_id`: File ID

**Returns**:
- `Result<Option<FileInfo>, Error>`: File info or None

##### `cancel_download(file_id: &str) -> Result<(), Error>`

Cancel a download.

**Parameters**:
- `file_id`: File ID

**Returns**:
- `Result<(), Error>`: Success or error

##### `cancel_upload(file_id: &str) -> Result<(), Error>`

Cancel an upload.

**Parameters**:
- `file_id`: File ID

**Returns**:
- `Result<(), Error>`: Success or error

### Event Handling

```rust
pub fn on_progress<F>(&mut self, handler: F) where F: Fn(ProgressEvent) + Send + Sync + 'static
```

**Parameters**:
- `handler`: Progress event handler

**ProgressEvent Types**:
- `ProgressEvent::Upload { file_id: String, transferred: u64, total: u64, percentage: f64, completed: bool }`
- `ProgressEvent::Download { file_id: String, transferred: u64, total: u64, percentage: f64, completed: bool }`

## Gigi Store API

The Gigi Store API provides persistence functionality for the Gigi P2P network.

### GigiStore Struct

#### Constructor

```rust
pub async fn new(database_path: &str) -> Result<Self, Error>
```

**Parameters**:
- `database_path`: Path to SQLite database file

**Returns**:
- `Result<GigiStore, Error>`: Store instance or error

#### Methods

##### `save_message(peer_id: &str, content: &str) -> Result<(), Error>`

Save a message.

**Parameters**:
- `peer_id`: Peer ID who sent the message
- `content`: Message content

**Returns**:
- `Result<(), Error>`: Success or error

##### `get_messages(peer_id: &str) -> Result<Vec<Message>, Error>`

Get messages from a peer.

**Parameters**:
- `peer_id`: Peer ID to get messages from

**Returns**:
- `Result<Vec<Message>, Error>`: List of messages

##### `add_contact(peer_id: &str, nickname: &str) -> Result<(), Error>`

Add a contact.

**Parameters**:
- `peer_id`: Peer ID of the contact
- `nickname`: Nickname for the contact

**Returns**:
- `Result<(), Error>`: Success or error

##### `get_contacts() -> Result<Vec<Contact>, Error>`

Get all contacts.

**Returns**:
- `Result<Vec<Contact>, Error>`: List of contacts

##### `update_contact(peer_id: &str, nickname: &str) -> Result<(), Error>`

Update a contact's nickname.

**Parameters**:
- `peer_id`: Peer ID of the contact
- `nickname`: New nickname

**Returns**:
- `Result<(), Error>`: Success or error

##### `remove_contact(peer_id: &str) -> Result<(), Error>`

Remove a contact.

**Parameters**:
- `peer_id`: Peer ID of the contact to remove

**Returns**:
- `Result<(), Error>`: Success or error

##### `create_group(name: &str) -> Result<String, Error>`

Create a group.

**Parameters**:
- `name`: Group name

**Returns**:
- `Result<String, Error>`: Group ID

##### `get_groups() -> Result<Vec<Group>, Error>`

Get all groups.

**Returns**:
- `Result<Vec<Group>, Error>`: List of groups

##### `add_peer_to_group(group_id: &str, peer_id: &str) -> Result<(), Error>`

Add a peer to a group.

**Parameters**:
- `group_id`: Group ID
- `peer_id`: Peer ID to add

**Returns**:
- `Result<(), Error>`: Success or error

##### `remove_peer_from_group(group_id: &str, peer_id: &str) -> Result<(), Error>`

Remove a peer from a group.

**Parameters**:
- `group_id`: Group ID
- `peer_id`: Peer ID to remove

**Returns**:
- `Result<(), Error>`: Success or error

##### `save_settings(key: &str, value: &str) -> Result<(), Error>`

Save a setting.

**Parameters**:
- `key`: Setting key
- `value`: Setting value

**Returns**:
- `Result<(), Error>`: Success or error

##### `get_settings(key: &str) -> Result<Option<String>, Error>`

Get a setting.

**Parameters**:
- `key`: Setting key

**Returns**:
- `Result<Option<String>, Error>`: Setting value or None

##### `queue_offline_message(peer_id: &str, content: &str) -> Result<(), Error>`

Queue a message for delivery when offline.

**Parameters**:
- `peer_id`: Peer ID to send the message to
- `content`: Message content

**Returns**:
- `Result<(), Error>`: Success or error

##### `get_offline_messages() -> Result<Vec<OfflineMessage>, Error>`

Get offline messages.

**Returns**:
- `Result<Vec<OfflineMessage>, Error>`: List of offline messages

##### `mark_message_as_delivered(message_id: &str) -> Result<(), Error>`

Mark a message as delivered.

**Parameters**:
- `message_id`: Message ID to mark as delivered

**Returns**:
- `Result<(), Error>`: Success or error

##### `save_thumbnail(file_id: &str, data: &[u8]) -> Result<(), Error>`

Save a thumbnail for a file.

**Parameters**:
- `file_id`: File ID
- `data`: Thumbnail data

**Returns**:
- `Result<(), Error>`: Success or error

##### `get_thumbnail(file_id: &str) -> Result<Option<Vec<u8>>, Error>`

Get a thumbnail for a file.

**Parameters**:
- `file_id`: File ID

**Returns**:
- `Result<Option<Vec<u8>>, Error>`: Thumbnail data or None

##### `begin_transaction() -> Result<Transaction, Error>`

Begin a transaction.

**Returns**:
- `Result<Transaction, Error>`: Transaction instance

## Conclusion

This API reference provides a comprehensive overview of the APIs exposed by the Gigi P2P ecosystem. It covers the core functionality of each component, including P2P communication, file sharing, authentication, and data persistence.

Developers can use this reference to understand how to integrate and use the Gigi P2P ecosystem in their applications. For more detailed information about each component, please refer to the component-specific guides in the documentation.