import { wordlist } from '@scure/bip39/wordlists/english'
import { generateMnemonic } from '@scure/bip39'

/**
 * Functions for generating mnemonics.
 * Note: Key derivation and encryption/decryption are handled by the backend.
 */

export function generateMnemonics(): string[] {
  const mnemonic = generateMnemonic(wordlist)
  return mnemonic.split(' ')
}
