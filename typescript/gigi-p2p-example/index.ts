#!/usr/bin/env node
import { P2pClient } from '@gigi/p2p-ts';
import { Command } from 'commander';
import readline from 'readline';
import { AmpMessageFactory, AmpMessageRouter, InMemoryAgentRegistry, AmpMessage } from '@gigi/amp-ts';

const program = new Command();

// Store file share messages for download lookup
const fileShareMessages: Map<string, { shareCode: string; fromPeerId: string; fromNickname: string; filename: string }> = new Map();

// AMP components
let agentRegistry: InMemoryAgentRegistry;
let messageRouter: AmpMessageRouter;
const AGENT_GROUP_NAME = 'gigi-agents'; // Dedicated group for agent communication

// Helper function to create a chat client
async function createChatClient(nickname: string): Promise<P2pClient> {
  const client = new P2pClient({
    nickname,
    outputDirectory: `./downloads-${nickname}`,
  });

  await client.start();
  console.log(`${nickname} started with peer ID: ${client.getPeerId()}`);
  console.log(`${nickname} listening on: ${client.getMultiaddrs().join(', ')}`);

  // Join the agent group
  try {
    await client.joinGroup(AGENT_GROUP_NAME);
    console.log(`Joined agent group: ${AGENT_GROUP_NAME}`);
  } catch (error) {
    console.warn(`Failed to join agent group: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }

  // Initialize AMP components
  agentRegistry = new InMemoryAgentRegistry();
  messageRouter = new AmpMessageRouter(agentRegistry);

  // Set up AMP message handlers
  messageRouter.registerMessageHandler('text', (message: any, agentId: string | undefined) => {
    console.log(`\n[AGENT] ${message.sender.name}: ${message.content}`);
  });

  messageRouter.registerMessageHandler('file', (message: any, agentId: string | undefined) => {
    console.log(`\n[AGENT] ${message.sender.name} shared a file: ${message.filename} (${message.fileSize} bytes)`);
    console.log(`File hash: ${message.fileHash}`);
  });

  messageRouter.registerMessageHandler('agent-settings-response', (message: any, agentId: string | undefined) => {
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
          console.log(`      Model: ${openclawAgent.model}`);
          console.log(`      Status: ${openclawAgent.status}`);
        });
      }
    });
  });

  // Set up event listeners
  client.onEvent(async (event) => {
    switch (event.type) {
      case 'peer-discovered':
        console.log(`\n${nickname} discovered peer: ${event.nickname} (${event.peerId})`);
        break;
      case 'group-message':
        // Check if this is an AMP message in the agent group
        if (event.content.type === 'text' && event.content.text.startsWith('{"type":"')) {
              try {
                const ampMessage = JSON.parse(event.content.text) as AmpMessage;
                if (ampMessage.type) {
                  console.log(`\n[AMP GROUP] ${event.fromNickname} sent an AMP message: ${ampMessage.type}`);
                  // Show detailed AMP message content
                  if (ampMessage.type === 'text' && 'content' in ampMessage) {
                    console.log(`  Content: ${ampMessage.content}`);
                  } else if (ampMessage.type === 'file' && 'filename' in ampMessage) {
                    console.log(`  File: ${ampMessage.filename} (${ampMessage.fileSize} bytes)`);
                    console.log(`  File hash: ${ampMessage.fileHash}`);
                  } else if (ampMessage.type === 'agent-settings-response' && 'agents' in ampMessage) {
                    console.log(`  Agent settings response received for ${ampMessage.agents.length} agents`);
                  }
                  messageRouter.routeMessage(ampMessage);
                } else {
                  // Regular text message
                  console.log(`\n[GROUP] ${event.fromNickname}: ${event.content.text}`);
                }
              } catch (e) {
                // Not an AMP message, treat as regular text
                console.log(`\n[GROUP] ${event.fromNickname}: ${event.content.text}`);
              }
            } else if (event.content.type === 'text') {
              console.log(`\n[GROUP] ${event.fromNickname}: ${event.content.text}`);
            } else if (event.content.type === 'fileShare') {
              console.log(`\n[GROUP] ${event.fromNickname} shared a file: ${event.content.filename} (${event.content.fileSize} bytes)`);
              console.log(`Use /download ${event.content.shareCode} to download this file`);
              // Store the file share message for later download
              fileShareMessages.set(event.content.shareCode, {
                shareCode: event.content.shareCode,
                fromPeerId: event.content.fromPeerId,
                fromNickname: event.content.fromNickname,
                filename: event.content.filename
              });
            }
        break;
      case 'direct-message':
        // Check if it's an AMP message
        try {
          const ampMessage = JSON.parse(event.message) as AmpMessage;
          if (ampMessage.type) {
            console.log(`\n[AMP DIRECT] ${event.fromNickname} sent an AMP message: ${ampMessage.type}`);
            messageRouter.routeMessage(ampMessage);
          } else {
            console.log(`\n[DIRECT] ${event.fromNickname}: ${event.message}`);
          }
        } catch (e) {
          // Not an AMP message, treat as regular text
          console.log(`\n[DIRECT] ${event.fromNickname}: ${event.message}`);
        }
        break;
      case 'connected':
        console.log(`\n${nickname} connected to: ${event.nickname}`);
        break;
      case 'disconnected':
        console.log(`\n${nickname} disconnected from: ${event.nickname}`);
        break;
      case 'listening-on':
        console.log(`${nickname} listening on: ${event.address}`);
        break;
      case 'file-shared':
        console.log(`\nFile shared successfully with share code: ${event.info.shareCode}`);
        console.log(`Use /group <group> /file ${event.info.shareCode} ${event.info.name} ${event.info.size} ${event.info.mimeType} to share this file in a group`);
        break;
      case 'file-download-started':
        console.log(`\nDownload started: ${event.filename} (Download ID: ${event.downloadId})`);
        break;
      case 'file-download-progress':
        console.log(`\nDownload progress: ${event.filename} - ${event.downloadedChunks}/${event.totalChunks} chunks`);
        break;
      case 'file-download-completed':
        console.log(`\nDownload completed: ${event.filename} saved to ${event.path}`);
        break;
      case 'file-share-request':
        console.log(`\n${event.fromNickname} requested to download file: ${event.filename}`);
        break;
    }
  });

  return client;
}

// Helper function to set up readline interface for user input
function setupReadline(nickname: string, client: P2pClient): void {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    prompt: `${nickname}> `
  });

  rl.prompt();

  rl.on('line', async (line) => {
    const input = line.trim();
    
    if (input.startsWith('/join ')) {
      const groupName = input.substring(6);
      await client.joinGroup(groupName);
      console.log(`Joined group: ${groupName}`);
    } else if (input.startsWith('/leave ')) {
      const groupName = input.substring(7);
      await client.leaveGroup(groupName);
      console.log(`Left group: ${groupName}`);
    } else if (input.startsWith('/group ')) {
      const parts = input.split(' ');
      if (parts.length < 3) {
        console.log('Usage: /group <group-name> <message>');
        console.log('Usage: /group <group-name> /file <share-code> <filename> <size> <mime-type>');
        rl.prompt();
        return;
      }
      const groupName = parts[1];
      
      if (parts[2] === '/file') {
        // Handle file share message
        if (parts.length < 4) {
          console.log('Usage: /group <group-name> /file <share-code>');
          rl.prompt();
          return;
        }
        const shareCode = parts[3];
        
        // Get file information from share code
        const file = client.getFileByShareCode(shareCode);
        if (!file) {
          console.error(`Error: File with share code ${shareCode} not found. Make sure you've shared this file first with /share command.`);
          rl.prompt();
          return;
        }
        
        await client.sendGroupMessage(groupName, {
          type: 'fileShare',
          shareCode,
          filename: file.info.name,
          fileSize: file.info.size,
          fileType: file.info.mimeType
        });
        console.log(`Shared file ${file.info.name} in group ${groupName}`);
      } else {
        // Handle text message
        const message = parts.slice(2).join(' ');
        await client.sendGroupMessage(groupName, {
          type: 'text',
          text: message
        });
      }
    } else if (input.startsWith('/direct ')) {
      const parts = input.split(' ');
      if (parts.length < 3) {
        console.log('Usage: /direct <nickname> <message>');
        rl.prompt();
        return;
      }
      const targetNickname = parts[1];
      const message = parts.slice(2).join(' ');
      try {
        await client.sendDirectMessageToNickname(targetNickname, message);
      } catch (error) {
        console.error(`Error sending direct message: ${error instanceof Error ? error.message : 'Unknown error'}`);
      }
    } else if (input === '/peers') {
      const peers = client.listPeers();
      console.log('Connected peers:');
      peers.forEach(peer => {
        console.log(`- ${peer.nickname || peer.peerId} (${peer.peerId})`);
      });
    } else if (input === '/groups') {
      const groups = client.getJoinedGroups();
      console.log('Joined groups:');
      groups.forEach(group => {
        console.log(`- ${group.name}`);
      });
    } else if (input.startsWith('/connect ')) {
      const multiaddr = input.substring(8);
      try {
        await client.connectToPeer(multiaddr);
        console.log(`Connected to peer at ${multiaddr}`);
      } catch (error) {
        console.error(`Error connecting to peer: ${error instanceof Error ? error.message : 'Unknown error'}`);
      }
    } else if (input.startsWith('/share ')) {
      const filePath = input.substring(7);
      try {
        const shareCode = await client.shareFile(filePath);
        console.log(`File shared successfully with share code: ${shareCode}`);
      } catch (error) {
        console.error(`Error sharing file: ${error instanceof Error ? error.message : 'Unknown error'}`);
      }
    } else if (input.startsWith('/download ')) {
      const parts = input.split(' ');
      if (parts.length < 2) {
        console.log('Usage: /download <share-code>');
        rl.prompt();
        return;
      }
      const shareCode = parts[1];
      const fileShare = fileShareMessages.get(shareCode);
      if (!fileShare) {
        console.error(`Error: File with share code ${shareCode} not found. Make sure you've received the file share message.`);
        rl.prompt();
        return;
      }
      try {
        // Use downloadFileByPeerId if we have the peer ID
        const downloadId = await client.downloadFileByPeerId(fileShare.fromPeerId, fileShare.fromNickname, shareCode);
        console.log(`Download started with ID: ${downloadId}`);
      } catch (error) {
        console.error(`Error downloading file: ${error instanceof Error ? error.message : 'Unknown error'}`);
      }
    } else if (input === '/files') {
      const files = client.listSharedFiles();
      console.log('Shared files:');
      files.forEach(file => {
        console.log(`- ${file.info.name} (${file.info.size} bytes) - Share code: ${file.info.shareCode}`);
      });
    } else if (input === '/downloads') {
      const downloads = client.getActiveDownloads();
      console.log('Active downloads:');
      downloads.forEach(download => {
        console.log(`- ${download.filename} (${download.downloadedChunks}/${download.totalChunks} chunks)`);
      });
    } else if (input.startsWith('/agent text ')) {
      const parts = input.split(' ');
      if (parts.length < 4) {
        console.log('Usage: /agent text <all|agent-id> <message>');
        rl.prompt();
        return;
      }
      const targetType = parts[2];
      const message = parts.slice(3).join(' ');
      
      let target;
      if (targetType === 'all') {
        target = { type: 'all' as const };
      } else {
        target = { type: 'specific' as const, agentIds: [targetType] };
      }
      
      const ampMessage = AmpMessageFactory.createTextMessage(
        message,
        target,
        { id: client.getPeerId(), name: nickname, type: 'owner' as const }
      );
      
      // Send AMP message to the agent group
      try {
        await client.sendGroupMessage(AGENT_GROUP_NAME, {
          type: 'text',
          text: JSON.stringify(ampMessage)
        });
        console.log(`Sent AMP text message to agent group ${AGENT_GROUP_NAME}`);
      } catch (error) {
        console.error(`Error sending message to agent group: ${error instanceof Error ? error.message : 'Unknown error'}`);
      }
    } else if (input.startsWith('/agent file ')) {
      const parts = input.split(' ');
      if (parts.length < 5) {
        console.log('Usage: /agent file <all|agent-id> <share-code>');
        rl.prompt();
        return;
      }
      const targetType = parts[2];
      const shareCode = parts[3];
      
      // Get file information from share code
      const file = client.getFileByShareCode(shareCode);
      if (!file) {
        console.error(`Error: File with share code ${shareCode} not found. Make sure you've shared this file first with /share command.`);
        rl.prompt();
        return;
      }
      
      let target;
      if (targetType === 'all') {
        target = { type: 'all' as const };
      } else {
        target = { type: 'specific' as const, agentIds: [targetType] };
      }
      
      const ampMessage = AmpMessageFactory.createFileMessage(
        file.info.name,
        file.info.size,
        shareCode, // Using shareCode as fileHash for simplicity
        target,
        { id: client.getPeerId(), name: nickname, type: 'owner' as const }
      );
      
      // Send AMP message to the agent group
      try {
        await client.sendGroupMessage(AGENT_GROUP_NAME, {
          type: 'text',
          text: JSON.stringify(ampMessage)
        });
        console.log(`Sent AMP file message to agent group ${AGENT_GROUP_NAME}`);
      } catch (error) {
        console.error(`Error sending message to agent group: ${error instanceof Error ? error.message : 'Unknown error'}`);
      }
    } else if (input.startsWith('/agent query')) {
      const parts = input.split(' ');
      let agentIds: string[] | undefined;
      if (parts.length > 2) {
        agentIds = parts.slice(2);
      }
      
      const ampMessage = AmpMessageFactory.createAgentSettingsQuery(
        { id: client.getPeerId(), name: nickname, type: 'owner' as const },
        agentIds
      );
      
      // Send AMP message to the agent group
      try {
        await client.sendGroupMessage(AGENT_GROUP_NAME, {
          type: 'text',
          text: JSON.stringify(ampMessage)
        });
        console.log(`Sent AMP agent settings query to agent group ${AGENT_GROUP_NAME} ${agentIds ? 'for agents: ' + agentIds.join(', ') : 'for all agents'}`);
      } catch (error) {
        console.error(`Error sending message to agent group: ${error instanceof Error ? error.message : 'Unknown error'}`);
      }
    } else if (input.startsWith('/agent register ')) {
      const parts = input.split(' ');
      if (parts.length < 5) {
        console.log('Usage: /agent register <id> <name> <type> <version>');
        rl.prompt();
        return;
      }
      const id = parts[2];
      const name = parts[3];
      const type = parts[4];
      const version = parts[5] || '1.0.0';
      
      agentRegistry.registerAgent({
        id,
        name,
        type,
        version,
        settings: [],
        status: 'online'
      });
      console.log(`Registered agent: ${name} (${id})`);
    } else if (input === '/agents') {
      const agents = agentRegistry.getAllAgents();
      console.log('Registered agents:');
      agents.forEach((agent: any) => {
        console.log(`- ${agent.name} (${agent.id}) - ${agent.status}`);
      });
    } else if (input === '/help') {
      console.log('Available commands:');
      console.log('  /join <group-name>                  - Join a group');
      console.log('  /leave <group-name>                 - Leave a group');
      console.log('  /group <group> <msg>                - Send text message to group');
      console.log('  /group <group> /file <share-code>    - Share file in group');
      console.log('  /share <file-path>                  - Share a file and get share code');
      console.log('  /download <share-code>              - Download a file using share code');
      console.log('  /direct <nick> <msg>                - Send direct message');
      console.log('  /connect <multiaddr>                - Connect to a peer');
      console.log('  /files                              - List shared files');
      console.log('  /downloads                          - List active downloads');
      console.log('  /peers                              - List connected peers');
      console.log('  /groups                             - List joined groups');
      console.log('  /agent text <all|agent-id> <msg>    - Send AMP text message to agent(s)');
      console.log('  /agent file <all|agent-id> <share-code> - Send AMP file message to agent(s)');
      console.log('  /agent query [agent-id...]          - Query agent settings');
      console.log('  /agent register <id> <name> <type> <version> - Register an agent');
      console.log('  /agents                             - List registered agents');
      console.log('  /help                               - Show this help');
      console.log('  /exit                               - Exit the program');
    } else if (input === '/exit') {
      await client.stop();
      rl.close();
      process.exit(0);
    } else if (input.length > 0) {
      console.log('Unknown command. Type /help for available commands.');
    }
    
    rl.prompt();
  });

  rl.on('close', async () => {
    await client.stop();
    process.exit(0);
  });
}

// Main function to run the chat client
async function main(): Promise<void> {
  program
    .name('gigi-p2p')
    .description('Gigi P2P Client for group chat, file sharing, and more')
    .version('1.0.0')
    .argument('<nickname>', 'Your nickname for the chat')
    .option('-b, --bootstrap <nodes...>', 'Bootstrap nodes to connect to')
    .option('--no-mdns', 'Disable mDNS discovery')
    .parse();

  const options = program.opts();
  const nickname = program.args[0];

  if (!nickname) {
    console.error('Error: Nickname is required');
    program.help();
    process.exit(1);
  }

  console.log(`Starting chat client as ${nickname}...`);
  if (options.bootstrap) {
    console.log(`Bootstrap nodes: ${options.bootstrap.join(', ')}`);
  }
  if (!options.mdns) {
    console.log('mDNS discovery disabled');
  }

  try {
    const client = await createChatClient(nickname);
    setupReadline(nickname, client);
  } catch (error) {
    console.error(`Error starting client: ${error instanceof Error ? error.message : 'Unknown error'}`);
    process.exit(1);
  }
}

main().catch(console.error);
