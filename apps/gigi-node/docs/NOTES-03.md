# GossipSub and Kademlia Relationship

GossipSub and Kademlia are **two independent protocols** that work together in libp2p:

## What Each Does

| Protocol | Purpose | How it Works |
|----------|---------|--------------|
| **Kademlia** | Peer Discovery | DHT (Distributed Hash Table) - stores (key, value) pairs, finds peers by ID |
| **GossipSub** | Message Routing | PubSub - broadcasts messages to subscribers of a topic |

## How They Work Together

### 1. Kademlia Finds Peers
```
Alice wants to find peers interested in "gigi-general"
→ Query Kademlia DHT
→ Get list of peer IDs interested in that topic
→ Connect to them
```

### 2. GossipSub Routes Messages
```
Alice publishes "hello" on "gigi-general"
→ GossipSub forwards to mesh peers (6 in default mesh)
→ Those peers forward to their mesh peers
→ Message propagates through network
```

## In Gigi's Architecture

```
┌─────────────────────────────────────────────────────┐
│                    Kademlia DHT                     │
│   (Peer discovery, finding who has what topic)      │
└─────────────────────┬───────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────┐
│                  GossipSub                          │
│   (Message routing, publishing/subscribing)          │
└─────────────────────────────────────────────────────┘
```

## Current Gigi Setup

In `docker_group_client.rs`:

```rust
// Kademlia: Find peers
swarm.behaviour_mut().kademlia.get_closest_peers(topic_hash.as_bytes().to_vec());

// GossipSub: Subscribe to topic
swarm.behaviour_mut().gossipsub.subscribe(&topic);

// When peer connects, add to mesh
swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
```

## Key Differences

| Aspect | Kademlia | GossipSub |
|--------|----------|-----------|
| **Purpose** | Discover peers | Route messages |
| **Lookup** | By peer ID or topic hash | By topic subscription |
| **Protocol** | DHT query | PubSub gossip |
| **Persistence** | Stores records | Just转发 messages |

## Summary

- **Kademlia** = "Who's out there?" (Discovery)
- **GossipSub** = "Send this message to them" (Routing)

They complement each other: Kademlia helps GossipSub find peers interested in a topic, then GossipSub handles message delivery.
