#!/usr/bin/env tsx
import { Command } from 'commander';
import { P2pClient, derivePeerId, generateMnemonic } from '@gigi/p2p';
import {
  AmpMessageRouter,
  AmpMessageFactory,
  InMemoryAgentRegistry,
} from '@gigi/amp';
import { createLogger } from '@gigi/logging';
import * as fs from 'fs';

const logger = createLogger({ name: 'p2p-example' });

const program = new Command();

// Default configuration
const DEFAULT_CONFIG = {
  bootstrapNodes: [
    '/dns4/bootstrap.libp2p.io/tcp/443/wss/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN',
    '/dns4/bootstrap.libp2p.io/tcp/443/wss/p2p/QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa',
  ],
};

// Global state
let p2pClient: P2pClient | null = null;
let agentRegistry: InMemoryAgentRegistry | null = null;
let messageRouter: AmpMessageRouter | null = null;
let currentNickname = 'anonymous';
let currentGroup: string | null = null;

// Initialize P2P client
async function initializeP2P(mnemonic: string, nickname: string) {
  logger.info({
    message: 'Initializing P2P client',
    nickname: nickname,
  });

  // Derive peer ID from mnemonic
  const peerId = await derivePeerId(mnemonic);
  logger.info({
    message: 'Derived peer ID',
    peerId: peerId,
  });

  // Create P2P client
  p2pClient = new P2pClient({
    nickname,
    mnemonic,
    config: {
      bootstrapNodes: DEFAULT_CONFIG.bootstrapNodes,
    },
  });

  // Initialize AMP components
  agentRegistry = new InMemoryAgentRegistry();
  messageRouter = new AmpMessageRouter(agentRegistry);

  // Set up AMP message handlers
  messageRouter.registerMessageHandler(
    'text',
    (message: any, _agentId: string | undefined) => {
      if (message.sender.type === 'node') {
        console.log(`\n[NODE] ${message.sender.name}: ${message.content}`);
      } else {
        console.log(`\n[AGENT] ${message.sender.name}: ${message.content}`);
      }
    }
  );

  messageRouter.registerMessageHandler(
    'file',
    (message: any, _agentId: string | undefined) => {
      if (message.sender.type === 'node') {
        console.log(
          `\n[NODE] ${message.sender.name} shared a file: ${message.filename} (${message.fileSize} bytes)`
        );
      } else {
        console.log(
          `\n[AGENT] ${message.sender.name} shared a file: ${message.filename} (${message.fileSize} bytes)`
        );
      }
      console.log(`File hash: ${message.fileHash}`);
    }
  );

  // Set up P2P event listeners
  p2pClient.onEvent((event) => {
    if (event.type === 'direct-message') {
      try {
        const parsedMsg = JSON.parse(event.message);
        if (parsedMsg.type && messageRouter) {
          messageRouter.routeMessage(parsedMsg);
        }
      } catch {
        // Not an AMP message, display as regular message
        console.log(`\n[${event.fromNickname}]: ${event.message}`);
      }
    } else if (event.type === 'group-message') {
      try {
        const parsedMsg = event.content;
        if (parsedMsg.type === 'text' && parsedMsg.text) {
          try {
            // Try to parse as AMP message
            const ampMsg = JSON.parse(parsedMsg.text);
            if (ampMsg.type && messageRouter) {
              // AMP message, route through message router
              messageRouter.routeMessage(ampMsg);
            } else {
              // Regular text message
              console.log(`\n[${event.fromNickname}]: ${parsedMsg.text}`);
            }
          } catch {
            // Regular text message
            console.log(`\n[${event.fromNickname}]: ${parsedMsg.text}`);
          }
        } else if (messageRouter) {
          // Check if it's an AMP message
          const ampMsg = parsedMsg as any;
          if (
            ampMsg.content &&
            ampMsg.target &&
            ampMsg.sender &&
            ampMsg.timestamp &&
            ampMsg.id
          ) {
            // AMP message, route through message router
            messageRouter.routeMessage(ampMsg);
          } else {
            // Other message types, display as JSON
            console.log(
              `\n[${event.fromNickname}] in ${event.group}: ${typeof parsedMsg === 'string' ? parsedMsg : JSON.stringify(parsedMsg)}`
            );
          }
        } else {
          // Other message types, display as JSON
          console.log(
            `\n[${event.fromNickname}] in ${event.group}: ${typeof parsedMsg === 'string' ? parsedMsg : JSON.stringify(parsedMsg)}`
          );
        }
      } catch {
        // Not an AMP message, display as regular message
        console.log(
          `\n[${event.fromNickname}] in ${event.group}: ${JSON.stringify(event.content)}`
        );
      }
    }
  });

  // Start the P2P client
  await p2pClient.start();
  console.log('P2P client started successfully!');
  console.log(`Your peer ID: ${p2pClient.getPeerId()}`);
  logger.info({
    message: 'P2P client started successfully',
    peerId: p2pClient.getPeerId(),
  });

  currentNickname = nickname;
}

// Interactive chat interface
async function startInteractiveChat() {
  if (!p2pClient) {
    console.error('P2P client not initialized. Please run init first.');
    return;
  }

  console.log('\n=== Gigi P2P Interactive Chat ===');
  console.log('Commands:');
  console.log('  /join <group>         - Join a group');
  console.log('  /leave                - Leave current group');
  console.log('  /msg <peer> <msg>     - Send direct message to a node');
  console.log(
    '  /agent-msg <node> <agent> <msg> - Send message to a specific agent'
  );
  console.log('  /share <file>         - Share a file');
  console.log('  /download <code>      - Download a file');
  console.log('  /peers                - List connected peers');
  console.log('  /quit                 - Exit');
  console.log('  <message>             - Send to current group\n');

  // Set up readline interface
  const readline = await import('readline');
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    prompt: currentGroup ? `[${currentGroup}]> ` : '> ',
  });

  rl.prompt();

  rl.on('line', async (line: string) => {
    const trimmed = line.trim();

    if (!trimmed) {
      rl.prompt();
      return;
    }

    if (trimmed.startsWith('/')) {
      const parts = trimmed.split(' ');
      const command = parts[0].toLowerCase();

      switch (command) {
        case '/join':
          if (parts.length < 2) {
            console.log('Usage: /join <group>');
          } else {
            const group = parts[1];
            await p2pClient!.joinGroup(group);
            currentGroup = group;
            console.log(`Joined group: ${group}`);
            rl.setPrompt(`[${group}]> `);
          }
          break;

        case '/leave':
          if (currentGroup) {
            await p2pClient!.leaveGroup(currentGroup);
            console.log(`Left group: ${currentGroup}`);
            currentGroup = null;
            rl.setPrompt('> ');
          } else {
            console.log('Not in any group');
          }
          break;

        case '/msg':
          if (parts.length < 3) {
            console.log('Usage: /msg <peer> <message>');
          } else {
            const peer = parts[1];
            const message = parts.slice(2).join(' ');

            // Create AMP node-to-node text message
            const textMessage = AmpMessageFactory.createNodeTextMessage(
              message,
              peer,
              {
                id: p2pClient!.getPeerId(),
                name: currentNickname,
                type: 'node',
              }
            );

            await p2pClient!.sendDirectMessage(
              peer,
              JSON.stringify(textMessage)
            );
            console.log(`Sent to ${peer}: ${message}`);
          }
          break;

        case '/agent-msg':
          if (parts.length < 4) {
            console.log('Usage: /agent-msg <node> <agent> <message>');
          } else {
            const node = parts[1];
            const agent = parts[2];
            const message = parts.slice(3).join(' ');

            // Create AMP node-to-agent text message
            const textMessage = AmpMessageFactory.createNodeAgentTextMessage(
              message,
              node,
              agent,
              {
                id: p2pClient!.getPeerId(),
                name: currentNickname,
                type: 'node',
              }
            );

            await p2pClient!.sendDirectMessage(
              node,
              JSON.stringify(textMessage)
            );
            console.log(`Sent to agent ${agent} on node ${node}: ${message}`);
          }
          break;

        case '/share':
          if (parts.length < 2) {
            console.log('Usage: /share <file>');
          } else {
            const filePath = parts[1];
            if (fs.existsSync(filePath)) {
              const shareCode = await p2pClient!.shareFile(filePath);
              const fileStats = fs.statSync(filePath);
              const fileName = filePath.split('/').pop() || filePath;

              console.log(`File shared! Share code: ${shareCode}`);

              // Create message for current group if in one
              if (currentGroup) {
                await p2pClient!.sendGroupMessage(currentGroup, {
                  type: 'text',
                  text: `Shared file: ${fileName} (${fileStats.size} bytes) - Share code: ${shareCode}`,
                });
              }
            } else {
              console.log(`File not found: ${filePath}`);
            }
          }
          break;

        case '/download':
          if (parts.length < 3) {
            console.log('Usage: /download <peer> <shareCode>');
          } else {
            const peer = parts[1];
            const shareCode = parts[2];
            try {
              const downloadPath = await p2pClient!.downloadFile(
                peer,
                shareCode
              );
              console.log(`File downloaded to: ${downloadPath}`);

              // Create message for download completion
              if (currentGroup) {
                await p2pClient!.sendGroupMessage(currentGroup, {
                  type: 'text',
                  text: `Downloaded file to: ${downloadPath}`,
                });
              }
            } catch (err) {
              console.error('Download failed:', err);
            }
          }
          break;

        case '/peers':
          const peers = p2pClient!.listPeers();
          if (peers.length === 0) {
            console.log('No peers discovered');
          } else {
            console.log('Discovered peers:');
            peers.forEach((peer) => {
              console.log(`  - ${peer.nickname} (${peer.peerId})`);
            });
          }
          break;

        case '/quit':
          console.log('Goodbye!');
          rl.close();
          process.exit(0);
          break;

        default:
          console.log(`Unknown command: ${command}`);
      }
    } else {
      // Send to current group
      if (currentGroup) {
        // Create AMP text message for group
        const textMessage = AmpMessageFactory.createTextMessage(
          trimmed,
          { type: 'all' },
          {
            id: p2pClient!.getPeerId(),
            name: currentNickname,
            type: 'node',
          }
        );

        // Send as JSON string in text message
        await p2pClient!.sendGroupMessage(currentGroup, {
          type: 'text',
          text: JSON.stringify(textMessage),
        });
        // Don't display user's own message - it will be received back from the P2P network
      } else {
        console.log('Not in any group. Use /join <group> to join a group.');
      }
    }

    rl.prompt();
  });

  rl.on('close', () => {
    console.log('\nGoodbye!');
    process.exit(0);
  });
}

// CLI commands
program.name('gigi-p2p').description('Gigi P2P Example CLI').version('1.0.0');

program
  .command('init')
  .description('Initialize P2P client')
  .option('-m, --mnemonic <mnemonic>', 'BIP-39 mnemonic phrase')
  .option('-n, --nickname <nickname>', 'Your nickname', 'anonymous')
  .action(async (options) => {
    console.log('Options received:', options);
    try {
      let mnemonic = options.mnemonic;
      if (!mnemonic) {
        console.log('Generating new mnemonic...');
        mnemonic = generateMnemonic();
        console.log(`Your mnemonic: ${mnemonic}`);
        console.log('Please save this mnemonic securely!');
        logger.info('Generated new mnemonic');
      } else {
        logger.info('Using provided mnemonic');
      }

      console.log('Calling initializeP2P with nickname:', options.nickname);
      await initializeP2P(mnemonic, options.nickname);
      await startInteractiveChat();
    } catch (err) {
      console.error('Failed to initialize:', err);
      logger.error({
        message: 'Failed to initialize P2P client',
        error: err,
      });
      process.exit(1);
    }
  });

program
  .command('chat')
  .description('Start interactive chat (requires init first)')
  .action(async () => {
    if (!p2pClient) {
      console.error('P2P client not initialized. Please run init first.');
      process.exit(1);
    }
    await startInteractiveChat();
  });

program
  .command('generate-mnemonic')
  .description('Generate a new BIP-39 mnemonic')
  .action(() => {
    const mnemonic = generateMnemonic();
    console.log(`Your mnemonic: ${mnemonic}`);
    console.log('Please save this mnemonic securely!');
  });

program
  .command('derive-peer-id')
  .description('Derive peer ID from mnemonic')
  .requiredOption('-m, --mnemonic <mnemonic>', 'BIP-39 mnemonic phrase')
  .action(async (options) => {
    try {
      const peerId = await derivePeerId(options.mnemonic);
      console.log(`Derived peer ID: ${peerId}`);
    } catch (err) {
      console.error('Failed to derive peer ID:', err);
      process.exit(1);
    }
  });

// Parse command line arguments
program.parse();

// If no command provided, show help
if (!process.argv.slice(2).length) {
  program.outputHelp();
}
