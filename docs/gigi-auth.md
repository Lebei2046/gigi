# Gigi Auth

Gigi Auth is the authentication and key management component of the Gigi P2P ecosystem. It provides secure account creation, login, and key management services, ensuring that only authorized users can access the network and that communications are properly encrypted. This guide provides detailed information about Gigi Auth's functionality, configuration, and usage.

## Overview

Gigi Auth is designed to secure the Gigi P2P network by providing robust authentication and key management. It handles account creation, password verification, key derivation, and secure storage of sensitive information. Gigi Auth uses industry-standard cryptography to ensure the security of user accounts and communications.

### Key Features

- **Account Management**: Create and manage user accounts
- **Key Derivation**: Secure key derivation from passwords
- **Encryption**: Encrypt sensitive data
- **Settings Management**: Store and retrieve user settings
- **Mnemonic Support**: Generate and use mnemonic phrases for key recovery
- **Password Verification**: Verify user passwords securely
- **Secure Storage**: Store keys and sensitive data securely

## Installation

### Prerequisites

- **Rust**: v1.60 or later
- **Cargo**: Latest version

### Installation Steps

1. **Clone the Gigi repository**:
   ```bash
   git clone https://github.com/gigi-project/gigi.git
   cd gigi
   ```

2. **Build Gigi Auth**:
   ```bash
   cd rust/gigi-auth
   cargo build
   ```

3. **Add Gigi Auth to your project**:
   ```toml
   # In your Cargo.toml
   [dependencies]
   gigi-auth = {
     path = "../gigi/rust/gigi-auth",
     version = "0.1.0"
   }
   ```

## Configuration

Gigi Auth can be configured with various options to customize its behavior:

### Basic Configuration

```rust
use gigi_auth::AuthManager;

// Create auth manager with default settings
let auth_manager = AuthManager::new("/path/to/keystore").await?;
```

### Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `keystore_path` | Path to store keys and account data | `~/.gigi/auth` |
| `kdf_iterations` | Number of iterations for key derivation | `390000` |
| `salt_length` | Length of salt for password hashing | `32` |
| `key_length` | Length of derived keys | `32` |
| `iv_length` | Length of initialization vector for encryption | `16` |

## Usage

### Basic Usage

```rust
use gigi_auth::AuthManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create auth manager
    let mut auth_manager = AuthManager::new("/path/to/keystore").await?;
    
    // Generate mnemonic
    let mnemonic = auth_manager.generate_mnemonic().await?;
    println!("Mnemonic: {}", mnemonic);
    
    // Create account
    auth_manager.signup("password", mnemonic).await?;
    println!("Account created successfully");
    
    // Login
    let account_info = auth_manager.login_with_password("password").await?;
    println!("Logged in as: {}", account_info.peer_id);
    
    // Get account info
    let account_info = auth_manager.get_account_info().await?;
    println!("Account info: {:?}", account_info);
    
    // Update password
    auth_manager.update_password("old_password", "new_password").await?;
    println!("Password updated successfully");
    
    // Delete account
    auth_manager.delete_account("password").await?;
    println!("Account deleted successfully");
    
    Ok(())
}
```

### Advanced Usage

#### Mnemonic Generation and Recovery

```rust
// Generate mnemonic
let mnemonic = auth_manager.generate_mnemonic().await?;
println!("Mnemonic: {}", mnemonic);

// Recover account from mnemonic
auth_manager.recover_from_mnemonic("password", mnemonic).await?;
println!("Account recovered successfully");
```

#### Settings Management

```rust
// Save settings
let settings = gigi_auth::Settings {
    nickname: "Alice".to_string(),
    theme: "dark".to_string(),
    auto_login: true,
};
auth_manager.save_settings(settings).await?;

// Get settings
let settings = auth_manager.get_settings().await?;
println!("Settings: {:?}", settings);
```

#### Key Management

```rust
// Get public key
let public_key = auth_manager.get_public_key().await?;
println!("Public key: {:?}", public_key);

// Sign data
let data = b"Hello, world!";
let signature = auth_manager.sign(data).await?;
println!("Signature: {:?}", signature);

// Verify signature
let is_valid = auth_manager.verify(data, &signature).await?;
println!("Signature valid: {}", is_valid);
```

## Architecture

### Auth Manager Structure

The Gigi Auth system consists of several key components:

1. **AuthManager**: Main authentication manager
2. **KeyDerivation**: Handles password-based key derivation
3. **Encryption**: Provides encryption and decryption services
4. **SettingsManager**: Manages user settings
5. **Storage**: Securely stores keys and sensitive data

### Data Flow

1. **Account Creation**: User creates an account with password and mnemonic
2. **Key Derivation**: Password is used to derive encryption keys
3. **Data Encryption**: Private keys and sensitive data are encrypted
4. **Storage**: Encrypted data is stored securely
5. **Authentication**: User authenticates with password
6. **Decryption**: Keys are decrypted for use

## Security

### Authentication

- **Password Security**: Passwords are hashed using Argon2id
- **Key Derivation**: Uses Argon2id for secure key derivation
- **Encryption**: Uses AES-256-GCM for encryption
- **Mnemonic Recovery**: BIP39-compatible mnemonic phrases for key recovery
- **Secure Storage**: Encrypted storage of keys and sensitive data

### Best Practices

- **Use Strong Passwords**: Use long, complex passwords
- **Backup Mnemonic**: Store mnemonic phrases securely
- **Update Regularly**: Keep Gigi Auth updated to the latest version
- **Limit Access**: Restrict access to keystore files
- **Monitor Activity**: Regularly check for unauthorized access

## Troubleshooting

### Common Issues

#### Account Creation Failed

- **Symptom**: Cannot create account
- **Solution**: Check keystore permissions, ensure password is strong enough

#### Login Failed

- **Symptom**: Cannot login to account
- **Solution**: Check password, recover from mnemonic if needed

#### Mnemonic Recovery Failed

- **Symptom**: Cannot recover account from mnemonic
- **Solution**: Check mnemonic phrase for errors, ensure correct order

#### Key Derivation Slow

- **Symptom**: Key derivation takes too long
- **Solution**: Adjust `kdf_iterations` in configuration (lower for faster performance, higher for better security)

### Debugging

Enable debug logging to troubleshoot issues:

```rust
// Enable debug logging
env::set_var("RUST_LOG", "gigi_auth=debug");

// Create auth manager with debug logging
let auth_manager = AuthManager::new("/path/to/keystore").await?;
```

## Advanced Features

### Custom Key Derivation

Configure custom key derivation parameters:

```rust
use gigi_auth::AuthManagerConfig;

let config = AuthManagerConfig {
    keystore_path: "/path/to/keystore".to_string(),
    kdf_iterations: 600000, // Higher for better security
    salt_length: 32,
    key_length: 32,
    iv_length: 16,
};

let auth_manager = AuthManager::new_with_config(config).await?;
```

### Multi-Account Support

Manage multiple accounts:

```rust
// Create auth manager for first account
let mut auth_manager1 = AuthManager::new("/path/to/keystore/account1").await?;

// Create auth manager for second account
let mut auth_manager2 = AuthManager::new("/path/to/keystore/account2").await?;

// Use different accounts
auth_manager1.signup("password1", mnemonic1).await?;
auth_manager2.signup("password2", mnemonic2).await?;
```

### Integration with Other Components

Integrate Gigi Auth with other Gigi components:

```rust
use gigi_auth::AuthManager;
use gigi_p2p::P2pClient;

// Create auth manager
let mut auth_manager = AuthManager::new("/path/to/keystore").await?;

// Login
let account_info = auth_manager.login_with_password("password").await?;

// Create P2P client with account info
let mut client = P2pClient::new_with_keypair(
    account_info.nickname,
    account_info.keypair,
    config
).await?;

// Start client
client.start().await?;
```

## API Reference

### AuthManager Struct

#### Constructor

```rust
let auth_manager = AuthManager::new(keystore_path).await?;
```

**Parameters**:
- `keystore_path`: Path to store keys and account data

#### Methods

##### `generate_mnemonic()`

Generate a mnemonic phrase for key recovery.

**Returns**:
- `Result<String, Error>` (mnemonic phrase)

##### `signup(password, mnemonic)`

Create a new account.

**Parameters**:
- `password`: Account password
- `mnemonic`: Mnemonic phrase for recovery

**Returns**:
- `Result<(), Error>`

##### `login_with_password(password)`

Login to an account with password.

**Parameters**:
- `password`: Account password

**Returns**:
- `Result<AccountInfo, Error>`

##### `login_with_mnemonic(mnemonic)`

Login to an account with mnemonic phrase.

**Parameters**:
- `mnemonic`: Mnemonic phrase

**Returns**:
- `Result<AccountInfo, Error>`

##### `recover_from_mnemonic(password, mnemonic)`

Recover an account from mnemonic phrase.

**Parameters**:
- `password`: New account password
- `mnemonic`: Mnemonic phrase

**Returns**:
- `Result<(), Error>`

##### `get_account_info()`

Get account information.

**Returns**:
- `Result<AccountInfo, Error>`

##### `update_password(old_password, new_password)`

Update account password.

**Parameters**:
- `old_password`: Current password
- `new_password`: New password

**Returns**:
- `Result<(), Error>`

##### `delete_account(password)`

Delete an account.

**Parameters**:
- `password`: Account password

**Returns**:
- `Result<(), Error>`

##### `get_public_key()`

Get the public key of the account.

**Returns**:
- `Result<Vec<u8>, Error>`

##### `sign(data)`

Sign data with the account's private key.

**Parameters**:
- `data`: Data to sign

**Returns**:
- `Result<Vec<u8>, Error>` (signature)

##### `verify(data, signature)`

Verify a signature.

**Parameters**:
- `data`: Signed data
- `signature`: Signature to verify

**Returns**:
- `Result<bool, Error>` (whether the signature is valid)

##### `save_settings(settings)`

Save user settings.

**Parameters**:
- `settings`: User settings

**Returns**:
- `Result<(), Error>`

##### `get_settings()`

Get user settings.

**Returns**:
- `Result<Settings, Error>`

## Examples

### Basic Account Management

```rust
use gigi_auth::AuthManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create auth manager
    let mut auth_manager = AuthManager::new("/path/to/keystore").await?;
    
    // Generate mnemonic
    let mnemonic = auth_manager.generate_mnemonic().await?;
    println!("Mnemonic: {}", mnemonic);
    
    // Create account
    auth_manager.signup("password123", mnemonic).await?;
    println!("Account created successfully");
    
    // Login
    let account_info = auth_manager.login_with_password("password123").await?;
    println!("Logged in as: {}", account_info.peer_id);
    
    // Get account info
    let account_info = auth_manager.get_account_info().await?;
    println!("Account info: {:?}", account_info);
    
    // Update password
    auth_manager.update_password("password123", "newpassword456").await?;
    println!("Password updated successfully");
    
    // Login with new password
    let account_info = auth_manager.login_with_password("newpassword456").await?;
    println!("Logged in with new password as: {}", account_info.peer_id);
    
    // Delete account
    auth_manager.delete_account("newpassword456").await?;
    println!("Account deleted successfully");
    
    Ok(())
}
```

### Mnemonic Recovery

```rust
use gigi_auth::AuthManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create auth manager
    let mut auth_manager = AuthManager::new("/path/to/keystore").await?;
    
    // Generate mnemonic
    let mnemonic = auth_manager.generate_mnemonic().await?;
    println!("Mnemonic: {}", mnemonic);
    
    // Create account
    auth_manager.signup("password123", mnemonic.clone()).await?;
    println!("Account created successfully");
    
    // Delete and recreate auth manager to simulate restart
    drop(auth_manager);
    let mut auth_manager = AuthManager::new("/path/to/keystore").await?;
    
    // Try to login with wrong password
    match auth_manager.login_with_password("wrongpassword").await {
        Ok(_) => println!("Login with wrong password succeeded (should have failed)"),
        Err(e) => println!("Login with wrong password failed as expected: {:?}"),
    }
    
    // Recover from mnemonic
    auth_manager.recover_from_mnemonic("newpassword456", mnemonic).await?;
    println!("Account recovered successfully");
    
    // Login with new password
    let account_info = auth_manager.login_with_password("newpassword456").await?;
    println!("Logged in after recovery as: {}", account_info.peer_id);
    
    Ok(())
}
```

### Settings Management

```rust
use gigi_auth::AuthManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create auth manager
    let mut auth_manager = AuthManager::new("/path/to/keystore").await?;
    
    // Generate mnemonic
    let mnemonic = auth_manager.generate_mnemonic().await?;
    
    // Create account
    auth_manager.signup("password123", mnemonic).await?;
    
    // Login
    auth_manager.login_with_password("password123").await?;
    
    // Save settings
    let settings = gigi_auth::Settings {
        nickname: "Alice".to_string(),
        theme: "dark".to_string(),
        auto_login: true,
    };
    auth_manager.save_settings(settings).await?;
    println!("Settings saved successfully");
    
    // Get settings
    let settings = auth_manager.get_settings().await?;
    println!("Settings: {:?}", settings);
    
    // Update settings
    let updated_settings = gigi_auth::Settings {
        nickname: "Alice Smith".to_string(),
        theme: "light".to_string(),
        auto_login: false,
    };
    auth_manager.save_settings(updated_settings).await?;
    println!("Settings updated successfully");
    
    // Get updated settings
    let updated_settings = auth_manager.get_settings().await?;
    println!("Updated settings: {:?}", updated_settings);
    
    Ok(())
}
```

## Conclusion

Gigi Auth provides robust authentication and key management for the Gigi P2P ecosystem, ensuring that user accounts are secure and communications are properly encrypted. By following this guide, you can integrate Gigi Auth into your applications, providing secure authentication and key management for your users.

For more information, see the [API Reference](api/auth-api.md) and [Troubleshooting Guide](guides/troubleshooting-guide.md).