# Tauri Plugin Gigi

The Tauri Plugin Gigi integrates Gigi P2P functionality into desktop and mobile applications built with Tauri. It provides a simple and efficient way to add P2P networking capabilities to your Tauri apps, enabling features like direct messaging, group messaging, and file sharing across platforms. This guide provides detailed information about the Tauri Plugin Gigi, including installation, configuration, usage, and API reference.

## Overview

The Tauri Plugin Gigi is designed to make it easy for Tauri applications to integrate with the Gigi P2P network. It provides a Rust backend for high performance and reliability, while offering a simple JavaScript/TypeScript API for frontend integration. This plugin enables Tauri apps to communicate directly with other peers in the network without relying on centralized servers.

### Key Features

- **Cross-Platform**: Works on desktop (Windows, macOS, Linux) and mobile (iOS, Android)
- **Native Performance**: Rust backend for high performance and reliability
- **Easy Integration**: Simple API for Tauri apps
- **Comprehensive Commands**: Full P2P functionality exposed
- **Secure**: Encrypted communications using Libp2p
- **Decentralized**: No central servers required
- **Offline Capable**: Works even when offline
- **Real-time Communication**: Instant messaging and file transfers
- **Group Messaging**: Support for group conversations
- **File Sharing**: Share files directly between peers

## Installation

### Prerequisites

- **Tauri**: v1.0 or later
- **Rust**: v1.60 or later
- **Node.js**: v16 or later
- **npm** or **yarn** or **pnpm**: For package management

### Installation Steps

1. **Add the plugin to your Tauri project**:
   ```bash
   # Using npm
   npm add tauri-plugin-gigi
   
   # Using yarn
   yarn add tauri-plugin-gigi
   
   # Using pnpm
   pnpm add tauri-plugin-gigi
   ```

2. **Update your Tauri configuration** (`tauri.conf.json`):
   ```json
   {
     "plugins": {
       "gigi": {
         "default_config": {
           "nickname": "My App",
           "bootstrap_nodes": ["/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer"],
           "enable_kademlia": true,
           "enable_relay": true,
           "enable_mdns": true,
           "listen_addrs": ["/ip4/0.0.0.0/tcp/0", "/ip4/0.0.0.0/ws/0"]
         }
       }
     }
   }
   ```

3. **Add the plugin to your `src-tauri/Cargo.toml`**:
   ```toml
   [dependencies]
   tauri-plugin-gigi = {
     path = "../pkgs/tauri-plugin-gigi",
     version = "0.1.0"
   }
   ```

4. **Initialize the plugin in your `src-tauri/src/main.rs`**:
   ```rust
   fn main() {
     tauri::Builder::default()
       .plugin(tauri_plugin_gigi::init())
       .run(tauri::generate_context!())
       .expect("error while running tauri application");
   }
   ```

## Configuration

### Plugin Configuration

The Tauri Plugin Gigi can be configured through the Tauri configuration file (`tauri.conf.json`). Here's a complete example:

```json
{
  "plugins": {
    "gigi": {
      "default_config": {
        "nickname": "My App",
        "peer_id": "your-peer-id",
        "bootstrap_nodes": ["/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer"],
        "enable_kademlia": true,
        "enable_relay": true,
        "enable_mdns": true,
        "listen_addrs": ["/ip4/0.0.0.0/tcp/0", "/ip4/0.0.0.0/ws/0"],
        "config": {
          "dm_policy": "open",
          "allow_from": ["*"]
        }
      },
      "debug": false,
      "log_level": "info"
    }
  }
}
```

### Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `nickname` | Display name for your node | `"Tauri App"` |
| `peer_id` | Optional predefined peer ID | Generated automatically |
| `bootstrap_nodes` | List of bootstrap nodes | `[]` |
| `enable_kademlia` | Enable Kademlia DHT for peer discovery | `true` |
| `enable_relay` | Enable circuit relay for NAT traversal | `true` |
| `enable_mdns` | Enable mDNS for local peer discovery | `true` |
| `listen_addrs` | Addresses to listen on | `["/ip4/0.0.0.0/tcp/0", "/ip4/0.0.0.0/ws/0"]` |
| `config.dm_policy` | Direct message policy (`"open"`, `"block"`) | `"open"` |
| `config.allow_from` | List of allowed peer IDs | `["*"]` |
| `debug` | Enable debug mode | `false` |
| `log_level` | Log level (`"trace"`, `"debug"`, `"info"`, `"warn"`, `"error"`) | `"info"` |

## Usage

### Basic Usage

```typescript
// Import the plugin
import { Gigi } from 'tauri-plugin-gigi';

// Initialize Gigi
await Gigi.init();

// Start the P2P client
await Gigi.start();

// Send a direct message
await Gigi.sendDirectMessage('peer-id', 'Hello!');

// Join a group
await Gigi.joinGroup('general');

// Send a group message
await Gigi.sendGroupMessage('general', 'Hello everyone!');

// Share a file
const fileId = await Gigi.shareFile('/path/to/file.txt');
console.log(`File shared with ID: ${fileId}`);

// Download a file
await Gigi.downloadFile(fileId, '/path/to/save/file.txt');
```

### Advanced Usage

#### Custom Configuration

```typescript
// Custom configuration
const config = {
  nickname: 'My Custom App',
  bootstrap_nodes: ['/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer'],
  enable_kademlia: true,
  enable_relay: true,
  enable_mdns: true,
  listen_addrs: ['/ip4/0.0.0.0/tcp/0', '/ip4/0.0.0.0/ws/0'],
  config: {
    dm_policy: 'open',
    allow_from: ['*']
  }
};

// Initialize with custom configuration
await Gigi.init(config);
```

#### Event Handling

```typescript
// Listen for events
Gigi.on('peer:connected', (peer) => {
  console.log(`Peer connected: ${peer.id} (${peer.nickname})`);
});

Gigi.on('peer:disconnected', (peerId) => {
  console.log(`Peer disconnected: ${peerId}`);
});

Gigi.on('message:direct', (message) => {
  console.log(`Direct message from ${message.from}: ${message.content}`);
});

Gigi.on('message:group', (message) => {
  console.log(`Group message in ${message.group}: ${message.content} from ${message.from}`);
});

Gigi.on('file:shared', (file) => {
  console.log(`File shared: ${file.id} (${file.name})`);
});

Gigi.on('file:downloaded', (file) => {
  console.log(`File downloaded: ${file.id} (${file.name}) to ${file.path}`);
});

Gigi.on('error', (error) => {
  console.error('Gigi error:', error);
});
```

#### File Sharing

```typescript
// Share a file
const fileId = await Gigi.shareFile('/path/to/large-file.zip');
console.log(`File shared with ID: ${fileId}`);

// Track upload progress
Gigi.on('file:upload:progress', (progress) => {
  console.log(`Upload progress: ${progress.percentage}% (${progress.transferred}/${progress.total} bytes)`);
  if (progress.completed) {
    console.log('Upload completed!');
  }
});

// Download a file
await Gigi.downloadFile(fileId, '/path/to/save/large-file.zip');

// Track download progress
Gigi.on('file:download:progress', (progress) => {
  console.log(`Download progress: ${progress.percentage}% (${progress.transferred}/${progress.total} bytes)`);
  if (progress.completed) {
    console.log('Download completed!');
  }
});
```

#### Group Management

```typescript
// Join a group
await Gigi.joinGroup('general');

// Leave a group
await Gigi.leaveGroup('general');

// List joined groups
const groups = await Gigi.getJoinedGroups();
console.log('Joined groups:', groups);

// Send a group message
await Gigi.sendGroupMessage('general', 'Hello everyone!');
```

## Architecture

### Plugin Structure

The Tauri Plugin Gigi consists of two main components:

1. **Rust Backend**: Provides the core P2P functionality using the Gigi P2P Rust client
2. **JavaScript/TypeScript Frontend**: Exposes a simple API for Tauri applications to use

### Data Flow

1. **Initialization**: The plugin is initialized with configuration
2. **Connection**: The P2P client connects to the network
3. **Discovery**: Peers are discovered using Kademlia DHT and mDNS
4. **Communication**: Messages and files are exchanged between peers
5. **Events**: Events are emitted to the frontend for UI updates

## Security

### Security Features

- **Encryption**: All communications are encrypted using the Noise protocol
- **Peer Verification**: Peers are verified by their public keys
- **Access Control**: Configure who can send you messages
- **File Safety**: Be cautious when downloading files from unknown peers
- **Secure Storage**: Sensitive data is stored securely

### Best Practices

- **Validate Inputs**: Validate all user inputs
- **Use HTTPS**: Use HTTPS for any web-based components
- **Update Regularly**: Keep the plugin updated to the latest version
- **Limit Permissions**: Request only necessary permissions
- **Monitor Connections**: Monitor for suspicious activity

## Troubleshooting

### Common Issues

#### Connection Problems

- **Symptom**: Cannot connect to the P2P network
- **Solution**: Check network connectivity, firewall settings, and bootstrap nodes

#### Peer Discovery

- **Symptom**: Cannot find other peers
- **Solution**: Ensure mDNS and Kademlia are enabled, check network settings

#### File Transfer

- **Symptom**: File transfer fails or is slow
- **Solution**: Check file permissions, network stability, and file size

#### Group Messaging

- **Symptom**: Cannot send or receive group messages
- **Solution**: Ensure all peers are subscribed to the same topic

### Debugging

Enable debug logging to troubleshoot issues:

```json
// In tauri.conf.json
{
  "plugins": {
    "gigi": {
      "debug": true,
      "log_level": "debug"
    }
  }
}
```

## Advanced Features

### Custom Protocols

Add custom protocols to extend the P2P functionality:

```rust
// In src-tauri/src/main.rs
fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_gigi::init_with_protocols(|mut client| {
      // Add custom protocol handler
      client.add_protocol("my-protocol", |data| {
        // Handle custom protocol data
        println!("Received custom protocol data: {:?}", data);
        Ok(Vec::new())
      });
      Ok(client)
    }))
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
```

### Custom Event Handlers

Add custom event handlers for application-specific events:

```typescript
// Listen for custom events
Gigi.on('custom:event', (data) => {
  console.log('Custom event:', data);
});

// Emit custom events from Rust
// In src-tauri/src/main.rs
#[tauri::command]
async fn emit_custom_event(app: tauri::AppHandle) -> Result<(), String> {
  app.emit_all("gigi:custom:event", Some(serde_json::json!({
    "message": "Hello from Rust!"
  })))?;
  Ok(())
}
```

### Integration with Other Tauri Plugins

Integrate with other Tauri plugins for enhanced functionality:

```typescript
// Example: Integrate with tauri-plugin-notification
import { notify } from '@tauri-apps/api/notification';

// Listen for messages and show notifications
Gigi.on('message:direct', (message) => {
  notify({
    title: `New message from ${message.from}`,
    body: message.content
  });
});

// Example: Integrate with tauri-plugin-store
import { Store } from 'tauri-plugin-store';

const store = new Store('gigi.json');

// Save messages to store
Gigi.on('message:direct', async (message) => {
  const messages = await store.get('messages') || [];
  messages.push(message);
  await store.set('messages', messages);
  await store.save();
});
```

## API Reference

### Core Methods

#### `init(config?: GigiConfig): Promise<void>`

Initialize the Gigi P2P client.

**Parameters**:
- `config`: Optional configuration object

**Returns**:
- `Promise<void>`

#### `start(): Promise<void>`

Start the Gigi P2P client and connect to the network.

**Returns**:
- `Promise<void>`

#### `stop(): Promise<void>`

Stop the Gigi P2P client and disconnect from the network.

**Returns**:
- `Promise<void>`

#### `sendDirectMessage(peerId: string, content: string): Promise<void>`

Send a direct message to a peer.

**Parameters**:
- `peerId`: Peer ID to send the message to
- `content`: Message content

**Returns**:
- `Promise<void>`

#### `joinGroup(group: string): Promise<void>`

Join a group.

**Parameters**:
- `group`: Group name

**Returns**:
- `Promise<void>`

#### `leaveGroup(group: string): Promise<void>`

Leave a group.

**Parameters**:
- `group`: Group name

**Returns**:
- `Promise<void>`

#### `sendGroupMessage(group: string, content: string): Promise<void>`

Send a message to a group.

**Parameters**:
- `group`: Group name
- `content`: Message content

**Returns**:
- `Promise<void>`

#### `shareFile(path: string): Promise<string>`

Share a file with the network.

**Parameters**:
- `path`: Path to the file to share

**Returns**:
- `Promise<string>` (file ID)

#### `downloadFile(fileId: string, path: string): Promise<void>`

Download a file from the network.

**Parameters**:
- `fileId`: File ID to download
- `path`: Path to save the downloaded file

**Returns**:
- `Promise<void>`

#### `getPeers(): Promise<Peer[]>`

Get a list of connected peers.

**Returns**:
- `Promise<Peer[]>`

#### `getJoinedGroups(): Promise<string[]>`

Get a list of joined groups.

**Returns**:
- `Promise<string[]>`

#### `getPeerId(): Promise<string>`

Get the current peer ID.

**Returns**:
- `Promise<string>`

#### `getNickname(): Promise<string>`

Get the current nickname.

**Returns**:
- `Promise<string>`

#### `setNickname(nickname: string): Promise<void>`

Set the current nickname.

**Parameters**:
- `nickname`: New nickname

**Returns**:
- `Promise<void>`

### Event Types

#### `peer:connected`

Emitted when a peer connects.

**Payload**:
```typescript
{
  id: string;      // Peer ID
  nickname: string; // Peer nickname
}
```

#### `peer:disconnected`

Emitted when a peer disconnects.

**Payload**:
```typescript
string; // Peer ID
```

#### `message:direct`

Emitted when a direct message is received.

**Payload**:
```typescript
{
  from: string;    // Sender peer ID
  content: string; // Message content
  timestamp: number; // Timestamp
}
```

#### `message:group`

Emitted when a group message is received.

**Payload**:
```typescript
{
  from: string;    // Sender peer ID
  group: string;   // Group name
  content: string; // Message content
  timestamp: number; // Timestamp
}
```

#### `file:shared`

Emitted when a file is successfully shared.

**Payload**:
```typescript
{
  id: string;      // File ID
  name: string;    // File name
  size: number;    // File size in bytes
}
```

#### `file:downloaded`

Emitted when a file is successfully downloaded.

**Payload**:
```typescript
{
  id: string;      // File ID
  name: string;    // File name
  path: string;    // Path to downloaded file
  size: number;    // File size in bytes
}
```

#### `file:upload:progress`

Emitted during file upload progress.

**Payload**:
```typescript
{
  fileId: string;  // File ID
  transferred: number; // Bytes transferred
  total: number;   // Total bytes
  percentage: number; // Percentage completed
  completed: boolean; // Whether upload is completed
}
```

#### `file:download:progress`

Emitted during file download progress.

**Payload**:
```typescript
{
  fileId: string;  // File ID
  transferred: number; // Bytes transferred
  total: number;   // Total bytes
  percentage: number; // Percentage completed
  completed: boolean; // Whether download is completed
}
```

#### `error`

Emitted when an error occurs.

**Payload**:
```typescript
{
  code: string;    // Error code
  message: string; // Error message
}
```

## Examples

### Basic Chat Application

```typescript
// Import the plugin
import { Gigi } from 'tauri-plugin-gigi';

// DOM elements
const messageInput = document.getElementById('message-input');
const sendButton = document.getElementById('send-button');
const messagesDiv = document.getElementById('messages');
const peerIdInput = document.getElementById('peer-id-input');

// Initialize Gigi
async function initGigi() {
  try {
    await Gigi.init({
      nickname: 'Chat App User',
      bootstrap_nodes: ['/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer'],
      enable_kademlia: true,
      enable_relay: true,
      enable_mdns: true
    });
    
    await Gigi.start();
    console.log('Gigi initialized and started');
    
    // Get and display our peer ID
    const peerId = await Gigi.getPeerId();
    console.log('Our peer ID:', peerId);
  } catch (error) {
    console.error('Error initializing Gigi:', error);
  }
}

// Send message
async function sendMessage() {
  const peerId = peerIdInput.value;
  const message = messageInput.value;
  
  if (!peerId || !message) return;
  
  try {
    await Gigi.sendDirectMessage(peerId, message);
    addMessage('You', message);
    messageInput.value = '';
  } catch (error) {
    console.error('Error sending message:', error);
  }
}

// Add message to UI
function addMessage(sender, content) {
  const messageElement = document.createElement('div');
  messageElement.className = 'message';
  messageElement.innerHTML = `<strong>${sender}:</strong> ${content}`;
  messagesDiv.appendChild(messageElement);
  messagesDiv.scrollTop = messagesDiv.scrollHeight;
}

// Listen for incoming messages
Gigi.on('message:direct', (message) => {
  addMessage(message.from, message.content);
});

// Listen for peer connections
Gigi.on('peer:connected', (peer) => {
  console.log(`Peer connected: ${peer.id} (${peer.nickname})`);
});

// Event listeners
sendButton.addEventListener('click', sendMessage);
messageInput.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') {
    sendMessage();
  }
});

// Initialize app
initGigi();
```

### Group Chat Application

```typescript
// Import the plugin
import { Gigi } from 'tauri-plugin-gigi';

// DOM elements
const messageInput = document.getElementById('message-input');
const sendButton = document.getElementById('send-button');
const messagesDiv = document.getElementById('messages');
const groupInput = document.getElementById('group-input');
const joinButton = document.getElementById('join-button');
const leaveButton = document.getElementById('leave-button');

let currentGroup = '';

// Initialize Gigi
async function initGigi() {
  try {
    await Gigi.init({
      nickname: 'Group Chat User',
      bootstrap_nodes: ['/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer'],
      enable_kademlia: true,
      enable_relay: true,
      enable_mdns: true
    });
    
    await Gigi.start();
    console.log('Gigi initialized and started');
  } catch (error) {
    console.error('Error initializing Gigi:', error);
  }
}

// Join group
async function joinGroup() {
  const group = groupInput.value;
  
  if (!group) return;
  
  try {
    await Gigi.joinGroup(group);
    currentGroup = group;
    console.log(`Joined group: ${group}`);
    addMessage('System', `Joined group: ${group}`);
  } catch (error) {
    console.error('Error joining group:', error);
  }
}

// Leave group
async function leaveGroup() {
  if (!currentGroup) return;
  
  try {
    await Gigi.leaveGroup(currentGroup);
    console.log(`Left group: ${currentGroup}`);
    addMessage('System', `Left group: ${currentGroup}`);
    currentGroup = '';
  } catch (error) {
    console.error('Error leaving group:', error);
  }
}

// Send group message
async function sendMessage() {
  const message = messageInput.value;
  
  if (!currentGroup || !message) return;
  
  try {
    await Gigi.sendGroupMessage(currentGroup, message);
    addMessage('You', message);
    messageInput.value = '';
  } catch (error) {
    console.error('Error sending message:', error);
  }
}

// Add message to UI
function addMessage(sender, content) {
  const messageElement = document.createElement('div');
  messageElement.className = 'message';
  messageElement.innerHTML = `<strong>${sender}:</strong> ${content}`;
  messagesDiv.appendChild(messageElement);
  messagesDiv.scrollTop = messagesDiv.scrollHeight;
}

// Listen for group messages
Gigi.on('message:group', (message) => {
  addMessage(`${message.from} (${message.group})`, message.content);
});

// Event listeners
joinButton.addEventListener('click', joinGroup);
leaveButton.addEventListener('click', leaveGroup);
sendButton.addEventListener('click', sendMessage);
messageInput.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') {
    sendMessage();
  }
});

// Initialize app
initGigi();
```

### File Sharing Application

```typescript
// Import the plugin
import { Gigi } from 'tauri-plugin-gigi';
import { open } from '@tauri-apps/api/dialog';
import { appDir } from '@tauri-apps/api/path';

// DOM elements
const shareButton = document.getElementById('share-button');
const downloadButton = document.getElementById('download-button');
const fileIdInput = document.getElementById('file-id-input');
const statusDiv = document.getElementById('status');

// Share file
async function shareFile() {
  try {
    // Open file dialog
    const files = await open({
      multiple: false,
      directory: false
    });
    
    if (!files) return;
    
    const filePath = Array.isArray(files) ? files[0] : files;
    
    // Share file
    statusDiv.textContent = 'Sharing file...';
    const fileId = await Gigi.shareFile(filePath);
    statusDiv.textContent = `File shared! ID: ${fileId}`;
    console.log(`File shared with ID: ${fileId}`);
  } catch (error) {
    console.error('Error sharing file:', error);
    statusDiv.textContent = `Error: ${error.message}`;
  }
}

// Download file
async function downloadFile() {
  try {
    const fileId = fileIdInput.value;
    
    if (!fileId) {
      statusDiv.textContent = 'Please enter a file ID';
      return;
    }
    
    // Get app directory for saving
    const appDirectory = await appDir();
    const savePath = `${appDirectory}/downloads`;
    
    // Create downloads directory if it doesn't exist
    await window.__TAURI__.fs.createDir(savePath, { recursive: true });
    
    const filePath = `${savePath}/downloaded-file`;
    
    // Download file
    statusDiv.textContent = 'Downloading file...';
    await Gigi.downloadFile(fileId, filePath);
    statusDiv.textContent = `File downloaded to: ${filePath}`;
    console.log(`File downloaded to: ${filePath}`);
  } catch (error) {
    console.error('Error downloading file:', error);
    statusDiv.textContent = `Error: ${error.message}`;
  }
}

// Listen for upload progress
Gigi.on('file:upload:progress', (progress) => {
  statusDiv.textContent = `Uploading: ${Math.round(progress.percentage)}%`;
  if (progress.completed) {
    statusDiv.textContent = 'Upload completed!';
  }
});

// Listen for download progress
Gigi.on('file:download:progress', (progress) => {
  statusDiv.textContent = `Downloading: ${Math.round(progress.percentage)}%`;
  if (progress.completed) {
    statusDiv.textContent = 'Download completed!';
  }
});

// Event listeners
shareButton.addEventListener('click', shareFile);
downloadButton.addEventListener('click', downloadFile);

// Initialize Gigi
async function initGigi() {
  try {
    await Gigi.init({
      nickname: 'File Sharing User',
      bootstrap_nodes: ['/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer'],
      enable_kademlia: true,
      enable_relay: true,
      enable_mdns: true
    });
    
    await Gigi.start();
    console.log('Gigi initialized and started');
  } catch (error) {
    console.error('Error initializing Gigi:', error);
    statusDiv.textContent = `Error initializing Gigi: ${error.message}`;
  }
}

// Initialize app
initGigi();
```

## Conclusion

The Tauri Plugin Gigi provides a powerful and easy-to-use integration of Gigi P2P functionality into Tauri applications. With its cross-platform support, native performance, and comprehensive API, it enables developers to build decentralized applications that can communicate directly between peers without relying on centralized servers.

By following the guidelines and examples in this guide, you can effectively integrate the Tauri Plugin Gigi into your Tauri application and take advantage of its powerful P2P features. Whether you're building a messaging application, a file sharing tool, or any other decentralized application, the Tauri Plugin Gigi provides the foundation you need to create a truly peer-to-peer experience.