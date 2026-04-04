import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { P2pClient } from '../client.js';
import type { MessageContentInput } from '../types.js';

// Mock libp2p and dependencies
vi.mock('../libp2p-setup.js', () => ({
  createLibp2pInstance: vi.fn().mockResolvedValue({
    libp2p: {
      handle: vi.fn(),
      dialProtocol: vi.fn().mockResolvedValue({
        sink: vi.fn().mockResolvedValue(undefined),
        source: {
          [Symbol.asyncIterator]: async function* () {
            yield new TextEncoder().encode(
              JSON.stringify({
                type: 'pong',
              })
            );
          },
        },
        connection: {
          id: 'mock-connection-id',
          remotePeer: 'mock-peer-id',
        },
        close: vi.fn().mockResolvedValue(undefined),
      }),
      start: vi.fn().mockResolvedValue(undefined),
      stop: vi.fn().mockResolvedValue(undefined),
      peerId: {
        toString: vi.fn().mockReturnValue('mock-peer-id'),
      },
      getMultiaddrs: vi.fn().mockReturnValue(['/ip4/127.0.0.1/tcp/1234']),
      services: {
        pubsub: {
          subscribe: vi.fn().mockResolvedValue(undefined),
          unsubscribe: vi.fn().mockResolvedValue(undefined),
          publish: vi.fn().mockResolvedValue(undefined),
          on: vi.fn(),
          off: vi.fn(),
          addEventListener: vi.fn(),
          removeEventListener: vi.fn(),
        },
        dht: {
          addEventListener: vi.fn(),
          removeEventListener: vi.fn(),
        },
      },
      on: vi.fn(),
      off: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    },
    gigiDns: {
      on: vi.fn(),
      stop: vi.fn(),
    },
  }),
}));

vi.mock('../peer-manager.js', () => ({
  PeerManager: vi.fn(function () {
    return {
      getPeerByNickname: vi.fn().mockResolvedValue('mock-peer-id'),
      getPeerById: vi
        .fn()
        .mockResolvedValue({ id: 'mock-peer-id', nickname: 'test-peer' }),
      listPeers: vi
        .fn()
        .mockReturnValue([{ id: 'mock-peer-id', nickname: 'test-peer' }]),
      addPeer: vi.fn(),
      removePeer: vi.fn(),
      list: vi
        .fn()
        .mockReturnValue([{ id: 'mock-peer-id', nickname: 'test-peer' }]),
      listConnected: vi.fn().mockReturnValue([]),
      discover: vi.fn(),
      addConnected: vi.fn(),
      removeConnected: vi.fn(),
      getNickname: vi.fn().mockReturnValue('test-peer'),
      getPeerId: vi.fn().mockReturnValue('mock-peer-id'),
      cleanup: vi.fn(),
    };
  }),
}));

vi.mock('../file-sharing.js', () => ({
  FileSharingManager: vi.fn(function () {
    return {
      share: vi.fn().mockResolvedValue('mock-share-code'),
      getFileInfo: vi.fn().mockResolvedValue({
        filename: 'test.txt',
        size: 1024,
        chunks: 1,
      }),
      download: vi.fn().mockResolvedValue('mock-download-id'),
      listSharedFiles: vi.fn().mockReturnValue([]),
      removeSharedFile: vi.fn(),
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
      revoke: vi.fn(),
    };
  }),
}));

vi.mock('../group.js', () => ({
  GroupManager: vi.fn(function () {
    const groups = new Set();
    return {
      join: vi.fn().mockImplementation((name: string, topic: string) => {
        groups.add({ name, topic });
        return Promise.resolve();
      }),
      leave: vi.fn().mockImplementation((name: string) => {
        groups.forEach((group: any) => {
          if (group.name === name) {
            groups.delete(group);
          }
        });
        return Promise.resolve();
      }),
      send: vi.fn().mockResolvedValue(undefined),
      list: vi.fn().mockImplementation(() => Array.from(groups)),
    };
  }),
}));

describe('Group Chat Functionality', () => {
  let alice: P2pClient;
  let bob: P2pClient;
  let charlie: P2pClient;

  beforeEach(async () => {
    // Setup three clients with different configurations
    alice = new P2pClient({
      nickname: 'alice',
      config: {
        bootstrapNodes: [],
        enableKademlia: false,
        enableRelay: true,
        enableMdns: false,
        listenAddrs: ['/ip4/0.0.0.0/tcp/0'],
      },
    });

    bob = new P2pClient({
      nickname: 'bob',
      config: {
        bootstrapNodes: [],
        enableKademlia: false,
        enableRelay: true,
        enableMdns: false,
        listenAddrs: ['/ip4/0.0.0.0/tcp/0'],
      },
    });

    charlie = new P2pClient({
      nickname: 'charlie',
      config: {
        bootstrapNodes: [],
        enableKademlia: false,
        enableRelay: true,
        enableMdns: false,
        listenAddrs: ['/ip4/0.0.0.0/tcp/0'],
      },
    });

    // Start all clients
    await Promise.all([alice.start(), bob.start(), charlie.start()]);
  });

  afterEach(async () => {
    // Stop all clients
    await Promise.all([alice.stop(), bob.stop(), charlie.stop()]);
  });

  it('should allow multiple peers to join the same group', async () => {
    // All join the same group
    await Promise.all([
      alice.joinGroup('general'),
      bob.joinGroup('general'),
      charlie.joinGroup('general'),
    ]);

    // Verify all joined the group
    const aliceGroups = alice.getJoinedGroups();
    const bobGroups = bob.getJoinedGroups();
    const charlieGroups = charlie.getJoinedGroups();

    expect(aliceGroups).toContainEqual({
      name: 'general',
      topic: 'gigi-group:general',
    });
    expect(bobGroups).toContainEqual({
      name: 'general',
      topic: 'gigi-group:general',
    });
    expect(charlieGroups).toContainEqual({
      name: 'general',
      topic: 'gigi-group:general',
    });
  });

  it('should allow a peer to leave a group', async () => {
    // Join group
    await alice.joinGroup('general');
    let groups = alice.getJoinedGroups();
    expect(groups).toContainEqual({
      name: 'general',
      topic: 'gigi-group:general',
    });

    // Leave group
    await alice.leaveGroup('general');
    groups = alice.getJoinedGroups();
    expect(groups).not.toContainEqual({
      name: 'general',
      topic: 'gigi-group:general',
    });
  });

  it('should send group messages without errors', async () => {
    // Join group
    await alice.joinGroup('general');

    // Send group message
    await expect(async () => {
      await alice.sendGroupMessage('general', {
        type: 'text',
        text: 'Hello everyone!',
      } as MessageContentInput);
    }).not.toThrow();
  });

  it('should get list of joined groups', async () => {
    // Join multiple groups
    await alice.joinGroup('general');
    await alice.joinGroup('development');

    // Get joined groups
    const groups = alice.getJoinedGroups();
    expect(groups).toContainEqual({
      name: 'general',
      topic: 'gigi-group:general',
    });
    expect(groups).toContainEqual({
      name: 'development',
      topic: 'gigi-group:development',
    });
  });

  it('should handle group operations when not started', async () => {
    // Create a client but don't start it
    const client = new P2pClient({
      nickname: 'test',
      config: {
        bootstrapNodes: [],
        enableKademlia: false,
        enableRelay: true,
        enableMdns: false,
        listenAddrs: ['/ip4/0.0.0.0/tcp/0'],
      },
    });

    // Verify client is not started
    expect(client.isStarted()).toBe(false);

    // Try to join a group (should fail)
    await expect(client.joinGroup('general')).rejects.toThrow();

    // Try to send a group message (should fail)
    await expect(
      client.sendGroupMessage('general', {
        type: 'text',
        text: 'Hello',
      } as MessageContentInput)
    ).rejects.toThrow();
  });
});
