# Gigi Direct-Messaging

A Rust library for peer-to-peer messaging using `libp2p`'s `request-response` protocol.

## Design

**The library(`lib.rs`) provides functions to implement the followings:**

- one peer listens on a port and accepts incoming connections
- one peer connects to another peer and sends messages
- the messages support text, images and larg-file sharing
  
**One example: `chat.rs` to help debug the library**

Arguments for `chat.rs`

- `--port`, the port for listening
- `--addr`, the peer addr for connecting to

## Implementation

In this directory, create a rust library project, write the code in `lib.rs`, and `examples/chat.rs`.

Make sure the dependencies come from the workspace to gurantee the compatibility.

## References
- /home/lebei/dev/rust-libp2p/examples/chat
- /home/lebei/dev/rust-libp2p/protocols/request-response