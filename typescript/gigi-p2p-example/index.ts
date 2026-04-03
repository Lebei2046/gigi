#!/usr/bin/env tsx
import { Command } from 'commander';
import { P2pClient, derivePeerId, generateMnemonic } from '@gigi/p2p-ts';
import {
  AmpMessageRouter,
  AmpMessageFactory,
  InMemoryAgentRegistry,
} from '@gigi/amp-ts';
import type { AgentInfo } from '@gigi/amp-ts';
import * as fs from 'fs';

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
  console.log(`Initializing P2P client with nickname: ${nickname}`);

  // Derive peer ID from mnemonic
  const peerId = await derivePeerId(mnemonic);
  console.log(`Derived peer ID: ${peerId}`);

  // Create P2P client
  p2pClient = new P2pClient({
    mnemonic,
    nickname,
    bootstrapNodes: DEFAULT_CONFIG.bootstrapNodes,
  });

  // Initialize AMP components
  agentRegistry = new InMemoryAgentRegistry();
  messageRouter = new AmpMessageRouter(agentRegistry);

  // Set up AMP message handlers
  messageRouter.registerMessageHandler(
    'text',
    (message: any, _agentId: string | undefined) => {
      console.log(`\n[AGENT] ${message.sender.name}: ${message.content}`);
    }
  );

  messageRouter.registerMessageHandler(
    'file',
    (message: any, _agentId: string | undefined) => {
      console.log(
        `\n[AGENT] ${message.sender.name} shared a file: ${message.filename} (${message.fileSize} bytes)`
      );
      console.log(`File hash: ${message.fileHash}`);
    }
  );

  messageRouter.registerMessageHandler(
    'agent-settings-response',
    (message: any, _agentId: string | undefined) => {
      const response = message as any;
      console.log('\n[AGENT SETTINGS RESPONSE]');
      response.agents.forEach((agent: any) => {
        console.log(`Agent: ${agent.name} (${agent.id})`);
        console.log(`  Type: ${agent.type}`);
        console.log(`  Version: ${agent.version}`);
        console.log(`  Status: ${agent.status}`);
        console.log(`  Settings:`);
        agent.settings.forEach((setting: any) => {
          console.log(`    - ${setting.name}: ${setting.value}`);
        });
        if (agent.openclawAgents && agent.openclawAgents.length > 0) {
          console.log(`  OpenClaw Agents:`);
          agent.openclawAgents.forEach((openclawAgent: any) => {
            console.log(`    - ${openclawAgent.name} (${openclawAgent.id})`);
          });
        }
      });
    }
  );

  // Set up P2P event listeners
  p2pClient.onMessage((msg) => {
    try {
      const parsedMsg = JSON.parse(msg.content);
      if (parsedMsg.type && messageRouter) {
        messageRouter.routeMessage(parsedMsg);
      }
    } catch {
      // Not an AMP message, display as regular message
      console.log(`\n[${msg.fromNickname}]: ${msg.content}`);
    }
  });

  // Start the P2P client
  await p2pClient.start();
  console.log('P2P client started successfully!');
  console.log(`Your peer ID: ${p2pClient.getPeerId()}`);

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
  console.log('  /join <group>     - Join a group');
  console.log('  /leave            - Leave current group');
  console.log('  /msg <peer> <msg> - Send direct message');
  console.log('  /share <file>     - Share a file');
  console.log('  /download <code>  - Download a file');
  console.log('  /peers            - List connected peers');
  console.log('  /agents           - List registered agents');
  console.log('  /settings         - Query agent settings');
  console.log('  /quit             - Exit');
  console.log('  <message>         - Send to current group\n');

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

            // Create AMP text message
            const textMessage = AmpMessageFactory.createTextMessage(
              message,
              { type: 'specific', agentIds: [peer] },
              {
                id: p2pClient!.getPeerId(),
                name: currentNickname,
                type: 'owner',
              }
            );

            await p2pClient!.sendDirectMessage(
              peer,
              JSON.stringify(textMessage)
            );
            console.log(`Sent to ${peer}: ${message}`);
          }
          break;

        case '/share':
          if (parts.length < 2) {
            console.log('Usage: /share <file>');
          } else {
            const filePath = parts[1];
            if (fs.existsSync(filePath)) {
              const shareCode = await p2pClient!.shareFile(filePath);
              console.log(`File shared! Share code: ${shareCode}`);
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
            } catch (err) {
              console.error('Download failed:', err);
            }
          }
          break;

        case '/peers':
          const peers = p2pClient!.getPeers();
          if (peers.length === 0) {
            console.log('No peers connected');
          } else {
            console.log('Connected peers:');
            peers.forEach((peer) => {
              console.log(`  - ${peer.nickname} (${peer.peerId})`);
            });
          }
          break;

        case '/agents':
          if (agentRegistry) {
            const agents = agentRegistry.getAllAgents();
            if (agents.length === 0) {
              console.log('No agents registered');
            } else {
              console.log('Registered agents:');
              agents.forEach((agent) => {
                console.log(
                  `  - ${agent.name} (${agent.id}) - ${agent.status}`
                );
              });
            }
          }
          break;

        case '/settings':
          if (messageRouter && agentRegistry) {
            // Register a temporary agent to receive the response
            const tempAgent: AgentInfo = {
              id: 'settings-query',
              name: 'Settings Query',
              type: 'query',
              version: '1.0.0',
              settings: [],
              status: 'online',
            };
            agentRegistry.registerAgent(tempAgent);

            // Create and send settings query
            const query = AmpMessageFactory.createAgentSettingsQuery(
              {
                id: p2pClient!.getPeerId(),
                name: currentNickname,
                type: 'owner',
              },
              undefined
            );

            messageRouter.routeMessage(query);
            console.log('Sent agent settings query');
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
        await p2pClient!.sendGroupMessage(currentGroup, trimmed);
        console.log(`[You]: ${trimmed}`);
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
    try {
      let mnemonic = options.mnemonic;
      if (!mnemonic) {
        console.log('Generating new mnemonic...');
        mnemonic = generateMnemonic();
        console.log(`Your mnemonic: ${mnemonic}`);
        console.log('Please save this mnemonic securely!');
      }

      await initializeP2P(mnemonic, options.nickname);
      await startInteractiveChat();
    } catch (err) {
      console.error('Failed to initialize:', err);
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
