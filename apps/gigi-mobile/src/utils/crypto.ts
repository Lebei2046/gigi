import { HDKey } from '@scure/bip32'
import { wordlist } from '@scure/bip39/wordlists/english'
import { generateMnemonic, mnemonicToSeedSync } from '@scure/bip39'
import { hkdf } from '@noble/hashes/hkdf'
import { keccak_256 } from '@noble/hashes/sha3'
import { getPublicKey } from '@noble/secp256k1'
import { xchacha20poly1305 } from '@noble/ciphers/chacha'
import { randomBytes, hexToBytes, bytesToHex } from '@noble/hashes/utils'
import { MessagingClient } from './messaging'

/**
 * Functions for generating mnemonics, deriving keys, encrypting and decrypting
 * mnemonics, and generating addresses.
 */

export function generateMnemonics(): string[] {
  const mnemonic = generateMnemonic(wordlist)
  return mnemonic.split(' ')
}

export function deriveKeys(mnemonic: string[]): {
  publicKey: Uint8Array
  privateKey: Uint8Array
} {
  if (mnemonic.length !== 12 && mnemonic.length !== 24) {
    throw new Error('Mnemonic must be 12 or 24 words long')
  }

  const seed = mnemonicToSeedSync(mnemonic.join(' '))
  const hdKey = HDKey.fromMasterSeed(seed)
  const childKey = hdKey.derive("m/44'/60'/0'/0/0")

  if (!childKey.publicKey || !childKey.privateKey) {
    throw new Error('Key derivation failed')
  }

  return {
    publicKey: childKey.publicKey,
    privateKey: childKey.privateKey,
  }
}

export function encryptMnemonics(
  mnemonic: string[],
  password: string
): {
  mnemonic: string
  nonce: string
} {
  const nonce = randomBytes(24) // XChaCha20 requires 24-byte nonce
  const cipher = xchacha20poly1305(
    expandTo32Bytes(stringToUint8Array(password)),
    nonce
  )
  const ciphertext = cipher.encrypt(stringToUint8Array(mnemonic.join(' ')))

  return {
    mnemonic: bytesToHex(ciphertext),
    nonce: bytesToHex(nonce),
  }
}

export function decryptMnemonics(
  ciphertext: string,
  key: string,
  nonce: string
): string[] {
  try {
    const cipher = xchacha20poly1305(
      expandTo32Bytes(stringToUint8Array(key)),
      hexToBytes(nonce)
    )
    const decrypted = cipher.decrypt(hexToBytes(ciphertext))
    return new TextDecoder().decode(decrypted).split(' ')
  } catch (error) {
    console.error('Decryption failed:', error)
    throw new Error('Decryption failed, please check if password is correct or data is corrupted')
  }
}

export interface AddressInfo {
  address: string
  peerId: string
}
export async function generateAddress(
  mnemonic: string[]
): Promise<AddressInfo> {
  const { privateKey } = deriveKeys(mnemonic)

  const address = getAddressByPrivateKey(privateKey)
  const peerId = await MessagingClient.tryGetPeerId(privateKey)

  return { address, peerId }
}

export function getAddress(mnemonic: string[]): string {
  const { privateKey } = deriveKeys(mnemonic)
  return getAddressByPrivateKey(privateKey)
}

function getAddressByPrivateKey(privKey: Uint8Array): string {
  // 1. Calculate public key from private key (uncompressed format, with 04 prefix)
  const publicKey = getPublicKey(privKey, false) // false means uncompressed

  // 2. Remove 04 prefix, get XY coordinates (32 bytes each)
  // 3. Calculate Keccak-256 hash
  // 4. Take last 20 bytes as address
  const hash_20 = keccak_256(publicKey.slice(1)).slice(-20)

  // 5. Convert to lowercase hexadecimal
  const address = `0x${bytesToHex(hash_20)}`.toLowerCase()

  return address
}

function stringToUint8Array(str: string): Uint8Array {
  const encoder = new TextEncoder()
  return encoder.encode(str)
}

function expandTo32Bytes(input: Uint8Array): Uint8Array {
  return hkdf(keccak_256, input, undefined, undefined, 32)
}

export function getPrivateKeyFromMnemonic(mnemonic: string[]): Uint8Array {
  const { privateKey } = deriveKeys(mnemonic)
  return privateKey
}

export function deriveGroupPrivateKey(mnemonic: string[]): Uint8Array {
  if (mnemonic.length !== 12 && mnemonic.length !== 24) {
    throw new Error('Mnemonic must be 12 or 24 words long')
  }

  const seed = mnemonicToSeedSync(mnemonic.join(' '))
  const hdKey = HDKey.fromMasterSeed(seed)
  const groupChildKey = hdKey.derive("m/44'/60'/1'/0/0")

  if (!groupChildKey.privateKey) {
    throw new Error('Group key derivation failed')
  }

  return groupChildKey.privateKey
}

export async function generateGroupPeerId(mnemonic: string[]): Promise<string> {
  const groupPrivateKey = deriveGroupPrivateKey(mnemonic)
  return await MessagingClient.tryGetPeerId(groupPrivateKey)
}
