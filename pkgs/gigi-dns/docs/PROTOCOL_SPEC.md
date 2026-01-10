# Gigi DNS Protocol Specification

## Overview

Gigi DNS is an mDNS-based protocol for peer discovery with rich metadata.

## Service Name

```
_gigi-dns._udp.local
```

## TXT Record Format

### Encoding

```
peer_id=<peer_id> nickname=<nickname> addr=<multiaddr> caps=<caps> meta=<meta>
```

### Fields

| Field | Description | Example |
|-------|-------------|---------|
| peer_id | libp2p PeerId | 12D3KooWBob... |
| nickname | Human-readable name | Alice |
| addr | Network multiaddr | /ip4/192.168.1.100/tcp/8000 |
| caps | Comma-separated capabilities | chat,file_sharing |
| meta | Comma-separated key:value pairs | version:1.0,os:Linux |

### Example

```
peer_id=12D3KooWBob nickname=Alice addr=/ip4/192.168.1.100/tcp/8000 caps=chat,file_sharing meta=version:1.0,os:Linux
```

## DNS Packet Structure

### Query Packet

```
+------------------+
| Transaction ID   | 2 bytes (random)
+------------------+
| Flags           | 2 bytes (0x0000)
+------------------+
| Questions        | 2 bytes (0x0001)
+------------------+
| Answers         | 2 bytes (0x0000)
+------------------+
| Authority       | 2 bytes (0x0000)
+------------------+
| Additional      | 2 bytes (0x0000)
+------------------+
| Question Name   | "_gigi-dns._udp.local"
+------------------+
| Type           | 0x000C (PTR)
+------------------+
| Class          | 0x0001 (IN)
+------------------+
```

### Response Packet

```
+------------------+
| Header          | 12 bytes
+------------------+
| Questions       | 0x0000
+------------------+
| Answers         | N (number of records)
+------------------+
| Authority       | 0x0000
+------------------+
| Additional      | 0x0000
+------------------+
| Answer Records  | Variable
+------------------+
| Name           | "_gigi-dns._udp.local"
+------------------+
| Type           | 0x0010 (TXT)
+------------------+
| Class          | 0x0001 (IN)
+------------------+
| TTL            | 4 bytes
+------------------+
| RDLENGTH        | 2 bytes (data length)
+------------------+
| RDATA          | TXT record (encoded peer info)
+------------------+
```

## Constraints

### Size Limits
```rust
MAX_TXT_LENGTH = 255 bytes  // RFC 1035 limit
```

### Typical Sizes
- peer_id: ~50 bytes
- nickname: ~30 bytes
- addr: ~40 bytes
- caps: ~40 bytes
- meta: ~80 bytes
- Total: ~240 bytes (within limit)

## Network Parameters

### Multicast Addresses
- IPv4: 224.0.0.251:5353
- IPv6: FF02::FB:5353

### Default Timing
- TTL: 6 minutes (360 seconds)
- Query Interval: 5 minutes (300 seconds)

## Events

### Discovered
New peer discovered with complete information.

### Updated
Peer information changed (nickname, address, capabilities, metadata).

### Expired
Peer record expired (TTL passed without refresh).

### Offline
Peer stopped advertising (no longer seen).

## Security Considerations

- All information broadcast in clear text
- No authentication mechanism
- Anyone on network can see all peer information
- Validate peer identity after connection establishment
