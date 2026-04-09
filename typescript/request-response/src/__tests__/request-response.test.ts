import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { RequestResponse, JsonCodec } from '../index.js';

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

describe('RequestResponse', () => {
  let requestResponse: RequestResponse<TestRequest, TestResponse, string>;
  let mockLibp2p: any;
  let mockStream: any;

  beforeEach(() => {
    // Create a mock stream
    mockStream = {
      sink: vi.fn().mockResolvedValue(undefined),
      source: {
        [Symbol.asyncIterator]: async function* () {
          yield new TextEncoder().encode(
            JSON.stringify({
              type: 'pong',
              timestamp: Date.now(),
              responseTime: 100,
            })
          );
        },
      },
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
    };

    // Create a mock libp2p instance
    mockLibp2p = {
      handle: vi.fn(),
      dialProtocol: vi.fn().mockResolvedValue(mockStream),
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

  it('should initialize with the correct protocol', () => {
    expect(mockLibp2p.handle).toHaveBeenCalledWith(
      '/test/1.0.0',
      expect.any(Function)
    );
  });

  it('should send a request and receive a response', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    const requestId = await requestResponse.sendRequest(
      'mock-peer-id',
      request
    );

    expect(mockLibp2p.dialProtocol).toHaveBeenCalledWith(
      'mock-peer-id',
      '/test/1.0.0'
    );
    expect(mockStream.sink).toHaveBeenCalled();
    expect(requestId).toBeDefined();
  });

  it('should emit an event when a response is received', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    const eventListener = vi.fn();
    requestResponse.onEvent(eventListener);

    await requestResponse.sendRequest('mock-peer-id', request);

    // Wait for events to be processed
    await new Promise((resolve) => setTimeout(resolve, 100));

    expect(eventListener).toHaveBeenCalled();
  });

  it('should handle errors when sending a request', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    // Make dialProtocol throw an error
    mockLibp2p.dialProtocol.mockRejectedValue(new Error('Dial failed'));

    await expect(
      requestResponse.sendRequest('mock-peer-id', request)
    ).rejects.toThrow('Dial failed');
  });

  it('should close and clean up resources', () => {
    requestResponse.close();
    // Verify that resources are cleaned up
    expect(requestResponse['pendingOutboundRequests']).toEqual(new Map());
    expect(requestResponse['pendingInboundRequests']).toEqual(new Map());
    expect(requestResponse['eventListeners']).toEqual([]);
  });
});

// Test JsonCodec
describe('JsonCodec', () => {
  it('should encode and decode requests', () => {
    const codec = new JsonCodec<TestRequest, TestResponse, string>(
      '/test/1.0.0'
    );
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    const encoded = codec.encodeRequest(request);
    const decoded = codec.decodeRequest(encoded);

    expect(decoded).toEqual(request);
  });

  it('should encode and decode responses', () => {
    const codec = new JsonCodec<TestRequest, TestResponse, string>(
      '/test/1.0.0'
    );
    const response: TestResponse = {
      type: 'pong',
      timestamp: Date.now(),
      responseTime: 100,
    };

    const encoded = codec.encodeResponse(response);
    const decoded = codec.decodeResponse(encoded);

    expect(decoded).toEqual(response);
  });

  it('should return the correct protocol', () => {
    const codec = new JsonCodec<TestRequest, TestResponse, string>(
      '/test/1.0.0'
    );
    expect(codec.getProtocol()).toBe('/test/1.0.0');
  });

  // Edge case tests for codec handling
  it('should handle empty request objects', () => {
    const codec = new JsonCodec<TestRequest, TestResponse, string>(
      '/test/1.0.0'
    );
    const request: any = {};

    const encoded = codec.encodeRequest(request);
    const decoded = codec.decodeRequest(encoded);

    expect(decoded).toEqual({});
  });

  it('should handle request objects with additional properties', () => {
    const codec = new JsonCodec<TestRequest, TestResponse, string>(
      '/test/1.0.0'
    );
    const request: any = {
      type: 'ping',
      timestamp: Date.now(),
      extraProperty: 'extra value',
      anotherProperty: 123,
    };

    const encoded = codec.encodeRequest(request);
    const decoded = codec.decodeRequest(encoded);

    expect(decoded.type).toBe('ping');
    expect(decoded.timestamp).toBeDefined();
    expect(decoded.extraProperty).toBe('extra value');
    expect(decoded.anotherProperty).toBe(123);
  });

  it('should handle null and undefined values', () => {
    const codec = new JsonCodec<TestRequest, TestResponse, string>(
      '/test/1.0.0'
    );
    const request: any = {
      type: 'ping',
      timestamp: null,
      extra: undefined,
    };

    const encoded = codec.encodeRequest(request);
    const decoded = codec.decodeRequest(encoded);

    expect(decoded.type).toBe('ping');
    expect(decoded.timestamp).toBeNull();
    expect(decoded.extra).toBeUndefined();
  });
});

// Edge case tests for RequestResponse
describe('RequestResponse - Edge Cases', () => {
  let requestResponse: RequestResponse<TestRequest, TestResponse, string>;
  let mockLibp2p: any;
  let mockStream: any;

  beforeEach(() => {
    // Create a mock stream
    mockStream = {
      sink: vi.fn().mockResolvedValue(undefined),
      source: {
        [Symbol.asyncIterator]: async function* () {
          yield new TextEncoder().encode(
            JSON.stringify({
              type: 'pong',
              timestamp: Date.now(),
              responseTime: 100,
            })
          );
        },
      },
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
    };

    // Create a mock libp2p instance
    mockLibp2p = {
      handle: vi.fn(),
      dialProtocol: vi.fn().mockResolvedValue(mockStream),
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

  it('should handle empty request data', async () => {
    const request: any = {};

    const requestId = await requestResponse.sendRequest(
      'mock-peer-id',
      request
    );

    expect(requestId).toBeDefined();
    expect(mockLibp2p.dialProtocol).toHaveBeenCalled();
  });

  it('should handle large request data', async () => {
    const largeRequest: any = {
      type: 'ping',
      timestamp: Date.now(),
      largeData: 'x'.repeat(10000), // 10KB of data
    };

    const requestId = await requestResponse.sendRequest(
      'mock-peer-id',
      largeRequest
    );

    expect(requestId).toBeDefined();
    expect(mockLibp2p.dialProtocol).toHaveBeenCalled();
  });

  it('should handle stream errors during response reading', async () => {
    // Create a mock stream that throws an error when reading
    const errorStream = {
      sink: vi.fn().mockResolvedValue(undefined),
      source: {
        [Symbol.asyncIterator]: async function* () {
          throw new Error('Stream read error');
        },
      },
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
    };

    mockLibp2p.dialProtocol.mockResolvedValue(errorStream);

    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    // Should throw an error when stream reading fails
    await expect(
      requestResponse.sendRequest('mock-peer-id', request)
    ).rejects.toThrow('Stream read error');
  });

  it('should handle multiple concurrent requests', async () => {
    const request1: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    const request2: TestRequest = {
      type: 'ping',
      timestamp: Date.now() + 1,
    };

    // Send two requests concurrently
    const [response1, response2] = await Promise.all([
      requestResponse.sendRequest('mock-peer-id', request1),
      requestResponse.sendRequest('mock-peer-id', request2),
    ]);

    expect(response1).toBeDefined();
    expect(response2).toBeDefined();
    expect(response1.type).toBe('pong');
    expect(response2.type).toBe('pong');
  });

  it('should handle closing with pending requests', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    // Send a request
    await requestResponse.sendRequest('mock-peer-id', request);

    // Close the request-response instance
    // Should not throw an error
    expect(() => requestResponse.close()).not.toThrow();
  });

  it('should handle multiple event listeners', () => {
    const listener1 = vi.fn();
    const listener2 = vi.fn();
    const listener3 = vi.fn();

    requestResponse.onEvent(listener1);
    requestResponse.onEvent(listener2);
    requestResponse.onEvent(listener3);

    // Simulate an event
    const testEvent = {
      type: 'TestEvent',
      data: 'test data',
    };

    // @ts-ignore - Accessing private method for testing
    requestResponse.emitEvent(testEvent);

    expect(listener1).toHaveBeenCalledWith(testEvent);
    expect(listener2).toHaveBeenCalledWith(testEvent);
    expect(listener3).toHaveBeenCalledWith(testEvent);
  });

  it('should handle removing event listeners', () => {
    const listener = vi.fn();

    // Add the listener
    const removeListener = requestResponse.onEvent(listener);

    // Remove the listener
    // Should not throw an error
    expect(() => removeListener()).not.toThrow();
  });
});
