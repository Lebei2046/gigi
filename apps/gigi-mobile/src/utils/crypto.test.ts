import {
  generateMnemonics,
  deriveKeys,
  encryptMnemonics,
  decryptMnemonics,
  getAddress,
} from './crypto'
import { describe, expect, it } from 'vitest'

describe('crypto utils', () => {
  describe('generateMnemonics', () => {
    it('should generate a valid mnemonic phrase', () => {
      const mnemonics = generateMnemonics()
      expect(mnemonics.length).toBeGreaterThanOrEqual(12)
      expect(mnemonics.length).toBeLessThanOrEqual(24)
    })
  })

  describe('deriveKeys', () => {
    it('should derive keys from a 12-word mnemonic', () => {
      const mnemonic = [
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'about',
      ]
      const { publicKey, privateKey } = deriveKeys(mnemonic)
      expect(publicKey).toBeDefined()
      expect(privateKey).toBeDefined()
    })

    it('should derive keys from a 24-word mnemonic', () => {
      const mnemonic = [
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'art',
      ]
      const { publicKey, privateKey } = deriveKeys(mnemonic)
      expect(publicKey).toBeDefined()
      expect(privateKey).toBeDefined()
    })

    it('should throw an error for invalid mnemonic length', () => {
      const mnemonic = ['abandon', 'abandon', 'abandon']
      expect(() => deriveKeys(mnemonic)).toThrow(
        'Mnemonic must be 12 or 24 words long'
      )
    })
  })

  describe('encryptMnemonics', () => {
    it('should encrypt a mnemonic phrase', () => {
      const mnemonic = [
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'about',
      ]
      const password = 'test-password'
      const { mnemonic: ciphertext, nonce } = encryptMnemonics(
        mnemonic,
        password
      )
      expect(ciphertext).toBeDefined()
      expect(nonce).toBeDefined()
    })
  })

  describe('decryptMnemonics', () => {
    it('should decrypt a mnemonic phrase', () => {
      const mnemonic = [
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'about',
      ]
      const password = 'test-password'
      const { mnemonic: ciphertext, nonce } = encryptMnemonics(
        mnemonic,
        password
      )
      const decrypted = decryptMnemonics(ciphertext, password, nonce)
      expect(decrypted).toEqual(mnemonic)
    })

    it('should throw an error for invalid key or nonce', () => {
      const ciphertext = 'invalid-ciphertext'
      const key = 'invalid-key'
      const nonce = 'invalid-nonce'
      expect(() => decryptMnemonics(ciphertext, key, nonce)).toThrow()
    })
  })

  describe('getAddress', () => {
    it('should get an address from a mnemonic', () => {
      const mnemonic = [
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'abandon',
        'about',
      ]
      const address = getAddress(mnemonic)
      expect(address).toBeDefined()
      expect(address.length).toBe(42) // 20 bytes in hex + `0x` prefix
    })
  })
})
