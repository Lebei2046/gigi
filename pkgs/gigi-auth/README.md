# gigi-auth

Authentication and account management for Gigi.

## Overview

`gigi-auth` provides a comprehensive authentication system for managing user accounts, including:

- **Account Creation**: Create accounts with BIP-39 mnemonics
- **Password Authentication**: Password-based login and verification
- **Encrypted Storage**: Secure mnemonic encryption
- **Key Derivation**: Derive peer IDs, group IDs, and EVM addresses from mnemonics

## Features

### Account Management

```rust
use gigi_auth::{AuthManager, AccountInfo, LoginResult};

// Create a new account
let account_info = auth_manager
    .create_account(
        "abandon abandon liar ...", // BIP-39 mnemonic
        "my_password",             // Password
        Some("Alice".to_string()) // Display name
    )
    .await?;

// Login with password - returns both account info and private key
let login_result = auth_manager
    .login("my_password")
    .await?;

// Access account information
let account_info = login_result.account_info;
let peer_id = account_info.peer_id;
let private_key = login_result.private_key; // For P2pClient initialization

// Check if account exists
let has_account = auth_manager.has_account().await?;

// Get account info (public data only)
let account_info = auth_manager.get_account_info().await?;

// Delete account
auth_manager.delete_account().await?;
```

### Password Management

```rust
// Change password
auth_manager
    .change_password("old_password", "new_password")
    .await?;

// Verify password
let is_valid = auth_manager.verify_password("password").await?;
```

### Key Derivation

```rust
use gigi_auth::{derive_peer_id, derive_group_id, derive_evm_address, derive_private_key};

// Derive peer ID from mnemonic (m/44'/60'/0'/0/0)
let peer_id = derive_peer_id(mnemonic)?;

// Derive group ID from mnemonic (m/44'/60'/1'/0/0)
let group_id = derive_group_id(mnemonic)?;

// Derive EVM address from mnemonic
let address = derive_evm_address(mnemonic)?;

// Derive private key from mnemonic (m/44'/60'/0'/0/0) - for P2pClient
let private_key = derive_private_key(mnemonic)?;
```

### Encryption

```rust
use gigi_auth::encryption::{encrypt_mnemonic, decrypt_mnemonic, EncryptedAccountData};

// Encrypt mnemonic with password
let encrypted_data = encrypt_mnemonic(
    mnemonic,
    password,
    &peer_id,
    &group_id,
    &address,
)?;

// Decrypt mnemonic
let decrypted = decrypt_mnemonic(&encrypted_data, password)?;
```

## Storage

The `AuthManager` uses a SQLite database through `sea-orm` with a settings table:

| Key | Value |
|-----|-------|
| `encrypted_mnemonic` | JSON with encrypted data |
| `peer_id` | Derived peer ID |
| `group_id` | Derived group ID |

## Authentication Flow

The password verification follows a decrypt-and-verify approach:

1. Retrieve encrypted mnemonic from storage
2. Attempt to decrypt with provided password
3. Derive peer_id from decrypted mnemonic
4. Compare derived peer_id with stored peer_id
5. If they match → password is correct

This approach is more secure than password hashing as the encrypted mnemonic itself serves as proof.

### Login Return Value

The `login` method returns a `LoginResult` struct containing:
- `account_info`: Public account information (address, peer_id, group_id, name)
- `private_key`: Hex-encoded private key derived from the mnemonic (path: m/44'/60'/0'/0/0)

The private key is included in the login response to enable immediate P2pClient initialization without requiring additional decryption steps.

## Implementation Status

### ✅ Implemented
- [x] Account creation with mnemonics
- [x] Password-based login
- [x] Password verification
- [x] Password change
- [x] Account deletion
- [x] Encrypted mnemonic storage
- [x] Key derivation (placeholder)
- [x] Settings management

### ⚠️ Placeholder Implementation

The following features use placeholder implementations and need to be upgraded for production:

1. **Encryption**: Uses simple XOR encryption
   - **Required**: Upgrade to AES-256-GCM or XChaCha20-Poly1305
   - **Crates**: `aes-gcm` or `chacha20poly1305`

2. **Key Derivation**: Uses SHA-256 hashing
   - **Required**: Implement proper BIP-32/BIP-39 derivation
   - **Crates**: `bip32`, `bip39`

3. **Nonce Generation**: Time-based (not cryptographically secure)
   - **Required**: Use proper CSPRNG
   - **Crates**: `rand`

## Dependencies

- `anyhow` - Error handling
- `chrono` - Timestamp handling
- `hex` - Hex encoding/decoding
- `sea-orm` - Database ORM
- `serde` - Serialization
- `sha2` - SHA-256 hashing (placeholder)
- `thiserror` - Error types
- `tracing` - Logging

## Security Considerations

⚠️ **Production Warning**: This crate uses placeholder implementations for demonstration. DO NOT use in production without:

1. **Upgrading encryption** to AES-256-GCM or XChaCha20-Poly1305
2. **Implementing proper BIP-32/BIP-39** key derivation
3. **Using cryptographically secure random number generators**
4. **Adding rate limiting** for login attempts
5. **Implementing memory zeroization** for sensitive data

## License

MIT OR Apache-2.0
