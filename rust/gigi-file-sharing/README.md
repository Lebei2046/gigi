# gigi-file-sharing

## Overview

File sharing management functionality for gigi. Provides a robust system for sharing files between peers in the Gigi P2P network.

### Features

- File sharing manager with chunked transfer support
- Support for both filesystem paths and URIs (content://, file://)
- Persistent storage via gigi-store
- File hash calculation (SHA256)
- Share code generation using BLAKE3

## Installation/Test

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
gigi-file-sharing = {
  path = "../gigi-file-sharing",
  version = "0.1.0"
}
```

### Testing

Run the tests to verify functionality:

```bash
cargo test
```
