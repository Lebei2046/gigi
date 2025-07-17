import { mnemonicToSeedSync } from '@scure/bip39';
import { HDKey } from '@scure/bip32';
import { getPublicKey } from '@noble/secp256k1';
import { keccak_256 } from '@noble/hashes/sha3';
import { bytesToHex } from '@noble/hashes/utils';
import { describe, expect, it } from 'vitest';

describe('Ethereum Address Generation', () => {
  it('should generate the correct Ethereum address from mnemonic', () => {
    // 1. 生成助记词（可选）
    const mnemonic = 'pioneer million sorry pipe cry garden private olive give apology inch foster';
    const eth_address = '0xebc936ea6729bc1b3f357c16245bde58af954981';

    // 2. 从助记词生成种子
    const seed = mnemonicToSeedSync(mnemonic);

    // 3. 从种子生成HD钱包
    const hdKey = HDKey.fromMasterSeed(seed);

    // 4. 派生以太坊路径 (m/44'/60'/0'/0/0)
    const ethDerivationPath = "m/44'/60'/0'/0/0";
    const childKey = hdKey.derive(ethDerivationPath);

    if (!childKey.privateKey || !childKey.publicKey) {
      throw new Error('Failed to derive private key');
    }

    // 5. 从私钥计算公钥 (非压缩格式，带04前缀)
    const publicKey = getPublicKey(childKey.privateKey, false); // false表示非压缩

    // 6. 去掉04前缀，得到XY坐标 (各32字节)
    const xyPubKey = publicKey.slice(1);

    // 7. 计算Keccak-256哈希
    const hash = keccak_256(xyPubKey);

    // 8. 取最后20字节作为地址
    const addressBytes = hash.slice(-20);

    // 9. 转换为小写十六进制
    const address = `0x${bytesToHex(addressBytes)}`.toLowerCase();

    expect(address).toEqual(eth_address);
  });
});
