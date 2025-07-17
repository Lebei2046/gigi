import { describe, it, expect } from 'vitest';
import { CryptoService } from './crypto';

describe('CryptoService', () => {
  describe('generateMnemonic', () => {
    it('should generate a valid mnemonic phrase', () => {
      const mnemonic = CryptoService.generateMnemonic();
      expect(mnemonic).toBeInstanceOf(Array);
      expect(mnemonic.length).toBeGreaterThan(0);
      expect(mnemonic.every(word => typeof word === 'string')).toBe(true);
    });
  });

  describe('deriveKeys', () => {
    it('should derive keys from a valid mnemonic', () => {
      const mnemonic = CryptoService.generateMnemonic();
      const keys = CryptoService.deriveKeys(mnemonic);
      expect(keys.publicKey).toBeInstanceOf(Uint8Array);
      expect(keys.privateKey).toBeInstanceOf(Uint8Array);
    });

    it('should throw an error for invalid mnemonic', () => {
      expect(() => CryptoService.deriveKeys(['invalid', 'mnemonic'])).toThrow('Mnemonic must be 12 or 24 words long');
    });
  });

  describe('encryptMnemonic and decryptMnemonic', () => {
    it('should encrypt and decrypt a mnemonic successfully', () => {
      const mnemonic = CryptoService.generateMnemonic();
      const password = 'test-password';
      const encrypted = CryptoService.encryptMnemonic(mnemonic, password);
      const decrypted = CryptoService.decryptMnemonic(encrypted.mnemonic, password, encrypted.nonce);
      expect(decrypted).toEqual(mnemonic);
    });

    it('should fail to decrypt with incorrect password', () => {
      const mnemonic = CryptoService.generateMnemonic();
      const password = 'test-password';
      const encrypted = CryptoService.encryptMnemonic(mnemonic, password);
      expect(() => CryptoService.decryptMnemonic(encrypted.mnemonic, 'wrong-password', encrypted.nonce)).toThrow();
    });
  });

  describe('generateAddress', () => {
    it('should generate a valid address from a mnemonic', () => {
      const mnemonic = CryptoService.generateMnemonic();
      const address = CryptoService.generateAddress(mnemonic);
      expect(typeof address).toBe('string');
      expect(address.length).toBe(40); // Ethereum address length
    });
  });
});