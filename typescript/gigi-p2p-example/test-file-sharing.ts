#!/usr/bin/env tsx
import { P2pClient } from '@gigi/p2p-ts';

async function testFileSharing() {
  console.log('Starting file sharing test...');
  
  // Create Alice and Bob clients
  const alice = new P2pClient({ nickname: 'Alice' });
  const bob = new P2pClient({ nickname: 'Bob' });
  
  try {
    // Start both clients
    console.log('Starting Alice...');
    await alice.start();
    console.log(`Alice started with peer ID: ${alice.getPeerId()}`);
    const aliceAddrs = alice.getMultiaddrs();
    console.log(`Alice addresses: ${aliceAddrs.join(', ')}`);
    
    console.log('Starting Bob...');
    await bob.start();
    console.log(`Bob started with peer ID: ${bob.getPeerId()}`);
    const bobAddrs = bob.getMultiaddrs();
    console.log(`Bob addresses: ${bobAddrs.join(', ')}`);
    
    // Explicitly add each other's addresses
    console.log('Adding Alice\'s addresses to Bob...');
    bob.addPeer('Alice', alice.getPeerId(), aliceAddrs);
    
    console.log('Adding Bob\'s addresses to Alice...');
    alice.addPeer('Bob', bob.getPeerId(), bobAddrs);
    
    // Wait for clients to discover each other
    console.log('Waiting for peer discovery...');
    await new Promise(resolve => setTimeout(resolve, 5000));
    
    // Alice shares a file
    console.log('Alice sharing file...');
    const shareCode = await alice.shareFile('/home/lebei/crdt.pdf');
    console.log(`Alice shared file with share code: ${shareCode}`);
    
    // Both join the chat group
    console.log('Alice joining group...');
    await alice.joinGroup('chat');
    
    console.log('Bob joining group...');
    await bob.joinGroup('chat');
    
    // Wait for group join to complete
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Alice shares the file in the group
    console.log('Alice sharing file in group...');
    await alice.sendGroupMessage('chat', {
      type: 'fileShare',
      shareCode,
      filename: 'crdt.pdf',
      fileSize: 342201,
      fileType: 'application/pdf'
    });
    
    // Wait for message to be received
    console.log('Waiting for Bob to receive file share message...');
    await new Promise(resolve => setTimeout(resolve, 3000));
    
    // Bob downloads the file
    console.log('Bob downloading file...');
    const downloadId = await bob.downloadFile('Alice', shareCode);
    console.log(`Bob started download with ID: ${downloadId}`);
    
    // Wait for download to complete
    console.log('Waiting for download to complete...');
    await new Promise(resolve => setTimeout(resolve, 10000));
    
    console.log('File sharing test completed successfully!');
    
  } catch (error) {
    console.error('Test failed:', error);
  } finally {
    // Stop both clients
    try {
      await alice.stop();
      await bob.stop();
    } catch (error) {
      console.error('Error stopping clients:', error);
    }
  }
}

testFileSharing();
