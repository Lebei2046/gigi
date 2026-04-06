# gigi-file-sharing

File sharing management functionality for gigi.

## Features

- File sharing manager with chunked transfer support
- Support for both filesystem paths and URIs (content://, file://)
- Persistent storage via gigi-store
- File hash calculation (SHA256)
- Share code generation using BLAKE3

## Usage

```rust
use gigi_file_sharing::{FileSharingManager, CHUNK_SIZE};

let mut manager = FileSharingManager::new();

// Share a file
let share_code = manager.share_file(&PathBuf::from("my_file.txt")).await?;

// List shared files
let files = manager.list_shared_files();

// Unshare a file
manager.unshare_file(&share_code)?;
```
