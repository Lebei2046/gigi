import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { P2pClient } from './client.js';

// Mock libp2p and dependencies
vi.mock('./libp2p-setup.js', () => ({
  createLibp2pInstance: vi.fn().mockResolvedValue({
    handle: vi.fn(),
    dialProtocol: vi.fn().mockResolvedValue({
      sink: vi.fn().mockResolvedValue(undefined),
      source: {
        [Symbol.asyncIterator]: async function* () {
          yield new TextEncoder().encode(JSON.stringify({
            type: 'pong'
          }));
        }
      },
      connection: {
        id: 'mock-connection-id',
        remotePeer: 'mock-peer-id'
      },
      close: vi.fn().mockResolvedValue(undefined)
    }),
    start: vi.fn().mockResolvedValue(undefined),
    stop: vi.fn().mockResolvedValue(undefined),
    peerId: {
      toString: vi.fn().mockReturnValue('mock-peer-id')
    },
    getMultiaddrs: vi.fn().mockReturnValue(['/ip4/127.0.0.1/tcp/1234'])
  })
}));

vi.mock('./peer-manager.js', () => ({
  PeerManager: vi.fn().mockImplementation(() => ({
    getPeerByNickname: vi.fn().mockResolvedValue('mock-peer-id'),
    getPeerById: vi.fn().mockResolvedValue({ id: 'mock-peer-id', nickname: 'test-peer' }),
    listPeers: vi.fn().mockReturnValue([{ id: 'mock-peer-id', nickname: 'test-peer' }]),
    addPeer: vi.fn(),
    removePeer: vi.fn()
  }))
}));

vi.mock('./file-sharing.js', () => ({
  FileSharingManager: vi.fn().mockImplementation(() => ({
    share: vi.fn().mockResolvedValue('mock-share-code'),
    getFileInfo: vi.fn().mockResolvedValue({
      filename: 'test.txt',
      size: 1024,
      chunks: 1
    }),
    download: vi.fn().mockResolvedValue('mock-download-id'),
    listSharedFiles: vi.fn().mockReturnValue([]),
    removeSharedFile: vi.fn()
  }))
}));

vi.mock('./group.js', () => ({
  GroupManager: vi.fn().mockImplementation(() => ({
    joinGroup: vi.fn().mockResolvedValue(undefined),
    leaveGroup: vi.fn().mockResolvedValue(undefined),
    sendGroupMessage: vi.fn().mockResolvedValue(undefined),
    getJoinedGroups: vi.fn().mockReturnValue(['general'])
  }))
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
      listenAddrs: ['/ip4/0.0.0.0/tcp/0']
    }
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
    expect(peerId).toBe('mock-peer-id');
  });

  it('should return the correct multiaddrs', async () => {
    await client.start();
    const multiaddrs = client.getMultiaddrs();
    expect(multiaddrs).toEqual(['/ip4/127.0.0.1/tcp/1234']);
  });

  it('should send a direct message', async () => {
    await client.start();
    await expect(client.sendDirectMessage('mock-peer-id', 'Hello')).resolves.not.toThrow();
  });

  it('should send a direct message to a nickname', async () => {
    await client.start();
    await expect(client.sendDirectMessageToNickname('test-peer', 'Hello')).resolves.not.toThrow();
  });

  it('should share a file', async () => {
    await client.start();
    const shareCode = await client.shareFile('./test.txt');
    expect(shareCode).toBe('mock-share-code');
  });

  it('should download a file', async () => {
    await client.start();
    const downloadId = await client.downloadFile('test-peer', 'mock-share-code');
    expect(downloadId).toBe('mock-download-id');
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
    await expect(client.sendGroupMessage('general', 'Hello everyone')).resolves.not.toThrow();
  });

  it('should get joined groups', async () => {
    await client.start();
    const groups = client.getJoinedGroups();
    expect(groups).toEqual(['general']);
  });

  it('should list peers', async () => {
    await client.start();
    const peers = client.listPeers();
    expect(peers).toEqual([{ id: 'mock-peer-id', nickname: 'test-peer' }]);
  });

  it('should get a peer by nickname', async () => {
    await client.start();
    const peer = await client.getPeerByNickname('test-peer');
    expect(peer).toBe('mock-peer-id');
  });

  it('should get a peer by ID', async () => {
    await client.start();
    const peer = await client.getPeerById('mock-peer-id');
    expect(peer).toEqual({ id: 'mock-peer-id', nickname: 'test-peer' });
  });

  it('should emit events', async () => {
    await client.start();
    const eventListener = vi.fn();
    client.onEvent(eventListener);
    
    // Simulate an event
    const testEvent = {
      type: 'test-event',
      data: 'test-data'
    };
    
    // Access the private emitEvent method to test event emission
    // @ts-ignore - Accessing private method for testing
    client.emitEvent(testEvent);
    
    expect(eventListener).toHaveBeenCalledWith(testEvent);
  });

  it('should remove event listeners', async () => {
    await client.start();
    const eventListener = vi.fn();
    const removeListener = client.onEvent(eventListener);
    
    // Remove the listener
    removeListener();
    
    // Simulate an event
    const testEvent = {
      type: 'test-event',
      data: 'test-data'
    };
    
    // @ts-ignore - Accessing private method for testing
    client.emitEvent(testEvent);
    
    expect(eventListener).not.toHaveBeenCalled();
  });
});
