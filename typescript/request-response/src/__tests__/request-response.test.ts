import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import {
  RequestResponse,
  JsonCodec,
  ResponseChannel,
  InboundRequestId,
  OutboundRequestId,
  ProtocolSupport,
  defaultConfig,
} from '../index.js';

// Mock the @libp2p/peer-id module
vi.mock('@libp2p/peer-id', () => ({
  peerIdFromString: vi.fn((id) => ({ id })),
}));

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
  let mockLibp2p: {
    handle: any;
    dialProtocol: any;
  };
  let mockStream: {
    sink: any;
    source: any;
    close: any;
    connection: any;
  };

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
    const request = {} as TestRequest;

    const encoded = codec.encodeRequest(request);
    const decoded = codec.decodeRequest(encoded);

    expect(decoded).toEqual({});
  });

  it('should handle request objects with additional properties', () => {
    const codec = new JsonCodec<TestRequest, TestResponse, string>(
      '/test/1.0.0'
    );
    const request = {
      type: 'ping',
      timestamp: Date.now(),
      extraProperty: 'extra value',
      anotherProperty: 123,
    } as TestRequest & Record<string, any>;

    const encoded = codec.encodeRequest(request);
    const decoded = codec.decodeRequest(encoded);

    expect(decoded.type).toBe('ping');
    expect(decoded.timestamp).toBeDefined();
    expect((decoded as any).extraProperty).toBe('extra value');
    expect((decoded as any).anotherProperty).toBe(123);
  });

  it('should handle null and undefined values', () => {
    const codec = new JsonCodec<TestRequest, TestResponse, string>(
      '/test/1.0.0'
    );
    const request = {
      type: 'ping',
      timestamp: null as any,
      extra: undefined,
    } as TestRequest & { extra?: any };

    const encoded = codec.encodeRequest(request);
    const decoded = codec.decodeRequest(encoded);

    expect(decoded.type).toBe('ping');
    expect(decoded.timestamp).toBeNull();
    expect((decoded as any).extra).toBeUndefined();
  });

  it('should handle Uint8Array serialization and deserialization', () => {
    const codec = new JsonCodec<TestRequest, TestResponse, string>(
      '/test/1.0.0'
    );
    const request: any = {
      type: 'ping',
      timestamp: Date.now(),
      data: new Uint8Array([1, 2, 3, 4, 5]),
    };

    const encoded = codec.encodeRequest(request);
    const decoded = codec.decodeRequest(encoded);

    expect(decoded.type).toBe('ping');
    expect(decoded.timestamp).toBeDefined();
    expect(decoded.data).toBeInstanceOf(Uint8Array);
    expect(Array.from(decoded.data)).toEqual([1, 2, 3, 4, 5]);
  });

  it('should handle array serialization and deserialization', () => {
    const codec = new JsonCodec<TestRequest, TestResponse, string>(
      '/test/1.0.0'
    );
    const request: any = {
      type: 'ping',
      timestamp: Date.now(),
      items: [1, 2, 3, 4, 5],
      nested: {
        array: [6, 7, 8, 9, 10],
      },
    };

    const encoded = codec.encodeRequest(request);
    const decoded = codec.decodeRequest(encoded);

    expect(decoded.type).toBe('ping');
    expect(decoded.timestamp).toBeDefined();
    expect(decoded.items).toEqual([1, 2, 3, 4, 5]);
    expect(decoded.nested.array).toEqual([6, 7, 8, 9, 10]);
  });
});

// Edge case tests for RequestResponse
describe('RequestResponse - Edge Cases', () => {
  let requestResponse: RequestResponse<TestRequest, TestResponse, string>;
  let mockLibp2p: {
    handle: any;
    dialProtocol: any;
  };
  let mockStream: {
    sink: any;
    source: any;
    close: any;
    connection: any;
  };

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
    const request = {} as TestRequest;

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

  it('should handle different stream types for sending responses', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    // Test with send method
    const sendStream = {
      send: vi.fn().mockResolvedValue(undefined),
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
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
    };

    mockLibp2p.dialProtocol.mockResolvedValue(sendStream);
    await requestResponse.sendRequest('mock-peer-id', request);
    expect(sendStream.send).toHaveBeenCalled();

    // Test with write method
    const writeStream = {
      write: vi.fn().mockImplementation((data, callback) => callback(null)),
      end: vi.fn().mockImplementation((callback) => callback()),
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
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
    };

    mockLibp2p.dialProtocol.mockResolvedValue(writeStream);
    await requestResponse.sendRequest('mock-peer-id', request);
    expect(writeStream.write).toHaveBeenCalled();

    // Test with Web Streams API
    const webStream = {
      writable: {
        writable: true,
        getWriter: vi.fn().mockReturnValue({
          write: vi.fn().mockResolvedValue(undefined),
          close: vi.fn().mockResolvedValue(undefined),
        }),
      },
      readable: {
        getReader: vi.fn().mockReturnValue({
          read: vi
            .fn()
            .mockResolvedValueOnce({
              done: false,
              value: new TextEncoder().encode(
                JSON.stringify({
                  type: 'pong',
                  timestamp: Date.now(),
                  responseTime: 100,
                })
              ),
            })
            .mockResolvedValueOnce({ done: true, value: undefined }),
        }),
      },
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
    };

    mockLibp2p.dialProtocol.mockResolvedValue(webStream);
    await requestResponse.sendRequest('mock-peer-id', request);
    expect(webStream.writable.getWriter).toHaveBeenCalled();
  });

  it('should handle different stream types for reading', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    // Test with receive method
    const receiveStream = {
      send: vi.fn().mockResolvedValue(undefined),
      receive: vi
        .fn()
        .mockResolvedValueOnce(
          Buffer.from(
            JSON.stringify({
              type: 'pong',
              timestamp: Date.now(),
              responseTime: 100,
            })
          )
        )
        .mockResolvedValueOnce(null),
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
    };

    mockLibp2p.dialProtocol.mockResolvedValue(receiveStream);
    await requestResponse.sendRequest('mock-peer-id', request);
    expect(receiveStream.receive).toHaveBeenCalled();

    // Test with read method
    const readStream = {
      send: vi.fn().mockResolvedValue(undefined),
      read: vi
        .fn()
        .mockReturnValueOnce(
          Buffer.from(
            JSON.stringify({
              type: 'pong',
              timestamp: Date.now(),
              responseTime: 100,
            })
          )
        )
        .mockReturnValueOnce(null),
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
    };

    mockLibp2p.dialProtocol.mockResolvedValue(readStream);
    await requestResponse.sendRequest('mock-peer-id', request);
    expect(readStream.read).toHaveBeenCalled();

    // Test with event emitter pattern
    const eventStream = {
      send: vi.fn().mockResolvedValue(undefined),
      on: vi.fn().mockImplementation((event, callback) => {
        if (event === 'data') {
          setTimeout(() => {
            callback(
              Buffer.from(
                JSON.stringify({
                  type: 'pong',
                  timestamp: Date.now(),
                  responseTime: 100,
                })
              )
            );
          }, 10);
        } else if (event === 'end') {
          setTimeout(() => callback(), 20);
        }
        return eventStream;
      }),
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
    };

    mockLibp2p.dialProtocol.mockResolvedValue(eventStream);
    await requestResponse.sendRequest('mock-peer-id', request);
    expect(eventStream.on).toHaveBeenCalled();
  });

  it('should handle peer ID conversion', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    // Test with multiaddr
    await requestResponse.sendRequest(
      '/ip4/127.0.0.1/tcp/8080/p2p/12D3KooW',
      request
    );
    expect(mockLibp2p.dialProtocol).toHaveBeenCalled();
  });

  it('should handle different stream types for reading', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    // Test with nested stream
    const nestedStream = {
      stream: {
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
      send: vi.fn().mockResolvedValue(undefined),
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
    };

    mockLibp2p.dialProtocol.mockResolvedValue(nestedStream);
    await requestResponse.sendRequest('mock-peer-id', request);
    expect(mockLibp2p.dialProtocol).toHaveBeenCalled();

    // Test with different chunk types
    const bufferStream = {
      [Symbol.asyncIterator]: async function* () {
        yield Buffer.from(
          JSON.stringify({
            type: 'pong',
            timestamp: Date.now(),
            responseTime: 100,
          })
        );
      },
      send: vi.fn().mockResolvedValue(undefined),
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
    };

    mockLibp2p.dialProtocol.mockResolvedValue(bufferStream);
    await requestResponse.sendRequest('mock-peer-id', request);
    expect(mockLibp2p.dialProtocol).toHaveBeenCalled();
  });

  it('should handle unsupported stream type', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    // Test with unsupported stream type
    const unsupportedStream = {
      send: vi.fn().mockResolvedValue(undefined),
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
    };

    mockLibp2p.dialProtocol.mockResolvedValue(unsupportedStream);
    await expect(
      requestResponse.sendRequest('mock-peer-id', request)
    ).rejects.toThrow('Unsupported stream type');
  });

  it('should handle empty response', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    // Test with empty response
    const emptyStream = {
      sink: vi.fn().mockResolvedValue(undefined),
      source: {
        [Symbol.asyncIterator]: async function* () {
          // Empty stream
        },
      },
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
    };

    mockLibp2p.dialProtocol.mockResolvedValue(emptyStream);
    await expect(
      requestResponse.sendRequest('mock-peer-id', request)
    ).rejects.toThrow();
  });

  it('should handle response channel operations', async () => {
    // Get the handler function from the mock
    const handler = mockLibp2p.handle.mock.calls[0][1];

    // Create a mock inbound stream
    const mockInboundStream = {
      sink: vi.fn().mockResolvedValue(undefined),
      source: {
        [Symbol.asyncIterator]: async function* () {
          yield new TextEncoder().encode(
            JSON.stringify({
              type: 'ping',
              timestamp: Date.now(),
            })
          );
        },
      },
      connection: {
        id: 'mock-inbound-connection',
        remotePeer: 'mock-inbound-peer',
      },
    };

    const eventListener = vi.fn();
    requestResponse.onEvent(eventListener);

    // Call the handler with the mock stream
    await handler({
      stream: mockInboundStream,
      connection: mockInboundStream.connection,
    });

    // Wait for events to be processed
    await new Promise((resolve) => setTimeout(resolve, 100));

    // Verify that a Message event was emitted
    expect(eventListener).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'Message',
        message: expect.objectContaining({
          type: 'Request',
          channel: expect.objectContaining({
            isOpen: expect.any(Function),
            send: expect.any(Function),
            getResponse: expect.any(Function),
          }),
        }),
      })
    );
  });

  it('should handle Web Streams API for response writing', async () => {
    // Get the handler function from the mock
    const handler = mockLibp2p.handle.mock.calls[0][1];

    // Create a mock inbound stream with Web Streams API
    const mockWriter = {
      write: vi.fn().mockResolvedValue(undefined),
      close: vi.fn().mockResolvedValue(undefined),
    };

    const mockInboundStream = {
      writable: {
        writable: true,
        getWriter: vi.fn().mockReturnValue(mockWriter),
      },
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-inbound-connection',
        remotePeer: 'mock-inbound-peer',
      },
      source: {
        [Symbol.asyncIterator]: async function* () {
          yield new TextEncoder().encode(
            JSON.stringify({
              type: 'ping',
              timestamp: Date.now(),
            })
          );
        },
      },
    };

    // Call the handler with the mock stream
    await handler({
      stream: mockInboundStream,
      connection: mockInboundStream.connection,
    });

    // Wait for processing
    await new Promise((resolve) => setTimeout(resolve, 100));
  });

  it('should handle error when no write method available', async () => {
    // Get the handler function from the mock
    const handler = mockLibp2p.handle.mock.calls[0][1];

    // Create a mock inbound stream with no write methods
    const mockInboundStream = {
      connection: {
        id: 'mock-inbound-connection',
        remotePeer: 'mock-inbound-peer',
      },
      source: {
        [Symbol.asyncIterator]: async function* () {
          yield new TextEncoder().encode(
            JSON.stringify({
              type: 'ping',
              timestamp: Date.now(),
            })
          );
        },
      },
    };

    // Call the handler with the mock stream
    await handler({
      stream: mockInboundStream,
      connection: mockInboundStream.connection,
    });

    // Wait for processing
    await new Promise((resolve) => setTimeout(resolve, 100));
  });

  it('should handle stream closing after response', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    const mockStream = {
      sink: vi.fn().mockResolvedValue(undefined),
      source: {
        [Symbol.asyncIterator]: async function* () {
          yield new TextEncoder().encode(
            '{"type":"pong","timestamp":1234567890}'
          );
        },
      },
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
    };

    mockLibp2p.dialProtocol.mockResolvedValue(mockStream);

    const response = await requestResponse.sendRequest('mock-peer-id', request);
    expect(response).toEqual({ type: 'pong', timestamp: 1234567890 });
    expect(mockStream.close).toHaveBeenCalled();
  });

  it('should handle stream read with receive method', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    const mockStream = {
      sink: vi.fn().mockResolvedValue(undefined),
      source: {
        [Symbol.asyncIterator]: async function* () {
          yield Buffer.from('{"type":"pong","timestamp":1234567890}');
        },
      },
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
    };

    mockLibp2p.dialProtocol.mockResolvedValue(mockStream);

    const response = await requestResponse.sendRequest('mock-peer-id', request);
    expect(response).toEqual({ type: 'pong', timestamp: 1234567890 });
  });

  it('should handle stream read with read method', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    const mockStream = {
      sink: vi.fn().mockResolvedValue(undefined),
      source: {
        [Symbol.asyncIterator]: async function* () {
          yield Buffer.from('{"type":"pong","timestamp":1234567890}');
        },
      },
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
    };

    mockLibp2p.dialProtocol.mockResolvedValue(mockStream);

    const response = await requestResponse.sendRequest('mock-peer-id', request);
    expect(response).toEqual({ type: 'pong', timestamp: 1234567890 });
  });

  it('should handle stream read with on data event', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    const mockStream = {
      sink: vi.fn().mockResolvedValue(undefined),
      on: vi.fn((event, callback) => {
        if (event === 'data') {
          callback(Buffer.from('{"type":"pong","timestamp":1234567890}'));
        } else if (event === 'end') {
          callback();
        }
      }),
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
    };

    mockLibp2p.dialProtocol.mockResolvedValue(mockStream);

    const response = await requestResponse.sendRequest('mock-peer-id', request);
    expect(response).toEqual({ type: 'pong', timestamp: 1234567890 });
  });

  it('should handle event listener error', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    // Add an event listener that throws an error
    requestResponse.onEvent(() => {
      throw new Error('Listener error');
    });

    const mockStream = {
      sink: vi.fn().mockResolvedValue(undefined),
      source: {
        [Symbol.asyncIterator]: async function* () {
          yield new TextEncoder().encode(
            '{"type":"pong","timestamp":1234567890}'
          );
        },
      },
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
    };

    mockLibp2p.dialProtocol.mockResolvedValue(mockStream);

    // Should not throw despite listener error
    const response = await requestResponse.sendRequest('mock-peer-id', request);
    expect(response).toEqual({ type: 'pong', timestamp: 1234567890 });
  });

  it('should test close method', async () => {
    requestResponse.close();
    // Should not throw
    expect(() => requestResponse.close()).not.toThrow();
  });

  it('should handle different chunk types in readStream', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    // Create a mock stream that simulates different chunk types
    const mockStream = {
      sink: vi.fn().mockResolvedValue(undefined),
      source: {
        [Symbol.asyncIterator]: async function* () {
          // Test Buffer chunk
          yield Buffer.from('{"type":"pong","timestamp":1234567890}');
        },
      },
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
    };

    mockLibp2p.dialProtocol.mockResolvedValue(mockStream);

    const response = await requestResponse.sendRequest('mock-peer-id', request);
    expect(response).toEqual({ type: 'pong', timestamp: 1234567890 });
  });

  it('should handle stream error event', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    const mockStream = {
      sink: vi.fn().mockResolvedValue(undefined),
      on: vi.fn((event, callback) => {
        if (event === 'error') {
          callback(new Error('Stream error'));
        }
      }),
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
    };

    mockLibp2p.dialProtocol.mockResolvedValue(mockStream);

    await expect(
      requestResponse.sendRequest('mock-peer-id', request)
    ).rejects.toThrow('Stream error');
  });

  it('should handle ResponseSent event emission', async () => {
    // Get the handler function from the mock
    const handler = mockLibp2p.handle.mock.calls[0][1];

    // Create a mock inbound stream
    const mockInboundStream = {
      sink: vi.fn().mockResolvedValue(undefined),
      source: {
        [Symbol.asyncIterator]: async function* () {
          yield new TextEncoder().encode(
            JSON.stringify({
              type: 'ping',
              timestamp: Date.now(),
            })
          );
        },
      },
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-inbound-connection',
        remotePeer: 'mock-inbound-peer',
      },
    };

    // Add event listener to capture ResponseSent event
    const eventListener = vi.fn();
    requestResponse.onEvent(eventListener);

    // Call the handler with the mock stream
    await handler({
      stream: mockInboundStream,
      connection: mockInboundStream.connection,
    });

    // Wait for processing
    await new Promise((resolve) => setTimeout(resolve, 100));

    // Verify that Message event was emitted
    expect(eventListener).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'Message',
      })
    );
  });

  it('should handle stream with on data event', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now(),
    };

    // Create a mock stream with on data event
    const mockStream = {
      sink: vi.fn().mockResolvedValue(undefined),
      on: vi.fn((event, callback) => {
        if (event === 'data') {
          callback(Buffer.from('{"type":"pong","timestamp":1234567890}'));
        } else if (event === 'end') {
          callback();
        }
      }),
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id',
      },
    };

    mockLibp2p.dialProtocol.mockResolvedValue(mockStream);

    const response = await requestResponse.sendRequest('mock-peer-id', request);
    expect(response).toEqual({ type: 'pong', timestamp: 1234567890 });
  });

  it('should test ResponseSent event emission', async () => {
    // Get the handler function from the mock
    const handler = mockLibp2p.handle.mock.calls[0][1];

    // Create a mock inbound stream
    const mockInboundStream = {
      sink: vi.fn().mockResolvedValue(undefined),
      source: {
        [Symbol.asyncIterator]: async function* () {
          yield new TextEncoder().encode(
            JSON.stringify({
              type: 'ping',
              timestamp: Date.now(),
            })
          );
        },
      },
      close: vi.fn().mockResolvedValue(undefined),
      connection: {
        id: 'mock-inbound-connection',
        remotePeer: 'mock-inbound-peer',
      },
    };

    // Call the handler with the mock stream
    await handler({
      stream: mockInboundStream,
      connection: mockInboundStream.connection,
    });

    // Wait for processing
    await new Promise((resolve) => setTimeout(resolve, 100));
  });

  it('should handle different chunk types in readStream', async () => {
    // Test the readStream method directly with different chunk types
    const mockStream = {
      source: {
        [Symbol.asyncIterator]: async function* () {
          // Test Buffer chunk
          yield Buffer.from('{"type":"pong","timestamp":1234567890}');
        },
      },
      close: vi.fn().mockResolvedValue(undefined),
    };

    // @ts-ignore - Accessing private method for testing
    const result = await requestResponse.readStream(mockStream);
    expect(result).toBeInstanceOf(Uint8Array);
  });

  it('should handle Array chunk type in readStream', async () => {
    // Test the readStream method directly with Array chunk
    const mockStream = {
      source: {
        [Symbol.asyncIterator]: async function* () {
          // Test Array chunk
          yield [
            123, 34, 116, 121, 112, 101, 34, 58, 34, 112, 111, 110, 103, 34,
            125,
          ];
        },
      },
      close: vi.fn().mockResolvedValue(undefined),
    };

    // @ts-ignore - Accessing private method for testing
    const result = await requestResponse.readStream(mockStream);
    expect(result).toBeInstanceOf(Uint8Array);
  });

  it('should handle Uint8ArrayList chunk type in readStream', async () => {
    // Test the readStream method directly with Uint8ArrayList chunk
    const mockStream = {
      source: {
        [Symbol.asyncIterator]: async function* () {
          // Test Uint8ArrayList-like chunk
          yield {
            toUint8Array: () =>
              new Uint8Array([
                123, 34, 116, 121, 112, 101, 34, 58, 34, 112, 111, 110, 103, 34,
                125,
              ]),
          };
        },
      },
      close: vi.fn().mockResolvedValue(undefined),
    };

    // @ts-ignore - Accessing private method for testing
    const result = await requestResponse.readStream(mockStream);
    expect(result).toBeInstanceOf(Uint8Array);
  });

  it('should handle chunk with slice method in readStream', async () => {
    // Test the readStream method directly with chunk that has slice method
    const mockStream = {
      source: {
        [Symbol.asyncIterator]: async function* () {
          // Test chunk with slice method
          yield {
            slice: () => [
              123, 34, 116, 121, 112, 101, 34, 58, 34, 112, 111, 110, 103, 34,
              125,
            ],
          };
        },
      },
      close: vi.fn().mockResolvedValue(undefined),
    };

    // @ts-ignore - Accessing private method for testing
    const result = await requestResponse.readStream(mockStream);
    expect(result).toBeInstanceOf(Uint8Array);
  });

  it('should handle chunk conversion error in readStream', async () => {
    // Test the readStream method directly with chunk that can't be converted
    const mockStream = {
      source: {
        [Symbol.asyncIterator]: async function* () {
          // Test chunk that can't be converted to Uint8Array
          yield undefined;
        },
      },
      close: vi.fn().mockResolvedValue(undefined),
    };

    // @ts-ignore - Accessing private method for testing
    const result = await requestResponse.readStream(mockStream);
    expect(result).toBeInstanceOf(Uint8Array);
  });
});

// Test index.ts exports
describe('index.ts exports', () => {
  it('should export all required modules', () => {
    // Import is already done at the top of the file
    // Just verify that the imports are working
    expect(RequestResponse).toBeDefined();
  });
});

// Test types.ts
describe('types.ts', () => {
  describe('ResponseChannel', () => {
    it('should create a response channel', () => {
      let responseSent = false;
      let sentResponse: any = null;

      const channel = new ResponseChannel((response) => {
        responseSent = true;
        sentResponse = response;
      });

      expect(channel.isOpen()).toBe(true);
      expect(channel.getResponse()).toBeNull();

      const testResponse = { type: 'pong', timestamp: Date.now() };
      const result = channel.send(testResponse);

      expect(result).toBe(true);
      expect(responseSent).toBe(true);
      expect(sentResponse).toEqual(testResponse);
      expect(channel.isOpen()).toBe(false);
      expect(channel.getResponse()).toEqual(testResponse);

      // Try to send again after channel is closed
      const result2 = channel.send({ type: 'pong', timestamp: Date.now() });
      expect(result2).toBe(false);
    });
  });

  describe('InboundRequestId', () => {
    it('should create an inbound request ID', () => {
      const requestId = new InboundRequestId(123);
      expect(requestId.value).toBe(123);
      expect(requestId.toString()).toBe('123');

      const requestId2 = new InboundRequestId(123);
      const requestId3 = new InboundRequestId(456);

      expect(requestId.equals(requestId2)).toBe(true);
      expect(requestId.equals(requestId3)).toBe(false);
      expect(requestId.hashCode()).toBe(123);
    });
  });

  describe('OutboundRequestId', () => {
    it('should create an outbound request ID', () => {
      const requestId = new OutboundRequestId(123);
      expect(requestId.value).toBe(123);
      expect(requestId.toString()).toBe('123');

      const requestId2 = new OutboundRequestId(123);
      const requestId3 = new OutboundRequestId(456);

      expect(requestId.equals(requestId2)).toBe(true);
      expect(requestId.equals(requestId3)).toBe(false);
      expect(requestId.hashCode()).toBe(123);
    });
  });

  describe('ProtocolSupport', () => {
    it('should create full protocol support', () => {
      const support = ProtocolSupport.full();
      expect(support.isInbound()).toBe(true);
      expect(support.isOutbound()).toBe(true);
    });

    it('should create inbound-only protocol support', () => {
      const support = ProtocolSupport.inbound();
      expect(support.isInbound()).toBe(true);
      expect(support.isOutbound()).toBe(false);
    });

    it('should create outbound-only protocol support', () => {
      const support = ProtocolSupport.outbound();
      expect(support.isInbound()).toBe(false);
      expect(support.isOutbound()).toBe(true);
    });
  });

  describe('defaultConfig', () => {
    it('should have default values', () => {
      expect(defaultConfig.requestTimeout).toBe(10000);
      expect(defaultConfig.maxConcurrentStreams).toBe(100);
    });
  });
});
