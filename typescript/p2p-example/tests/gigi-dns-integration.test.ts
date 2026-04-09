import { describe, it, expect, afterEach } from 'vitest';
import { P2pClient } from '@gigi/p2p';

// Function to connect with retries to handle race conditions
async function connectWithRetry(
  client: P2pClient,
  multiaddr: string,
  maxRetries = 5,
  delay = 1000
): Promise<boolean> {
  for (let i = 0; i < maxRetries; i++) {
    try {
      // Try to connect
      await client.connectToPeer(multiaddr);
      return true;
    } catch {
      if (i < maxRetries - 1) {
        await new Promise((resolve) => setTimeout(resolve, delay));
      }
    }
  }
  return false;
}

describe('Gigi DNS Integration Tests', () => {
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

  it('should discover peers via Gigi DNS', async () => {
    // Create Alice and Bob clients with nicknames and disable Kademlia to only use local discovery
    alice = new P2pClient({
      nickname: 'Alice',
      config: {
        enableKademlia: false,
      },
    });
    bob = new P2pClient({
      nickname: 'Bob',
      config: {
        enableKademlia: false,
      },
    });

    // Start both clients
    await alice.start();
    await bob.start();

    const aliceMultiaddrs = alice.getMultiaddrs();
    const aliceMultiaddr = aliceMultiaddrs[0];

    // Store discovered peer IDs and addresses
    let aliceDiscoveredBob = false;
    let bobDiscoveredAlice = false;
    let bobMultiaddr = '';

    // Listen for peer discovery events
    alice.onEvent((event) => {
      if (event.type === 'peer-discovered' && event.nickname === 'Bob') {
        aliceDiscoveredBob = true;
        bobMultiaddr = event.address;
      }
    });

    bob.onEvent((event) => {
      if (event.type === 'peer-discovered' && event.nickname === 'Alice') {
        bobDiscoveredAlice = true;
      }
    });

    // Wait for discovery with timeout
    const discoveryTimeout = 10000;
    const startTime = Date.now();
    while (!aliceDiscoveredBob || !bobDiscoveredAlice) {
      if (Date.now() - startTime > discoveryTimeout) {
        throw new Error('Peer discovery timeout');
      }
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }

    // Try to connect with retries
    const aliceConnected = await connectWithRetry(alice, bobMultiaddr);
    const bobConnected = await connectWithRetry(bob, aliceMultiaddr);

    expect(aliceConnected).toBe(true);
    expect(bobConnected).toBe(true);

    // Wait a bit for connections to establish
    await new Promise((resolve) => setTimeout(resolve, 3000));

    // Check connected peers
    const alicePeers = alice.listPeers();
    const bobPeers = bob.listPeers();

    expect(alicePeers.length).toBeGreaterThan(0);
    expect(bobPeers.length).toBeGreaterThan(0);

    const aliceHasBob = alicePeers.some((peer) => peer.nickname === 'Bob');
    const bobHasAlice = bobPeers.some((peer) => peer.nickname === 'Alice');

    expect(aliceHasBob).toBe(true);
    expect(bobHasAlice).toBe(true);
  }, 60000);
});
