#!/usr/bin/env ts-node
import { derivePeerId } from '@gigi/p2p';

// 使用提供的助记词
const mnemonic = 'mango valley develop put bleak runway rocket proud attend shell comfort angry';

console.log('Generating peer ID from mnemonic...');
console.log('Mnemonic:', mnemonic);

try {
  // 从助记词派生对等节点 ID
  const peerId = await derivePeerId(mnemonic);
  console.log('Generated Peer ID:', peerId);
} catch (error) {
  console.error('Error generating peer ID:', error);
}
