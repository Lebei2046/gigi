#!/usr/bin/env node
import { P2pClient } from '@gigi/p2p-ts';
import { Command } from 'commander';
import readline from 'readline';

const program = new Command();

// Store file share messages for download lookup
const fileShareMessages: Map<string, { shareCode: string; fromPeerId: string; fromNickname: string; filename: string }> = new Map();

// Helper function to create a chat client
async function createChatClient(nickname: string): Promise<P2pClient> {
  const client = new P2pClient({
    nickname,
    outputDirectory: `./downloads-${nickname}`,
  });

  await client.start();
  console.log(`${nickname} started with peer ID: ${client.getPeerId()}`);
  console.log(`${nickname} listening on: ${client.getMultiaddrs().join(', ')}`);

  // Set up event listeners
  client.onEvent(async (event) => {
    switch (event.type) {
      case 'peer-discovered':
        console.log(`\n${nickname} discovered peer: ${event.nickname} (${event.peerId})`);
        break;
      case 'group-message':
        if (event.fromNickname !== nickname) {
          if (event.content.type === 'text') {
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
        }
        break;
      case 'direct-message':
        console.log(`\n[DIRECT] ${event.fromNickname}: ${event.message}`);
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
