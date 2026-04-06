## Explain

```rust
ttl: Duration::from_secs(6 * 60),
query_interval: Duration::from_secs(5 * 60),
```

These two settings control the timing behavior of the Gigi DNS protocol:

**`ttl: Duration::from_secs(6 * 60)`** (6 minutes)
- **Time To Live** for peer records
- When a peer is discovered, its information is cached for 6 minutes
- After 6 minutes without receiving an announcement from that peer, the peer record expires
- Expired peers are removed from the discovered peers list and generate an `Expired` event

**`query_interval: Duration::from_secs(5 * 60)`** (5 minutes)
- How often the local node sends discovery **queries** to the network
- Every 5 minutes, the node sends a query asking "who is out there?"
- Other nodes respond with their current information, refreshing their TTL

**Why 5 vs 6 minutes:**
- The query interval (5 min) is shorter than the TTL (6 min)
- This ensures that before a peer expires, the local node will query the network
- When queried, other peers announce themselves, refreshing their TTL
- This prevents peers from disappearing due to timeout when they're still active


## what's the standard values for mdns?

Standard mDNS values are:

**TTL: 120 seconds (2 minutes)**
- DNS resource records have a recommended TTL of 120 seconds
- Some implementations use 75 seconds or 4500 seconds (75 min) for PTR records

**Query Interval:**
- First query sent immediately
- Repeated queries after 1 second, then 2 seconds (up to 3 retries for robustness)
- After initial discovery, active hosts announce themselves every 120 seconds
- Passive hosts respond to queries but don't periodically announce

**Current Gigi DNS values (6 min TTL, 5 min query) are much longer** than standard mDNS. For typical local network discovery, you might want:
- `ttl: Duration::from_secs(120)` (2 min)
- `query_interval: Duration::from_secs(60)` (1 min) - or use the initial+retry pattern

## explain ttl, query_interval, announce_interval, cleanup_interval

Here's an explanation of all four timing parameters:

## `ttl` (Time To Live) - 6 minutes
- **Purpose**: How long discovered peer information remains valid
- **What happens**: When a peer is discovered, its info is cached for this duration
- **Expiration**: After `ttl` seconds without receiving an announcement from that peer, the peer record expires
- **Result**: Generates an `Expired` event and removes the peer from discovered list

## `query_interval` - 5 minutes
- **Purpose**: How often to actively ask "who is out there?"
- **What happens**: Sends a query packet to the multicast group
- **Response**: Other peers respond with their current peer information
- **Why shorter than TTL**: Ensures peers refresh their TTL before expiring

## `announce_interval` - 15 seconds
- **Purpose**: How often to broadcast "I'm here!" to the network
- **What happens**: Sends an unsolicited announcement with your peer info
- **Benefit**: Helps peers discover you quickly without waiting for queries
- **Trade-off**: More frequent = faster discovery, but more network traffic

## `cleanup_interval` - 30 seconds
- **Purpose**: How often to check for and remove expired peers
- **What happens**: Iterates through discovered peers and removes those past their TTL
- **Result**: Generates `Expired` events for cleaned up peers
- **Separate from TTL**: TTL determines *when* a peer expires, cleanup determines *how often* we check

## Timing Relationship
```
Every 15 seconds:  You announce your presence
Every 5 minutes:   You ask "who's out there?"
Every 30 seconds:  You remove expired peers from your list
Every 6 minutes:   Peers expire if you haven't heard from them
```

This design ensures peers stay discovered while keeping network traffic reasonable.