# Gigi DNS Usage Guide

## Quick Start

```rust
use gigi_dns::{GigiDnsConfig, GigiDnsBehaviour};

// Create config with nickname
let config = GigiDnsConfig {
    nickname: "Alice".to_string(),
    capabilities: vec!["chat".to_string(), "file_sharing".to_string()],
    ..Default::default()
};

// Create behavior
let mdns = GigiDnsBehaviour::new(local_peer_id, config)?;

// Add to swarm
let mut swarm = SwarmBuilder::new()
    .with_behaviour(|_| mdns)?
    .build();
```

## Handling Events

```rust
loop {
    match swarm.select_next_some() {
        SwarmEvent::Behaviour(GigiDnsEvent::Discovered(info)) => {
            println!("Discovered {} at {}", info.nickname, info.multiaddr);
            println!("Capabilities: {:?}", info.capabilities);
            app_state.add_peer(info);
        }
        SwarmEvent::Behaviour(GigiDnsEvent::Updated { peer_id, old_info, new_info }) => {
            println!("Peer updated: {} → {}", old_info.nickname, new_info.nickname);
        }
        SwarmEvent::Behaviour(GigiDnsEvent::Expired { peer_id, info }) => {
            println!("Peer expired: {}", info.nickname);
            app_state.remove_peer(peer_id);
        }
        _ => {}
    }
}
```

## Finding Peers

```rust
// Find by nickname
if let Some(peer) = mdns.find_peer_by_nickname("Alice") {
    println!("Found Alice at {}", peer.multiaddr);
    swarm.dial(peer.multiaddr)?;
}

// Get all peers
let peers = mdns.get_discovered_peers();
for peer in peers {
    println!("{} ({})", peer.nickname, peer.peer_id);
}

// Find by capability
let chat_peers: Vec<_> = peers
    .iter()
    .filter(|p| p.capabilities.contains(&"chat".to_string()))
    .collect();
```

## Configuration Options

```rust
let config = GigiDnsConfig {
    nickname: "Alice".to_string(),
    ttl: Duration::from_secs(6 * 60),        // 6 minutes
    query_interval: Duration::from_secs(5 * 60),  // 5 minutes
    enable_ipv6: false,                      // IPv4 only
    capabilities: vec![
        "chat".to_string(),
        "file_sharing".to_string(),
    ],
    metadata: {
        let mut map = HashMap::new();
        map.insert("version".to_string(), "2.0".to_string());
        map.insert("os".to_string(), "Linux".to_string());
        map
    },
};
```

## Real-World Examples

### Chat Application
```rust
let config = GigiDnsConfig {
    nickname: "Alice".to_string(),
    capabilities: vec!["chat".to_string()],
    metadata: {
        let mut meta = HashMap::new();
        meta.insert("status".to_string(), "available".to_string());
        meta.insert("chat_version".to_string(), "2.0".to_string());
        meta
    },
    ..Default::default()
};
```

### File Sharing Network
```rust
let config = GigiDnsConfig {
    nickname: "Bob-FileServer".to_string(),
    capabilities: vec!["file_sharing".to_string(), "download".to_string()],
    metadata: {
        let mut meta = HashMap::new();
        meta.insert("storage".to_string(), "1TB".to_string());
        meta.insert("bandwidth".to_string(), "1Gbps".to_string());
        meta
    },
    ..Default::default()
};
```

### IoT Device Discovery
```rust
let config = GigiDnsConfig {
    nickname: "TemperatureSensor-1".to_string(),
    capabilities: vec!["temperature_sensor".to_string()],
    metadata: {
        let mut meta = HashMap::new();
        meta.insert("location".to_string(), "Living Room".to_string());
        meta.insert("type".to_string(), "DHT22".to_string());
        meta.insert("battery".to_string(), "85%".to_string());
        meta
    },
    ..Default::default()
};
```
