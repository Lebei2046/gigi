import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { RequestResponse, JsonCodec } from './index.js';

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

  it('should initialize with the correct protocol', () => {
    expect(mockLibp2p.handle).toHaveBeenCalledWith('/test/1.0.0', expect.any(Function));
  });

  it('should send a request and receive a response', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now()
    };

    const requestId = await requestResponse.sendRequest('mock-peer-id', request);

    expect(mockLibp2p.dialProtocol).toHaveBeenCalledWith('mock-peer-id', '/test/1.0.0');
    expect(mockStream.sink).toHaveBeenCalled();
    expect(requestId).toBeDefined();
  });

  it('should emit an event when a response is received', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now()
    };

    const eventListener = vi.fn();
    requestResponse.onEvent(eventListener);

    await requestResponse.sendRequest('mock-peer-id', request);

    // Wait for events to be processed
    await new Promise(resolve => setTimeout(resolve, 100));

    expect(eventListener).toHaveBeenCalled();
  });

  it('should handle errors when sending a request', async () => {
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now()
    };

    // Make dialProtocol throw an error
    mockLibp2p.dialProtocol.mockRejectedValue(new Error('Dial failed'));

    await expect(requestResponse.sendRequest('mock-peer-id', request)).rejects.toThrow('Dial failed');
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
    const codec = new JsonCodec<TestRequest, TestResponse, string>('/test/1.0.0');
    const request: TestRequest = {
      type: 'ping',
      timestamp: Date.now()
    };

    const encoded = codec.encodeRequest(request);
    const decoded = codec.decodeRequest(encoded);

    expect(decoded).toEqual(request);
  });

  it('should encode and decode responses', () => {
    const codec = new JsonCodec<TestRequest, TestResponse, string>('/test/1.0.0');
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
    const codec = new JsonCodec<TestRequest, TestResponse, string>('/test/1.0.0');
    expect(codec.getProtocol()).toBe('/test/1.0.0');
  });
});
