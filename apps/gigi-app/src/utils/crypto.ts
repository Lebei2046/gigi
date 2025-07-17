import { HDKey } from '@scure/bip32';
import { wordlist } from '@scure/bip39/wordlists/english';
import { generateMnemonic, mnemonicToSeedSync } from '@scure/bip39';
import { hkdf } from '@noble/hashes/hkdf';
import { keccak_256 } from '@noble/hashes/sha3';
import { xchacha20poly1305 } from '@noble/ciphers/chacha';
import { randomBytes, hexToBytes, bytesToHex } from '@noble/hashes/utils';

/**
 * Functions for generating mnemonics, deriving keys, encrypting and decrypting
 * mnemonics, and generating addresses.
 */

export function generateMnemonics(): string[] {
  const mnemonic = generateMnemonic(wordlist);
  return mnemonic.split(' ');
}

export function deriveKeys(mnemonic: string[]): {
  publicKey: Uint8Array;
  privateKey: Uint8Array;
} {
  if (mnemonic.length !== 12 && mnemonic.length !== 24) {
    throw new Error('Mnemonic must be 12 or 24 words long');
  }

  const seed = mnemonicToSeedSync(mnemonic.join(' '));
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

export function encryptMnemonics(mnemonic: string[], password: string): {
  mnemonic: string,
  nonce: string,
} {
  const nonce = randomBytes(24); // XChaCha20需要24字节nonce
  const cipher = xchacha20poly1305(expandTo32Bytes(stringToUint8Array(password)), nonce);
  const ciphertext = cipher.encrypt(stringToUint8Array(mnemonic.join(' ')));

  return {
    mnemonic: bytesToHex(ciphertext),
    nonce: bytesToHex(nonce),
  };
}

export function decryptMnemonics(ciphertext: string, key: string, nonce: string): string[] {
  const cipher = xchacha20poly1305(expandTo32Bytes(stringToUint8Array(key)), hexToBytes(nonce));
  const decrypted = cipher.decrypt(hexToBytes(ciphertext));
  return new TextDecoder().decode(decrypted).split(' ');
}


export function generateAddress(mnemonic: string[]): string {
  const { publicKey } = deriveKeys(mnemonic);
  const hash = keccak_256(publicKey.slice(1));
  return bytesToHex(hash.slice(-20));
}

function stringToUint8Array(str: string): Uint8Array {
  const encoder = new TextEncoder();
  return encoder.encode(str);
}

function expandTo32Bytes(input: Uint8Array): Uint8Array {
  return hkdf(keccak_256, input, undefined, undefined, 32);
}
