import { generateMnemonic, mnemonicToSeedSync } from '@scure/bip39';
import { HDKey } from '@scure/bip32';
import { wordlist } from '@scure/bip39/wordlists/english';

export class CryptoService {
  static generateMnemonic(): string[] {
    const mnemonic = generateMnemonic(wordlist);
    return mnemonic.split(' ');
  }

  static deriveKeys(mnemonic: string[], password?: string): {
    publicKey: Uint8Array;
    privateKey: Uint8Array;
  } {
    const seed = mnemonicToSeedSync(mnemonic.join(' '), password);
    const hdKey = HDKey.fromMasterSeed(seed);
    const childKey = hdKey.derive("m/44'/60'/0'/0/0");

    if (!childKey.publicKey || !childKey.privateKey) {
      throw new Error('Key derivation failed');
    }

    return {
      publicKey: childKey.publicKey,
      privateKey: childKey.privateKey,
    };
  }
}
