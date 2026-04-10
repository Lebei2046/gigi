// Using any types to avoid module resolution issues
// These types will be provided by the libp2p instance at runtime
type Libp2p = any;
type PeerId = any;
type ConnectionId = any;
type Stream = any;
import {
  Codec,
  ResponseChannel,
  InboundRequestId,
  OutboundRequestId,
  RequestResponseEvent,
  RequestResponseConfig,
  defaultConfig,
  InboundFailure,
  OutboundFailure,
} from './types';

// Import peerIdFromString to convert string peer IDs to PeerId objects
import { peerIdFromString } from '@libp2p/peer-id';

// Incoming stream data interface
export interface IncomingStreamData {
  stream: Stream;
  connection: {
    id: ConnectionId;
    remotePeer: PeerId;
  };
}

/**
 * A request-response protocol implementation for js-libp2p.
 * Based on the rust-libp2p request-response protocol.
 */
export class RequestResponse<TRequest, TResponse, TProtocol extends string> {
  private libp2p: Libp2p;
  private codec: Codec<TRequest, TResponse, TProtocol>;
  private config: RequestResponseConfig;
  private nextOutboundRequestId = 1;
  private nextInboundRequestId = 1;
  private pendingOutboundRequests = new Map<
    string,
    { peerId: PeerId; connectionId: ConnectionId; timeout: NodeJS.Timeout }
  >();
  private pendingInboundRequests = new Map<
    string,
    { peerId: PeerId; connectionId: ConnectionId; timeout: NodeJS.Timeout }
  >();
  private eventListeners: ((
    event: RequestResponseEvent<TRequest, TResponse>
  ) => void)[] = [];

  /**
   * Create a new RequestResponse instance.
   * @param libp2p - The libp2p instance
   * @param codec - The codec for encoding/decoding messages
   * @param config - Optional configuration
   */
  constructor(
    libp2p: Libp2p,
    codec: Codec<TRequest, TResponse, TProtocol>,
    config: Partial<RequestResponseConfig> = {}
  ) {
    this.libp2p = libp2p;
    this.codec = codec;
    this.config = { ...defaultConfig, ...config };
    this.registerProtocolHandler();
  }

  /**
   * Register the protocol handler with libp2p.
   */
  private registerProtocolHandler(): void {
    const protocol = this.codec.getProtocol();

    this.libp2p.handle(protocol, async (stream: any) => {
      console.log(
        `[RequestResponse] Received stream: ${JSON.stringify(stream)}`
      );

      // Get connection from stream
      let connection = stream.connection;

      // Try different ways to get connection from stream
      if (!connection) {
        connection = stream.conn || stream._connection;
      }

      console.log(
        `[RequestResponse] Using stream: ${stream}, connection: ${connection}`
      );

      // Ensure we have at least a stream
      if (stream) {
        await this.handleIncomingStream({ stream, connection });
      } else {
        console.error('[RequestResponse] Missing stream in incoming data');
      }
    });
  }

  /**
   * Handle an incoming stream.
   */
  private async handleIncomingStream(data: IncomingStreamData): Promise<void> {
    const { stream, connection } = data;
    console.log(
      `[RequestResponse] Incoming stream from connection: ${JSON.stringify(connection)}`
    );

    // Try to get peerId from different possible locations
    let peerId: any = 'unknown';
    if (connection) {
      peerId = connection.remotePeer || 'unknown';
    }

    // Try to get peerId from stream if not found in connection
    if (peerId === 'unknown' && stream) {
      peerId =
        stream.remotePeer || stream.peerId || stream._remotePeer || 'unknown';
    }

    const connectionId = connection?.id || stream?.id || 'unknown';

    try {
      // Log stream details for debugging
      console.log('[RequestResponse] Stream object:', stream);
      console.log(
        '[RequestResponse] Stream constructor:',
        stream.constructor.name
      );
      console.log('[RequestResponse] Stream has sink:', typeof stream.sink);
      console.log('[RequestResponse] Stream has source:', typeof stream.source);
      console.log(
        '[RequestResponse] Stream has asyncIterator:',
        typeof stream[Symbol.asyncIterator]
      );
      console.log(
        '[RequestResponse] Stream source has asyncIterator:',
        typeof stream.source?.[Symbol.asyncIterator]
      );

      // Read request from stream
      const requestData = await this.readStream(stream);
      const request = this.codec.decodeRequest(requestData);

      // Generate inbound request ID
      const requestId = new InboundRequestId(this.nextInboundRequestId++);

      // Set up timeout
      const timeout = setTimeout(() => {
        this.handleInboundTimeout(peerId, connectionId, requestId);
      }, this.config.requestTimeout);

      // Store pending inbound request
      this.pendingInboundRequests.set(requestId.toString(), {
        peerId,
        connectionId,
        timeout,
      });

      // Create response channel
      const channel = new ResponseChannel<TResponse>((response) => {
        this.sendResponse(stream, response);
        this.clearInboundRequest(requestId);
        this.emitEvent({
          type: 'ResponseSent',
          peer: peerId,
          connectionId,
          requestId,
        });
      });

      // Emit request event
      this.emitEvent({
        type: 'Message',
        peer: peerId,
        connectionId,
        message: {
          type: 'Request',
          requestId,
          request,
          channel,
        },
      });
    } catch (error) {
      console.error('Error handling incoming stream:', error);
      this.emitEvent({
        type: 'InboundFailure',
        peer: peerId,
        connectionId,
        requestId: new InboundRequestId(0),
        error: InboundFailure.Timeout,
      });
    } finally {
      // Stream will be closed by the sender
    }
  }

  /**
   * Send a response through the stream.
   */
  private async sendResponse(
    stream: Stream,
    response: TResponse
  ): Promise<void> {
    try {
      const encodedResponse = this.codec.encodeResponse(response);

      // Check if stream has send method (YamuxStream)
      if (typeof (stream as any).send === 'function') {
        console.log(`[RequestResponse] Using send method for response`);
        await (stream as any).send(encodedResponse);
        // End the stream
        if (typeof (stream as any).close === 'function') {
          await (stream as any).close();
        }
      } else if (stream.sink) {
        console.log(`[RequestResponse] Using sink method for response`);
        await stream.sink([encodedResponse]);
      } else if (typeof stream.write === 'function') {
        // Try to use write method if available
        console.log(`[RequestResponse] Using write method for response`);
        await new Promise<void>((resolve, reject) => {
          stream.write(encodedResponse, (error: Error | null) => {
            if (error) {
              reject(error);
            } else {
              if (typeof stream.end === 'function') {
                stream.end(resolve);
              } else {
                resolve();
              }
            }
          });
        });
      } else if (
        typeof stream.writable === 'object' &&
        stream.writable.writable
      ) {
        // Try Web Streams API
        console.log(`[RequestResponse] Using Web Streams API for response`);
        const writer = stream.writable.getWriter();
        await writer.write(encodedResponse);
        await writer.close();
      } else {
        console.error(
          `[RequestResponse] No valid write method found for stream`
        );
        throw new Error('No write method available');
      }
    } catch (error) {
      console.error('Error sending response:', error);
    } finally {
      if (typeof stream.close === 'function') {
        try {
          await stream.close();
        } catch (error) {
          console.error('Error closing stream:', error);
        }
      }
    }
  }

  /**
   * Handle inbound request timeout.
   */
  private handleInboundTimeout(
    peerId: PeerId,
    connectionId: ConnectionId,
    requestId: InboundRequestId
  ): void {
    if (this.pendingInboundRequests.has(requestId.toString())) {
      this.clearInboundRequest(requestId);
      this.emitEvent({
        type: 'InboundFailure',
        peer: peerId,
        connectionId,
        requestId,
        error: InboundFailure.Timeout,
      });
    }
  }

  /**
   * Clear an inbound request.
   */
  private clearInboundRequest(requestId: InboundRequestId): void {
    const key = requestId.toString();
    const request = this.pendingInboundRequests.get(key);
    if (request) {
      clearTimeout(request.timeout);
      this.pendingInboundRequests.delete(key);
    }
  }

  /**
   * Send a request to a peer.
   * @param peerId - The peer to send the request to (can be a PeerId object, string peer ID, or multiaddr string)
   * @param request - The request to send
   * @returns The response from the peer
   */
  async sendRequest(
    peerId: PeerId | string,
    request: TRequest
  ): Promise<TResponse> {
    const requestId = new OutboundRequestId(this.nextOutboundRequestId++);
    const protocol = this.codec.getProtocol();

    // Convert peerId to appropriate format for dialProtocol
    let dialTarget = peerId;

    // Extract peer ID from multiaddr if needed
    if (typeof dialTarget === 'string') {
      if (dialTarget.startsWith('/')) {
        // Extract peer ID from multiaddr string
        const match = dialTarget.match(/\/p2p\/(\w+)/);
        if (match && match[1]) {
          dialTarget = match[1];
        }
      }
      // Convert string peer ID to PeerId object only if it looks like a valid peer ID
      // Skip conversion for mock peer IDs used in tests
      if (dialTarget.match(/^[1Q][a-zA-Z0-9]+$/)) {
        try {
          dialTarget = peerIdFromString(dialTarget);
        } catch (error) {
          console.log(`[RequestResponse] Failed to parse peer ID:`, error);
          throw error;
        }
      }
    }

    try {
      console.log(
        `[RequestResponse] Dialing protocol ${protocol} to peer ${dialTarget}`
      );

      // Try to dial using the peer ID or multiaddr
      let stream;
      try {
        // Pass the peer ID or multiaddr directly to dialProtocol
        stream = await this.libp2p.dialProtocol(dialTarget, protocol);
      } catch (error) {
        console.log(`[RequestResponse] Dial attempt failed:`, error);
        throw error;
      }
      const connectionId = stream.connection?.id || 'unknown';

      console.log(
        `[RequestResponse] Got stream with connection ID: ${connectionId}`
      );

      // Set up timeout
      const timeout = setTimeout(() => {
        this.handleOutboundTimeout(dialTarget, connectionId, requestId);
      }, this.config.requestTimeout);

      // Store pending outbound request
      this.pendingOutboundRequests.set(requestId.toString(), {
        peerId: dialTarget,
        connectionId,
        timeout,
      });

      // Send request
      const encodedRequest = this.codec.encodeRequest(request);
      console.log(
        `[RequestResponse] Sending request: ${JSON.stringify(request)}`
      );

      // Log stream object to understand its structure
      console.log(`[RequestResponse] Stream object:`, stream);
      console.log(
        `[RequestResponse] Stream constructor:`,
        stream.constructor.name
      );
      console.log(`[RequestResponse] Stream has sink:`, typeof stream.sink);
      console.log(`[RequestResponse] Stream has source:`, typeof stream.source);
      console.log(`[RequestResponse] Stream has write:`, typeof stream.write);
      console.log(`[RequestResponse] Stream has send:`, typeof stream.send);
      console.log(`[RequestResponse] Stream has push:`, typeof stream.push);
      console.log(`[RequestResponse] Stream has end:`, typeof stream.end);
      console.log(`[RequestResponse] Stream has close:`, typeof stream.close);

      // Try to use the correct API for Libp2p streams
      if (typeof stream.send === 'function') {
        // Try to use send method if available (YamuxStream)
        console.log(`[RequestResponse] Using send method`);
        // Convert Uint8Array to Buffer before sending
        const buffer = Buffer.from(encodedRequest);
        console.log(
          `[RequestResponse] Sending buffer length: ${buffer.length}`
        );
        // Send the data and wait for it to complete
        await stream.send(buffer);
        console.log(`[RequestResponse] Data sent successfully`);
        // Close only the write side of the stream to signal we're done sending
        // This allows the other side to know when to stop reading, but leaves the read side open
        if (typeof stream.sendCloseWrite === 'function') {
          console.log(`[RequestResponse] Closing write side of stream`);
          await stream.sendCloseWrite();
        } else if (typeof stream.end === 'function') {
          // Fallback to end if sendCloseWrite is not available
          console.log(`[RequestResponse] Ending stream write`);
          await new Promise<void>((resolve, reject) => {
            stream.end((error: Error | null) => {
              if (error) {
                reject(error);
              } else {
                resolve();
              }
            });
          });
        }
      } else if (typeof stream.write === 'function') {
        // Try to use write method if available
        console.log(`[RequestResponse] Using write method`);
        await new Promise<void>((resolve, reject) => {
          stream.write(encodedRequest, (error: Error | null) => {
            if (error) {
              reject(error);
            } else {
              // Don't end the stream yet - we need to read the response
              resolve();
            }
          });
        });
      } else if (typeof stream.sink === 'function') {
        console.log(`[RequestResponse] Using sink method with array`);
        // Pass an array of Uint8Array chunks directly to sink
        await stream.sink([encodedRequest]);
      } else if (
        typeof stream.writable === 'object' &&
        stream.writable.writable
      ) {
        // Try Web Streams API
        console.log(`[RequestResponse] Using Web Streams API`);
        const writer = stream.writable.getWriter();
        await writer.write(encodedRequest);
        // Close the writer to signal we're done sending, but leave the readable side open
        await writer.close();
      } else {
        // Last resort - throw error
        throw new Error('No write method available');
      }

      console.log(`[RequestResponse] Request sent`);

      // Wait for response
      console.log(`[RequestResponse] Waiting for response`);
      const responseData = await this.readStream(stream);
      console.log(
        `[RequestResponse] Received response data: ${responseData.length} bytes`
      );
      const response = this.codec.decodeResponse(responseData);
      // Avoid logging large chunk data
      const responseObj = response as any;
      if (responseObj.type === 'chunk' && responseObj.chunk) {
        const responseWithoutChunk = { ...responseObj };
        delete responseWithoutChunk.chunk;
        console.log(
          `[RequestResponse] Decoded response: ${JSON.stringify(responseWithoutChunk)} (chunk data omitted)`
        );
      } else {
        console.log(
          `[RequestResponse] Decoded response: ${JSON.stringify(response)}`
        );
      }

      // Clear timeout and pending request
      this.clearOutboundRequest(requestId);

      // Emit response event
      this.emitEvent({
        type: 'Message',
        peer: dialTarget,
        connectionId,
        message: {
          type: 'Response',
          requestId,
          response,
        },
      });

      // Close the stream after processing the response
      if (typeof stream.close === 'function') {
        console.log(`[RequestResponse] Closing stream`);
        try {
          await stream.close();
        } catch (error) {
          console.error(`[RequestResponse] Error closing stream:`, error);
        }
      }

      return response;
    } catch (error) {
      // Clear pending request
      this.clearOutboundRequest(requestId);

      console.error(`[RequestResponse] Error in sendRequest:`, error);

      // Emit failure event
      this.emitEvent({
        type: 'OutboundFailure',
        peer: dialTarget,
        connectionId: 'unknown' as ConnectionId,
        requestId,
        error: OutboundFailure.DialFailure,
      });

      throw error;
    }
  }

  /**
   * Handle outbound request timeout.
   */
  private handleOutboundTimeout(
    peerId: PeerId | string,
    connectionId: ConnectionId,
    requestId: OutboundRequestId
  ): void {
    if (this.pendingOutboundRequests.has(requestId.toString())) {
      this.clearOutboundRequest(requestId);
      this.emitEvent({
        type: 'OutboundFailure',
        peer: peerId,
        connectionId,
        requestId,
        error: OutboundFailure.Timeout,
      });
    }
  }

  /**
   * Clear an outbound request.
   */
  private clearOutboundRequest(requestId: OutboundRequestId): void {
    const key = requestId.toString();
    const request = this.pendingOutboundRequests.get(key);
    if (request) {
      clearTimeout(request.timeout);
      this.pendingOutboundRequests.delete(key);
    }
  }

  /**
   * Read data from a stream.
   */
  private async readStream(stream: Stream): Promise<Uint8Array> {
    const chunks: Uint8Array[] = [];

    try {
      // Handle nested stream objects (common in some libp2p implementations)
      if (stream.stream) {
        stream = stream.stream;
      }

      // Try for streams with async iterator first (YamuxStream supports this)
      if (typeof stream[Symbol.asyncIterator] === 'function') {
        for await (const chunk of stream) {
          // Handle different chunk types
          if (Buffer.isBuffer(chunk)) {
            chunks.push(new Uint8Array(chunk));
          } else if (chunk instanceof Uint8Array) {
            chunks.push(chunk);
          } else if (Array.isArray(chunk)) {
            chunks.push(new Uint8Array(chunk));
          } else if (chunk && typeof chunk.toUint8Array === 'function') {
            // Handle Uint8ArrayList
            chunks.push(chunk.toUint8Array());
          } else if (chunk && typeof chunk.slice === 'function') {
            // Try to slice as a last resort
            chunks.push(new Uint8Array(chunk.slice(0)));
          } else {
            // Try to convert to Uint8Array anyway
            try {
              chunks.push(new Uint8Array(chunk));
            } catch (error) {
              console.error('[RequestResponse] Error converting chunk:', error);
            }
          }
        }
      } else if (typeof (stream as any).receive === 'function') {
        // Try YamuxStream receive method
        let chunk;
        while ((chunk = await (stream as any).receive()) !== null) {
          chunks.push(new Uint8Array(chunk));
        }
      } else if (typeof (stream as any).read === 'function') {
        // Try stream.read() method
        let chunk;
        while ((chunk = (stream as any).read()) !== null) {
          chunks.push(new Uint8Array(chunk));
        }
      }
      // Try the most common duplex stream API
      else if (stream.source) {
        for await (const chunk of stream.source) {
          chunks.push(chunk);
        }
      }
      // Try Web Streams API
      else if (stream.readable) {
        const reader = stream.readable.getReader();
        while (true) {
          const { done, value } = await reader.read();
          if (done) {
            break;
          }
          if (value) {
            chunks.push(value);
          }
        }
      }
      // Try event emitter pattern
      else if (typeof stream.on === 'function') {
        await new Promise<void>((resolve, reject) => {
          stream.on('data', (chunk: Buffer) => {
            chunks.push(new Uint8Array(chunk));
          });
          stream.on('end', () => {
            resolve();
          });
          stream.on('error', (error: Error) => {
            reject(error);
          });
        });
      }
      // Try to get data from stream directly
      else {
        // For some stream types, the data might be available immediately
        throw new Error('Unsupported stream type');
      }
    } catch (error) {
      console.error('[RequestResponse] Error reading stream:', error);
      throw error;
    }

    if (chunks.length === 0) {
      return new Uint8Array(0);
    }

    const totalLength = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
    const data = new Uint8Array(totalLength);
    let offset = 0;

    for (const chunk of chunks) {
      data.set(chunk, offset);
      offset += chunk.length;
    }

    return data;
  }

  /**
   * Add an event listener.
   * @param listener - The event listener
   * @returns A function to remove the listener
   */
  onEvent(
    listener: (event: RequestResponseEvent<TRequest, TResponse>) => void
  ): () => void {
    this.eventListeners.push(listener);
    return () => {
      const index = this.eventListeners.indexOf(listener);
      if (index > -1) {
        this.eventListeners.splice(index, 1);
      }
    };
  }

  /**
   * Emit an event to all listeners.
   */
  private emitEvent(event: RequestResponseEvent<TRequest, TResponse>): void {
    for (const listener of this.eventListeners) {
      try {
        listener(event);
      } catch (error) {
        console.error('Error in event listener:', error);
      }
    }
  }

  /**
   * Close the request-response instance and clean up resources.
   */
  close(): void {
    // Clear all timeouts
    for (const request of this.pendingOutboundRequests.values()) {
      clearTimeout(request.timeout);
    }
    for (const request of this.pendingInboundRequests.values()) {
      clearTimeout(request.timeout);
    }

    // Clear pending requests
    this.pendingOutboundRequests.clear();
    this.pendingInboundRequests.clear();

    // Remove event listeners
    this.eventListeners = [];
  }
}
