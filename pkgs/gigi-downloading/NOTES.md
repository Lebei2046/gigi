# Gigi Downloading

A Rust library for peer-to-peer file-transfering using `libp2p`'s `request-response` protocol.

The `gigi-downloading` is designed to be integrated into gigi direct-messaging program for large-file sharing. 

## Features

- **File Transfer**: Transfer files between peers over the network
- **Chunked Transfer**: Large files are automatically split into chunks for efficient transfer
- **File Integrity**: SHA-256 hash verification ensures file integrity
- **Event-Driven**: Async event handling for transfer progress
- **Discovery**: Built-in file listing and metadata exchange
- **Secure**: Uses libp2p's noise protocol for encrypted communication

## Design

**The library(`lib.rs`) provides functions to implement the followings:**
 
- applying client/server pattern and supporting multi clients
- the server and the client support recovery of transfering
- the server shares files, records sharing info for recovery
- the server supports listing shared file info and revoking file sharing
- the server responses the client with the requesting chunk of data
- the client uses the sharing code to retrieve the file info
- the client designs the file meta data to store the downloading file
- the client supports stream transfer from server and displays the progress of downloading
- the client finish the last chunk of data, check the sum of the file, and rename the downloading file to the normal one

**Two examples: `server.rs` and `client.rs` to help debug the library**

Arguments for `server.rs`

- `--info-path`, the directory for recording shared file info
- `--files`, the space-seperated file path
- `--port`, the port for listening

Arguments for `client.rs`

- `--addr`, the server addr for connecting to
- `--code`, the shared code for file downloading
- `--output`, the directory for saving files

## Implementation

In this directory, create a rust library project, write the code in `lib.rs`, `examples/server.rs` and `examples/client.rs`.

Make sure the dependencies come from the workspace to gurantee the compatibility.

## References
- /home/lebei/dev/rust-libp2p/examples/file-sharing
- /home/lebei/dev/rust-libp2p/protocols/request-response