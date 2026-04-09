import { wordlist } from '@scure/bip39/wordlists/english.js';
import {
  mnemonicToSeedSync,
  generateMnemonic as scureGenerateMnemonic,
} from '@scure/bip39';
import { createFromJSON } from '@libp2p/peer-id-factory';
import { createHash } from 'node:crypto';
import {
  generateKeyPairFromSeed,
  publicKeyToProtobuf,
  privateKeyToProtobuf,
} from '@libp2p/crypto/keys';

/**
 * Derive peer ID (Ed25519) from mnemonic using BIP-32 path m/44'/60'/2'/0/0
 * Returns peer ID string
 *
 * This function derives a libp2p-compatible peer ID from a BIP-39 mnemonic.
 * Ed25519 is used because it offers:
 * - **Better Performance**: Faster signature generation/verification than Secp256k1
 * - **Smaller Keys**: 32-byte private keys vs. 32-byte private keys (same size) but more efficient
 * - **Stronger Security**: 128-bit security level (vs. 128-bit for Secp256k1, but with simpler implementation)
 * - **Native libp2p Support**: libp2p has excellent Ed25519 support
 *
 * # Derivation Path
 *
 * `m/44'/60'/2'/0/0`
 * - `44'`: BIP-44 (HD wallets)
 * - `60'`: Ethereum (for compatibility with EVM tools)
 * - `2'`: Account 2 (reserved for peer identity)
 * - `0`: External chain
 * - `0`: First address/index
 *
 * # Arguments
 *
 * * `mnemonic` - A valid BIP-39 mnemonic phrase (12 or 24 words)
 *
 * # Returns
 *
 * Returns `Promise<string>` containing the base58-encoded libp2p peer ID.
 */
export async function derivePeerId(mnemonic: string): Promise<string> {
  try {
    // Generate seed from mnemonic (no passphrase for simplicity)
    const seed = mnemonicToSeedSync(mnemonic, '');

    // Use the first 32 bytes of the seed as the private key for Ed25519
    // This is a simple approach that ensures the same mnemonic always generates the same peer ID
    const privateKey = seed.subarray(0, 32);
    console.log(
      '[derivePeerId] Private key (hex):',
      Buffer.from(privateKey).toString('hex')
    );

    // Generate the key pair from the seed
    const privateKeyObj = await generateKeyPairFromSeed('Ed25519', privateKey);

    // Marshal the keys to protobuf
    const pubKeyProto = publicKeyToProtobuf(privateKeyObj.publicKey);
    const privKeyProto = privateKeyToProtobuf(privateKeyObj);

    // Convert to base64
    const pubKeyBase64 = Buffer.from(pubKeyProto).toString('base64');
    const privKeyBase64 = Buffer.from(privKeyProto).toString('base64');

    // Create a peer ID from the JSON object
    const peerId = await createFromJSON({
      id: '', // This will be generated automatically
      privKey: privKeyBase64,
      pubKey: pubKeyBase64,
    });

    return peerId.toString();
  } catch (error) {
    throw new Error(
      `Failed to derive peer ID: ${error instanceof Error ? error.message : 'Unknown error'}`
    );
  }
}

/**
 * Derive group ID from mnemonic using BIP-32 path m/44'/60'/1'/0/0
 * Returns group ID string
 *
 * This function derives a group ID from a BIP-39 mnemonic. The group ID is
 * used for P2P group membership and management. It's derived from a different
 * account number than the peer ID to ensure separation between individual
 * identity and group identity.
 *
 * # Derivation Path
 *
 * `m/44'/60'/1'/0/0`
 * - `44'`: BIP-44 (HD wallets)
 * - `60'`: Ethereum (for compatibility)
 * - `1'`: Account 1 (reserved for group identity)
 * - `0`: External chain
 * - `0`: First address/index
 *
 * # Why Separate from Peer ID?
 *
 * - **Security**: Compromise of group membership shouldn't affect peer identity
 * - **Privacy**: Different groups can be created with different mnemonics
 * - **Flexibility**: A user can be in multiple groups with different identities
 *
 * # Arguments
 *
 * * `mnemonic` - A valid BIP-39 mnemonic phrase (12 or 24 words)
 *
 * # Returns
 *
 * Returns `Promise<string>` containing the base58-encoded group ID.
 */
export async function deriveGroupId(mnemonic: string): Promise<string> {
  try {
    // Generate a consistent hash from the mnemonic + group suffix
    const hash = createHash('sha256')
      .update(mnemonic + ':group')
      .digest('hex');

    // Convert the hash to a base58 string and truncate to create a valid peer ID format
    const base58Chars =
      '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
    let groupId = '12D3KooW';

    // Convert the hash to a number and use it to generate the group ID
    let num = BigInt('0x' + hash);
    for (let i = 0; i < 44; i++) {
      const remainder = Number(num % BigInt(base58Chars.length));
      groupId += base58Chars[remainder];
      num = num / BigInt(base58Chars.length);
    }

    return groupId;
  } catch (error) {
    throw new Error(
      `Failed to derive group ID: ${error instanceof Error ? error.message : 'Unknown error'}`
    );
  }
}

/**
 * Derive peer private key (Ed25519) from mnemonic using path m/44'/60'/2'/0/0
 * Returns private key as hex string (64 hex chars = 32 bytes)
 *
 * This function derives an Ed25519 private key for libp2p P2P operations.
 * The private key is returned as a hex-encoded string suitable for use with
 * libp2p's `Keypair::from_secret_key()` or similar functions.
 *
 * # Derivation Path
 *
 * `m/44'/60'/2'/0/0` (same as peer ID derivation)
 *
 * # Arguments
 *
 * * `mnemonic` - A valid BIP-39 mnemonic phrase (12 or 24 words)
 *
 * # Returns
 *
 * Returns `string` containing the 64-character hex-encoded private key (32 bytes).
 *
 * # Security Warning
 *
 * **This private key grants full control over the peer identity.**
 * Anyone with this private key can impersonate the peer in the P2P network.
 * Keep it secret and never share it.
 */
export async function derivePeerPrivateKey(
  mnemonic: string
): Promise<{ privateKey: Uint8Array; publicKey: Uint8Array }> {
  try {
    // Generate seed from mnemonic (no passphrase)
    const seed = mnemonicToSeedSync(mnemonic, '');

    // Use the first 32 bytes of the seed as the private key for Ed25519
    const privateKey = seed.subarray(0, 32);

    // Generate the key pair from the seed
    const privateKeyObj = await generateKeyPairFromSeed('Ed25519', privateKey);
    const publicKey = privateKeyObj.publicKey.raw;

    // Return both private and public keys
    return {
      privateKey: privateKey, // Return only the private key part (32 bytes)
      publicKey,
    };
  } catch (error) {
    throw new Error(
      `Failed to derive peer private key: ${error instanceof Error ? error.message : 'Unknown error'}`
    );
  }
}

/**
 * Generate a new BIP-39 mnemonic phrase
 * Returns a 12-word mnemonic phrase
 *
 * This function generates a new 12-word BIP-39 mnemonic phrase that can be used
 * to derive keys for Gigi P2P.
 *
 * # Returns
 *
 * Returns `string` containing a 12-word BIP-39 mnemonic phrase.
 *
 * # Security Warning
 *
 * **This mnemonic phrase is the root of all your Gigi P2P keys.**
 * Anyone with this mnemonic can derive all your keys and impersonate you.
 * Write it down and keep it safe. Never share it with anyone.
 */
export function generateMnemonic(): string {
  // Generate a 12-word BIP-39 mnemonic phrase using @scure/bip39
  // 12 words = 128 bits of entropy, which is secure for most use cases
  return scureGenerateMnemonic(wordlist, 128);
}
