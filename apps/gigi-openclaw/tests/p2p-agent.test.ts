/**
 * Gigi OpenClaw P2P Agent Test
 *
 * This test verifies the functionality of the Gigi P2P plugin for OpenClaw.
 * It tests the following steps:
 * 1. Discovering the target OpenClaw node via P2P
 * 2. Joining the gigi-agents group
 * 3. Sending a message to the main agent
 * 4. Receiving a response from the main agent
 *
 * The test sends the message "Who are you?" to the main agent and expects a response.
 */
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { P2pClient } from '../../typescript/p2p/dist/client.js';
import { AmpMessageFactory } from '../../typescript/amp/dist/index.js';
import { createLogger } from '../../typescript/logging/dist/index.js';

// Create a logger for test output
const logger = createLogger({ name: 'gigi-openclaw-test' });

describe('Gigi OpenClaw P2P Agent Test', () => {
  // P2P client instance for the test
  let p2pClient: P2pClient;
  // Peer ID generated for this test client
  let testPeerId: string;

  // Target OpenClaw node peer ID (derived from the provided mnemonic)
  const targetPeerId = '12D3KooWKrVwR4tFJgMBt1LoEFN4eVVyJsP9cNGZpfYkwVGK8Cac';

  /**
   * Set up the test environment before each test
   * - Create a new P2pClient instance
   * - Start the client and get its peer ID
   */
  beforeEach(async () => {
    // Create P2pClient without a mnemonic (generates a new peer ID)
    p2pClient = new P2pClient({
      nickname: 'Test Client',
      config: {
        bootstrapNodes: [],
        enableKademlia: true,
        enableRelay: true,
        enableMdns: true,
        listenAddrs: ['/ip4/0.0.0.0/tcp/0', '/ip4/0.0.0.0/tcp/0/ws'],
      },
      // Don't use a mnemonic, let P2pClient generate a new peer ID
    });

    // Start the P2pClient
    await p2pClient.start();
    testPeerId = p2pClient.getPeerId();
    logger.info(`Test peer ID: ${testPeerId}`);
  });

  /**
   * Clean up after each test
   * - Stop the P2pClient if it's running
   */
  afterEach(async () => {
    // Stop P2pClient
    if (p2pClient && p2pClient.isStarted()) {
      await p2pClient.stop();
    }
  });

  /**
   * Test the complete flow: discover node, join group, send message, receive response
   */
  it('should discover the OpenClaw node, join gigi-agents group, and send group message to main agent', async () => {
    logger.info('Starting test to discover OpenClaw node, join group, and send message...');
    logger.info(`Target OpenClaw peer ID: ${targetPeerId}`);

    // Listen for peer-discovered events to log all discovered nodes
    p2pClient.onEvent(async (event) => {
      if (event.type === 'peer-discovered') {
        logger.info(`Discovered peer: ${event.peerId} (${event.nickname})`);
      }
    });

    // Create AMP message to send to main agent
    const ampMessage = AmpMessageFactory.createTextMessage(
      'Who are you?',  // Message content
      { type: 'specific', agentIds: ['main'] },  // Target main agent
      {
        id: testPeerId,  // Sender ID (test client's peer ID)
        name: 'Test Client',  // Sender name
        type: 'agent'  // Sender type
      }
    );

    // Wait for the target node to be discovered
    let peerDiscovered = false;

    // Check if target node is already discovered
    const peers = p2pClient.listPeers();
    for (const peer of peers) {
      if (peer.peerId === targetPeerId) {
        peerDiscovered = true;
        logger.info(`Target OpenClaw node already discovered: ${peer.peerId} (${peer.nickname})`);
        break;
      }
    }

    // If not discovered yet, wait for discovery
    if (!peerDiscovered) {
      logger.info('Waiting for peer discovery...');

      // Create promise to wait for peer discovery
      const peerDiscoveredPromise = new Promise<void>((resolve) => {
        const unsubscribe = p2pClient.onEvent(async (event) => {
          if (event.type === 'peer-discovered' && event.peerId === targetPeerId) {
            logger.info(`Discovered target OpenClaw node: ${event.peerId} (${event.nickname})`);
            unsubscribe();
            resolve();
          }
        });
      });

      // Wait for peer discovery with 60-second timeout
      await Promise.race([
        peerDiscoveredPromise,
        new Promise<void>((_, reject) => {
          setTimeout(() => reject(new Error('Peer discovery timeout')), 60000);
        })
      ]);
    }

    // Wait for connection to be established
    await new Promise(resolve => setTimeout(resolve, 5000));

    // Join the gigi-agents group
    logger.info('Joining gigi-agents group...');
    await p2pClient.joinGroup('gigi-agents');
    logger.info('Joined gigi-agents group successfully');

    // Wait for group join to complete
    await new Promise(resolve => setTimeout(resolve, 5000));

    // Set up listener for response from main agent before sending message
    const responsePromise = new Promise<string>((resolve) => {
      // Listen for all events to ensure we don't miss any messages
      const unsubscribe = p2pClient.onEvent(async (event) => {
        logger.info(`Received event: ${event.type}`);
        if (event.type === 'direct-message') {
          logger.info(`Received direct message from: ${event.from}`);
          try {
            // Parse message data
            const messageData = typeof event.message === 'string' ? JSON.parse(event.message) : event.message;
            logger.info(`Message data: ${JSON.stringify(messageData)}`);
            // Check if message is from main agent
            if (messageData.type === 'text' && messageData.sender && messageData.sender.id === 'main') {
              logger.info(`Received response from main agent: ${messageData.content}`);
              unsubscribe();
              resolve(messageData.content);
            }
          } catch (error) {
            logger.error('Error parsing response message:', error);
          }
        }
      });
    });

    // Send group message to gigi-agents group
    logger.info('Sending group message to gigi-agents group...');
    try {
      // Format message correctly for P2pClient
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await p2pClient.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent successfully');
    } catch (error) {
      logger.error('Error sending group message:', error);
      // Retry sending message
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await p2pClient.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent after retry');
    }

    // Wait for response with 120-second timeout
    const response = await Promise.race([
      responsePromise,
      new Promise<string>((_, reject) => {
        setTimeout(() => reject(new Error('Response timeout')), 120000);
      })
    ]);

    // Log and verify response
    logger.info(`Response received: ${response}`);
    expect(response).toBeDefined();
    expect(typeof response).toBe('string');
  }, 180000); // Increase test timeout to 180 seconds
});
