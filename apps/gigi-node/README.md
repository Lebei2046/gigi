# Gigi Node

A standalone P2P network node for the Gigi messaging system. Supports bootstrap, relay, and full node modes for enabling cross-network mobile communication.

## Features

- **Bootstrap Mode**: Provides DHT entry points for new peers
- **Relay Mode**: Enables NAT traversal for mobile devices behind routers
- **Full Mode**: Combined bootstrap and relay capabilities
- **Kademlia DHT**: Distributed peer discovery
- **GossipSub**: Pub/sub messaging support
- **Circuit Relay v2**: NAT traversal for P2P connections
- **Dual Transport**: TCP and QUIC support

## Usage

### Build

```bash
cargo build --release
```

### Run as Bootstrap Node

```bash
./target/release/gigi-node \
  --mode bootstrap \
  --listen /ip4/0.0.0.0/tcp/4001 \
  --listen /ip4/0.0.0.0/udp/4001/quic-v1 \
  --external /ip4/203.0.113.10/tcp/4001 \
  --identity /data/bootstrap.key
```

### Run as Relay Node

```bash
./target/release/gigi-node \
  --mode relay \
  --listen /ip4/0.0.0.0/tcp/4003 \
  --listen /ip4/0.0.0.0/udp/4003/quic-v1 \
  --external /ip4/203.0.113.12/tcp/4003 \
  --bootstrap /ip4/203.0.113.10/tcp/4001/p2p/12D3KooW... \
  --identity /data/relay.key
```

### Run as Full Node

```bash
./target/release/gigi-node \
  --mode full \
  --listen /ip4/0.0.0.0/tcp/4002 \
  --listen /ip4/0.0.0.0/udp/4002/quic-v1 \
  --external /ip4/203.0.113.11/tcp/4002 \
  --bootstrap /ip4/203.0.113.10/tcp/4001/p2p/12D3KooW... \
  --identity /data/fullnode.key
```

## Command Line Options

| Option | Description | Example |
|--------|-------------|---------|
| `--mode` | Node mode: bootstrap, relay, or full | `--mode bootstrap` |
| `--listen` | Listen addresses (can specify multiple) | `--listen /ip4/0.0.0.0/tcp/4001` |
| `--external` | External addresses to advertise | `--external /ip4/203.0.113.10/tcp/4001` |
| `--bootstrap` | Bootstrap peer addresses | `--bootstrap /ip4/203.0.113.10/tcp/4001/p2p/12D3KooW...` |
| `--identity` | Path to identity key file | `--identity /data/node.key` |
| `--topics` | Topics to subscribe to (relay/full mode) | `--topics gigi-general` |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Gigi Node                            │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │ Kademlia │  │GossipSub │  │ Identify │  │  Relay   │    │
│  │   DHT    │  │ Pub/Sub  │  │ Protocol │  │ Circuit  │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
├─────────────────────────────────────────────────────────────┤
│              TCP + QUIC + Noise + Yamux                     │
└─────────────────────────────────────────────────────────────┘
```

## Deployment

See the Docker deployment configuration in the project root for cloud deployment.

## License

Same as Gigi project
