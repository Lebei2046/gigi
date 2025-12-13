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
  const nonce = randomBytes(24) // XChaCha20需要24字节nonce
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
    throw new Error('解密失败，请检查密码是否正确或数据是否损坏')
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
  // 1. 从私钥计算公钥 (非压缩格式，带04前缀)
  const publicKey = getPublicKey(privKey, false) // false表示非压缩

  // 2. 去掉04前缀，得到XY坐标 (各32字节)
  // 3. 计算Keccak-256哈希
  // 4. 取最后20字节作为地址
  const hash_20 = keccak_256(publicKey.slice(1)).slice(-20)

  // 5. 转换为小写十六进制
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
