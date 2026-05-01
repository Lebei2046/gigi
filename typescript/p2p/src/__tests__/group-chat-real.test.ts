import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { P2pClient } from '../client.js';
import type { MessageContentInput } from '../types.js';

// 不使用模拟，使用真实的实现
describe('Group Chat Functionality (Real Implementation)', () => {
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
        enableRelay: false,
        enableMdns: false,
        listenAddrs: ['/ip4/127.0.0.1/tcp/15001'],
      },
    });

    bob = new P2pClient({
      nickname: 'bob',
      config: {
        bootstrapNodes: [],
        enableKademlia: false,
        enableRelay: false,
        enableMdns: false,
        listenAddrs: ['/ip4/127.0.0.1/tcp/15002'],
      },
    });

    charlie = new P2pClient({
      nickname: 'charlie',
      config: {
        bootstrapNodes: [],
        enableKademlia: false,
        enableRelay: false,
        enableMdns: false,
        listenAddrs: ['/ip4/127.0.0.1/tcp/15003'],
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

    expect(aliceGroups).toContainEqual(
      expect.objectContaining({
        name: 'general',
        topic: 'gigi-group:general',
      })
    );
    expect(bobGroups).toContainEqual(
      expect.objectContaining({
        name: 'general',
        topic: 'gigi-group:general',
      })
    );
    expect(charlieGroups).toContainEqual(
      expect.objectContaining({
        name: 'general',
        topic: 'gigi-group:general',
      })
    );
  });

  it('should allow a peer to leave a group', async () => {
    // Join group
    await alice.joinGroup('general');
    let groups = alice.getJoinedGroups();
    expect(groups).toContainEqual(
      expect.objectContaining({
        name: 'general',
        topic: 'gigi-group:general',
      })
    );

    // Leave group
    await alice.leaveGroup('general');
    groups = alice.getJoinedGroups();
    expect(groups).not.toContainEqual(
      expect.objectContaining({
        name: 'general',
        topic: 'gigi-group:general',
      })
    );
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
    expect(groups).toContainEqual(
      expect.objectContaining({
        name: 'general',
        topic: 'gigi-group:general',
      })
    );
    expect(groups).toContainEqual(
      expect.objectContaining({
        name: 'development',
        topic: 'gigi-group:development',
      })
    );
  });

  it('should handle group operations when not started', async () => {
    // Create a client but don't start it
    const client = new P2pClient({
      nickname: 'test',
      config: {
        bootstrapNodes: [],
        enableKademlia: false,
        enableRelay: false,
        enableMdns: false,
        listenAddrs: ['/ip4/127.0.0.1/tcp/15004'],
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

  it('should handle multiple groups', async () => {
    // Join different groups
    await alice.joinGroup('general');
    await alice.joinGroup('development');
    await alice.joinGroup('testing');

    // Get joined groups
    const groups = alice.getJoinedGroups();
    expect(groups.length).toBe(3);

    // Leave one group
    await alice.leaveGroup('development');
    const groupsAfterLeave = alice.getJoinedGroups();
    expect(groupsAfterLeave.length).toBe(2);
    expect(groupsAfterLeave).not.toContainEqual(
      expect.objectContaining({
        name: 'development',
        topic: 'gigi-group:development',
      })
    );
  });
});
