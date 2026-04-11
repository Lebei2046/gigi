# Tauri Plugin gigi-p2p

## Overview

A Tauri plugin for integrating Gigi P2P functionality into desktop and mobile applications. This plugin provides a bridge between Tauri applications and the Gigi P2P network, enabling secure, decentralized communication.

### Features

- P2P peer discovery and communication
- Group messaging support
- File sharing capabilities
- Cross-platform compatibility (desktop and mobile)
- Seamless integration with Tauri applications

## Installation/Test

### Installation

Add the plugin to your Tauri project:

```bash
cargo add tauri-plugin-gigi
```

Or add it to your `Cargo.toml`:

```toml
[dependencies]
tauri-plugin-gigi = {
  path = "../tauri-plugin-gigi",
  version = "0.1.0"
}
```

### Testing

Run the tests to verify functionality:

```bash
cargo test
```
