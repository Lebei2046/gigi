import { describe, it, expect, afterEach } from 'vitest';
import { P2pClient } from '@gigi/p2p';

describe('File Sharing Integration Tests', () => {
  let alice: P2pClient;
  let bob: P2pClient;

  afterEach(async () => {
    // Stop both clients
    try {
      if (alice) await alice.stop();
      if (bob) await bob.stop();
    } catch {
      // Ignore errors when stopping clients
    }
  });

  it('should share and download files between peers', async () => {
    // Create Alice and Bob clients with unique nicknames to avoid interference between tests
    const timestamp = Date.now();
    const aliceNickname = `Alice-${timestamp}`;
    const bobNickname = `Bob-${timestamp}`;
    alice = new P2pClient({ nickname: aliceNickname });
    bob = new P2pClient({ nickname: bobNickname });

    // Start both clients
    await alice.start();
    await bob.start();

    const alicePeerId = alice.getPeerId();
    const aliceAddrs = alice.getMultiaddrs();
    const bobAddrs = bob.getMultiaddrs();

    // Explicitly add each other's addresses
    bob.addPeer(aliceNickname, alicePeerId, aliceAddrs);
    alice.addPeer(bobNickname, bob.getPeerId(), bobAddrs);

    // Connect to each other
    if (aliceAddrs.length > 0 && bobAddrs.length > 0) {
      await alice.connectToPeer(bobAddrs[0]);
      await bob.connectToPeer(aliceAddrs[0]);
    }

    // Wait for connections to establish
    await new Promise((resolve) => setTimeout(resolve, 3000));

    // Alice shares a file
    const shareCode = await alice.shareFile('/home/lebei/crdt.pdf');
    expect(shareCode).toBeTruthy();

    // Wait for file to be fully shared
    await new Promise((resolve) => setTimeout(resolve, 2000));

    // Both join the chat group
    await alice.joinGroup('chat');
    await bob.joinGroup('chat');

    // Wait for group join to complete
    await new Promise((resolve) => setTimeout(resolve, 2000));

    // Alice shares the file in the group
    await alice.sendGroupMessage('chat', {
      type: 'fileShare',
      shareCode,
      filename: 'crdt.pdf',
      fileSize: 342201,
      fileType: 'application/pdf',
    });

    // Wait for message to be received
    await new Promise((resolve) => setTimeout(resolve, 3000));

    // Bob downloads the file
    const downloadId = await bob.downloadFile(aliceNickname, shareCode);
    expect(downloadId).toBeTruthy();

    // Wait for download to complete
    await new Promise((resolve) => setTimeout(resolve, 10000));
  }, 60000);
});
