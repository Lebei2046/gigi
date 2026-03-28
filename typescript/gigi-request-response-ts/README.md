# Gigi Request-Response Protocol (TypeScript)

A TypeScript implementation of the request-response protocol for js-libp2p, based on the rust-libp2p request-response protocol.

## Overview

This package provides a request-response protocol implementation for js-libp2p that is compatible with the rust-libp2p request-response protocol. It allows peers to send requests and receive responses over libp2p streams.

## Features

- Request-response protocol implementation
- JSON and CBOR codecs for message serialization
- Timeout handling
- Event-based architecture
- Support for multiple protocols
- TypeScript type safety

## Installation

```bash
pnpm add @gigi/request-response-ts
```

## Usage

### Basic Usage

```typescript
import { createLibp2p } from 'libp2p';
import { RequestResponse, JsonCodec } from '@gigi/request-response-ts';

// Define request and response types
interface MyRequest {
  type: string;
  data: any;
}

interface MyResponse {
  status: number;
  data: any;
}

// Create libp2p instance
const libp2p = await createLibp2p({
  // libp2p configuration
});

// Create codec
const codec = new JsonCodec<MyRequest, MyResponse>('/my-protocol/1.0.0');

// Create request-response instance
const requestResponse = new RequestResponse(libp2p, codec);

// Add event listener
requestResponse.onEvent((event) => {
  if (event.type === 'Message' && event.message.type === 'Request') {
    // Handle incoming request
    const { request, channel } = event.message;
    
    // Process request
    const response: MyResponse = {
      status: 200,
      data: { result: 'success' }
    };
    
    // Send response
    channel.send(response);
  } else if (event.type === 'Message' && event.message.type === 'Response') {
    // Handle incoming response
    const { response } = event.message;
    console.log('Received response:', response);
  }
});

// Send request to a peer
const peerId = /* get peer id */;
const request: MyRequest = {
  type: 'ping',
  data: { timestamp: Date.now() }
};

const response = await requestResponse.sendRequest(peerId, request);
console.log('Received response:', response);
```

### Using CBOR Codec

```typescript
import { CborCodec } from '@gigi/request-response-ts';

// Create CBOR codec
const codec = new CborCodec<MyRequest, MyResponse>('/my-protocol/1.0.0');

// Create request-response instance
const requestResponse = new RequestResponse(libp2p, codec);
```

## API

### RequestResponse Class

#### Constructor

```typescript
new RequestResponse(libp2p: Libp2p, codec: Codec<TRequest, TResponse, TProtocol>, config?: Partial<RequestResponseConfig>)
```

#### Methods

- `sendRequest(peerId: PeerId | string, request: TRequest): Promise<TResponse>` - Send a request to a peer and return the response
- `onEvent(listener: (event: RequestResponseEvent<TRequest, TResponse>) => void): () => void` - Add an event listener
- `close(): void` - Close the request-response instance and clean up resources

### Events

#### Message Event

Emitted when a request or response is received.

```typescript
{
  type: 'Message',
  peer: PeerId,
  connectionId: ConnectionId,
  message: {
    type: 'Request',
    requestId: InboundRequestId,
    request: TRequest,
    channel: ResponseChannel<TResponse>
  }
}

// or

{
  type: 'Message',
  peer: PeerId,
  connectionId: ConnectionId,
  message: {
    type: 'Response',
    requestId: OutboundRequestId,
    response: TResponse
  }
}
```

#### OutboundFailure Event

Emitted when an outbound request fails.

```typescript
{
  type: 'OutboundFailure',
  peer: PeerId,
  connectionId: ConnectionId,
  requestId: OutboundRequestId,
  error: OutboundFailure
}
```

#### InboundFailure Event

Emitted when an inbound request fails.

```typescript
{
  type: 'InboundFailure',
  peer: PeerId,
  connectionId: ConnectionId,
  requestId: InboundRequestId,
  error: InboundFailure
}
```

#### ResponseSent Event

Emitted when a response is sent for an inbound request.

```typescript
{
  type: 'ResponseSent',
  peer: PeerId,
  connectionId: ConnectionId,
  requestId: InboundRequestId
}
```

## Configuration

The `RequestResponseConfig` interface allows you to configure the request-response protocol:

```typescript
interface RequestResponseConfig {
  /**
   * Timeout for requests in milliseconds.
   */
  requestTimeout: number;
  
  /**
   * Maximum number of concurrent streams per connection.
   */
  maxConcurrentStreams: number;
}
```

Default values:

```typescript
{
  requestTimeout: 10000, // 10 seconds
  maxConcurrentStreams: 100
}
```

## Codecs

### JsonCodec

A codec that uses JSON for message serialization.

```typescript
new JsonCodec<TRequest, TResponse, TProtocol>(protocol: TProtocol)
```

### CborCodec

A codec that uses CBOR for message serialization, which is more efficient for binary data.

```typescript
new CborCodec<TRequest, TResponse, TProtocol>(protocol: TProtocol)
```

## License

MIT
