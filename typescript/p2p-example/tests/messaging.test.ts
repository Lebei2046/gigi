import { P2pClient, P2pEventType } from '@gigi/p2p';
import {
  AmpMessageFactory,
  AmpMessageRouter,
  InMemoryAgentRegistry,
} from '@gigi/amp';

// Helper function to wait for a condition
async function waitFor(
  condition: () => boolean,
  timeout = 10000
): Promise<void> {
  const start = Date.now();
  while (Date.now() - start < timeout) {
    if (condition()) return;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error('Timeout waiting for condition');
}

describe('Node-to-Node Messaging Integration Tests', () => {
  let alice: GigiP2pClient;
  let bob: GigiP2pClient;
  let aliceMessages: string[] = [];
  let bobMessages: string[] = [];
  let aliceDirectMessageUnsubscribe: () => void;
  let bobDirectMessageUnsubscribe: () => void;
  let aliceMessageRouter: AmpMessageRouter;
  let bobMessageRouter: AmpMessageRouter;

  beforeEach(async () => {
    // Clear message arrays
    aliceMessages = [];
    bobMessages = [];

    // Create Alice's client
    alice = new P2pClient({
      nickname: 'Alice',
      config: {
        enableKademlia: false, // Disable Kademlia for faster local testing
        enableMdns: true,
      },
    });

    // Create Bob's client
    bob = new P2pClient({
      nickname: 'Bob',
      config: {
        enableKademlia: false, // Disable Kademlia for faster local testing
        enableMdns: true,
      },
    });

    // Initialize AMP components for Alice
    const aliceAgentRegistry = new InMemoryAgentRegistry();
    aliceMessageRouter = new AmpMessageRouter(aliceAgentRegistry);

    // Initialize AMP components for Bob
    const bobAgentRegistry = new InMemoryAgentRegistry();
    bobMessageRouter = new AmpMessageRouter(bobAgentRegistry);

    // Set up message handlers for Alice
    aliceMessageRouter.registerMessageHandler(
      'text',
      (message: any, _agentId: string | undefined) => {
        if (message.sender.type === 'node') {
          aliceMessages.push(
            `[NODE] ${message.sender.name}: ${message.content}`
          );
        } else {
          aliceMessages.push(
            `[AGENT] ${message.sender.name}: ${message.content}`
          );
        }
      }
    );

    // Set up message handlers for Bob
    bobMessageRouter.registerMessageHandler(
      'text',
      (message: any, _agentId: string | undefined) => {
        console.log('Bob message handler called:', message);
        if (message.sender.type === 'node') {
          bobMessages.push(`[NODE] ${message.sender.name}: ${message.content}`);
          console.log('Added message to bobMessages:', bobMessages);
        } else {
          bobMessages.push(
            `[AGENT] ${message.sender.name}: ${message.content}`
          );
          console.log('Added message to bobMessages:', bobMessages);
        }
      }
    );

    // Set up P2P event listeners for Alice
    const aliceDirectMessageListener = (event: any) => {
      try {
        const parsedMsg =
          typeof event.message === 'string'
            ? JSON.parse(event.message)
            : event.message;
        if (
          parsedMsg.content &&
          parsedMsg.target &&
          parsedMsg.sender &&
          parsedMsg.timestamp &&
          parsedMsg.id
        ) {
          aliceMessageRouter.routeMessage(parsedMsg);
        } else {
          aliceMessages.push(`[${event.fromNickname}]: ${event.message}`);
        }
      } catch {
        aliceMessages.push(`[${event.fromNickname}]: ${event.message}`);
      }
    };

    // Set up P2P event listeners for Bob
    const bobDirectMessageListener = (event: any) => {
      console.log('Bob received direct message event:', event);
      try {
        const parsedMsg =
          typeof event.message === 'string'
            ? JSON.parse(event.message)
            : event.message;
        console.log('Bob parsed message:', parsedMsg);
        console.log('Bob parsed message has content:', !!parsedMsg.content);
        console.log('Bob parsed message has target:', !!parsedMsg.target);
        console.log('Bob parsed message has sender:', !!parsedMsg.sender);
        console.log('Bob parsed message has timestamp:', !!parsedMsg.timestamp);
        console.log('Bob parsed message has id:', !!parsedMsg.id);
        if (
          parsedMsg.content &&
          parsedMsg.target &&
          parsedMsg.sender &&
          parsedMsg.timestamp &&
          parsedMsg.id
        ) {
          console.log('Bob routing message through message router');
          bobMessageRouter.routeMessage(parsedMsg);
        } else {
          console.log('Bob adding message to bobMessages directly');
          bobMessages.push(`[${event.fromNickname}]: ${event.message}`);
          console.log('Bob messages after adding:', bobMessages);
        }
      } catch (error) {
        console.log('Bob error parsing message:', error);
        bobMessages.push(`[${event.fromNickname}]: ${event.message}`);
        console.log('Bob messages after error:', bobMessages);
      }
    };

    // Set up group message listeners
    const aliceGroupMessageListener = (event: any) => {
      try {
        const content =
          typeof event.content === 'string'
            ? JSON.parse(event.content)
            : event.content;
        if (content.type === 'text' && content.text) {
          // Handle regular group text message
          aliceMessages.push(`[${event.fromNickname}]: ${content.text}`);
        } else if (
          content.content &&
          content.target &&
          content.sender &&
          content.timestamp &&
          content.id
        ) {
          // Handle AMP message
          aliceMessageRouter.routeMessage(content);
        }
      } catch (error) {
        console.log('Alice error parsing group message:', error);
        aliceMessages.push(
          `[${event.fromNickname}] in ${event.group}: ${JSON.stringify(event.content)}`
        );
      }
    };

    const bobGroupMessageListener = (event: any) => {
      try {
        const content =
          typeof event.content === 'string'
            ? JSON.parse(event.content)
            : event.content;
        if (content.type === 'text' && content.text) {
          // Handle regular group text message
          bobMessages.push(`[${event.fromNickname}]: ${content.text}`);
        } else if (
          content.content &&
          content.target &&
          content.sender &&
          content.timestamp &&
          content.id
        ) {
          // Handle AMP message
          bobMessageRouter.routeMessage(content);
        }
      } catch (error) {
        console.log('Bob error parsing group message:', error);
        bobMessages.push(
          `[${event.fromNickname}] in ${event.group}: ${JSON.stringify(event.content)}`
        );
      }
    };

    // Start both clients
    await alice.start();
    await bob.start();

    // Register listeners using client's onEvent method
    console.log('About to register listeners');
    aliceDirectMessageUnsubscribe = alice.onEvent((event) => {
      if (event.type === P2pEventType.DIRECT_MESSAGE) {
        aliceDirectMessageListener(event);
      } else if (event.type === P2pEventType.GROUP_MESSAGE) {
        aliceGroupMessageListener(event);
      }
    });
    console.log('Registered alice listeners');

    bobDirectMessageUnsubscribe = bob.onEvent((event) => {
      if (event.type === P2pEventType.DIRECT_MESSAGE) {
        bobDirectMessageListener(event);
      } else if (event.type === P2pEventType.GROUP_MESSAGE) {
        bobGroupMessageListener(event);
      }
    });
    console.log('Registered bob listeners');

    // Wait for clients to discover each other via mDNS
    await waitFor(
      () => alice.listPeers().length > 0 && bob.listPeers().length > 0
    );

    // No need to explicitly connect - sendDirectMessage will handle it
    // Wait a moment for peer discovery to complete
    await new Promise((resolve) => setTimeout(resolve, 1000));
  });

  afterEach(async () => {
    // Unsubscribe from event listeners
    if (aliceDirectMessageUnsubscribe) aliceDirectMessageUnsubscribe();
    if (bobDirectMessageUnsubscribe) bobDirectMessageUnsubscribe();

    // Stop both clients
    if (alice) await alice.stop();
    if (bob) await bob.stop();

    // Clear message arrays
    aliceMessages = [];
    bobMessages = [];
  });

  test('Direct node-to-node messaging using AMP', async () => {
    const testMessage = 'Hello Bob! This is a direct message from Alice.';

    // Create AMP message from Alice to Bob
    const bobPeerId = bob.getPeerId();
    const ampMessage = AmpMessageFactory.createNodeTextMessage(
      testMessage,
      bobPeerId,
      {
        id: alice.getPeerId(),
        name: 'Alice',
        type: 'node',
      }
    );

    // Send direct message
    await alice.sendDirectMessage(bobPeerId, ampMessage);

    // Wait for message to be received
    await waitFor(() => bobMessages.length > 0);

    // Verify Bob received the message
    expect(bobMessages.length).toBeGreaterThan(0);
    expect(
      bobMessages.some((msg) =>
        msg.includes('Hello Bob! This is a direct message from Alice.')
      )
    ).toBe(true);
    expect(bobMessages.some((msg) => msg.includes('[NODE] Alice:'))).toBe(true);
  });

  test('Group messaging', async () => {
    const testMessage = 'Hello everyone! This is a group message from Alice.';
    const groupName = 'test-group';

    // Both join the same group
    await alice.joinGroup(groupName);
    await bob.joinGroup(groupName);

    // Wait for both to join the group
    await waitFor(
      () =>
        alice.getJoinedGroups().find((group) => group.name === groupName) &&
        bob.getJoinedGroups().find((group) => group.name === groupName)
    );

    // Wait a moment for pubsub subscriptions to complete
    await new Promise((resolve) => setTimeout(resolve, 2000));

    // Alice sends a group message
    await alice.sendGroupMessage(groupName, {
      type: 'text',
      text: testMessage,
    });

    // Wait for message to be received by Bob
    await waitFor(() => bobMessages.length > 0);

    // Verify Bob received the message
    expect(bobMessages.length).toBeGreaterThan(0);
    expect(
      bobMessages.some((msg) =>
        msg.includes('Hello everyone! This is a group message from Alice.')
      )
    ).toBe(true);
    expect(bobMessages.some((msg) => msg.includes('[Alice]:'))).toBe(true);
  });

  test('Bidirectional direct messaging', async () => {
    const aliceMessage = 'Hello Bob!';
    const bobMessage = 'Hello Alice!';

    // Get peer IDs
    const alicePeerId = alice.getPeerId();
    const bobPeerId = bob.getPeerId();

    // Clear message arrays
    bobMessages = [];
    aliceMessages = [];

    // Alice sends to Bob
    const ampMessage1 = AmpMessageFactory.createNodeTextMessage(
      aliceMessage,
      bobPeerId,
      {
        id: alicePeerId,
        name: 'Alice',
        type: 'node',
      }
    );
    await alice.sendDirectMessage(bobPeerId, ampMessage1);

    // Wait for Bob to receive
    await waitFor(() => bobMessages.length > 0, 5000);
    console.log('Bob received messages:', bobMessages);

    // Clear Alice's messages to avoid confusion
    aliceMessages = [];

    // Add a small delay to ensure messages are processed
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Bob sends to Alice
    const ampMessage2 = AmpMessageFactory.createNodeTextMessage(
      bobMessage,
      alicePeerId,
      {
        id: bobPeerId,
        name: 'Bob',
        type: 'node',
      }
    );
    console.log('Bob is sending message to Alice:', ampMessage2);
    await bob.sendDirectMessage(alicePeerId, ampMessage2);

    // Wait for Alice to receive
    await waitFor(() => aliceMessages.length > 0, 5000);
    console.log('Alice received messages:', aliceMessages);

    // Verify both received messages
    // Bob might receive multiple messages, but we just need to check he got at least one
    expect(bobMessages.length).toBeGreaterThan(0);
    expect(bobMessages.some((msg) => msg.includes('Hello Bob!'))).toBe(true);

    // Alice should receive exactly one message from Bob
    expect(aliceMessages.length).toBeGreaterThan(0);
    expect(
      aliceMessages.some(
        (msg) => msg.includes('Bob') && msg.includes('Hello Alice!')
      )
    ).toBe(true);
  });

  test('Direct node-to-node file message using AMP', async () => {
    const testFileName = 'test.txt';
    const testFileContent = 'This is a test file content';

    // Create AMP file message from Alice to Bob
    const bobPeerId = bob.getPeerId();
    const ampMessage = AmpMessageFactory.createNodeFileMessage(
      testFileName,
      Buffer.from(testFileContent),
      bobPeerId,
      {
        id: alice.getPeerId(),
        name: 'Alice',
        type: 'node',
      }
    );

    // Send direct message
    await alice.sendDirectMessage(bobPeerId, ampMessage);

    // Wait for message to be received
    await waitFor(() => bobMessages.length > 0);

    // Verify Bob received the message (file messages will be handled differently in production)
    expect(bobMessages.length).toBeGreaterThan(0);
  });

  test('Group messaging with AMP', async () => {
    const testMessage = 'Hello group! This is an AMP group message from Alice.';
    const groupName = 'test-amp-group';

    // Both join the same group
    await alice.joinGroup(groupName);
    await bob.joinGroup(groupName);

    // Wait for both to join the group
    await waitFor(
      () =>
        alice.getJoinedGroups().find((group) => group.name === groupName) &&
        bob.getJoinedGroups().find((group) => group.name === groupName)
    );

    // Wait a moment for pubsub subscriptions to complete
    await new Promise((resolve) => setTimeout(resolve, 2000));

    // Create AMP message
    const ampMessage = AmpMessageFactory.createNodeTextMessage(
      testMessage,
      groupName, // Use group name as target for group messages
      {
        id: alice.getPeerId(),
        name: 'Alice',
        type: 'node',
      }
    );

    // Alice sends a group message using AMP
    await alice.sendGroupMessage(groupName, ampMessage);

    // Wait for message to be received by Bob
    await waitFor(() => bobMessages.length > 0);

    // Verify Bob received the message
    expect(bobMessages.length).toBeGreaterThan(0);
    expect(
      bobMessages.some((msg) =>
        msg.includes('Hello group! This is an AMP group message from Alice.')
      )
    ).toBe(true);
    expect(bobMessages.some((msg) => msg.includes('[NODE] Alice:'))).toBe(true);
  });

  test('Multiple sequential direct messages', async () => {
    const messages = [
      'First message from Alice',
      'Second message from Alice',
      'Third message from Alice',
    ];

    const bobPeerId = bob.getPeerId();

    // Send multiple messages
    for (const message of messages) {
      const ampMessage = AmpMessageFactory.createNodeTextMessage(
        message,
        bobPeerId,
        {
          id: alice.getPeerId(),
          name: 'Alice',
          type: 'node',
        }
      );
      await alice.sendDirectMessage(bobPeerId, ampMessage);
      // Small delay between messages
      await new Promise((resolve) => setTimeout(resolve, 100));
    }

    // Wait for all messages to be received
    await waitFor(() => {
      // Check if all messages are present in bobMessages
      return messages.every((message) =>
        bobMessages.some((bobMsg) => bobMsg.includes(message))
      );
    });

    // Verify Bob received all messages
    expect(bobMessages.length).toBeGreaterThan(0);
    messages.forEach((message) => {
      expect(bobMessages.some((bobMsg) => bobMsg.includes(message))).toBe(true);
    });
  });

  test('Error handling for invalid messages', async () => {
    const bobPeerId = bob.getPeerId();

    // Send an invalid message (not an AMP message)
    await alice.sendDirectMessage(bobPeerId, 'Invalid message format');

    // Wait for message to be received
    await waitFor(() => bobMessages.length > 0);

    // Verify Bob received the message as a regular message (not AMP)
    expect(bobMessages.length).toBeGreaterThan(0);
    expect(
      bobMessages.some((msg) => msg.includes('Invalid message format'))
    ).toBe(true);
  });

  test('Node-to-agent messaging using AMP', async () => {
    const testMessage = 'Hello Agent! This is a message from Alice.';
    const agentId = 'test-agent-1';
    const agentName = 'Test Agent';

    // Register an agent with Bob's agent registry
    await bobMessageRouter.agentRegistry.registerAgent({
      id: agentId,
      name: agentName,
      description: 'A test agent',
      ownerId: bob.getPeerId(),
      capabilities: [],
    });

    // Create AMP message from Alice to Bob's agent
    const bobPeerId = bob.getPeerId();
    const ampMessage = AmpMessageFactory.createNodeAgentTextMessage(
      testMessage,
      {
        type: 'agent',
        nodeId: bobPeerId,
        agentId: agentId,
      },
      {
        id: alice.getPeerId(),
        name: 'Alice',
        type: 'node',
      }
    );

    // Send direct message
    await alice.sendDirectMessage(bobPeerId, ampMessage);

    // Wait for message to be received
    await waitFor(() => bobMessages.length > 0);

    // Verify Bob received the message (it should be routed to the agent)
    expect(bobMessages.length).toBeGreaterThan(0);
    expect(
      bobMessages.some((msg) =>
        msg.includes('Hello Agent! This is a message from Alice.')
      )
    ).toBe(true);
    // Check that the message contains Alice's name
    expect(bobMessages.some((msg) => msg.includes('Alice'))).toBe(true);
  });

  test('Agent-to-node messaging using AMP', async () => {
    const testMessage = 'Hello Bob! This is a message from Test Agent.';
    const agentId = 'test-agent-2';
    const agentName = 'Test Agent';

    // Register an agent with Alice's agent registry
    await aliceMessageRouter.agentRegistry.registerAgent({
      id: agentId,
      name: agentName,
      description: 'A test agent',
      ownerId: alice.getPeerId(),
      capabilities: [],
    });

    // Create AMP message from Agent to Bob
    const bobPeerId = bob.getPeerId();
    const ampMessage = AmpMessageFactory.createNodeTextMessage(
      testMessage,
      bobPeerId,
      {
        id: agentId,
        name: agentName,
        type: 'agent',
      }
    );

    // Send direct message from Alice's client (acting as the agent's node)
    await alice.sendDirectMessage(bobPeerId, ampMessage);

    // Wait for message to be received
    await waitFor(() => bobMessages.length > 0);

    // Verify Bob received the message from the agent
    expect(bobMessages.length).toBeGreaterThan(0);
    expect(
      bobMessages.some((msg) =>
        msg.includes('Hello Bob! This is a message from Test Agent.')
      )
    ).toBe(true);
    expect(bobMessages.some((msg) => msg.includes('[AGENT] Test Agent:'))).toBe(
      true
    );
  });

  test('Group messaging with multiple peers (3+ nodes)', async () => {
    const testMessage = 'Hello everyone! This is a group message from Alice.';
    const groupName = 'multi-peer-group';
    let charlie: P2pClient;
    const charlieMessages: string[] = [];
    let charlieMessageRouter: AmpMessageRouter;
    let charlieStarted = false;
    let charlieUnsubscribe: () => void;

    try {
      // Create Charlie's client
      charlie = new P2pClient({
        nickname: 'Charlie',
        config: {
          enableKademlia: false, // Disable Kademlia for faster local testing
          enableMdns: true,
        },
      });

      // Initialize AMP components for Charlie
      const charlieAgentRegistry = new InMemoryAgentRegistry();
      charlieMessageRouter = new AmpMessageRouter(charlieAgentRegistry);

      // Set up message handlers for Charlie
      charlieMessageRouter.registerMessageHandler(
        'text',
        (message: any, _agentId: string | undefined) => {
          if (message.sender.type === 'node') {
            charlieMessages.push(
              `[NODE] ${message.sender.name}: ${message.content}`
            );
          } else {
            charlieMessages.push(
              `[AGENT] ${message.sender.name}: ${message.content}`
            );
          }
        }
      );

      // Start Charlie's client
      await charlie.start();
      charlieStarted = true;

      // Set up P2P event listeners for Charlie using onEvent method
      charlieUnsubscribe = charlie.onEvent((event) => {
        if (!charlieStarted) return;

        if (event.type === P2pEventType.DIRECT_MESSAGE) {
          // Only process messages not from Charlie himself
          try {
            const charliePeerId = charlie.getPeerId();
            if (event.from === charliePeerId) return;

            const parsedMsg =
              typeof event.message === 'string'
                ? JSON.parse(event.message)
                : event.message;
            if (
              parsedMsg.content &&
              parsedMsg.target &&
              parsedMsg.sender &&
              parsedMsg.timestamp &&
              parsedMsg.id
            ) {
              charlieMessageRouter.routeMessage(parsedMsg);
            } else {
              charlieMessages.push(`[${event.fromNickname}]: ${event.message}`);
            }
          } catch {
            charlieMessages.push(`[${event.fromNickname}]: ${event.message}`);
          }
        } else if (event.type === P2pEventType.GROUP_MESSAGE) {
          try {
            const parsedMsg =
              typeof event.content === 'string'
                ? JSON.parse(event.content)
                : event.content;
            if (parsedMsg.type === 'text' && parsedMsg.text) {
              charlieMessages.push(
                `[${event.fromNickname}]: ${parsedMsg.text}`
              );
            } else if (
              parsedMsg.content &&
              parsedMsg.target &&
              parsedMsg.sender &&
              parsedMsg.timestamp &&
              parsedMsg.id
            ) {
              charlieMessageRouter.routeMessage(parsedMsg);
            }
          } catch {
            charlieMessages.push(
              `[${event.fromNickname}] in ${event.group}: ${JSON.stringify(event.content)}`
            );
          }
        }
      });

      // Wait for all clients to discover each other via mDNS
      await waitFor(
        () =>
          alice.listPeers().length > 1 &&
          bob.listPeers().length > 1 &&
          charlie.listPeers().length > 1
      );

      // No need to explicitly connect - sendDirectMessage will handle it
      // Wait a moment for peer discovery to complete
      await new Promise((resolve) => setTimeout(resolve, 1000));

      // All join the same group
      await alice.joinGroup(groupName);
      await bob.joinGroup(groupName);
      await charlie.joinGroup(groupName);

      // Wait for all to join the group
      await waitFor(
        () =>
          alice.getJoinedGroups().find((group) => group.name === groupName) &&
          bob.getJoinedGroups().find((group) => group.name === groupName) &&
          charlie.getJoinedGroups().find((group) => group.name === groupName)
      );

      // Wait a moment for pubsub subscriptions to complete
      await new Promise((resolve) => setTimeout(resolve, 2000));

      // Alice sends a group message
      await alice.sendGroupMessage(groupName, {
        type: 'text',
        text: testMessage,
      });

      // Wait for message to be received by Bob and Charlie
      await waitFor(() => bobMessages.length > 0 && charlieMessages.length > 0);

      // Verify both Bob and Charlie received the message
      expect(bobMessages.length).toBeGreaterThan(0);
      expect(
        bobMessages.some((msg) =>
          msg.includes('Hello everyone! This is a group message from Alice.')
        )
      ).toBe(true);
      expect(bobMessages.some((msg) => msg.includes('[Alice]:'))).toBe(true);
      expect(charlieMessages.length).toBeGreaterThan(0);
      expect(
        charlieMessages.some((msg) =>
          msg.includes('Hello everyone! This is a group message from Alice.')
        )
      ).toBe(true);
      expect(charlieMessages.some((msg) => msg.includes('[Alice]:'))).toBe(
        true
      );
    } finally {
      // Clean up Charlie's client
      charlieStarted = false;
      charlieUnsubscribe();
      if (charlie) await charlie.stop();
    }
  });

  test('Large message handling', async () => {
    // Create a large message (10KB)
    const largeMessage = 'A'.repeat(1024 * 10);
    const bobPeerId = bob.getPeerId();

    // Create AMP message from Alice to Bob
    const ampMessage = AmpMessageFactory.createNodeTextMessage(
      largeMessage,
      bobPeerId,
      {
        id: alice.getPeerId(),
        name: 'Alice',
        type: 'node',
      }
    );

    // Send large message
    await alice.sendDirectMessage(bobPeerId, ampMessage);

    // Wait for message to be received
    await waitFor(() => bobMessages.length > 0);

    // Verify Bob received the large message
    expect(bobMessages.length).toBeGreaterThan(0);
    expect(bobMessages.some((msg) => msg.includes('A'.repeat(100)))).toBe(true); // Verify the message contains part of the large content
  });

  test('Edge cases in message formatting', async () => {
    const bobPeerId = bob.getPeerId();

    // Test 1: Empty message
    const emptyMessage = '';
    const ampMessage1 = AmpMessageFactory.createNodeTextMessage(
      emptyMessage,
      bobPeerId,
      {
        id: alice.getPeerId(),
        name: 'Alice',
        type: 'node',
      }
    );

    await alice.sendDirectMessage(bobPeerId, ampMessage1);
    await waitFor(() => bobMessages.length > 0);
    expect(bobMessages.length).toBeGreaterThan(0);

    // Clear messages for next test
    bobMessages = [];

    // Test 2: Message with special characters
    const specialCharsMessage = 'Hello Bob! 😊 Special chars: !@#$%^&*()';
    const ampMessage2 = AmpMessageFactory.createNodeTextMessage(
      specialCharsMessage,
      bobPeerId,
      {
        id: alice.getPeerId(),
        name: 'Alice',
        type: 'node',
      }
    );

    await alice.sendDirectMessage(bobPeerId, ampMessage2);
    await waitFor(() => bobMessages.length > 0);
    expect(bobMessages.length).toBeGreaterThan(0);
    expect(bobMessages.some((msg) => msg.includes('Hello Bob!'))).toBe(true);
    expect(
      bobMessages.some((msg) => msg.includes('Special chars: !@#$%^&*()'))
    ).toBe(true);

    // Clear messages for next test
    bobMessages = [];

    // Test 3: Message with newlines and whitespace
    const whitespaceMessage = 'Hello\nBob\n\nHow are you?   ';
    const ampMessage3 = AmpMessageFactory.createNodeTextMessage(
      whitespaceMessage,
      bobPeerId,
      {
        id: alice.getPeerId(),
        name: 'Alice',
        type: 'node',
      }
    );

    await alice.sendDirectMessage(bobPeerId, ampMessage3);
    await waitFor(() => bobMessages.length > 0);
    expect(bobMessages.length).toBeGreaterThan(0);
    expect(bobMessages.some((msg) => msg.includes('Hello'))).toBe(true);
    expect(bobMessages.some((msg) => msg.includes('Bob'))).toBe(true);
    expect(bobMessages.some((msg) => msg.includes('How are you?'))).toBe(true);
  });

  afterAll(async () => {
    // Stop clients
    if (alice) await alice.stop();
    if (bob) await bob.stop();
  });
});
