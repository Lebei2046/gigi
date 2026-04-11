# Gigi Request-Response Protocol (TypeScript)

## Overview

A TypeScript implementation of the request-response protocol for js-libp2p, based on the rust-libp2p request-response protocol. It allows peers to send requests and receive responses over libp2p streams with support for multiple codecs and event-based architecture.

### Features

- Request-response protocol implementation
- JSON and CBOR codecs for message serialization
- Timeout handling
- Event-based architecture
- Support for multiple protocols
- TypeScript type safety

## Installation/Test

### Installation

```bash
pnpm add @gigi/request-response
```

### Testing

```bash
pnpm test
```

## License

MIT
