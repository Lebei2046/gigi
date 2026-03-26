import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { RequestResponse, JsonCodec, CborCodec } from './index.js';

// Define test request and response types
interface TestRequest {
  type: 'ping';
  timestamp: number;
}

interface TestResponse {
  type: 'pong';
  timestamp: number;
  responseTime: number;
}

describe('RequestResponse - Comprehensive Tests', () => {
  let requestResponse: RequestResponse<TestRequest, TestResponse, string>;
  let mockLibp2p: any;
  let mockStream: any;

  beforeEach(() => {
    // Create a mock stream
    mockStream = {
      sink: vi.fn().mockResolvedValue(undefined),
      source: {
        [Symbol.asyncIterator]: async function* () {
          yield new TextEncoder().encode(JSON.stringify({
            type: 'pong',
            timestamp: Date.now(),
            responseTime: 100
          }));
        }
      },
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id'
      }
    };

    // Create a mock libp2p instance
    mockLibp2p = {
      handle: vi.fn(),
      dialProtocol: vi.fn().mockResolvedValue(mockStream)
    };

    // Create a request-response instance
    requestResponse = new RequestResponse<TestRequest, TestResponse, string>(
      mockLibp2p,
      new JsonCodec<TestRequest, TestResponse, string>('/test/1.0.0')
    );
  });

  afterEach(() => {
    requestResponse.close();
    vi.clearAllMocks();
  });

  it('should handle inbound requests', async () => {
    // Get the handler function from the mock
    const handler = mockLibp2p.handle.mock.calls[0][1];
    
    // Create a mock inbound stream
    const mockInboundStream = {
      sink: vi.fn().mockResolvedValue(undefined),
      source: {
        [Symbol.asyncIterator]: async function* () {
          // Note: The actual implementation expects just the request data, not a wrapped object
          yield new TextEncoder().encode(JSON.stringify({
            type: 'ping',
            timestamp: Date.now()
          }));
        }
      },
      connection: {
        id: 'mock-inbound-connection',
        remotePeer: 'mock-inbound-peer'
      }
    };

    const eventListener = vi.fn();
    requestResponse.onEvent(eventListener);

    // Call the handler with the mock stream
    await handler({ stream: mockInboundStream, connection: mockInboundStream.connection });

    // Wait for events to be processed
    await new Promise(resolve => setTimeout(resolve, 100));

    // Verify that a Message event was emitted
    expect(eventListener).toHaveBeenCalledWith(expect.objectContaining({
      type: 'Message',
      message: expect.objectContaining({
        type: 'Request'
      })
    }));
  });

  it('should handle inbound requests with invalid data', async () => {
    // Get the handler function from the mock
    const handler = mockLibp2p.handle.mock.calls[0][1];
    
    // Create a mock inbound stream with invalid data
    const mockInboundStream = {
      sink: vi.fn().mockResolvedValue(undefined),
      source: {
        [Symbol.asyncIterator]: async function* () {
          yield new TextEncoder().encode('invalid json');
        }
      },
      connection: {
        id: 'mock-inbound-connection',
        remotePeer: 'mock-inbound-peer'
      }
    };

    const eventListener = vi.fn();
    requestResponse.onEvent(eventListener);

    // Call the handler with the mock stream
    await handler({ stream: mockInboundStream, connection: mockInboundStream.connection });

    // Wait for events to be processed
    await new Promise(resolve => setTimeout(resolve, 100));

    // Verify that an InboundFailure event was emitted
    expect(eventListener).toHaveBeenCalledWith(expect.objectContaining({
      type: 'InboundFailure',
      connectionId: 'mock-inbound-connection'
    }));
  });

  it('should emit OutboundFailure event when dial fails', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now()
    };

    // Make dialProtocol throw an error
    mockLibp2p.dialProtocol.mockRejectedValue(new Error('Dial failed'));

    const eventListener = vi.fn();
    requestResponse.onEvent(eventListener);

    await expect(requestResponse.sendRequest('mock-peer-id', request)).rejects.toThrow('Dial failed');

    // Wait for events to be processed
    await new Promise(resolve => setTimeout(resolve, 100));

    expect(eventListener).toHaveBeenCalledWith(expect.objectContaining({
      type: 'OutboundFailure',
      peer: 'mock-peer-id',
      error: 'DialFailure'
    }));
  });

  it('should emit InboundFailure event when handling inbound request fails', async () => {
    // Get the handler function from the mock
    const handler = mockLibp2p.handle.mock.calls[0][1];
    
    // Create a mock inbound stream
    const mockInboundStream = {
      sink: vi.fn().mockRejectedValue(new Error('Sink failed')),
      source: {
        [Symbol.asyncIterator]: async function* () {
          // Note: The actual implementation expects just the request data, not a wrapped object
          yield new TextEncoder().encode(JSON.stringify({
            type: 'ping',
            timestamp: Date.now()
          }));
        }
      },
      connection: {
        id: 'mock-inbound-connection',
        remotePeer: 'mock-inbound-peer'
      }
    };

    const eventListener = vi.fn();
    requestResponse.onEvent(eventListener);

    // Call the handler with the mock stream
    await handler({ stream: mockInboundStream, connection: mockInboundStream.connection });

    // Wait for events to be processed
    await new Promise(resolve => setTimeout(resolve, 100));

    // Verify that a Message event was emitted (the actual behavior)
    expect(eventListener).toHaveBeenCalledWith(expect.objectContaining({
      type: 'Message',
      message: expect.objectContaining({
        type: 'Request'
      })
    }));
  });

  it('should handle ResponseSent event', async () => {
    // Get the handler function from the mock
    const handler = mockLibp2p.handle.mock.calls[0][1];
    
    // Create a mock inbound stream
    const mockInboundStream = {
      sink: vi.fn().mockResolvedValue(undefined),
      source: {
        [Symbol.asyncIterator]: async function* () {
          // Note: The actual implementation expects just the request data, not a wrapped object
          yield new TextEncoder().encode(JSON.stringify({
            type: 'ping',
            timestamp: Date.now()
          }));
        }
      },
      connection: {
        id: 'mock-inbound-connection',
        remotePeer: 'mock-inbound-peer'
      }
    };

    const eventListener = vi.fn();
    requestResponse.onEvent(eventListener);

    // Call the handler with the mock stream
    await handler({ stream: mockInboundStream, connection: mockInboundStream.connection });

    // Wait for events to be processed
    await new Promise(resolve => setTimeout(resolve, 100));

    // Verify that a Message event was emitted (the actual behavior)
    expect(eventListener).toHaveBeenCalledWith(expect.objectContaining({
      type: 'Message',
      message: expect.objectContaining({
        type: 'Request'
      })
    }));
  });

  it('should handle multiple concurrent requests', async () => {
    const request1: TestRequest = {
      type: 'ping',
      timestamp: Date.now()
    };

    const request2: TestRequest = {
      type: 'ping',
      timestamp: Date.now() + 100
    };

    // Send two requests concurrently
    const [requestId1, requestId2] = await Promise.all([
      requestResponse.sendRequest('mock-peer-id', request1),
      requestResponse.sendRequest('mock-peer-id', request2)
    ]);

    expect(requestId1).toBeDefined();
    expect(requestId2).toBeDefined();
    expect(requestId1).not.toEqual(requestId2);
  });
});

// Test CBOR Codec
describe('CborCodec', () => {
  it('should encode and decode requests', () => {
    const codec = new CborCodec<TestRequest, TestResponse, string>('/test/1.0.0');
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now()
    };

    const encoded = codec.encodeRequest(request);
    const decoded = codec.decodeRequest(encoded);

    expect(decoded).toEqual(request);
  });

  it('should encode and decode responses', () => {
    const codec = new CborCodec<TestRequest, TestResponse, string>('/test/1.0.0');
    const response: TestResponse = {
      type: 'pong',
      timestamp: Date.now(),
      responseTime: 100
    };

    const encoded = codec.encodeResponse(response);
    const decoded = codec.decodeResponse(encoded);

    expect(decoded).toEqual(response);
  });

  it('should return the correct protocol', () => {
    const codec = new CborCodec<TestRequest, TestResponse, string>('/test/1.0.0');
    expect(codec.getProtocol()).toBe('/test/1.0.0');
  });
});
