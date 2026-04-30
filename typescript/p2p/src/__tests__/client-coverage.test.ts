import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { P2pClient } from '../client';
import { createLibp2pInstance } from '../libp2p-setup';
import { eventEmitter } from '../events';

// Mock dependencies
vi.mock('../libp2p-setup');
vi.mock('../events');
vi.mock('../errors', () => ({
  P2pError: {
    notStarted: vi.fn().mockReturnValue(new Error('Client not started')),
    peerNotFound: vi.fn().mockReturnValue(new Error('Peer not found')),
    fileNotFound: vi.fn().mockReturnValue(new Error('File not found')),
    networkError: vi.fn().mockReturnValue(new Error('Network error')),
    timeout: vi.fn().mockReturnValue(new Error('Timeout')),
    alreadyStarted: vi.fn().mockReturnValue(new Error('Already started')),
  },
}));
vi.mock('@libp2p/peer-id', () => ({
  peerIdFromString: vi.fn().mockReturnValue({ toString: () => 'mock-peer-id' }),
}));

const mockCreateLibp2pInstance = vi.mocked(createLibp2pInstance);
const mockEventEmitter = vi.mocked(eventEmitter);

// Mock implementations
const mockLibp2p = {
  peerId: { toString: vi.fn().mockReturnValue('mock-peer-id') },
  getMultiaddrs: vi.fn().mockReturnValue(['/ip4/127.0.0.1/tcp/1234']),
  start: vi.fn().mockResolvedValue(undefined),
  stop: vi.fn().mockResolvedValue(undefined),
  dial: vi.fn().mockResolvedValue(undefined),
  dialProtocol: vi.fn().mockResolvedValue({
    sink: vi.fn().mockResolvedValue(undefined),
    source: {
      [Symbol.asyncIterator]: async function* () {
        yield new TextEncoder().encode('test message');
      },
    },
    close: vi.fn().mockResolvedValue(undefined),
  }),
  handle: vi.fn().mockResolvedValue(undefined),
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
  services: {
    pubsub: {
      subscribe: vi.fn(),
      unsubscribe: vi.fn(),
      publish: vi.fn().mockResolvedValue(undefined),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    },
    dht: {
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    },
  },
};

const mockGigiDns = {
  on: vi.fn(),
  stop: vi.fn(),
  updateListenAddresses: vi.fn(),
  startService: vi.fn(),
  sendQuery: vi.fn(),
  emitDiscoveredPeers: vi.fn(),
};

describe('P2pClient Coverage Tests', () => {
  let client: P2pClient;
  const config = {
    nickname: 'test-client',
    config: {
      bootstrapNodes: [],
      enableKademlia: false,
      enableRelay: true,
      enableMdns: false,
      listenAddrs: ['/ip4/127.0.0.1/tcp/0'],
    },
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockCreateLibp2pInstance.mockResolvedValue({
      libp2p: mockLibp2p as unknown as Awaited<
        ReturnType<typeof import('../libp2p-setup').createLibp2pInstance>
      >['libp2p'],
      gigiDns: mockGigiDns as unknown as Awaited<
        ReturnType<typeof import('../libp2p-setup').createLibp2pInstance>
      >['gigiDns'],
    });
    mockEventEmitter.emit = vi.fn().mockResolvedValue(undefined);
    client = new P2pClient(config);
  });

  afterEach(async () => {
    if (client) {
      await client.stop();
    }
  });

  it('should initialize with mnemonic', async () => {
    const mnemonic =
      'test test test test test test test test test test test test';
    client = new P2pClient({
      ...config,
      mnemonic,
    });
    await client.start();
    expect(mockCreateLibp2pInstance).toHaveBeenCalledWith(
      expect.objectContaining({ mnemonic })
    );
  });

  it('should handle Gigi DNS events', async () => {
    // Setup Gigi DNS mock to emit events
    const mockDiscoveredEvent = {
      peerInfo: {
        peerId: { toString: vi.fn().mockReturnValue('mock-discovered-peer') },
        nickname: 'discovered-peer',
        multiaddr: {
          toString: vi.fn().mockReturnValue('/ip4/127.0.0.1/tcp/5678'),
        },
      },
    };

    const mockOfflineEvent = {
      peerInfo: {
        peerId: { toString: vi.fn().mockReturnValue('mock-offline-peer') },
        nickname: 'offline-peer',
      },
    };

    const mockErrorEvent = {
      error: new Error('DNS error'),
      context: 'test context',
    };

    await client.start();

    // Simulate Gigi DNS events
    const discoveredCall = mockGigiDns.on.mock.calls.find(
      (call: any) => call[0] === 'Discovered'
    );
    const offlineCall = mockGigiDns.on.mock.calls.find(
      (call: any) => call[0] === 'Offline'
    );
    const errorCall = mockGigiDns.on.mock.calls.find(
      (call: any) => call[0] === 'Error'
    );

    expect(discoveredCall).toBeDefined();
    expect(offlineCall).toBeDefined();
    expect(errorCall).toBeDefined();

    const discoveredListener = discoveredCall![1];
    const offlineListener = offlineCall![1];
    const errorListener = errorCall![1];

    // Trigger the listeners
    discoveredListener(mockDiscoveredEvent);
    offlineListener(mockOfflineEvent);
    errorListener(mockErrorEvent);

    // Verify events were emitted
    expect(mockEventEmitter.emit).toHaveBeenCalled();
  });

  it('should handle peer:connect and peer:disconnect events', async () => {
    await client.start();

    // Get the event listeners
    const connectCall = mockLibp2p.addEventListener.mock.calls.find(
      (call: any) => call[0] === 'peer:connect'
    );
    const disconnectCall = mockLibp2p.addEventListener.mock.calls.find(
      (call: any) => call[0] === 'peer:disconnect'
    );

    expect(connectCall).toBeDefined();
    expect(disconnectCall).toBeDefined();

    const connectListener = connectCall![1];
    const disconnectListener = disconnectCall![1];

    // Simulate events
    connectListener({
      detail: {
        remotePeer: { toString: vi.fn().mockReturnValue('mock-peer') },
      },
    });
    disconnectListener({
      detail: {
        remotePeer: { toString: vi.fn().mockReturnValue('mock-peer') },
      },
    });

    // Verify events were emitted
    expect(mockEventEmitter.emit).toHaveBeenCalled();
  });

  it('should handle pubsub messages', async () => {
    await client.start();

    // Get the pubsub listener
    const pubsubCall =
      mockLibp2p.services.pubsub.addEventListener.mock.calls.find(
        (call: any) => call[0] === 'message'
      );

    expect(pubsubCall).toBeDefined();
    const pubsubListener = pubsubCall![1];

    // Simulate a structured message
    pubsubListener({
      detail: {
        topic: 'gigi-group:general',
        data: new TextEncoder().encode(
          JSON.stringify({
            senderPeerId: 'mock-sender',
            senderNickname: 'sender',
            content: { type: 'text', text: 'Hello' },
            timestamp: Date.now(),
          })
        ),
      },
    });

    // Simulate a plain text message
    pubsubListener({
      detail: {
        topic: 'gigi-group:general',
        data: new TextEncoder().encode('Plain text message'),
        from: 'mock-sender',
      },
    });

    // Verify events were emitted
    expect(mockEventEmitter.emit).toHaveBeenCalled();
  });

  it('should handle DHT peer events when DHT is enabled', async () => {
    client = new P2pClient({
      ...config,
      config: {
        ...config.config,
        enableKademlia: true,
      },
    });
    await client.start();

    // Get the DHT listener
    const dhtCall = mockLibp2p.services.dht.addEventListener.mock.calls.find(
      (call: any) => call[0] === 'peer'
    );

    expect(dhtCall).toBeDefined();
    const dhtListener = dhtCall![1];

    // Simulate DHT peer discovery
    dhtListener({
      detail: { id: { toString: vi.fn().mockReturnValue('mock-dht-peer') } },
    });

    // Verify the listener was called
    expect(mockLibp2p.services.dht.addEventListener).toHaveBeenCalled();
  });

  it('should handle direct message protocol', async () => {
    await client.start();

    // Get the protocol handler
    const protocolCall = mockLibp2p.handle.mock.calls.find(
      (call: any) => call[0] === '/gigi/direct/1.0.0'
    );

    expect(protocolCall).toBeDefined();
    const protocolHandler = protocolCall![1];

    // Simulate a direct message
    const mockStream = {
      [Symbol.asyncIterator]: async function* () {
        yield new TextEncoder().encode('Direct message');
      },
    };

    await protocolHandler({
      stream: mockStream,
      connection: {
        remotePeer: { toString: vi.fn().mockReturnValue('mock-sender') },
      },
    });

    // Verify event was emitted
    expect(mockEventEmitter.emit).toHaveBeenCalled();
  });

  it('should handle file download with error', async () => {
    await client.start();

    // Mock sendFileMessage to return error
    const originalSendFileMessage = (client as any).sendFileMessage;
    (client as any).sendFileMessage = vi.fn().mockResolvedValue({
      type: 'error',
      message: 'File not found',
    });

    await expect(
      client.downloadFile('test-peer', 'invalid-share-code')
    ).rejects.toThrow();

    // Restore original method
    (client as any).sendFileMessage = originalSendFileMessage;
  });

  it('should handle connectToPeer method', async () => {
    await client.start();

    await expect(
      client.connectToPeer('/ip4/127.0.0.1/tcp/1234')
    ).resolves.not.toThrow();
    expect(mockLibp2p.dial).toHaveBeenCalled();
  });

  it('should handle offEvent method', async () => {
    const listener = vi.fn();
    client.onEvent(listener);
    client.offEvent(listener);
    expect(mockEventEmitter.off).toHaveBeenCalled();
  });

  it('should handle waitForEvent method', async () => {
    await client.start();

    // Test timeout case
    const timeoutPromise = client.waitForEvent('test-event', 100);
    await expect(timeoutPromise).rejects.toThrow();
  });

  it('should handle revokeFile method with non-existent file', async () => {
    await client.start();
    await expect(
      client.revokeFile('non-existent-share-code')
    ).rejects.toThrow();
  });

  it('should handle listSharedFiles method', async () => {
    await client.start();
    const files = client.listSharedFiles();
    expect(Array.isArray(files)).toBe(true);
  });

  it('should handle getFileByShareCode method', async () => {
    await client.start();
    const file = client.getFileByShareCode('test-share-code');
    expect(file).toBeUndefined();
  });

  it('should handle getActiveDownloads method', async () => {
    await client.start();
    const downloads = client.getActiveDownloads();
    expect(Array.isArray(downloads)).toBe(true);
  });

  it('should handle cancelDownload method', async () => {
    await client.start();
    await expect(
      client.cancelDownload('test-download-id')
    ).resolves.not.toThrow();
  });

  it('should handle listConnectedPeers method', async () => {
    await client.start();
    const peers = client.listConnectedPeers();
    expect(Array.isArray(peers)).toBe(true);
  });

  it('should handle readStreamMessage with different stream types', async () => {
    await client.start();

    // Test with async iterator
    const asyncIteratorStream = {
      [Symbol.asyncIterator]: async function* () {
        yield new TextEncoder().encode('Hello');
        yield new TextEncoder().encode(' World');
      },
    };

    const result1 = await (client as any).readStreamMessage(
      asyncIteratorStream
    );
    expect(result1).toBe('Hello World');

    // Test with nested stream
    const nestedStream = {
      stream: {
        [Symbol.asyncIterator]: async function* () {
          yield new TextEncoder().encode('Nested');
        },
      },
    };

    const result2 = await (client as any).readStreamMessage(nestedStream);
    expect(result2).toBe('Nested');

    // Test with empty stream
    const emptyStream = {
      [Symbol.asyncIterator]: async function* () {},
    };

    const result3 = await (client as any).readStreamMessage(emptyStream);
    expect(result3).toBe('');
  });

  it('should handle handleFileChunk method', async () => {
    await client.start();

    // Add a mock download
    const downloadId = 'test-download-id';
    const download = {
      downloadId,
      filename: 'test.txt',
      shareCode: 'test-share-code',
      fromPeerId: 'test-peer',
      fromNickname: 'test-nickname',
      totalChunks: 2,
      downloadedChunks: 1,
      startedAt: Date.now(),
      completed: false,
      failed: false,
      data: [],
    };

    (client as any).downloadManager.add(download);

    // Mock event emitter
    mockEventEmitter.emit = vi.fn().mockResolvedValue(undefined);

    // Create a mock channel
    const mockChannel = {
      send: vi.fn(),
    };

    // Test with incomplete download
    const chunkRequest = {
      type: 'chunk',
      downloadId,
      shareCode: 'test-share-code',
      chunkIndex: 1,
      totalChunks: 2,
      chunk: new Uint8Array([1, 2, 3]),
    };

    await (client as any).handleFileChunk(
      'test-peer',
      chunkRequest,
      mockChannel
    );
    expect(mockChannel.send).toHaveBeenCalled();
  });

  it('should handle sendDirectMessage method', async () => {
    await client.start();

    // Test with send method
    const mockStream = {
      send: vi.fn().mockResolvedValue(undefined),
      close: vi.fn().mockResolvedValue(undefined),
    };

    mockLibp2p.dialProtocol.mockResolvedValue(mockStream);
    await client.sendDirectMessage('mock-peer-id', 'Hello');
    expect(mockStream.send).toHaveBeenCalled();
  });

  it('should handle sendDirectMessageToNickname method', async () => {
    await client.start();

    // Add a mock peer
    client.addPeer('test-nickname', 'test-peer-id', [
      '/ip4/127.0.0.1/tcp/1234',
    ]);

    // Mock sendDirectMessage
    const originalSendDirectMessage = client.sendDirectMessage;
    (client as any).sendDirectMessage = vi.fn().mockResolvedValue(undefined);

    await client.sendDirectMessageToNickname('test-nickname', 'Hello');
    expect((client as any).sendDirectMessage).toHaveBeenCalledWith(
      'test-peer-id',
      'Hello'
    );

    // Restore original method
    (client as any).sendDirectMessage = originalSendDirectMessage;
  });

  it('should handle joinGroup and leaveGroup methods', async () => {
    await client.start();

    await client.joinGroup('test-group');
    expect(mockLibp2p.services.pubsub.subscribe).toHaveBeenCalledWith(
      'gigi-group:test-group'
    );

    await client.leaveGroup('test-group');
    expect(mockLibp2p.services.pubsub.unsubscribe).toHaveBeenCalledWith(
      'gigi-group:test-group'
    );
  });

  it('should handle sendGroupMessage method', async () => {
    await client.start();

    await client.sendGroupMessage('test-group', {
      type: 'text',
      text: 'Hello',
    });
    expect(mockLibp2p.services.pubsub.publish).toHaveBeenCalled();
  });

  it('should handle getJoinedGroups method', async () => {
    await client.start();

    await client.joinGroup('test-group');
    const groups = client.getJoinedGroups();
    expect(Array.isArray(groups)).toBe(true);
  });

  it('should handle getPeerByNickname and getPeerById methods', async () => {
    await client.start();

    // Add a mock peer
    client.addPeer('test-nickname', 'test-peer-id', [
      '/ip4/127.0.0.1/tcp/1234',
    ]);

    const peerByNickname = client.getPeerByNickname('test-nickname');
    expect(peerByNickname).toBeDefined();
    expect(peerByNickname?.nickname).toBe('test-nickname');

    const peerById = client.getPeerById('test-peer-id');
    expect(peerById).toBeDefined();
    expect(peerById?.peerId).toBe('test-peer-id');
  });

  it('should handle listPeers method', async () => {
    await client.start();

    // Add a mock peer
    client.addPeer('test-nickname', 'test-peer-id', [
      '/ip4/127.0.0.1/tcp/1234',
    ]);

    const peers = client.listPeers();
    expect(Array.isArray(peers)).toBe(true);
    expect(peers.length).toBeGreaterThan(0);
  });

  it('should handle shareFile method', async () => {
    await client.start();

    // Mock fileManager.share
    const originalShare = (client as any).fileManager.share;
    (client as any).fileManager.share = vi.fn().mockResolvedValue({
      fileId: 'test-file-id',
      info: {
        name: 'test.txt',
        size: 1024,
        mimeType: 'text/plain',
        chunkCount: 1,
        hash: 'test-hash',
      },
      shareCode: 'test-share-code',
    });

    const shareCode = await client.shareFile('test.txt');
    expect(shareCode).toBe('test-share-code');

    // Restore original method
    (client as any).fileManager.share = originalShare;
  });

  it('should handle downloadFileByPeerId method', async () => {
    await client.start();

    // Mock sendFileMessage
    const originalSendFileMessage = (client as any).sendFileMessage;
    (client as any).sendFileMessage = vi
      .fn()
      .mockResolvedValueOnce({
        type: 'file-info',
        fileId: 'test-file-id',
        name: 'test.txt',
        size: 1024,
        mimeType: 'text/plain',
        chunkCount: 1,
        hash: 'test-hash',
      })
      .mockResolvedValueOnce({
        type: 'chunk',
        downloadId: 'test-download-id',
        chunkIndex: 0,
        totalChunks: 1,
        chunk: new Uint8Array([1, 2, 3]),
      });

    // Mock fileManager.saveFile
    const originalSaveFile = (client as any).fileManager.saveFile;
    (client as any).fileManager.saveFile = vi.fn().mockResolvedValue(undefined);

    const downloadId = await (client as any).downloadFileByPeerId(
      'test-peer-id',
      'test-nickname',
      'test-share-code'
    );
    expect(downloadId).toBeDefined();

    // Restore original methods
    (client as any).sendFileMessage = originalSendFileMessage;
    (client as any).fileManager.saveFile = originalSaveFile;
  });

  it('should handle downloadFileByPeerId with error response', async () => {
    await client.start();

    // Mock sendFileMessage to return error
    const originalSendFileMessage = (client as any).sendFileMessage;
    (client as any).sendFileMessage = vi.fn().mockResolvedValue({
      type: 'error',
      message: 'File not found',
    });

    await expect(
      (client as any).downloadFileByPeerId(
        'test-peer-id',
        'test-nickname',
        'test-share-code'
      )
    ).rejects.toThrow();

    // Restore original method
    (client as any).sendFileMessage = originalSendFileMessage;
  });

  it('should handle downloadFileByPeerId with network error', async () => {
    await client.start();

    // Mock sendFileMessage to throw error
    const originalSendFileMessage = (client as any).sendFileMessage;
    (client as any).sendFileMessage = vi
      .fn()
      .mockRejectedValue(new Error('Network error'));

    await expect(
      (client as any).downloadFileByPeerId(
        'test-peer-id',
        'test-nickname',
        'test-share-code'
      )
    ).rejects.toThrow();

    // Restore original method
    (client as any).sendFileMessage = originalSendFileMessage;
  });

  it('should handle getPeerId method when not started', () => {
    expect(() => client.getPeerId()).toThrow();
  });

  it('should handle getMultiaddrs method when not started', () => {
    expect(() => client.getMultiaddrs()).toThrow();
  });

  it('should handle sendDirectMessage when not started', async () => {
    await expect(
      client.sendDirectMessage('test-peer-id', 'Hello')
    ).rejects.toThrow();
  });

  it('should handle file request events', async () => {
    await client.start();

    // Mock the channel object
    const mockChannel = {
      send: vi.fn(),
    };

    // Call handleFileRequest directly
    await (client as any).handleFileRequest(
      'test-peer-id',
      {
        type: 'request',
        shareCode: 'test-share-code',
      },
      mockChannel
    );

    // Verify the channel.send was called with an error (file not found)
    expect(mockChannel.send).toHaveBeenCalledWith({
      type: 'error',
      message: 'File not found',
    });

    await client.stop();
  });

  it('should handle file chunk request events', async () => {
    await client.start();

    // Mock the channel object
    const mockChannel = {
      send: vi.fn(),
    };

    // Call handleFileChunkRequest directly
    await (client as any).handleFileChunkRequest(
      'test-peer-id',
      {
        type: 'chunk',
        shareCode: 'test-share-code',
        downloadId: 'test-download-id',
        chunkIndex: 0,
      },
      mockChannel
    );

    // Verify the channel.send was called with an error (file not found)
    expect(mockChannel.send).toHaveBeenCalledWith({
      type: 'error',
      message: 'File not found',
    });

    await client.stop();
  });

  it('should handle peer:connect event without detail', async () => {
    await client.start();

    // Mock the libp2p addEventListener method
    const mockAddEventListener = vi.spyOn(
      (client as any).libp2p,
      'addEventListener'
    );

    // Get the peer:connect event handler
    let connectHandler: any;
    mockAddEventListener.mock.calls.forEach((call) => {
      if (call[0] === 'peer:connect') {
        connectHandler = call[1];
      }
    });

    // Trigger the event handler without detail
    if (connectHandler) {
      await connectHandler({});
    }

    await client.stop();
  });

  it('should handle peer:disconnect event without detail', async () => {
    await client.start();

    // Mock the libp2p addEventListener method
    const mockAddEventListener = vi.spyOn(
      (client as any).libp2p,
      'addEventListener'
    );

    // Get the peer:disconnect event handler
    let disconnectHandler: any;
    mockAddEventListener.mock.calls.forEach((call) => {
      if (call[0] === 'peer:disconnect') {
        disconnectHandler = call[1];
      }
    });

    // Trigger the event handler without detail
    if (disconnectHandler) {
      await disconnectHandler({});
    }

    await client.stop();
  });

  it('should handle pubsub message event without detail', async () => {
    await client.start();

    // Mock the pubsub addEventListener method
    const mockAddEventListener = vi.spyOn(
      (client as any).libp2p.services.pubsub,
      'addEventListener'
    );

    // Get the message event handler
    let messageHandler: any;
    mockAddEventListener.mock.calls.forEach((call) => {
      if (call[0] === 'message') {
        messageHandler = call[1];
      }
    });

    // Trigger the event handler without detail
    if (messageHandler) {
      await messageHandler({});
    }

    await client.stop();
  });

  it('should handle pubsub message event without topic', async () => {
    await client.start();

    // Mock the pubsub addEventListener method
    const mockAddEventListener = vi.spyOn(
      (client as any).libp2p.services.pubsub,
      'addEventListener'
    );

    // Get the message event handler
    let messageHandler: any;
    mockAddEventListener.mock.calls.forEach((call) => {
      if (call[0] === 'message') {
        messageHandler = call[1];
      }
    });

    // Trigger the event handler without topic
    if (messageHandler) {
      await messageHandler({ detail: {} });
    }

    await client.stop();
  });

  it('should handle pubsub message event without data', async () => {
    await client.start();

    // Mock the pubsub addEventListener method
    const mockAddEventListener = vi.spyOn(
      (client as any).libp2p.services.pubsub,
      'addEventListener'
    );

    // Get the message event handler
    let messageHandler: any;
    mockAddEventListener.mock.calls.forEach((call) => {
      if (call[0] === 'message') {
        messageHandler = call[1];
      }
    });

    // Trigger the event handler without data
    if (messageHandler) {
      await messageHandler({ detail: { topic: 'gigi-group:test-group' } });
    }

    await client.stop();
  });

  it('should handle direct message protocol without stream', async () => {
    await client.start();

    // Mock the libp2p handle method
    const mockHandle = vi.spyOn((client as any).libp2p, 'handle');

    // Get the protocol handler
    let protocolHandler: any;
    mockHandle.mock.calls.forEach((call) => {
      if (call[0] === '/gigi/direct/1.0.0') {
        protocolHandler = call[1];
      }
    });

    // Trigger the handler without stream
    if (protocolHandler) {
      await protocolHandler({
        connection: { remotePeer: { toString: () => 'test-peer-id' } },
      });
    }

    await client.stop();
  });

  it('should test getJoinedGroups method', async () => {
    await client.start();
    const groups = client.getJoinedGroups();
    expect(Array.isArray(groups)).toBe(true);
    await client.stop();
  });

  it('should test shareFile method logging', async () => {
    await client.start();

    // Mock the fileManager.share method
    vi.spyOn((client as any).fileManager, 'share').mockReturnValue({
      info: { name: 'test.txt' },
      shareCode: 'test-share-code',
    });

    const shareCode = await client.shareFile('./test.txt');
    expect(shareCode).toBe('test-share-code');

    await client.stop();
  });

  it('should test downloadFileByPeerId when file not found', async () => {
    await client.start();

    // Mock the peerManager.getPeerId method
    vi.spyOn((client as any).peerManager, 'getPeerId').mockReturnValue(
      'test-peer-id'
    );

    // Mock the sendFileMessage method to return an error
    vi.spyOn(client as any, 'sendFileMessage').mockRejectedValue(
      new Error('File not found')
    );

    await expect(
      client.downloadFileByPeerId(
        'test-peer-id',
        'non-existent-share-code',
        './downloads'
      )
    ).rejects.toThrow();

    await client.stop();
  });

  it('should test stream writing with write method', async () => {
    await client.start();

    // Mock the libp2p.dialProtocol method to return a stream with write method
    const mockStream = {
      write: vi.fn((data, callback) => callback(null)),
      end: vi.fn((callback) => callback()),
    };

    vi.spyOn((client as any).libp2p, 'dialProtocol').mockResolvedValue(
      mockStream as any
    );

    await client.sendDirectMessage('test-peer-id', 'Hello');

    expect(mockStream.write).toHaveBeenCalled();
    expect(mockStream.end).toHaveBeenCalled();

    await client.stop();
  });

  it('should test stream writing with sink method', async () => {
    await client.start();

    // Mock the libp2p.dialProtocol method to return a stream with sink method
    const mockStream = {
      sink: vi.fn().mockResolvedValue(undefined),
    };

    vi.spyOn((client as any).libp2p, 'dialProtocol').mockResolvedValue(
      mockStream as any
    );

    await client.sendDirectMessage('test-peer-id', 'Hello');

    expect(mockStream.sink).toHaveBeenCalled();

    await client.stop();
  });

  it('should test stream writing with Web Streams API', async () => {
    await client.start();

    // Mock the libp2p.dialProtocol method to return a stream with writable property
    const mockWriter = {
      write: vi.fn().mockResolvedValue(undefined),
      close: vi.fn().mockResolvedValue(undefined),
    };

    const mockStream = {
      writable: {
        writable: true,
        getWriter: vi.fn().mockReturnValue(mockWriter),
      },
    };

    vi.spyOn((client as any).libp2p, 'dialProtocol').mockResolvedValue(
      mockStream as any
    );

    await client.sendDirectMessage('test-peer-id', 'Hello');

    expect(mockStream.writable.getWriter).toHaveBeenCalled();
    expect(mockWriter.write).toHaveBeenCalled();
    expect(mockWriter.close).toHaveBeenCalled();

    await client.stop();
  });

  it('should test stream writing with error', async () => {
    await client.start();

    // Mock the libp2p.dialProtocol method to return a stream with write method that errors
    const mockStream = {
      write: vi.fn((data, callback) => callback(new Error('Write error'))),
    };

    vi.spyOn((client as any).libp2p, 'dialProtocol').mockResolvedValue(
      mockStream as any
    );

    await expect(
      client.sendDirectMessage('test-peer-id', 'Hello')
    ).rejects.toThrow();

    await client.stop();
  });

  it('should test stream writing with no write method', async () => {
    await client.start();

    // Mock the libp2p.dialProtocol method to return a stream with no write methods
    const mockStream = {};

    vi.spyOn((client as any).libp2p, 'dialProtocol').mockResolvedValue(
      mockStream as any
    );

    await expect(
      client.sendDirectMessage('test-peer-id', 'Hello')
    ).rejects.toThrow('Network error');

    await client.stop();
  });

  it('should test readStreamMessage with error event', async () => {
    await client.start();

    // Create a mock stream with error event
    const mockStream = {
      on: vi.fn((event, callback) => {
        if (event === 'error') {
          // Trigger error immediately
          setTimeout(() => callback(new Error('Stream error')), 10);
        }
      }),
    };

    // Call readStreamMessage directly
    const readStreamMessage = (client as any).readStreamMessage;
    await expect(readStreamMessage(mockStream as any)).rejects.toThrow(
      'Stream error'
    );

    await client.stop();
  });

  it('should handle sendDirectMessageToNickname when peer not found', async () => {
    await client.start();
    await expect(
      client.sendDirectMessageToNickname('non-existent-nickname', 'Hello')
    ).rejects.toThrow();
  });

  it('should handle joinGroup when not started', async () => {
    await expect(client.joinGroup('test-group')).rejects.toThrow();
  });

  it('should handle leaveGroup when not started', async () => {
    await expect(client.leaveGroup('test-group')).rejects.toThrow();
  });

  it('should handle sendGroupMessage when not started', async () => {
    await expect(
      client.sendGroupMessage('test-group', { type: 'text', text: 'Hello' })
    ).rejects.toThrow();
  });

  it('should handle shareFile when not started', async () => {
    await expect(client.shareFile('test.txt')).rejects.toThrow();
  });

  it('should handle downloadFile when not started', async () => {
    await expect(
      client.downloadFile('test-peer', 'test-share-code')
    ).rejects.toThrow();
  });

  it('should handle downloadFile when peer not found', async () => {
    await client.start();
    await expect(
      client.downloadFile('non-existent-peer', 'test-share-code')
    ).rejects.toThrow();
  });

  it('should handle connectToPeer when not started', async () => {
    await expect(
      client.connectToPeer('/ip4/127.0.0.1/tcp/1234')
    ).rejects.toThrow();
  });

  it('should handle connectToPeer with connection error', async () => {
    await client.start();
    mockLibp2p.dial.mockRejectedValue(new Error('Connection failed'));
    await expect(
      client.connectToPeer('/ip4/127.0.0.1/tcp/1234')
    ).rejects.toThrow();
  });

  it('should handle handleFileRequest method', async () => {
    await client.start();

    // Add a mock file to fileManager
    const mockFile = {
      fileId: 'test-file-id',
      info: {
        name: 'test.txt',
        size: 1024,
        mimeType: 'text/plain',
        chunkCount: 1,
        hash: 'test-hash',
      },
      shareCode: 'test-share-code',
    };
    (client as any).fileManager.getByShareCode = vi
      .fn()
      .mockReturnValue(mockFile);

    const mockChannel = {
      send: vi.fn(),
    };

    await (client as any).handleFileRequest(
      'test-peer-id',
      {
        type: 'request',
        action: 'request',
        shareCode: 'test-share-code',
        downloadId: 'test-download-id',
      },
      mockChannel
    );

    expect(mockChannel.send).toHaveBeenCalled();
  });

  it('should handle handleFileRequest with file not found', async () => {
    await client.start();

    // Mock fileManager.getByShareCode to return undefined
    (client as any).fileManager.getByShareCode = vi
      .fn()
      .mockReturnValue(undefined);

    const mockChannel = {
      send: vi.fn(),
    };

    await (client as any).handleFileRequest(
      'test-peer-id',
      {
        type: 'request',
        action: 'request',
        shareCode: 'non-existent-share-code',
        downloadId: 'test-download-id',
      },
      mockChannel
    );

    expect(mockChannel.send).toHaveBeenCalledWith({
      type: 'error',
      message: 'File not found',
    });
  });

  it('should handle handleFileChunkRequest method', async () => {
    await client.start();

    // Add a mock file to fileManager
    const mockFile = {
      fileId: 'test-file-id',
      info: {
        name: 'test.txt',
      },
      shareCode: 'test-share-code',
    };
    (client as any).fileManager.getByShareCode = vi
      .fn()
      .mockReturnValue(mockFile);
    (client as any).fileManager.getChunk = vi
      .fn()
      .mockResolvedValue(new Uint8Array([1, 2, 3]));

    const mockChannel = {
      send: vi.fn(),
    };

    await (client as any).handleFileChunkRequest(
      'test-peer-id',
      {
        type: 'chunk',
        downloadId: 'test-download-id',
        shareCode: 'test-share-code',
        chunkIndex: 0,
        totalChunks: 1,
        chunk: new Uint8Array(0),
      },
      mockChannel
    );

    expect(mockChannel.send).toHaveBeenCalled();
  });

  it('should handle handleFileChunkRequest with file not found', async () => {
    await client.start();

    // Mock fileManager.getByShareCode to return undefined
    (client as any).fileManager.getByShareCode = vi
      .fn()
      .mockReturnValue(undefined);

    const mockChannel = {
      send: vi.fn(),
    };

    await (client as any).handleFileChunkRequest(
      'test-peer-id',
      {
        type: 'chunk',
        downloadId: 'test-download-id',
        shareCode: 'non-existent-share-code',
        chunkIndex: 0,
        totalChunks: 1,
        chunk: new Uint8Array(0),
      },
      mockChannel
    );

    expect(mockChannel.send).toHaveBeenCalledWith({
      type: 'error',
      message: 'File not found',
    });
  });

  it('should handle handleFileChunkRequest with error', async () => {
    await client.start();

    // Add a mock file to fileManager
    const mockFile = {
      fileId: 'test-file-id',
      info: {
        name: 'test.txt',
      },
      shareCode: 'test-share-code',
    };
    (client as any).fileManager.getByShareCode = vi
      .fn()
      .mockReturnValue(mockFile);
    (client as any).fileManager.getChunk = vi
      .fn()
      .mockRejectedValue(new Error('Chunk error'));

    const mockChannel = {
      send: vi.fn(),
    };

    await (client as any).handleFileChunkRequest(
      'test-peer-id',
      {
        type: 'chunk',
        downloadId: 'test-download-id',
        shareCode: 'test-share-code',
        chunkIndex: 0,
        totalChunks: 1,
        chunk: new Uint8Array(0),
      },
      mockChannel
    );

    expect(mockChannel.send).toHaveBeenCalledWith({
      type: 'error',
      message: 'Failed to send chunk',
    });
  });

  it('should handle handleFileChunk method', async () => {
    await client.start();

    // Add a mock download
    const downloadId = 'test-download-id';
    const download = {
      downloadId,
      filename: 'test.txt',
      shareCode: 'test-share-code',
      fromPeerId: 'test-peer',
      fromNickname: 'test-nickname',
      totalChunks: 1,
      downloadedChunks: 0,
      startedAt: Date.now(),
      completed: false,
      failed: false,
      data: [],
    };

    (client as any).downloadManager.add(download);

    // Mock fileManager.saveFile
    (client as any).fileManager.saveFile = vi.fn().mockResolvedValue(undefined);

    // Mock event emitter
    mockEventEmitter.emit = vi.fn().mockResolvedValue(undefined);

    // Create a mock channel
    const mockChannel = {
      send: vi.fn(),
    };

    // Test with complete download
    const chunkRequest = {
      type: 'chunk',
      downloadId,
      shareCode: 'test-share-code',
      chunkIndex: 0,
      totalChunks: 1,
      chunk: new Uint8Array([1, 2, 3]),
    };

    await (client as any).handleFileChunk(
      'test-peer',
      chunkRequest,
      mockChannel
    );
    expect(mockChannel.send).toHaveBeenCalled();
  });

  it('should handle handleFileChunk with download not found', async () => {
    await client.start();

    // Create a mock channel
    const mockChannel = {
      send: vi.fn(),
    };

    // Test with non-existent download
    const chunkRequest = {
      type: 'chunk',
      downloadId: 'non-existent-download-id',
      shareCode: 'test-share-code',
      chunkIndex: 0,
      totalChunks: 1,
      chunk: new Uint8Array([1, 2, 3]),
    };

    await (client as any).handleFileChunk(
      'test-peer',
      chunkRequest,
      mockChannel
    );
    expect(mockChannel.send).toHaveBeenCalledWith({
      type: 'error',
      message: 'Download not found',
    });
  });

  it('should handle sendFileMessage when not started', async () => {
    await expect(
      (client as any).sendFileMessage('test-peer-id', {
        type: 'request',
        action: 'request',
        shareCode: 'test-share-code',
        downloadId: 'test-download-id',
      })
    ).rejects.toThrow();
  });

  it('should handle readStreamMessage with different chunk types', async () => {
    await client.start();

    // Test with Buffer chunk
    const bufferStream = {
      [Symbol.asyncIterator]: async function* () {
        yield Buffer.from('Buffer chunk');
      },
    };

    const result1 = await (client as any).readStreamMessage(bufferStream);
    expect(result1).toBe('Buffer chunk');

    // Test with Array chunk
    const arrayStream = {
      [Symbol.asyncIterator]: async function* () {
        yield [72, 101, 108, 108, 111]; // ASCII for 'Hello'
      },
    };

    const result2 = await (client as any).readStreamMessage(arrayStream);
    expect(result2).toBe('Hello');

    // Test with Uint8ArrayList chunk
    const uint8ArrayListStream = {
      [Symbol.asyncIterator]: async function* () {
        yield { toUint8Array: () => new Uint8Array([87, 111, 114, 108, 100]) }; // ASCII for 'World'
      },
    };

    const result3 = await (client as any).readStreamMessage(
      uint8ArrayListStream
    );
    expect(result3).toBe('World');

    // Test with sliceable chunk
    const sliceableStream = {
      [Symbol.asyncIterator]: async function* () {
        yield { slice: () => [72, 101, 108, 108, 111] }; // ASCII for 'Hello'
      },
    };

    const result4 = await (client as any).readStreamMessage(sliceableStream);
    expect(result4).toBe('Hello');
  });

  it('should handle readStreamMessage with different stream types', async () => {
    await client.start();

    // Test with receive method
    const receiveStream = {
      receive: vi
        .fn()
        .mockResolvedValueOnce(Buffer.from('Receive test'))
        .mockResolvedValueOnce(null),
    };

    const result1 = await (client as any).readStreamMessage(receiveStream);
    expect(result1).toBe('Receive test');

    // Test with read method
    const readStream = {
      read: vi
        .fn()
        .mockReturnValueOnce(Buffer.from('Read test'))
        .mockReturnValueOnce(null),
    };

    const result2 = await (client as any).readStreamMessage(readStream);
    expect(result2).toBe('Read test');

    // Test with source property
    const sourceStream = {
      source: {
        [Symbol.asyncIterator]: async function* () {
          yield new Uint8Array([84, 101, 115, 116]); // ASCII for 'Test'
        },
      },
    };

    const result3 = await (client as any).readStreamMessage(sourceStream);
    expect(result3).toBe('Test');

    // Test with readable property (Web Streams API)
    const webStream = {
      readable: {
        getReader: vi.fn().mockReturnValue({
          read: vi
            .fn()
            .mockResolvedValueOnce({
              done: false,
              value: new Uint8Array([87, 101, 98]),
            }) // ASCII for 'Web'
            .mockResolvedValueOnce({ done: true, value: undefined }),
        }),
      },
    };

    const result4 = await (client as any).readStreamMessage(webStream);
    expect(result4).toBe('Web');

    // Test with event emitter pattern
    const eventStream = {
      on: vi.fn().mockImplementation((event, callback) => {
        if (event === 'data') {
          setTimeout(() => {
            callback(Buffer.from('Event test'));
          }, 10);
        } else if (event === 'end') {
          setTimeout(() => callback(), 20);
        } else if (event === 'error') {
          // Do nothing
        }
        return eventStream;
      }),
    };

    const result5 = await (client as any).readStreamMessage(eventStream);
    expect(result5).toBe('Event test');
  });

  it('should handle readStreamMessage with error', async () => {
    await client.start();

    // Test with error in stream
    const errorStream = {
      [Symbol.asyncIterator]: async function* () {
        throw new Error('Stream error');
      },
    };

    await expect(
      (client as any).readStreamMessage(errorStream)
    ).rejects.toThrow('Stream error');
  });

  it('should handle readStreamMessage with unsupported stream type', async () => {
    await client.start();

    // Test with unsupported stream type
    const unsupportedStream = {};

    await expect(
      (client as any).readStreamMessage(unsupportedStream)
    ).rejects.toThrow('Unsupported stream type');
  });

  it('should handle readStreamMessage with empty stream', async () => {
    await client.start();

    // Test with empty stream
    const emptyStream = {
      [Symbol.asyncIterator]: async function* () {},
    };

    const result = await (client as any).readStreamMessage(emptyStream);
    expect(result).toBe('');
  });
});
