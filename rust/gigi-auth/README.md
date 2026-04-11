# gigi-auth

## Overview

Authentication and account management for Gigi. Provides a comprehensive authentication system for managing user accounts, including account creation with BIP-39 mnemonics, password authentication, encrypted storage, and key derivation.

### Features

- **Account Creation**: Create accounts with BIP-39 mnemonics
- **Password Authentication**: Password-based login and verification
- **Encrypted Storage**: Secure mnemonic encryption
- **Key Derivation**: Derive peer IDs, group IDs, and EVM addresses from mnemonics
- **Account Management**: Create, login, verify, change password, and delete accounts

### Authentication Flow

The password verification follows a decrypt-and-verify approach:

1. Retrieve encrypted mnemonic from storage
2. Attempt to decrypt with provided password
3. Derive peer_id from decrypted mnemonic
4. Compare derived peer_id with stored peer_id
5. If they match → password is correct

## Installation/Test

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
gigi-auth = {
  path = "../gigi-auth",
  version = "0.1.0"
}
```

### Testing

Run the tests to verify functionality:

```bash
cargo test
```

## Security Considerations

⚠️ **Production Warning**: This crate uses placeholder implementations for demonstration. DO NOT use in production without:

1. **Upgrading encryption** to AES-256-GCM or XChaCha20-Poly1305
2. **Implementing proper BIP-32/BIP-39** key derivation
3. **Using cryptographically secure random number generators**
4. **Adding rate limiting** for login attempts
5. **Implementing memory zeroization** for sensitive data

## License

MIT OR Apache-2.0
