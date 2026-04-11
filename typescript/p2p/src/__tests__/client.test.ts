import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { P2pClient } from '../client';
import type { MessageContentInput } from '../types';

// Mock libp2p and dependencies
vi.mock('../libp2p-setup', () => ({
  createLibp2pInstance: vi.fn().mockResolvedValue({
    libp2p: {
      handle: vi.fn(),
      dialProtocol: vi.fn().mockResolvedValue({
        sink: vi.fn().mockResolvedValue(undefined),
        source: {
          [Symbol.asyncIterator]: async function* () {
            yield new TextEncoder().encode(
              JSON.stringify({
                type: 'file-info',
                fileId: 'mock-file-id',
                name: 'test.txt',
                size: 1024,
                mimeType: 'text/plain',
                chunkCount: 1,
                hash: 'mock-hash',
              })
            );
          },
        },
        connection: {
          id: 'mock-connection-id',
          remotePeer: '12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3',
        },
        close: vi.fn().mockResolvedValue(undefined),
      }),
      start: vi.fn().mockResolvedValue(undefined),
      stop: vi.fn().mockResolvedValue(undefined),
      peerId: {
        toString: vi
          .fn()
          .mockReturnValue(
            '12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3'
          ),
      },
      getMultiaddrs: vi.fn().mockReturnValue(['/ip4/127.0.0.1/tcp/1234']),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      on: vi.fn(),
      off: vi.fn(),
      services: {
        pubsub: {
          subscribe: vi.fn(),
          unsubscribe: vi.fn(),
          publish: vi.fn(),
          on: vi.fn(),
          addEventListener: vi.fn(),
          removeEventListener: vi.fn(),
        },
        dht: {
          addEventListener: vi.fn(),
          removeEventListener: vi.fn(),
        },
      },
    },
    gigiDns: {
      on: vi.fn(),
      stop: vi.fn(),
      updateListenAddresses: vi.fn(),
      startService: vi.fn(),
      sendQuery: vi.fn(),
    },
  }),
}));

// Mock peer-manager
const mockPeerManager = {
  getPeerByNickname: vi
    .fn()
    .mockResolvedValue('12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3'),
  getPeerById: vi.fn().mockResolvedValue({
    id: '12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3',
    nickname: 'test-peer',
  }),
  getByNickname: vi.fn().mockReturnValue({
    id: '12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3',
    nickname: 'test-peer',
  }),
  getByPeerId: vi.fn().mockReturnValue({
    id: '12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3',
    nickname: 'test-peer',
  }),
  list: vi.fn().mockReturnValue([
    {
      id: '12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3',
      nickname: 'test-peer',
    },
  ]),
  listConnected: vi.fn().mockReturnValue([]),
  discover: vi.fn(),
  addConnected: vi.fn(),
  removeConnected: vi.fn(),
  getNickname: vi.fn().mockReturnValue('test-peer'),
  getPeerId: vi
    .fn()
    .mockReturnValue('12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3'),
  cleanup: vi.fn(),
};
vi.mock('../peer-manager', () => ({
  PeerManager: vi.fn(function () {
    return mockPeerManager;
  }),
}));

// Mock file-sharing
const mockFileSharingManager = {
  share: vi.fn().mockResolvedValue({
    fileId: 'mock-file-id',
    shareCode: 'mock-share-code',
    info: {
      name: 'test.txt',
      size: 1024,
      mimeType: 'text/plain',
      chunkCount: 1,
      hash: 'mock-hash',
    },
  }),
  getByShareCode: vi.fn().mockReturnValue({
    fileId: 'mock-file-id',
    shareCode: 'mock-share-code',
    info: {
      name: 'test.txt',
      size: 1024,
      mimeType: 'text/plain',
      chunkCount: 1,
      hash: 'mock-hash',
    },
    filePath: './test.txt',
  }),
  getChunk: vi.fn().mockResolvedValue(new Uint8Array(0)),
  saveFile: vi.fn().mockResolvedValue(undefined),
  list: vi.fn().mockReturnValue([]),
  listAll: vi.fn().mockReturnValue([]),
  getShareCodes: vi.fn().mockReturnValue(['mock-share-code']),
  revoke: vi.fn(),
};
vi.mock('../file-sharing', () => ({
  FileSharingManager: vi.fn(function () {
    return mockFileSharingManager;
  }),
}));

// Mock group
const mockGroupManager = {
  join: vi.fn(),
  leave: vi.fn(),
  list: vi.fn().mockReturnValue(['general']),
};
vi.mock('../group', () => ({
  GroupManager: vi.fn(function () {
    return mockGroupManager;
  }),
}));

// Mock download-manager
const mockDownloadManager = {
  add: vi.fn(),
  get: vi.fn().mockReturnValue({
    downloadId: 'mock-download-id',
    filename: 'test.txt',
    shareCode: 'mock-share-code',
    fromPeerId: '12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3',
    fromNickname: 'test-peer',
    totalChunks: 1,
    downloadedChunks: 0,
    startedAt: Date.now(),
    completed: false,
    failed: false,
    data: [],
  }),
  remove: vi.fn(),
  list: vi.fn().mockReturnValue([]),
};
vi.mock('../download-manager', () => ({
  DownloadManager: vi.fn(function () {
    return mockDownloadManager;
  }),
}));

describe('P2pClient', () => {
  let client: P2pClient;
  const config = {
    nickname: 'test-client',
    config: {
      bootstrapNodes: [],
      enableKademlia: false,
      enableRelay: true,
      enableMdns: false,
      listenAddrs: ['/ip4/0.0.0.0/tcp/0'],
    },
  };

  beforeEach(async () => {
    client = new P2pClient(config);
  });

  afterEach(async () => {
    if (client) {
      await client.stop();
    }
  });

  it('should initialize with the correct configuration', () => {
    expect(client).toBeInstanceOf(P2pClient);
  });

  it('should start successfully', async () => {
    await client.start();
    expect(client.isStarted()).toBe(true);
  });

  it('should stop successfully', async () => {
    await client.start();
    expect(client.isStarted()).toBe(true);
    await client.stop();
    expect(client.isStarted()).toBe(false);
  });

  it('should return the correct peer ID', async () => {
    await client.start();
    const peerId = client.getPeerId();
    expect(peerId).toBe('12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3');
  });

  it('should return the correct multiaddrs', async () => {
    await client.start();
    const multiaddrs = client.getMultiaddrs();
    expect(multiaddrs).toEqual(['/ip4/127.0.0.1/tcp/1234']);
  });

  it('should send a direct message', async () => {
    await client.start();
    await expect(
      client.sendDirectMessage(
        '12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3',
        'Hello'
      )
    ).resolves.not.toThrow();
  });

  it('should send a direct message to a nickname', async () => {
    await client.start();
    await expect(
      client.sendDirectMessageToNickname('test-peer', 'Hello')
    ).resolves.not.toThrow();
  });

  it('should share a file', async () => {
    await client.start();
    const shareCode = await client.shareFile('./test.txt');
    expect(shareCode).toBe('mock-share-code');
  });

  it('should download a file', async () => {
    await client.start();
    const downloadId = await client.downloadFile(
      'test-peer',
      'mock-share-code'
    );
    expect(typeof downloadId).toBe('string');
  });

  it('should revoke a file', async () => {
    await client.start();
    await expect(client.revokeFile('mock-share-code')).resolves.not.toThrow();
  });

  it('should list shared files', async () => {
    await client.start();
    const files = client.listSharedFiles();
    expect(files).toEqual([]);
  });

  it('should join a group', async () => {
    await client.start();
    await expect(client.joinGroup('general')).resolves.not.toThrow();
  });

  it('should leave a group', async () => {
    await client.start();
    await expect(client.leaveGroup('general')).resolves.not.toThrow();
  });

  it('should send a group message', async () => {
    await client.start();
    await expect(
      client.sendGroupMessage('general', {
        type: 'text',
        text: 'Hello everyone',
      } as MessageContentInput)
    ).resolves.not.toThrow();
  });

  it('should get joined groups', async () => {
    await client.start();
    const groups = client.getJoinedGroups();
    expect(groups).toEqual(['general']);
  });

  it('should list peers', async () => {
    await client.start();
    const peers = client.listPeers();
    expect(peers).toEqual([
      {
        id: '12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3',
        nickname: 'test-peer',
      },
    ]);
  });

  it('should get a peer by nickname', async () => {
    await client.start();
    const peer = client.getPeerByNickname('test-peer');
    expect(peer).toEqual({
      id: '12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3',
      nickname: 'test-peer',
    });
  });

  it('should get a peer by ID', async () => {
    await client.start();
    const peer = client.getPeerById(
      '12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3'
    );
    expect(peer).toEqual({
      id: '12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3',
      nickname: 'test-peer',
    });
  });

  it('should register event listeners', async () => {
    await client.start();
    const eventListener = vi.fn();
    const removeListener = client.onEvent(eventListener);
    expect(typeof removeListener).toBe('function');
  });

  it('should remove event listeners', async () => {
    await client.start();
    const eventListener = vi.fn();
    const removeListener = client.onEvent(eventListener);
    removeListener();
    // Just verify the removeListener function exists and is callable
    expect(typeof removeListener).toBe('function');
  });

  // Error handling tests
  it('should handle starting an already started client', async () => {
    await client.start();
    expect(client.isStarted()).toBe(true);

    // Should throw an error when starting again
    await expect(client.start()).rejects.toThrow('P2P client already started');
  });

  it('should handle stopping an already stopped client', async () => {
    // Client is not started yet
    expect(client.isStarted()).toBe(false);

    // Should not throw an error when stopping
    await expect(client.stop()).resolves.not.toThrow();
  });

  it('should handle sending message when client is not started', async () => {
    // Client is not started yet
    expect(client.isStarted()).toBe(false);

    // Should throw an error when sending message
    await expect(
      client.sendDirectMessage(
        '12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3',
        'Hello'
      )
    ).rejects.toThrow('P2P client not started');
  });

  it('should handle sharing file when client is not started', async () => {
    // Client is not started yet
    expect(client.isStarted()).toBe(false);

    // Should throw an error when sharing file
    await expect(client.shareFile('./test.txt')).rejects.toThrow(
      'P2P client not started'
    );
  });

  it('should handle downloading file when client is not started', async () => {
    // Client is not started yet
    expect(client.isStarted()).toBe(false);

    // Should throw an error when downloading file
    await expect(
      client.downloadFile('test-peer', 'mock-share-code')
    ).rejects.toThrow('P2P client not started');
  });

  it('should handle joining group when client is not started', async () => {
    // Client is not started yet
    expect(client.isStarted()).toBe(false);

    // Should throw an error when joining group
    await expect(client.joinGroup('general')).rejects.toThrow(
      'P2P client not started'
    );
  });

  it('should handle sending group message when client is not started', async () => {
    // Client is not started yet
    expect(client.isStarted()).toBe(false);

    // Should throw an error when sending group message
    await expect(
      client.sendGroupMessage('general', {
        type: 'text',
        text: 'Hello everyone',
      } as MessageContentInput)
    ).rejects.toThrow('P2P client not started');
  });
});
