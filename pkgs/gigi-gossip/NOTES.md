# Gigi Gossip

A Rust library for group messaging using `libp2p`'s `gossipsub` protocol.

## Design

**The library(`lib.rs`) provides functions to implement the followings:**

- peers listen on a random port and accepts incoming connections
- peers use `gigi-mdns` to discover other peers
- peers subscribe to the same topic to send and receive messages
- the messages support text, images
  
**One example: `chat.rs` to help debug the library**

Arguments for `chat.rs`

- `--topic`, the topic for group messaging
- `--nickname`, the nickname for the peer

## Implementation

✅ **COMPLETED** - In this directory, created a rust library project with:

- `lib.rs` - Main library implementation with:
  - `GossipBehaviour` combining gossipsub and mDNS
  - `GossipChat` struct with manual swarm management
  - Factory methods: `with_swarm()`, `create_behaviour()`
  - Event-driven architecture with `tokio::select!`
  - Support for text and image messages

- `examples/chat.rs` - Complete chat example demonstrating:
  - Manual event loop with tokio::select!
  - Peer discovery via mDNS
  - Message sending/receiving
  - Image file sharing
  - Command-line interface with CLI arguments

- `Cargo.toml` - Package configuration with workspace dependencies
- `README.md` - Documentation with API reference and usage examples

The implementation follows the same refactoring pattern as gigi-dm, gigi-mdns, and gigi-downloading:
- Removed blocking methods (new(), run(), poll_events())
- Made swarm public for manual event handling
- Added factory methods for external swarm creation
- Implemented manual event loops with tokio::select!

All dependencies come from the workspace to guarantee compatibility.

## References

- [Libp2p Chat Example](https://github.com/libp2p/rust-libp2p/tree/master/examples/chat)
- [Libp2p Gossipsub Protocol](https://github.com/libp2p/rust-libp2p/tree/master/protocols/gossipsub)