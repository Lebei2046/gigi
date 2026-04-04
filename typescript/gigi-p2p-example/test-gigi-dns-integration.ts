#!/usr/bin/env tsx

import { P2pClient } from '@gigi/p2p-ts';

// Add global error handler to catch uncaught errors
process.on('uncaughtException', (error) => {
  console.error('Uncaught Exception:', error);
  console.error('Stack:', error.stack);
});

process.on('unhandledRejection', (reason, promise) => {
  console.error('Unhandled Rejection at:', promise);
  console.error('Reason:', reason);
});

// Function to connect with retries to handle race conditions
async function connectWithRetry(
  client: P2pClient,
  peerId: string,
  maxRetries = 5,
  delay = 1000
): Promise<boolean> {
  for (let i = 0; i < maxRetries; i++) {
    try {
      // Try to connect
      await client.connectToPeer(peerId);
      console.log(`Successfully connected after ${i} retries`);
      return true;
    } catch (error: any) {
      console.log(`Connection attempt ${i + 1} failed: ${error.message}`);
      if (i < maxRetries - 1) {
        await new Promise((resolve) => setTimeout(resolve, delay));
      }
    }
  }
  return false;
}

async function testGigiDnsIntegration() {
  console.log('Starting Gigi DNS integration test...\n');

  // Create Alice and Bob clients with nicknames and disable Kademlia to only use local discovery
  const alice = new P2pClient({
    nickname: 'Alice',
    config: {
      enableKademlia: false,
    },
  });
  const bob = new P2pClient({
    nickname: 'Bob',
    config: {
      enableKademlia: false,
    },
  });

  try {
    // Start both clients
    console.log('Starting Alice...');
    await alice.start();
    console.log(`Alice started with peer ID: ${alice.getPeerId()}`);
    const aliceAddrs = alice.getMultiaddrs();
    console.log(`Alice addresses: ${aliceAddrs.join(', ')}\n`);

    console.log('Starting Bob...');
    await bob.start();
    console.log(`Bob started with peer ID: ${bob.getPeerId()}`);
    const bobAddrs = bob.getMultiaddrs();
    console.log(`Bob addresses: ${bobAddrs.join(', ')}\n`);

    // Wait for peer discovery via Gigi DNS
    console.log('Waiting for peer discovery via Gigi DNS...');
    console.log(
      '(This should happen automatically via mDNS announcements with nicknames)\n'
    );

    // Store discovered peer IDs
    let aliceDiscoveredBob = false;
    let bobDiscoveredAlice = false;
    let bobPeerId = '';
    const alicePeerId = alice.getPeerId();

    // Listen for peer discovery events
    alice.onEvent((event) => {
      if (event.type === 'peer-discovered') {
        console.log('\n=== Alice discovered a peer ===');
        console.log(`Nickname: ${event.nickname}`);
        console.log(`Peer ID: ${event.peerId}`);
        console.log(`Address: ${event.address}`);
        if (event.nickname === 'Bob') {
          aliceDiscoveredBob = true;
          bobPeerId = event.peerId;
        }
      }
    });

    bob.onEvent((event) => {
      if (event.type === 'peer-discovered') {
        console.log('\n=== Bob discovered a peer ===');
        console.log(`Nickname: ${event.nickname}`);
        console.log(`Peer ID: ${event.peerId}`);
        console.log(`Address: ${event.address}`);
        if (event.nickname === 'Alice') {
          bobDiscoveredAlice = true;
        }
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

    console.log('\n=== Peers discovered successfully ===');

    // Try to connect with retries
    console.log('\n=== Attempting to connect peers ===');
    await connectWithRetry(alice, bobPeerId);
    await connectWithRetry(bob, alicePeerId);

    // Wait a bit for connections to establish
    await new Promise((resolve) => setTimeout(resolve, 3000));

    // Check connected peers
    console.log('\n=== Checking connected peers ===');
    const alicePeers = alice.listPeers();
    console.log(`Alice has ${alicePeers.length} peer(s):`);
    alicePeers.forEach((peer) => {
      console.log(`  - ${peer.nickname} (${peer.peerId})`);
    });

    const bobPeers = bob.listPeers();
    console.log(`\nBob has ${bobPeers.length} peer(s):`);
    bobPeers.forEach((peer) => {
      console.log(`  - ${peer.nickname} (${peer.peerId})`);
    });

    console.log('\n=== Test completed ===');
  } catch (error) {
    console.error('Test failed:', error);
  } finally {
    // Stop both clients
    try {
      await alice.stop();
      await bob.stop();
    } catch {
      console.error('Error stopping clients');
    }
  }
}

testGigiDnsIntegration();
