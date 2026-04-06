# Gigi DNS

Gigi DNS is the decentralized DNS service for the Gigi P2P network, allowing peers to resolve names to peer IDs. It provides a human-friendly way to address peers without needing to remember long peer IDs. This guide provides detailed information about Gigi DNS's functionality, configuration, and usage.

## Overview

Gigi DNS is designed to simplify peer addressing in the Gigi P2P network by allowing users to use human-readable names instead of long, hexadecimal peer IDs. It is a decentralized service, meaning there is no central authority or single point of failure. Gigi DNS uses cryptographically verified records to ensure the security and integrity of name resolutions.

### Key Features

- **Name Resolution**: Resolve human-readable names to peer IDs
- **Decentralized**: No central authority or single point of failure
- **Secure**: Cryptographically verified records
- **Efficient**: Cached for performance
- **Fault-tolerant**: Graceful degradation if name resolution fails
- **Extensible**: Support for different record types and name spaces

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

2. **Build Gigi DNS**:
   ```bash
   cd rust/gigi-dns
   cargo build
   ```

3. **Add Gigi DNS to your project**:
   ```toml
   # In your Cargo.toml
   [dependencies]
   gigi-dns = {
     path = "../gigi/rust/gigi-dns",
     version = "0.1.0"
   }
   ```

## Configuration

Gigi DNS can be configured with various options to customize its behavior:

### Basic Configuration

```rust
use gigi_dns::GigiDns;

// Create DNS instance with default settings
let dns = GigiDns::new().await?;
```

### Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `cache_size` | Size of the DNS cache | `1000` |
| `cache_ttl` | Time-to-live for cached records (seconds) | `3600` |
| `record_ttl` | Time-to-live for published records (seconds) | `86400` |
| `max_name_length` | Maximum length of a DNS name | `63` |
| `enable_mdns` | Enable mDNS for local resolution | `true` |
| `enable_dht` | Enable DHT for global resolution | `true` |

## Usage

### Basic Usage

```rust
use gigi_dns::GigiDns;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create DNS instance
    let dns = GigiDns::new().await?;
    
    // Register a name
    dns.register("alice", "QmAlicePeerId").await?;
    println!("Registered 'alice' to QmAlicePeerId");
    
    // Resolve a name
    let peer_id = dns.resolve("alice").await?;
    println!("Resolved 'alice' to: {}", peer_id);
    
    // List registered names
    let names = dns.list_names().await?;
    println!("Registered names: {:?}", names);
    
    // Unregister a name
    dns.unregister("alice").await?;
    println!("Unregistered 'alice'");
    
    Ok(())
}
```

### Advanced Usage

#### Custom Cache Configuration

```rust
use gigi_dns::GigiDnsConfig;

let config = GigiDnsConfig {
    cache_size: 5000,
    cache_ttl: 7200, // 2 hours
    record_ttl: 172800, // 2 days
    max_name_length: 63,
    enable_mdns: true,
    enable_dht: true,
};

let dns = GigiDns::new_with_config(config).await?;
```

#### Name Resolution with Fallback

```rust
// Try to resolve a name, with fallback
let peer_id = match dns.resolve("alice").await {
    Ok(peer_id) => peer_id,
    Err(_) => {
        println!("Name resolution failed, using fallback peer ID");
        "QmFallbackPeerId".to_string()
    }
};

println!("Using peer ID: {}", peer_id);
```

#### Batch Operations

```rust
// Register multiple names
let records = vec![
    ("alice", "QmAlicePeerId"),
    ("bob", "QmBobPeerId"),
    ("charlie", "QmCharliePeerId"),
];

for (name, peer_id) in records {
    dns.register(name, peer_id).await?;
    println!("Registered '{}' to {}", name, peer_id);
}

// Resolve multiple names
for name in vec!["alice", "bob", "charlie"] {
    match dns.resolve(name).await {
        Ok(peer_id) => println!("Resolved '{}' to: {}", name, peer_id),
        Err(e) => println!("Failed to resolve '{}': {:?}", name, e),
    }
}
```

## Architecture

### DNS Structure

The Gigi DNS system consists of several key components:

1. **GigiDns**: Main DNS manager
2. **Cache**: In-memory cache for resolved records
3. **mDNS**: Local name resolution via multicast DNS
4. **DHT**: Global name resolution via distributed hash table
5. **Record Manager**: Manages DNS records
6. **Validator**: Verifies record signatures

### Data Flow

1. **Name Resolution**: Client requests resolution of a name
2. **Cache Check**: Check if the name is in the local cache
3. **Local Resolution**: Try mDNS for local names
4. **Global Resolution**: Try DHT for global names
5. **Cache Update**: Cache the resolved record
6. **Response**: Return the resolved peer ID to the client

## Security

### Authentication

- **Record Signing**: DNS records are signed with the peer's private key
- **Signature Verification**: Records are verified before being accepted
- **Name Ownership**: Only the owner of a peer ID can register names for it
- **Anti-Spoofing**: Cryptographic verification prevents name spoofing

### Best Practices

- **Use Unique Names**: Choose unique names to avoid conflicts
- **Keep Names Short**: Shorter names are easier to remember and resolve faster
- **Verify Resolutions**: Always verify resolved peer IDs before communicating
- **Update Regularly**: Keep Gigi DNS updated to the latest version

## Troubleshooting

### Common Issues

#### Name Resolution Failed

- **Symptom**: Cannot resolve a name to a peer ID
- **Solution**: Check network connectivity, ensure the name is registered, verify the peer is online

#### Name Conflict

- **Symptom**: Multiple peers trying to use the same name
- **Solution**: Choose a unique name, check for typos

#### Slow Resolution

- **Symptom**: Name resolution takes too long
- **Solution**: Check network connectivity, ensure DHT is functioning properly

#### Cache Issues

- **Symptom**: Resolving to old peer ID
- **Solution**: Clear the DNS cache, wait for TTL to expire

### Debugging

Enable debug logging to troubleshoot issues:

```rust
// Enable debug logging
env::set_var("RUST_LOG", "gigi_dns=debug");

// Create DNS instance with debug logging
let dns = GigiDns::new().await?;
```

## Advanced Features

### Custom Name Spaces

Gigi DNS supports different name spaces for better organization:

```rust
// Register a name in a specific namespace
dns.register_in_namespace("alice", "QmAlicePeerId", "users").await?;

// Resolve a name from a specific namespace
let peer_id = dns.resolve_from_namespace("alice", "users").await?;
```

### Record Types

Gigi DNS supports different record types for extended functionality:

```rust
// Register a service record
dns.register_service("chat", "QmChatServicePeerId", "tcp", 5001).await?;

// Resolve a service record
let service_info = dns.resolve_service("chat").await?;
println!("Service chat: {}:{}:{}", service_info.peer_id, service_info.protocol, service_info.port);
```

### Integration with Other Components

Integrate Gigi DNS with other Gigi components:

```rust
use gigi_dns::GigiDns;
use gigi_p2p::P2pClient;

// Create DNS instance
let dns = GigiDns::new().await?;

// Create P2P client
let mut client = P2pClient::new("My Node", config).await?;

// Register the client's peer ID with a name
dns.register("my-node", client.peer_id()).await?;

// Resolve a name and connect to the peer
let peer_id = dns.resolve("alice").await?;
client.connect_to_peer(&peer_id).await?;
```

## API Reference

### GigiDns Struct

#### Constructor

```rust
let dns = GigiDns::new().await?;
```

**Parameters**:
- None (uses default configuration)

#### Methods

##### `register(name, peer_id)`

Register a name for a peer ID.

**Parameters**:
- `name`: Name to register
- `peer_id`: Peer ID to associate with the name

**Returns**:
- `Result<(), Error>`

##### `register_in_namespace(name, peer_id, namespace)`

Register a name in a specific namespace.

**Parameters**:
- `name`: Name to register
- `peer_id`: Peer ID to associate with the name
- `namespace`: Namespace to register the name in

**Returns**:
- `Result<(), Error>`

##### `resolve(name)`

Resolve a name to a peer ID.

**Parameters**:
- `name`: Name to resolve

**Returns**:
- `Result<String, Error>` (peer ID)

##### `resolve_from_namespace(name, namespace)`

Resolve a name from a specific namespace.

**Parameters**:
- `name`: Name to resolve
- `namespace`: Namespace to resolve from

**Returns**:
- `Result<String, Error>` (peer ID)

##### `unregister(name)`

Unregister a name.

**Parameters**:
- `name`: Name to unregister

**Returns**:
- `Result<(), Error>`

##### `unregister_from_namespace(name, namespace)`

Unregister a name from a specific namespace.

**Parameters**:
- `name`: Name to unregister
- `namespace`: Namespace to unregister from

**Returns**:
- `Result<(), Error>`

##### `list_names()`

List all registered names.

**Returns**:
- `Result<Vec<String>, Error>` (list of names)

##### `list_names_in_namespace(namespace)`

List all registered names in a specific namespace.

**Parameters**:
- `namespace`: Namespace to list names from

**Returns**:
- `Result<Vec<String>, Error>` (list of names)

##### `register_service(name, peer_id, protocol, port)`

Register a service record.

**Parameters**:
- `name`: Service name
- `peer_id`: Peer ID providing the service
- `protocol`: Service protocol (e.g., "tcp")
- `port`: Service port

**Returns**:
- `Result<(), Error>`

##### `resolve_service(name)`

Resolve a service record.

**Parameters**:
- `name`: Service name to resolve

**Returns**:
- `Result<ServiceInfo, Error>`

##### `clear_cache()`

Clear the DNS cache.

**Returns**:
- `Result<(), Error>`

##### `get_cache_size()`

Get the current cache size.

**Returns**:
- `usize` (cache size)

## Examples

### Basic Name Resolution

```rust
use gigi_dns::GigiDns;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create DNS instance
    let dns = GigiDns::new().await?;
    
    // Register names
    dns.register("alice", "QmAlicePeerId").await?;
    dns.register("bob", "QmBobPeerId").await?;
    
    println!("Registered names: alice, bob");
    
    // Resolve names
    let alice_id = dns.resolve("alice").await?;
    let bob_id = dns.resolve("bob").await?;
    
    println!("Resolved alice to: {}", alice_id);
    println!("Resolved bob to: {}", bob_id);
    
    // List registered names
    let names = dns.list_names().await?;
    println!("All registered names: {:?}", names);
    
    // Unregister a name
    dns.unregister("alice").await?;
    println!("Unregistered alice");
    
    // Try to resolve the unregistered name
    match dns.resolve("alice").await {
        Ok(peer_id) => println!("Resolved alice to: {}", peer_id),
        Err(e) => println!("Failed to resolve alice: {:?}", e),
    }
    
    Ok(())
}
```

### Namespace Usage

```rust
use gigi_dns::GigiDns;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create DNS instance
    let dns = GigiDns::new().await?;
    
    // Register names in different namespaces
    dns.register_in_namespace("alice", "QmAlicePeerId", "users").await?;
    dns.register_in_namespace("bob", "QmBobPeerId", "users").await?;
    dns.register_in_namespace("chat", "QmChatServicePeerId", "services").await?;
    dns.register_in_namespace("file", "QmFileServicePeerId", "services").await?;
    
    println!("Registered names in namespaces");
    
    // Resolve names from specific namespaces
    let alice_id = dns.resolve_from_namespace("alice", "users").await?;
    let chat_id = dns.resolve_from_namespace("chat", "services").await?;
    
    println!("Resolved alice (users) to: {}", alice_id);
    println!("Resolved chat (services) to: {}", chat_id);
    
    // List names in each namespace
    let user_names = dns.list_names_in_namespace("users").await?;
    let service_names = dns.list_names_in_namespace("services").await?;
    
    println!("User names: {:?}", user_names);
    println!("Service names: {:?}", service_names);
    
    Ok(())
}
```

### Service Registration

```rust
use gigi_dns::GigiDns;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create DNS instance
    let dns = GigiDns::new().await?;
    
    // Register services
    dns.register_service("chat", "QmChatServicePeerId", "tcp", 5001).await?;
    dns.register_service("file", "QmFileServicePeerId", "tcp", 5002).await?;
    
    println!("Registered services: chat, file");
    
    // Resolve services
    let chat_service = dns.resolve_service("chat").await?;
    let file_service = dns.resolve_service("file").await?;
    
    println!("Chat service: {}:{}:{}", chat_service.peer_id, chat_service.protocol, chat_service.port);
    println!("File service: {}:{}:{}", file_service.peer_id, file_service.protocol, file_service.port);
    
    Ok(())
}
```

## Conclusion

Gigi DNS provides a decentralized name resolution service for the Gigi P2P network, making it easier for users to address peers using human-readable names instead of long peer IDs. By following this guide, you can integrate Gigi DNS into your applications, providing a more user-friendly way to identify and connect to peers in the network.

For more information, see the [API Reference](api/dns-api.md) and [Troubleshooting Guide](guides/troubleshooting-guide.md).