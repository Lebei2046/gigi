# Gigi P2P Ecosystem Examples

This document provides practical examples of how to use the Gigi P2P ecosystem in real-world applications. It covers various use cases, including basic P2P messaging, group messaging, file sharing, authentication, and platform integration.

## Table of Contents

1. [Basic P2P Messaging](#basic-p2p-messaging)
2. [Group Messaging](#group-messaging)
3. [File Sharing](#file-sharing)
4. [Authentication](#authentication)
5. [Name Resolution](#name-resolution)
6. [Tauri Application Integration](#tauri-application-integration)
7. [OpenClaw Integration](#openclaw-integration)
8. [Network Node Deployment](#network-node-deployment)
9. [Advanced Use Cases](#advanced-use-cases)

## Basic P2P Messaging

### Rust Client Example

```rust
use gigi_p2p::P2pClient;
use gigi_p2p::config::P2pConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create P2P client configuration
    let config = P2pConfig {
        bootstrap_nodes: vec!["/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer"],
        enable_kademlia: true,
        enable_relay: true,
        enable_mdns: true,
        listen_addrs: vec!["/ip4/0.0.0.0/tcp/0", "/ip4/0.0.0.0/ws/0"],
        ..Default::default()
    };
    
    // Create P2P client
    let mut client = P2pClient::new("My Node", config).await?;
    
    // Start the client
    client.start().await?;
    
    // Get our peer ID
    let peer_id = client.get_peer_id();
    println!("Our peer ID: {}", peer_id);
    
    // Send a direct message
    let recipient_peer_id = "QmRecipientPeerId";
    client.send_direct_message(recipient_peer_id, "Hello from Gigi P2P!").await?;
    println!("Message sent to {}", recipient_peer_id);
    
    // Listen for incoming messages
    client.on_event(|event| {
        match event {
            gigi_p2p::event::Event::DirectMessage { from, content, timestamp } => {
                println!("Received message from {} at {}: {}", from, timestamp, content);
            },
            gigi_p2p::event::Event::PeerConnected(peer) => {
                println!("Peer connected: {} ({})
", peer.id, peer.nickname);
            },
            gigi_p2p::event::Event::PeerDisconnected(peer_id) => {
                println!("Peer disconnected: {}", peer_id);
            },
            _ => {}
        }
    });
    
    // Keep the client running
    tokio::signal::ctrl_c().await?;
    
    // Stop the client
    client.stop().await?;
    
    Ok(())
}
```

### TypeScript Client Example

```typescript
import { P2pClient } from '@gigi/p2p';

async function main() {
  try {
    // Create P2P client
    const client = new P2pClient('My Node', {
      bootstrapNodes: ['/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer'],
      enableKademlia: true,
      enableRelay: true,
      enableMdns: true,
      listenAddrs: ['/ip4/0.0.0.0/tcp/0', '/ip4/0.0.0.0/ws/0']
    });
    
    // Start the client
    await client.start();
    
    // Get our peer ID
    const peerId = await client.getPeerId();
    console.log(`Our peer ID: ${peerId}`);
    
    // Send a direct message
    const recipientPeerId = 'QmRecipientPeerId';
    await client.sendDirectMessage(recipientPeerId, 'Hello from Gigi P2P!');
    console.log(`Message sent to ${recipientPeerId}`);
    
    // Listen for incoming messages
    client.on('message:direct', (message) => {
      console.log(`Received message from ${message.from} at ${message.timestamp}: ${message.content}`);
    });
    
    // Listen for peer connections
    client.on('peer:connected', (peer) => {
      console.log(`Peer connected: ${peer.id} (${peer.nickname})`);
    });
    
    // Listen for peer disconnections
    client.on('peer:disconnected', (peerId) => {
      console.log(`Peer disconnected: ${peerId}`);
    });
    
    // Keep the client running
    await new Promise(() => {});
  } catch (error) {
    console.error('Error:', error);
  }
}

main();
```

## Group Messaging

### Rust Client Example

```rust
use gigi_p2p::P2pClient;
use gigi_p2p::config::P2pConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create P2P client configuration
    let config = P2pConfig {
        bootstrap_nodes: vec!["/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer"],
        enable_kademlia: true,
        enable_relay: true,
        enable_mdns: true,
        listen_addrs: vec!["/ip4/0.0.0.0/tcp/0", "/ip4/0.0.0.0/ws/0"],
        ..Default::default()
    };
    
    // Create P2P client
    let mut client = P2pClient::new("My Node", config).await?;
    
    // Start the client
    client.start().await?;
    
    // Join a group
    let group_name = "general";
    client.join_group(group_name).await?;
    println!("Joined group: {}", group_name);
    
    // Send a group message
    client.send_group_message(group_name, "Hello everyone!").await?;
    println!("Message sent to group: {}", group_name);
    
    // Listen for group messages
    client.on_event(|event| {
        match event {
            gigi_p2p::event::Event::GroupMessage { from, group, content, timestamp } => {
                println!("Received message in {} from {} at {}: {}", group, from, timestamp, content);
            },
            _ => {}
        }
    });
    
    // Keep the client running
    tokio::signal::ctrl_c().await?;
    
    // Leave the group
    client.leave_group(group_name).await?;
    println!("Left group: {}", group_name);
    
    // Stop the client
    client.stop().await?;
    
    Ok(())
}
```

### TypeScript Client Example

```typescript
import { P2pClient } from '@gigi/p2p';

async function main() {
  try {
    // Create P2P client
    const client = new P2pClient('My Node', {
      bootstrapNodes: ['/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer'],
      enableKademlia: true,
      enableRelay: true,
      enableMdns: true,
      listenAddrs: ['/ip4/0.0.0.0/tcp/0', '/ip4/0.0.0.0/ws/0']
    });
    
    // Start the client
    await client.start();
    
    // Join a group
    const groupName = 'general';
    await client.joinGroup(groupName);
    console.log(`Joined group: ${groupName}`);
    
    // Send a group message
    await client.sendGroupMessage(groupName, 'Hello everyone!');
    console.log(`Message sent to group: ${groupName}`);
    
    // Listen for group messages
    client.on('message:group', (message) => {
      console.log(`Received message in ${message.group} from ${message.from} at ${message.timestamp}: ${message.content}`);
    });
    
    // Keep the client running
    await new Promise(() => {});
  } catch (error) {
    console.error('Error:', error);
  }
}

main();
```

## File Sharing

### Rust Client Example

```rust
use gigi_p2p::P2pClient;
use gigi_p2p::config::P2pConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create P2P client configuration
    let config = P2pConfig {
        bootstrap_nodes: vec!["/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer"],
        enable_kademlia: true,
        enable_relay: true,
        enable_mdns: true,
        listen_addrs: vec!["/ip4/0.0.0.0/tcp/0", "/ip4/0.0.0.0/ws/0"],
        ..Default::default()
    };
    
    // Create P2P client
    let mut client = P2pClient::new("My Node", config).await?;
    
    // Start the client
    client.start().await?;
    
    // Share a file
    let file_path = "/path/to/file.txt";
    let file_id = client.share_file(file_path).await?;
    println!("File shared with ID: {}", file_id);
    
    // Download a file
    let download_path = "/path/to/save/file.txt";
    client.download_file(&file_id, download_path).await?;
    println!("File downloaded to: {}", download_path);
    
    // Listen for file events
    client.on_event(|event| {
        match event {
            gigi_p2p::event::Event::FileShared { id, name, size } => {
                println!("File shared: {} ({} bytes)", name, size);
            },
            gigi_p2p::event::Event::FileDownloaded { id, name, path, size } => {
                println!("File downloaded: {} ({} bytes) to {}", name, size, path);
            },
            _ => {}
        }
    });
    
    // Keep the client running
    tokio::signal::ctrl_c().await?;
    
    // Stop the client
    client.stop().await?;
    
    Ok(())
}
```

### TypeScript Client Example

```typescript
import { P2pClient } from '@gigi/p2p';

async function main() {
  try {
    // Create P2P client
    const client = new P2pClient('My Node', {
      bootstrapNodes: ['/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer'],
      enableKademlia: true,
      enableRelay: true,
      enableMdns: true,
      listenAddrs: ['/ip4/0.0.0.0/tcp/0', '/ip4/0.0.0.0/ws/0']
    });
    
    // Start the client
    await client.start();
    
    // Share a file
    const filePath = '/path/to/file.txt';
    const fileId = await client.shareFile(filePath);
    console.log(`File shared with ID: ${fileId}`);
    
    // Download a file
    const downloadPath = '/path/to/save/file.txt';
    await client.downloadFile(fileId, downloadPath);
    console.log(`File downloaded to: ${downloadPath}`);
    
    // Listen for file events
    client.on('file:shared', (file) => {
      console.log(`File shared: ${file.name} (${file.size} bytes)`);
    });
    
    client.on('file:downloaded', (file) => {
      console.log(`File downloaded: ${file.name} (${file.size} bytes) to ${file.path}`);
    });
    
    // Listen for upload progress
    client.on('file:upload:progress', (progress) => {
      console.log(`Upload progress: ${Math.round(progress.percentage)}% (${progress.transferred}/${progress.total} bytes)`);
    });
    
    // Listen for download progress
    client.on('file:download:progress', (progress) => {
      console.log(`Download progress: ${Math.round(progress.percentage)}% (${progress.transferred}/${progress.total} bytes)`);
    });
    
    // Keep the client running
    await new Promise(() => {});
  } catch (error) {
    console.error('Error:', error);
  }
}

main();
```

## Authentication

### Rust Example

```rust
use gigi_auth::AuthManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create auth manager
    let mut auth = AuthManager::new("/path/to/auth/data")?;
    
    // Create an account
    let username = "alice";
    let password = "secure_password";
    
    let account = auth.create_account(username, password)?;
    println!("Account created: {}", account.username);
    
    // Login to the account
    let logged_in_account = auth.login(username, password)?;
    println!("Logged in as: {}", logged_in_account.username);
    
    // Get mnemonic phrase for recovery
    let mnemonic = auth.get_mnemonic(username, password)?;
    println!("Mnemonic phrase: {}", mnemonic);
    
    // Recover account from mnemonic
    let recovered_account = auth.recover_from_mnemonic(&mnemonic, "new_secure_password")?;
    println!("Account recovered: {}", recovered_account.username);
    
    Ok(())
}
```

## Name Resolution

### Rust Example

```rust
use gigi_p2p::P2pClient;
use gigi_p2p::config::P2pConfig;
use gigi_dns::DnsClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create P2P client configuration
    let config = P2pConfig {
        bootstrap_nodes: vec!["/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer"],
        enable_kademlia: true,
        enable_relay: true,
        enable_mdns: true,
        listen_addrs: vec!["/ip4/0.0.0.0/tcp/0", "/ip4/0.0.0.0/ws/0"],
        ..Default::default()
    };
    
    // Create P2P client
    let client = P2pClient::new("My Node", config).await?;
    client.start().await?;
    
    // Create DNS client
    let mut dns_client = DnsClient::new(&client).await?;
    
    // Get our peer ID
    let peer_id = client.get_peer_id();
    
    // Register a name
    let name = "alice";
    dns_client.register_name(name, &peer_id).await?;
    println!("Registered name '{}' for peer ID '{}'", name, peer_id);
    
    // Resolve a name
    let resolved_peer_id = dns_client.resolve_name(name).await?;
    match resolved_peer_id {
        Some(id) => println!("Resolved name '{}' to peer ID '{}'", name, id),
        None => println!("Name '{}' not found", name),
    }
    
    // Get all registered names
    let registered_names = dns_client.get_registered_names().await?;
    println!("Registered names:");
    for (name, id) in registered_names {
        println!("- {}: {}", name, id);
    }
    
    // Unregister a name
    dns_client.unregister_name(name).await?;
    println!("Unregistered name '{}'", name);
    
    // Stop the client
    client.stop().await?;
    
    Ok(())
}
```

## Tauri Application Integration

### Tauri App Example

```typescript
// src/App.tsx
import { useState, useEffect } from 'react';
import { Gigi } from 'tauri-plugin-gigi';

function App() {
  const [peerId, setPeerId] = useState('');
  const [message, setMessage] = useState('');
  const [messages, setMessages] = useState<string[]>([]);
  const [recipientId, setRecipientId] = useState('');

  useEffect(() => {
    async function initGigi() {
      try {
        // Initialize Gigi
        await Gigi.init({
          nickname: 'Tauri App User',
          bootstrap_nodes: ['/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer'],
          enable_kademlia: true,
          enable_relay: true,
          enable_mdns: true
        });

        // Start Gigi
        await Gigi.start();

        // Get peer ID
        const id = await Gigi.getPeerId();
        setPeerId(id);

        // Listen for messages
        Gigi.on('message:direct', (msg) => {
          setMessages(prev => [...prev, `From ${msg.from}: ${msg.content}`]);
        });

        // Listen for peer connections
        Gigi.on('peer:connected', (peer) => {
          setMessages(prev => [...prev, `Peer connected: ${peer.id} (${peer.nickname})`]);
        });
      } catch (error) {
        console.error('Error initializing Gigi:', error);
      }
    }

    initGigi();

    return () => {
      // Cleanup
      Gigi.stop().catch(console.error);
    };
  }, []);

  const sendMessage = async () => {
    if (!recipientId || !message) return;

    try {
      await Gigi.sendDirectMessage(recipientId, message);
      setMessages(prev => [...prev, `To ${recipientId}: ${message}`]);
      setMessage('');
    } catch (error) {
      console.error('Error sending message:', error);
    }
  };

  return (
    <div className="app">
      <h1>Gigi P2P Chat</h1>
      <div className="peer-id">
        <p>Your peer ID: {peerId}</p>
      </div>
      <div className="messages">
        {messages.map((msg, index) => (
          <div key={index} className="message">{msg}</div>
        ))}
      </div>
      <div className="input-area">
        <input
          type="text"
          placeholder="Recipient peer ID"
          value={recipientId}
          onChange={(e) => setRecipientId(e.target.value)}
        />
        <input
          type="text"
          placeholder="Message"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          onKeyPress={(e) => e.key === 'Enter' && sendMessage()}
        />
        <button onClick={sendMessage}>Send</button>
      </div>
    </div>
  );
}

export default App;
```

## OpenClaw Integration

### OpenClaw Plugin Example

```typescript
// openclaw-plugin-gigi.js
const { Plugin } = require('openclaw-plugin');

class GigiPlugin extends Plugin {
  constructor() {
    super('gigi');
    this.client = null;
  }

  async init() {
    // Initialize the plugin
    this.registerCommand('init', this.initGigi.bind(this));
    this.registerCommand('start', this.startGigi.bind(this));
    this.registerCommand('stop', this.stopGigi.bind(this));
    this.registerCommand('sendDirectMessage', this.sendDirectMessage.bind(this));
    this.registerCommand('joinGroup', this.joinGroup.bind(this));
    this.registerCommand('leaveGroup', this.leaveGroup.bind(this));
    this.registerCommand('sendGroupMessage', this.sendGroupMessage.bind(this));
    this.registerCommand('shareFile', this.shareFile.bind(this));
    this.registerCommand('downloadFile', this.downloadFile.bind(this));
    this.registerCommand('getPeers', this.getPeers.bind(this));
    this.registerCommand('getJoinedGroups', this.getJoinedGroups.bind(this));
    this.registerCommand('getPeerId', this.getPeerId.bind(this));
    this.registerCommand('getNickname', this.getNickname.bind(this));
    this.registerCommand('setNickname', this.setNickname.bind(this));

    console.log('Gigi plugin initialized');
  }

  async initGigi(config) {
    try {
      const { P2pClient } = require('@gigi/p2p');
      this.client = new P2pClient(config.nickname || 'OpenClaw User', {
        bootstrapNodes: config.bootstrap_nodes || [],
        enableKademlia: config.enable_kademlia !== false,
        enableRelay: config.enable_relay !== false,
        enableMdns: config.enable_mdns !== false,
        listenAddrs: config.listen_addrs || ['/ip4/0.0.0.0/tcp/0', '/ip4/0.0.0.0/ws/0']
      });

      // Listen for events
      this.client.on('peer:connected', (peer) => {
        this.emit('gigi:peer:connected', peer);
      });

      this.client.on('peer:disconnected', (peerId) => {
        this.emit('gigi:peer:disconnected', peerId);
      });

      this.client.on('message:direct', (message) => {
        this.emit('gigi:message:direct', message);
      });

      this.client.on('message:group', (message) => {
        this.emit('gigi:message:group', message);
      });

      this.client.on('file:shared', (file) => {
        this.emit('gigi:file:shared', file);
      });

      this.client.on('file:downloaded', (file) => {
        this.emit('gigi:file:downloaded', file);
      });

      this.client.on('file:upload:progress', (progress) => {
        this.emit('gigi:file:upload:progress', progress);
      });

      this.client.on('file:download:progress', (progress) => {
        this.emit('gigi:file:download:progress', progress);
      });

      this.client.on('error', (error) => {
        this.emit('gigi:error', error);
      });

      return { success: true };
    } catch (error) {
      console.error('Error initializing Gigi:', error);
      return { success: false, error: error.message };
    }
  }

  async startGigi() {
    try {
      if (!this.client) {
        return { success: false, error: 'Gigi not initialized' };
      }

      await this.client.start();
      return { success: true };
    } catch (error) {
      console.error('Error starting Gigi:', error);
      return { success: false, error: error.message };
    }
  }

  async stopGigi() {
    try {
      if (!this.client) {
        return { success: false, error: 'Gigi not initialized' };
      }

      await this.client.stop();
      return { success: true };
    } catch (error) {
      console.error('Error stopping Gigi:', error);
      return { success: false, error: error.message };
    }
  }

  async sendDirectMessage(params) {
    try {
      if (!this.client) {
        return { success: false, error: 'Gigi not initialized' };
      }

      await this.client.sendDirectMessage(params.peerId, params.content);
      return { success: true };
    } catch (error) {
      console.error('Error sending direct message:', error);
      return { success: false, error: error.message };
    }
  }

  async joinGroup(params) {
    try {
      if (!this.client) {
        return { success: false, error: 'Gigi not initialized' };
      }

      await this.client.joinGroup(params.group);
      return { success: true };
    } catch (error) {
      console.error('Error joining group:', error);
      return { success: false, error: error.message };
    }
  }

  async leaveGroup(params) {
    try {
      if (!this.client) {
        return { success: false, error: 'Gigi not initialized' };
      }

      await this.client.leaveGroup(params.group);
      return { success: true };
    } catch (error) {
      console.error('Error leaving group:', error);
      return { success: false, error: error.message };
    }
  }

  async sendGroupMessage(params) {
    try {
      if (!this.client) {
        return { success: false, error: 'Gigi not initialized' };
      }

      await this.client.sendGroupMessage(params.group, params.content);
      return { success: true };
    } catch (error) {
      console.error('Error sending group message:', error);
      return { success: false, error: error.message };
    }
  }

  async shareFile(params) {
    try {
      if (!this.client) {
        return { success: false, error: 'Gigi not initialized' };
      }

      const fileId = await this.client.shareFile(params.path);
      return { success: true, fileId };
    } catch (error) {
      console.error('Error sharing file:', error);
      return { success: false, error: error.message };
    }
  }

  async downloadFile(params) {
    try {
      if (!this.client) {
        return { success: false, error: 'Gigi not initialized' };
      }

      await this.client.downloadFile(params.fileId, params.path);
      return { success: true };
    } catch (error) {
      console.error('Error downloading file:', error);
      return { success: false, error: error.message };
    }
  }

  async getPeers() {
    try {
      if (!this.client) {
        return { success: false, error: 'Gigi not initialized' };
      }

      const peers = await this.client.getPeers();
      return { success: true, peers };
    } catch (error) {
      console.error('Error getting peers:', error);
      return { success: false, error: error.message };
    }
  }

  async getJoinedGroups() {
    try {
      if (!this.client) {
        return { success: false, error: 'Gigi not initialized' };
      }

      const groups = await this.client.getJoinedGroups();
      return { success: true, groups };
    } catch (error) {
      console.error('Error getting joined groups:', error);
      return { success: false, error: error.message };
    }
  }

  async getPeerId() {
    try {
      if (!this.client) {
        return { success: false, error: 'Gigi not initialized' };
      }

      const peerId = await this.client.getPeerId();
      return { success: true, peerId };
    } catch (error) {
      console.error('Error getting peer ID:', error);
      return { success: false, error: error.message };
    }
  }

  async getNickname() {
    try {
      if (!this.client) {
        return { success: false, error: 'Gigi not initialized' };
      }

      const nickname = await this.client.getNickname();
      return { success: true, nickname };
    } catch (error) {
      console.error('Error getting nickname:', error);
      return { success: false, error: error.message };
    }
  }

  async setNickname(params) {
    try {
      if (!this.client) {
        return { success: false, error: 'Gigi not initialized' };
      }

      await this.client.setNickname(params.nickname);
      return { success: true };
    } catch (error) {
      console.error('Error setting nickname:', error);
      return { success: false, error: error.message };
    }
  }
}

module.exports = GigiPlugin;
```

## Network Node Deployment

### Bootstrap Node Example

```bash
# Start a bootstrap node
./gigi-node --mode bootstrap --listen /ip4/0.0.0.0/tcp/4001
```

### Relay Node Example

```bash
# Start a relay node
./gigi-node --mode relay --listen /ip4/0.0.0.0/tcp/4002 --bootstrap /ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer
```

### Full Node Example

```bash
# Start a full node
./gigi-node --mode full --listen /ip4/0.0.0.0/tcp/4003 --bootstrap /ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer
```

## Advanced Use Cases

### Custom Protocol Implementation

```rust
use gigi_p2p::P2pClient;
use gigi_p2p::config::P2pConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create P2P client configuration
    let config = P2pConfig {
        bootstrap_nodes: vec!["/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer"],
        enable_kademlia: true,
        enable_relay: true,
        enable_mdns: true,
        listen_addrs: vec!["/ip4/0.0.0.0/tcp/0", "/ip4/0.0.0.0/ws/0"],
        ..Default::default()
    };
    
    // Create P2P client
    let mut client = P2pClient::new("My Node", config).await?;
    
    // Add custom protocol
    client.add_protocol("my-custom-protocol", |data| {
        // Handle custom protocol data
        println!("Received custom protocol data: {:?}", data);
        
        // Return response
        Ok(vec![1, 2, 3, 4, 5])
    });
    
    // Start the client
    client.start().await?;
    
    // Send custom protocol data to a peer
    let peer_id = "QmPeerId";
    let data = vec![6, 7, 8, 9, 10];
    let response = client.send_protocol_message(peer_id, "my-custom-protocol", data).await?;
    println!("Received response: {:?}", response);
    
    // Keep the client running
    tokio::signal::ctrl_c().await?;
    
    // Stop the client
    client.stop().await?;
    
    Ok(())
}
```

### Secure Messaging with End-to-End Encryption

```typescript
import { P2pClient } from '@gigi/p2p';
import * as crypto from 'crypto';

async function main() {
  try {
    // Create P2P client
    const client = new P2pClient('My Node', {
      bootstrapNodes: ['/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer'],
      enableKademlia: true,
      enableRelay: true,
      enableMdns: true,
      listenAddrs: ['/ip4/0.0.0.0/tcp/0', '/ip4/0.0.0.0/ws/0']
    });
    
    // Start the client
    await client.start();
    
    // Generate encryption key
    const encryptionKey = crypto.randomBytes(32);
    
    // Encrypt message
    function encryptMessage(message: string, key: Buffer): string {
      const iv = crypto.randomBytes(16);
      const cipher = crypto.createCipheriv('aes-256-cbc', key, iv);
      let encrypted = cipher.update(message, 'utf8', 'base64');
      encrypted += cipher.final('base64');
      return `${iv.toString('base64')}:${encrypted}`;
    }
    
    // Decrypt message
    function decryptMessage(encryptedMessage: string, key: Buffer): string {
      const [ivStr, encrypted] = encryptedMessage.split(':');
      const iv = Buffer.from(ivStr, 'base64');
      const decipher = crypto.createDecipheriv('aes-256-cbc', key, iv);
      let decrypted = decipher.update(encrypted, 'base64', 'utf8');
      decrypted += decipher.final('utf8');
      return decrypted;
    }
    
    // Send encrypted message
    const recipientPeerId = 'QmRecipientPeerId';
    const message = 'Hello with end-to-end encryption!';
    const encryptedMessage = encryptMessage(message, encryptionKey);
    await client.sendDirectMessage(recipientPeerId, encryptedMessage);
    console.log('Encrypted message sent');
    
    // Listen for encrypted messages
    client.on('message:direct', (msg) => {
      try {
        const decryptedMessage = decryptMessage(msg.content, encryptionKey);
        console.log(`Received decrypted message from ${msg.from}: ${decryptedMessage}`);
      } catch (error) {
        console.error('Error decrypting message:', error);
      }
    });
    
    // Keep the client running
    await new Promise(() => {});
  } catch (error) {
    console.error('Error:', error);
  }
}

main();
```

## Conclusion

These examples demonstrate how to use the Gigi P2P ecosystem in various scenarios, from basic messaging to advanced use cases. By following these examples, developers can quickly integrate Gigi P2P functionality into their applications and build decentralized systems that leverage the power of peer-to-peer communication.

For more detailed information about each component, please refer to the component-specific guides in the documentation.