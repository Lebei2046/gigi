import { mnemonicToSeedSync } from '@scure/bip39';
import { HDKey } from '@scure/bip32';
import { getPublicKey } from '@noble/secp256k1';
import { keccak_256 } from '@noble/hashes/sha3';
import { bytesToHex } from '@noble/hashes/utils';
import { describe, expect, it } from 'vitest';

describe('Ethereum Address Generation', () => {
  it('should generate the correct Ethereum address from mnemonic', () => {
    // 1. Generate mnemonic (optional)
    const mnemonic = 'pioneer million sorry pipe cry garden private olive give apology inch foster';
    const eth_address = '0xebc936ea6729bc1b3f357c16245bde58af954981';

    // 2. Generate seed from mnemonic
    const seed = mnemonicToSeedSync(mnemonic);

    // 3. Generate HD wallet from seed
    const hdKey = HDKey.fromMasterSeed(seed);

    // 4. Derive Ethereum path (m/44'/60'/0'/0/0)
    const ethDerivationPath = "m/44'/60'/0'/0/0";
    const childKey = hdKey.derive(ethDerivationPath);

    if (!childKey.privateKey || !childKey.publicKey) {
      throw new Error('Failed to derive private key');
    }

    // 5. Calculate public key from private key (uncompressed format, with 04 prefix)
    const publicKey = getPublicKey(childKey.privateKey, false); // false means uncompressed

    // 6. Remove 04 prefix, get XY coordinates (32 bytes each)
    const xyPubKey = publicKey.slice(1);

    // 7. Calculate Keccak-256 hash
    const hash = keccak_256(xyPubKey);

    // 8. Take last 20 bytes as address
    const addressBytes = hash.slice(-20);

    // 9. Convert to lowercase hexadecimal
    const address = `0x${bytesToHex(addressBytes)}`.toLowerCase();

    expect(address).toEqual(eth_address);
  });
});
