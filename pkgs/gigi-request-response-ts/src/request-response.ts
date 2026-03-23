import type { Libp2p } from '@libp2p/interface-libp2p';
import type { PeerId } from '@libp2p/interface-peer-id';
import type { ConnectionId } from '@libp2p/interface-connection';
import type { Stream } from '@libp2p/interface-connection';
import type { Multiaddr } from '@multiformats/multiaddr';
import { Codec, ResponseChannel, InboundRequestId, OutboundRequestId, RequestResponseEvent, ProtocolSupport, RequestResponseConfig, defaultConfig } from './types.js';

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
  private pendingOutboundRequests = new Map<string, { peerId: PeerId; connectionId: ConnectionId; timeout: NodeJS.Timeout }>();
  private pendingInboundRequests = new Map<string, { peerId: PeerId; connectionId: ConnectionId; timeout: NodeJS.Timeout }>();
  private eventListeners: ((event: RequestResponseEvent<TRequest, TResponse>) => void)[] = [];

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
    
    this.libp2p.handle(protocol, async (data: IncomingStreamData) => {
      await this.handleIncomingStream(data);
    });
  }

  /**
   * Handle an incoming stream.
   */
  private async handleIncomingStream(data: IncomingStreamData): Promise<void> {
    const { stream, connection } = data;
    const peerId = connection.remotePeer;
    const connectionId = connection.id;

    try {
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
        timeout
      });

      // Create response channel
      const channel = new ResponseChannel<TResponse>((response) => {
        this.sendResponse(stream, response);
        this.clearInboundRequest(requestId);
        this.emitEvent({
          type: 'ResponseSent',
          peer: peerId,
          connectionId,
          requestId
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
          channel
        }
      });
    } catch (error) {
      console.error('Error handling incoming stream:', error);
      // TODO: Emit inbound failure event
    } finally {
      // Stream will be closed by the sender
    }
  }

  /**
   * Send a response through the stream.
   */
  private async sendResponse(stream: Stream, response: TResponse): Promise<void> {
    try {
      const encodedResponse = this.codec.encodeResponse(response);
      await stream.sink([encodedResponse]);
    } catch (error) {
      console.error('Error sending response:', error);
    } finally {
      await stream.close();
    }
  }

  /**
   * Handle inbound request timeout.
   */
  private handleInboundTimeout(peerId: PeerId, connectionId: ConnectionId, requestId: InboundRequestId): void {
    if (this.pendingInboundRequests.has(requestId.toString())) {
      this.clearInboundRequest(requestId);
      this.emitEvent({
        type: 'InboundFailure',
        peer: peerId,
        connectionId,
        requestId,
        error: 'Timeout' as any
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
   * @param peerId - The peer to send the request to
   * @param request - The request to send
   * @returns The request ID
   */
  async sendRequest(peerId: PeerId, request: TRequest): Promise<OutboundRequestId> {
    const requestId = new OutboundRequestId(this.nextOutboundRequestId++);
    const protocol = this.codec.getProtocol();

    try {
      // Dial the protocol
      const stream = await this.libp2p.dialProtocol(peerId, protocol);
      const connectionId = stream.connection.id;

      // Set up timeout
      const timeout = setTimeout(() => {
        this.handleOutboundTimeout(peerId, connectionId, requestId);
      }, this.config.requestTimeout);

      // Store pending outbound request
      this.pendingOutboundRequests.set(requestId.toString(), {
        peerId,
        connectionId,
        timeout
      });

      // Send request
      const encodedRequest = this.codec.encodeRequest(request);
      await stream.sink([encodedRequest]);

      // Wait for response
      const responseData = await this.readStream(stream);
      const response = this.codec.decodeResponse(responseData);

      // Clear timeout and pending request
      this.clearOutboundRequest(requestId);

      // Emit response event
      this.emitEvent({
        type: 'Message',
        peer: peerId,
        connectionId,
        message: {
          type: 'Response',
          requestId,
          response
        }
      });

      return requestId;
    } catch (error) {
      // Clear pending request
      this.clearOutboundRequest(requestId);
      
      // Emit failure event
      this.emitEvent({
        type: 'OutboundFailure',
        peer: peerId,
        connectionId: 'unknown' as ConnectionId,
        requestId,
        error: 'DialFailure' as any
      });

      throw error;
    }
  }

  /**
   * Handle outbound request timeout.
   */
  private handleOutboundTimeout(peerId: PeerId, connectionId: ConnectionId, requestId: OutboundRequestId): void {
    if (this.pendingOutboundRequests.has(requestId.toString())) {
      this.clearOutboundRequest(requestId);
      this.emitEvent({
        type: 'OutboundFailure',
        peer: peerId,
        connectionId,
        requestId,
        error: 'Timeout' as any
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
    for await (const chunk of stream.source) {
      chunks.push(chunk);
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
  onEvent(listener: (event: RequestResponseEvent<TRequest, TResponse>) => void): () => void {
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
