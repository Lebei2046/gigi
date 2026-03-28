#!/usr/bin/env node

/**
 * Generate a BIP-39 mnemonic phrase for Gigi P2P
 * 
 * This script generates a new 12-word BIP-39 mnemonic phrase that can be used
 * to configure the Gigi OpenClaw plugin. The mnemonic is used to derive the
 * peer ID and private key for the Gigi P2P client.
 */

import { generateMnemonic, derivePeerId } from '@gigi/p2p-ts';

console.log('Generating BIP-39 mnemonic phrase for Gigi P2P...\n');

async function generate() {
  try {
    // Generate a new 12-word mnemonic phrase
    const mnemonic = generateMnemonic();
    
    // Derive peer ID from the mnemonic
    const peerId = await derivePeerId(mnemonic);
    
    console.log('Generated mnemonic phrase:');
    console.log('========================');
    console.log(mnemonic);
    console.log('\nDerived peer ID:');
    console.log('================');
    console.log(peerId);
    console.log('\nAdd this to your OpenClaw channel configuration:');
    console.log('==============================================');
    console.log(`{
  "channels": {
    "gigi-p2p-bundled": {
      "peerId": "${peerId}",
      "multiaddrs": [
        "/ip4/0.0.0.0/tcp/0",
        "/ip4/0.0.0.0/tcp/0/ws"
      ],
      "peerIdJson": {
        "id": "${peerId}",
        "mnemonic": "${mnemonic}"
      },
      "displayName": "My Gigi Node",
      "enabled": true
    }
  }
}`);
    
    console.log('\nImportant:');
    console.log('- This mnemonic phrase is the root of all your Gigi P2P keys');
    console.log('- Keep it secure and never share it with anyone');
    console.log('- If you lose this mnemonic, you will lose access to your Gigi P2P identity');
    console.log('- Make a backup of this mnemonic in a safe place');
    
  } catch (error) {
    console.error('Error generating mnemonic or deriving peer ID:', error);
    process.exit(1);
  }
}

// Run the async function
generate();
