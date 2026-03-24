// Using any types to avoid module resolution issues
// These types will be provided by the libp2p instance at runtime
type PeerId = any;
type ConnectionId = any;

/**
 * A response channel for sending a response to an inbound request.
 */
export class ResponseChannel<TResponse> {
  private sender: ((response: TResponse) => void) | null = null;
  private response: TResponse | null = null;
  private resolved = false;

  constructor(sender: (response: TResponse) => void) {
    this.sender = sender;
  }

  /**
   * Checks whether the response channel is still open.
   */
  isOpen(): boolean {
    return !this.resolved && this.sender !== null;
  }

  /**
   * Send a response through the channel.
   */
  send(response: TResponse): boolean {
    if (!this.isOpen()) {
      return false;
    }

    this.sender!(response);
    this.response = response;
    this.resolved = true;
    this.sender = null;
    return true;
  }

  /**
   * Get the response if it was sent.
   */
  getResponse(): TResponse | null {
    return this.response;
  }
}

/**
 * The ID of an inbound request.
 */
export class InboundRequestId {
  constructor(public readonly value: number) {}

  toString(): string {
    return this.value.toString();
  }

  equals(other: InboundRequestId): boolean {
    return this.value === other.value;
  }

  hashCode(): number {
    return this.value;
  }
}

/**
 * The ID of an outbound request.
 */
export class OutboundRequestId {
  constructor(public readonly value: number) {}

  toString(): string {
    return this.value.toString();
  }

  equals(other: OutboundRequestId): boolean {
    return this.value === other.value;
  }

  hashCode(): number {
    return this.value;
  }
}

/**
 * Possible failures for outbound requests.
 */
export enum OutboundFailure {
  DialFailure = 'DialFailure',
  Timeout = 'Timeout',
  ConnectionClosed = 'ConnectionClosed',
  UnsupportedProtocols = 'UnsupportedProtocols',
  Io = 'Io'
}

/**
 * Possible failures for inbound requests.
 */
export enum InboundFailure {
  Timeout = 'Timeout',
  ConnectionClosed = 'ConnectionClosed',
  UnsupportedProtocols = 'UnsupportedProtocols',
  ResponseOmission = 'ResponseOmission',
  Io = 'Io'
}

/**
 * An inbound request or response message.
 */
export type Message<TRequest, TResponse> = 
  | {
      type: 'Request';
      requestId: InboundRequestId;
      request: TRequest;
      channel: ResponseChannel<TResponse>;
    }
  | {
      type: 'Response';
      requestId: OutboundRequestId;
      response: TResponse;
    };

/**
 * Events emitted by the request-response behaviour.
 */
export type RequestResponseEvent<TRequest, TResponse> = 
  | {
      type: 'Message';
      peer: PeerId;
      connectionId: ConnectionId;
      message: Message<TRequest, TResponse>;
    }
  | {
      type: 'OutboundFailure';
      peer: PeerId;
      connectionId: ConnectionId;
      requestId: OutboundRequestId;
      error: OutboundFailure;
    }
  | {
      type: 'InboundFailure';
      peer: PeerId;
      connectionId: ConnectionId;
      requestId: InboundRequestId;
      error: InboundFailure;
    }
  | {
      type: 'ResponseSent';
      peer: PeerId;
      connectionId: ConnectionId;
      requestId: InboundRequestId;
    };

/**
 * Protocol support direction.
 */
export class ProtocolSupport {
  private readonly inbound: boolean;
  private readonly outbound: boolean;

  private constructor(inbound: boolean, outbound: boolean) {
    this.inbound = inbound;
    this.outbound = outbound;
  }

  /**
   * Create a ProtocolSupport that supports both inbound and outbound requests.
   */
  static full(): ProtocolSupport {
    return new ProtocolSupport(true, true);
  }

  /**
   * Create a ProtocolSupport that only supports inbound requests.
   */
  static inbound(): ProtocolSupport {
    return new ProtocolSupport(true, false);
  }

  /**
   * Create a ProtocolSupport that only supports outbound requests.
   */
  static outbound(): ProtocolSupport {
    return new ProtocolSupport(false, true);
  }

  /**
   * Check if inbound requests are supported.
   */
  isInbound(): boolean {
    return this.inbound;
  }

  /**
   * Check if outbound requests are supported.
   */
  isOutbound(): boolean {
    return this.outbound;
  }
}

/**
 * Configuration for the request-response behaviour.
 */
export interface RequestResponseConfig {
  /**
   * Timeout for requests in milliseconds.
   */
  requestTimeout: number;
  
  /**
   * Maximum number of concurrent streams per connection.
   */
  maxConcurrentStreams: number;
}

/**
 * Default configuration values.
 */
export const defaultConfig: RequestResponseConfig = {
  requestTimeout: 10000, // 10 seconds
  maxConcurrentStreams: 100
};

/**
 * Codec interface for encoding and decoding requests and responses.
 */
export interface Codec<TRequest, TResponse, TProtocol extends string> {
  /**
   * Encode a request to a Uint8Array.
   */
  encodeRequest(request: TRequest): Uint8Array;
  
  /**
   * Decode a request from a Uint8Array.
   */
  decodeRequest(data: Uint8Array): TRequest;
  
  /**
   * Encode a response to a Uint8Array.
   */
  encodeResponse(response: TResponse): Uint8Array;
  
  /**
   * Decode a response from a Uint8Array.
   */
  decodeResponse(data: Uint8Array): TResponse;
  
  /**
   * Get the protocol identifier.
   */
  getProtocol(): TProtocol;
}
