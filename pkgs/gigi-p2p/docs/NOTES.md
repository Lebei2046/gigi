# Gigi P2P

A comprehensive peer-to-peer networking library that combines direct messaging, group messaging, and file transfer capabilities into a unified interface.

## Tasks

We have developed `gigi-mdns` for peer discovery with nickname, `gigi-dm` for direct messaging, `gigi-gossip` for group messaging with nickname, and `gigi-downloading` for file sharing between two peers. With those experiences, we have accumulated a solid understanding of P2P networking concepts and best practices. Now, we are working on integrating these functions into a unified P2P library.

**The library(`lib.rs`)  implementing the followings:**

- **Auto Discovery**: Automatically discover peers on the network via mDNS
- **Nickname Exchange**: Exchange nicknames between peers for easy identification(by request-response protocol)
- **Direct Messaging**: One-to-one communication between peers(by request-response protocol)
- **Group Messaging**: Many-to-many communication via Gossipsub(by gossipsub protocol)
- **File Transfer**: Chunked largefile sharing between peers(by request-response protocol)
- **Event-driven**: Unified event system for all P2P operations
- **Error Handling**: Error handling for all P2P operations
- **Testing**: Testing for all P2P operations
- **Documentation**: Documentation for all P2P operations

**One example: `chat.rs` to help debug the library**

Arguments for `chat.rs`

- `--port`, the port for listening
- `--nickname`, the nickname for the peer
- `--shared`, the file name for recording info of shared files
- `--output`, the directory for saving files

Commands to implement

- `peers`, list all peers discovered via mDNS including their nicknames and addresses
- `connect <nickname>`, connect to a peer with the given nickname
- `send <nickname> <message>`, send a message to the peer with the given nickname
- `send-image <nickname> <image-path>`, send an image to the peer with the given nickname
- `join <group>`, join a group with the given name
- `leave <group>`, leave a group with the given name
- `send-group <group> <message>`, send a message to the group with the given name
- `send-group-image <group> <image-path>`, send an image to the group with the given name
- `share <file-path>`, record the file path for sharing
- `list-files`, list all files shared by the peer
- `download <nickname> <code>`, download a file by shared code from the peer with the given nickname
  
**Don't use `gigi-mdns`, `gigi-dm`, `gigi-gossip`, `gigi-downloading` directly, create a new interface in `lib.rs` instead.**

**Don't use `tui` in `chat.rs`**

**Make sure the dependencies come from the workspace to gurantee the compatibility.**
# Modified NOTES.md
