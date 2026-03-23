# Gigi P2P Rust Client

The Gigi P2P Rust Client is a Rust implementation of the Gigi P2P client, providing core P2P functionality with high performance and reliability. This guide provides detailed information about the client's functionality, configuration, and usage.

## Overview

The Gigi P2P Rust Client is designed for use in performance-critical applications and as the foundation for other Gigi components. It provides a high-performance, memory-safe implementation of the Gigi P2P protocol, suitable for use in resource-constrained environments and as a reference implementation.

### Key Features

- **Rust Performance**: Fast and memory-safe implementation
- **Full P2P Stack**: Complete P2P networking stack
- **File Sharing**: Efficient file transfer between peers
- **Group Management**: Create and manage groups
- **Message Reliability**: Ensure message delivery
- **Connection Recovery**: Automatically reconnect to peers
- **Download Management**: Track and manage file downloads

## Installation

### Prerequisites

- **Rust**: v1.60 or later
- **Cargo**: Latest version

### Installation Steps

1. **Clone the Gigi repository**:
   ```bash
   git clone https://github.com/gigi-project/gigi.git
   cd gigi
   ```

2. **Build the Rust client**:
   ```bash
   cd pkgs/gigi-p2p
   cargo build
   ```

3. **Add the client to your project**:
   ```toml
   # In your Cargo.toml
   [dependencies]
   gigi-p2p = {
     path = "../gigi/pkgs/gigi-p2p",
     version = "0.1.0"
   }
   ```

## Configuration

The Gigi P2P Rust Client can be configured with various options to customize its behavior:

### Basic Configuration

```rust
use gigi_p2p::P2pConfig;

let config = P2pConfig {
    bootstrap_nodes: vec!["/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer"],
    enable_kademlia: true,
    enable_relay: true,
    enable_mdns: true,
    listen_addrs: vec!["/ip4/0.0.0.0/tcp/0"],
};
```

### Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `bootstrap_nodes` | List of bootstrap nodes to connect to | `vec![]` |
| `enable_kademlia` | Enable Kademlia DHT for peer discovery | `true` |
| `enable_relay` | Enable circuit relay for NAT traversal | `true` |
| `enable_mdns` | Enable mDNS for local peer discovery | `true` |
| `listen_addrs` | List of addresses to listen on | `vec!["/ip4/0.0.0.0/tcp/0"]` |
| `max_connections` | Maximum number of connections | `100` |
| `connection_timeout` | Connection timeout in milliseconds | `30000` |
| `relay_hop_limit` | Maximum number of relay hops | `3` |

## Usage

### Basic Usage

```rust
use gigi_p2p::P2pClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create config
    let config = P2pConfig {
        bootstrap_nodes: vec!["/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer"],
        enable_kademlia: true,
        enable_relay: true,
        enable_mdns: true,
        listen_addrs: vec!["/ip4/0.0.0.0/tcp/0"],
    };
    
    // Create client
    let mut client = P2pClient::new("My Node", config).await?;
    
    // Start client
    client.start().await?;
    println!("Started with peer ID: {}", client.peer_id());
    
    // Send direct message
    client.send_direct_message("peer-id", "Hello!").await?;
    
    // Join group
    client.join_group("general").await?;
    
    // Send group message
    client.send_group_message("general", "Hello everyone!").await?;
    
    // Share file
    let share_code = client.share_file("/path/to/file.txt").await?;
    println!("File shared with code: {}", share_code);
    
    // Download file
    let download_id = client.download_file("peer-id", &share_code).await?;
    println!("Download started with ID: {}", download_id);
    
    // Listen for events
    client.on_event(|event| {
        match event {
            Event::DirectMessage { from, message } => {
                println!("Received message from {}: {}", from, message);
            }
            Event::GroupMessage { from, group, message } => {
                println!("Received group message from {} in {}: {}", from, group, message);
            }
            _ => {}
        }
    });
    
    // Stop client
    client.stop().await?;
    
    Ok(())
}
```

### Advanced Usage

#### Custom Event Handling

```rust
// Listen for specific events
client.on_event(|event| {
    match event {
        Event::PeerDiscovered { peer_id } => {
            println!("Discovered peer: {}", peer_id);
        }
        Event::PeerConnected { peer_id } => {
            println!("Connected to peer: {}", peer_id);
        }
        Event::PeerDisconnected { peer_id } => {
            println!("Disconnected from peer: {}", peer_id);
        }
        Event::DirectMessage { from, message } => {
            println!("Received message from {}: {}", from, message);
        }
        Event::GroupMessage { from, group, message } => {
            println!("Received group message from {} in {}: {}", from, group, message);
        }
        Event::FileShared { share_code } => {
            println!("File shared with code: {}", share_code);
        }
        Event::FileDownloadStarted { download_id } => {
            println!("File download started with ID: {}", download_id);
        }
        Event::FileDownloadProgress { download_id, progress } => {
            println!("Download {} progress: {}%", download_id, progress * 100.0);
        }
        Event::FileDownloadCompleted { download_id, file_path } => {
            println!("Download {} completed: {:?}", download_id, file_path);
        }
        Event::FileDownloadFailed { download_id, error } => {
            println!("Download {} failed: {:?}", download_id, error);
        }
        Event::Error { error } => {
            eprintln!("Error: {:?}", error);
        }
    }
});
```

#### Peer Management

```rust
// List all discovered peers
let peers = client.list_peers().await?;
println!("Discovered {} peers:", peers.len());
for peer in peers {
    println!("- {}: {}", peer.id, peer.nickname.unwrap_or_else(|| "Unknown".to_string()));
}

// Connect to a specific peer
client.connect_to_peer("peer-id").await?;

// Disconnect from a peer
client.disconnect_from_peer("peer-id").await?;
```

#### Group Management

```rust
// List all joined groups
let groups = client.list_groups().await?;
println!("Joined {} groups:", groups.len());
for group in groups {
    println!("- {}", group.name);
}

// Join a group
client.join_group("general").await?;

// Leave a group
client.leave_group("general").await?;
```

#### File Sharing

```rust
// Share a file
let share_code = client.share_file("/path/to/file.txt").await?;
println!("File shared with code: {}", share_code);

// Download a file
let download_id = client.download_file("peer-id", &share_code).await?;
println!("Download started with ID: {}", download_id);

// Cancel a download
client.cancel_download(&download_id).await?;

// List active downloads
let downloads = client.list_active_downloads().await?;
println!("Active downloads: {}", downloads.len());
for download in downloads {
    println!("- {}: {}%", download.id, download.progress * 100.0);
}
```

## Architecture

### Client Structure

The Gigi P2P Rust Client consists of several key components:

1. **P2pClient**: Main client class that orchestrates all functionality
2. **Behaviour**: Libp2p behaviour that handles network events
3. **PeerManager**: Manages peer discovery and connections
4. **GroupManager**: Handles group messaging and subscriptions
5. **FileSharing**: Manages file sharing and downloads
6. **EventHandler**: Processes network events

### Data Flow

1. **Client Initialization**: Client is created and configured
2. **Network Connection**: Client connects to the P2P network
3. **Peer Discovery**: Client discovers other peers
4. **Message Processing**: Client sends and receives messages
5. **File Transfer**: Client shares and downloads files

## Security

### Authentication

- **Peer Verification**: Peers are verified by their public keys
- **Encryption**: All communications are encrypted using Libp2p's noise protocol
- **Access Control**: Implement application-level access control

### Best Practices

- **Use Secure Transport**: Always use encrypted transports
- **Validate Peers**: Verify peer identities before communicating
- **Limit Exposure**: Only share necessary information
- **Update Regularly**: Keep the client updated to the latest version

## Troubleshooting

### Common Issues

#### Connection Problems

- **Symptom**: Cannot connect to the network
- **Solution**: Check network connectivity, firewall settings, and bootstrap nodes

#### Peer Discovery

- **Symptom**: Cannot find other peers
- **Solution**: Ensure mDNS and DHT are enabled, check network multicast settings

#### File Transfer

- **Symptom**: File transfer fails
- **Solution**: Check file permissions, disk space, and network stability

#### Group Messaging

- **Symptom**: Group messages not received
- **Solution**: Ensure all peers are subscribed to the same group topic

### Debugging

Enable debug logging to troubleshoot issues:

```rust
// Enable debug logging
client.enable_debug_logging();

// Check client status
let status = client.get_status().await?;
println!("Client status: {:?}", status);

// Check network stats
let stats = client.get_network_stats().await?;
println!("Network stats: {:?}", stats);
```

## Advanced Features

### Custom Libp2p Configuration

You can provide a custom Libp2p configuration to fine-tune the client's behavior:

```rust
use libp2p::identity::Keypair;
use libp2p::noise::Config as NoiseConfig;
use libp2p::tcp::Config as TcpConfig;
use libp2p::yamux::Config as YamuxConfig;
use libp2p::kad::KademliaConfig;
use libp2p::gossipsub::GossipsubConfig;

// Create custom Libp2p configuration
let mut libp2p_config = Libp2pConfig {
    keypair: Keypair::generate_ed25519(),
    transports: vec![TcpConfig::default().into()],
    connection_encryption: NoiseConfig::new,
    stream_muxers: vec![YamuxConfig::default().into()],
    kademlia_config: Some(KademliaConfig::default()),
    gossipsub_config: Some(GossipsubConfig::default()),
    ..Default::default()
};

// Create Gigi client with custom Libp2p configuration
let mut client = P2pClient::new_with_libp2p_config("My Node", libp2p_config).await?;
```

### NAT Traversal

The client automatically handles NAT traversal using Libp2p's circuit relay protocol. For better results, you can configure relay options:

```rust
let config = P2pConfig {
    enable_relay: true,
    relay_hop_limit: 3,
    // Additional relay configuration
    ..Default::default()
};

let mut client = P2pClient::new("My Node", config).await?;
```

### Performance Optimization

To optimize performance:

- **Limit Connections**: Configure `max_connections` in client settings
- **Enable Relay**: Use relay nodes for better NAT traversal
- **Optimize DHT**: Adjust DHT parameters for your network size
- **Use Fast Transports**: Prioritize faster transports like QUIC

## API Reference

### P2pClient Struct

#### Constructor

```rust
let client = P2pClient::new(nickname, config).await?;
```

**Parameters**:
- `nickname`: Human-readable name for your node
- `config`: Configuration options

#### Methods

##### `start()`

Start the P2P client.

**Returns**:
- `Result<(), Error>`

##### `stop()`

Stop the P2P client.

**Returns**:
- `Result<(), Error>`

##### `peer_id()`

Get the peer ID of the client.

**Returns**:
- `String` (peer ID)

##### `multiaddrs()`

Get the multiaddresses the client is listening on.

**Returns**:
- `Vec<String>` (multiaddresses)

##### `is_connected()`

Check if the client is connected to the network.

**Returns**:
- `bool`

##### `list_peers()`

List all discovered peers.

**Returns**:
- `Result<Vec<PeerInfo>, Error>`

##### `connect_to_peer(peer_id)`

Connect to a specific peer.

**Parameters**:
- `peer_id`: Peer ID to connect to

**Returns**:
- `Result<(), Error>`

##### `disconnect_from_peer(peer_id)`

Disconnect from a peer.

**Parameters**:
- `peer_id`: Peer ID to disconnect from

**Returns**:
- `Result<(), Error>`

##### `send_direct_message(peer_id, message)`

Send a direct message to a peer.

**Parameters**:
- `peer_id`: Target peer ID
- `message`: Message content

**Returns**:
- `Result<(), Error>`

##### `list_groups()`

List all joined groups.

**Returns**:
- `Result<Vec<GroupInfo>, Error>`

##### `join_group(group_name)`

Join a group.

**Parameters**:
- `group_name`: Group name

**Returns**:
- `Result<(), Error>`

##### `leave_group(group_name)`

Leave a group.

**Parameters**:
- `group_name`: Group name

**Returns**:
- `Result<(), Error>`

##### `send_group_message(group_name, message)`

Send a message to a group.

**Parameters**:
- `group_name`: Group name
- `message`: Message content

**Returns**:
- `Result<(), Error>`

##### `share_file(file_path)`

Share a file.

**Parameters**:
- `file_path`: Path to file

**Returns**:
- `Result<String, Error>` (share code)

##### `download_file(peer_id, share_code)`

Download a file.

**Parameters**:
- `peer_id`: Peer ID to download from
- `share_code`: Share code

**Returns**:
- `Result<String, Error>` (download ID)

##### `cancel_download(download_id)`

Cancel a download.

**Parameters**:
- `download_id`: Download ID to cancel

**Returns**:
- `Result<(), Error>`

##### `list_active_downloads()`

List active downloads.

**Returns**:
- `Result<Vec<DownloadInfo>, Error>`

##### `on_event(handler)`

Register an event handler.

**Parameters**:
- `handler`: Event handler function

##### `enable_debug_logging()`

Enable debug logging.

##### `get_status()`

Get the client status.

**Returns**:
- `Result<ClientStatus, Error>`

##### `get_network_stats()`

Get network statistics.

**Returns**:
- `Result<NetworkStats, Error>`

## Examples

### Basic Messaging

```rust
use gigi_p2p::P2pClient;
use gigi_p2p::P2pConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create config
    let config = P2pConfig {
        bootstrap_nodes: vec!["/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer"],
        ..Default::default()
    };
    
    // Create and start client
    let mut client = P2pClient::new("Alice", config).await?;
    client.start().await?;
    
    println!("Alice started with peer ID: {}", client.peer_id());
    
    // Send message to Bob
    client.send_direct_message("bob-peer-id", "Hello Bob!").await?;
    
    // Listen for messages
    client.on_event(|event| {
        match event {
            gigi_p2p::Event::DirectMessage { from, message } => {
                println!("Alice received: {} from {}", message, from);
            }
            _ => {}
        }
    });
    
    // Keep the program running
    tokio::signal::ctrl_c().await?;
    
    // Stop client
    client.stop().await?;
    
    Ok(())
}
```

### Group Chat

```rust
use gigi_p2p::P2pClient;
use gigi_p2p::P2pConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create config
    let config = P2pConfig {
        bootstrap_nodes: vec!["/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer"],
        ..Default::default()
    };
    
    // Create and start client
    let mut client = P2pClient::new("Alice", config).await?;
    client.start().await?;
    
    // Join general group
    client.join_group("general").await?;
    println!("Alice joined general group");
    
    // Send group message
    client.send_group_message("general", "Hello everyone!").await?;
    
    // Listen for group messages
    client.on_event(|event| {
        match event {
            gigi_p2p::Event::GroupMessage { from, group, message } => {
                println!("Alice received group message in {}: {} from {}", group, message, from);
            }
            _ => {}
        }
    });
    
    // Keep the program running
    tokio::signal::ctrl_c().await?;
    
    // Stop client
    client.stop().await?;
    
    Ok(())
}
```

### File Sharing

```rust
use gigi_p2p::P2pClient;
use gigi_p2p::P2pConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create config
    let config = P2pConfig {
        bootstrap_nodes: vec!["/ip4/127.0.0.1/tcp/4001/p2p/QmBootstrapPeer"],
        ..Default::default()
    };
    
    // Create and start client
    let mut client = P2pClient::new("Alice", config).await?;
    client.start().await?;
    
    // Share a file
    let share_code = client.share_file("/path/to/document.pdf").await?;
    println!("File shared with code: {}", share_code);
    
    // Listen for download events
    client.on_event(|event| {
        match event {
            gigi_p2p::Event::FileDownloadProgress { download_id, progress } => {
                println!("Download progress: {}%", progress * 100.0);
            }
            gigi_p2p::Event::FileDownloadCompleted { download_id, file_path } => {
                println!("Download completed: {:?}", file_path);
            }
            _ => {}
        }
    });
    
    // Download a file from Bob
    let download_id = client.download_file("bob-peer-id", "share-code-from-bob").await?;
    println!("Download started with ID: {}", download_id);
    
    // Keep the program running
    tokio::signal::ctrl_c().await?;
    
    // Stop client
    client.stop().await?;
    
    Ok(())
}
```

## Conclusion

The Gigi P2P Rust Client provides a high-performance, memory-safe implementation of the Gigi P2P protocol, suitable for use in performance-critical applications and as a reference implementation. By following this guide, you can integrate P2P functionality into your Rust applications, creating robust, distributed systems that work across networks and devices.

For more information, see the [API Reference](api/rust-api.md) and [Troubleshooting Guide](guides/troubleshooting-guide.md).