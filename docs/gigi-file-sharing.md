# Gigi File Sharing

Gigi File Sharing is the file sharing component of the Gigi P2P ecosystem, providing efficient, reliable file transfer between peers with support for large files and resumable transfers. This guide provides detailed information about Gigi File Sharing's functionality, configuration, and usage.

## Overview

Gigi File Sharing is designed to enable secure, efficient file sharing between peers in the Gigi P2P network without relying on centralized servers or cloud storage. It supports large file transfers, resumable downloads, and progress tracking, making it ideal for sharing files of any size across the network.

### Key Features

- **File Transfer**: Efficient file transfer between peers
- **Chunking**: Split large files for easier transfer
- **Progress Tracking**: Monitor download/upload progress
- **Error Handling**: Handle network errors gracefully
- **Resumable Transfers**: Resume interrupted downloads
- **Share Codes**: Generate and use share codes for file sharing
- **Security**: Encrypted file transfers
- **Multi-peer Transfer**: Download from multiple peers simultaneously

## Installation

### Prerequisites

- **Rust**: v1.60 or later
- **Cargo**: Latest version

### Installation Steps

1. **Clone the Gigi repository**:
   ```bash
   git clone https://github.com/Lebei2046/gigi.git
   cd gigi
   ```

2. **Build Gigi File Sharing**:
   ```bash
   cd rust/gigi-file-sharing
   cargo build
   ```

3. **Add Gigi File Sharing to your project**:
   ```toml
   # In your Cargo.toml
   [dependencies]
   gigi-file-sharing = {
     path = "../gigi/rust/gigi-file-sharing",
     version = "0.1.0"
   }
   ```

## Configuration

Gigi File Sharing can be configured with various options to customize its behavior:

### Basic Configuration

```rust
use gigi_file_sharing::FileSharingManager;

// Create file sharing manager with default settings
let file_sharing = FileSharingManager::new("/path/to/downloads").await?;
```

### Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `download_dir` | Directory to save downloaded files | `./downloads` |
| `chunk_size` | Size of file chunks (bytes) | `1048576` (1MB) |
| `max_concurrent_downloads` | Maximum number of concurrent downloads | `5` |
| `max_concurrent_uploads` | Maximum number of concurrent uploads | `5` |
| `download_timeout` | Download timeout (seconds) | `300` |
| `upload_timeout` | Upload timeout (seconds) | `300` |
| `max_retries` | Maximum number of retry attempts | `3` |
| `retry_delay` | Delay between retries (seconds) | `5` |

## Usage

### Basic Usage

```rust
use gigi_file_sharing::FileSharingManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create file sharing manager
    let mut file_sharing = FileSharingManager::new("/path/to/downloads").await?;

    // Share a file
    let share_code = file_sharing.share("/path/to/file.txt").await?;
    println!("File shared with code: {}", share_code);

    // Download a file
    let download_id = file_sharing.download("peer-id", &share_code).await?;
    println!("Download started with ID: {}", download_id);

    // Track download progress
    file_sharing.on_progress(|download_id, progress| {
        println!("Download {} progress: {}%", download_id, progress * 100.0);
    });

    // Wait for download to complete
    let file_path = file_sharing.wait_for_download(&download_id).await?;
    println!("Download completed: {:?}", file_path);

    Ok(())
}
```

### Advanced Usage

#### Custom Configuration

```rust
use gigi_file_sharing::FileSharingConfig;

let config = FileSharingConfig {
    download_dir: "/path/to/downloads".to_string(),
    chunk_size: 2097152, // 2MB
    max_concurrent_downloads: 10,
    max_concurrent_uploads: 10,
    download_timeout: 600, // 10 minutes
    upload_timeout: 600, // 10 minutes
    max_retries: 5,
    retry_delay: 10,
};

let file_sharing = FileSharingManager::new_with_config(config).await?;
```

#### Multiple File Downloads

```rust
// Start multiple downloads
let download_ids = vec![];

for (peer_id, share_code) in file_shares {
    let download_id = file_sharing.download(&peer_id, &share_code).await?;
    download_ids.push(download_id);
    println!("Started download: {}", download_id);
}

// Wait for all downloads to complete
for download_id in download_ids {
    let file_path = file_sharing.wait_for_download(&download_id).await?;
    println!("Download completed: {:?}", file_path);
}
```

#### Cancel Download

```rust
// Start download
let download_id = file_sharing.download("peer-id", "share-code").await?;

// Cancel download after 5 seconds
tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
file_sharing.cancel_download(&download_id).await?;
println!("Download cancelled: {}", download_id);
```

#### List Active Downloads

```rust
// List active downloads
let downloads = file_sharing.list_active_downloads().await?;
println!("Active downloads: {}", downloads.len());

for download in downloads {
    println!("- {}: {}% ({} of {})
",
        download.id,
        download.progress * 100.0,
        download.downloaded_bytes,
        download.total_bytes
    );
}
```

## Architecture

### File Sharing Structure

The Gigi File Sharing system consists of several key components:

1. **FileSharingManager**: Main file sharing manager
2. **DownloadManager**: Manages file downloads
3. **UploadManager**: Manages file uploads
4. **ChunkManager**: Handles file chunking and reassembly
5. **ShareCodeManager**: Generates and validates share codes
6. **ProgressTracker**: Tracks transfer progress
7. **ErrorHandler**: Handles network errors and retries

### Data Flow

1. **File Sharing**: User shares a file, generating a share code
2. **File Request**: Recipient requests the file using the share code
3. **Chunking**: File is split into chunks for transfer
4. **Transfer**: Chunks are transferred between peers
5. **Reassembly**: Chunks are reassembled into the original file
6. **Completion**: Transfer is marked as complete

## Security

### Authentication

- **Share Codes**: Unique, cryptographically generated share codes
- **Peer Verification**: Peers are verified before file transfer
- **Encryption**: File transfers are encrypted using Libp2p's noise protocol
- **Access Control**: Share codes can be restricted to specific peers

### Best Practices

- **Use Strong Share Codes**: Longer share codes are more secure
- **Limit Access**: Share files only with trusted peers
- **Verify Sources**: Verify the source peer before downloading files
- **Scan Files**: Scan downloaded files for malware
- **Update Regularly**: Keep Gigi File Sharing updated to the latest version

## Troubleshooting

### Common Issues

#### File Transfer Failed

- **Symptom**: File transfer fails to complete
- **Solution**: Check network connectivity, ensure both peers are online, verify share code

#### Slow Transfer Speeds

- **Symptom**: File transfer is slow
- **Solution**: Check network speed, reduce concurrent transfers, use wired connections

#### Corrupted Files

- **Symptom**: Downloaded file is corrupted
- **Solution**: Check network stability, retry download, verify file hash

#### Share Code Invalid

- **Symptom**: Share code is not recognized
- **Solution**: Check share code for typos, ensure the sharing peer is online, regenerate share code

### Debugging

Enable debug logging to troubleshoot issues:

```rust
// Enable debug logging
env::set_var("RUST_LOG", "gigi_file_sharing=debug");

// Create file sharing manager with debug logging
let file_sharing = FileSharingManager::new("/path/to/downloads").await?;
```

## Advanced Features

### Multi-Peer Download

Download from multiple peers simultaneously for faster transfers:

```rust
// Start multi-peer download
let download_id = file_sharing.download_from_multiple(
    vec!["peer-id-1", "peer-id-2", "peer-id-3"],
    "share-code"
).await?;

// Track progress
file_sharing.on_progress(|download_id, progress| {
    println!("Multi-peer download {} progress: {}%", download_id, progress * 100.0);
});
```

### Custom Chunk Size

Adjust chunk size for different network conditions:

```rust
let config = FileSharingConfig {
    chunk_size: 524288, // 512KB for slow networks
    // Other configuration options
    ..Default::default()
};

let file_sharing = FileSharingManager::new_with_config(config).await?;
```

### Integration with Other Components

Integrate Gigi File Sharing with other Gigi components:

```rust
use gigi_file_sharing::FileSharingManager;
use gigi_p2p::P2pClient;

// Create P2P client
let mut client = P2pClient::new("My Node", config).await?;

// Create file sharing manager
let mut file_sharing = FileSharingManager::new("/path/to/downloads").await?;

// Share a file
let share_code = file_sharing.share("/path/to/file.txt").await?;

// Send share code to peer
client.send_direct_message("peer-id", format!("Here's the file: {}", share_code)).await?;

// Download file from peer
let download_id = file_sharing.download("peer-id", &share_code).await?;
```

## API Reference

### FileSharingManager Struct

#### Constructor

```rust
let file_sharing = FileSharingManager::new(download_dir).await?;
```

**Parameters**:
- `download_dir`: Directory to save downloaded files

#### Methods

##### `share(file_path)`

Share a file.

**Parameters**:
- `file_path`: Path to file to share

**Returns**:
- `Result<String, Error>` (share code)

##### `download(peer_id, share_code)`

Download a file from a peer.

**Parameters**:
- `peer_id`: Peer ID to download from
- `share_code`: Share code for the file

**Returns**:
- `Result<String, Error>` (download ID)

##### `download_from_multiple(peer_ids, share_code)`

Download a file from multiple peers.

**Parameters**:
- `peer_ids`: List of peer IDs to download from
- `share_code`: Share code for the file

**Returns**:
- `Result<String, Error>` (download ID)

##### `cancel_download(download_id)`

Cancel a download.

**Parameters**:
- `download_id`: Download ID to cancel

**Returns**:
- `Result<(), Error>`

##### `wait_for_download(download_id)`

Wait for a download to complete.

**Parameters**:
- `download_id`: Download ID to wait for

**Returns**:
- `Result<PathBuf, Error>` (path to downloaded file)

##### `list_active_downloads()`

List active downloads.

**Returns**:
- `Result<Vec<DownloadInfo>, Error>`

##### `on_progress(handler)`

Register a progress handler.

**Parameters**:
- `handler`: Progress handler function

##### `get_download_info(download_id)`

Get information about a download.

**Parameters**:
- `download_id`: Download ID to get info for

**Returns**:
- `Result<DownloadInfo, Error>`

##### `set_max_concurrent_downloads(max)`

Set maximum concurrent downloads.

**Parameters**:
- `max`: Maximum number of concurrent downloads

**Returns**:
- `Result<(), Error>`

##### `set_max_concurrent_uploads(max)`

Set maximum concurrent uploads.

**Parameters**:
- `max`: Maximum number of concurrent uploads

**Returns**:
- `Result<(), Error>`

## Examples

### Basic File Sharing

```rust
use gigi_file_sharing::FileSharingManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create file sharing manager
    let mut file_sharing = FileSharingManager::new("/path/to/downloads").await?;

    // Share a file
    let share_code = file_sharing.share("/path/to/document.pdf").await?;
    println!("File shared with code: {}", share_code);

    // Download the file (from another peer)
    let download_id = file_sharing.download("peer-id", &share_code).await?;
    println!("Download started with ID: {}", download_id);

    // Track progress
    file_sharing.on_progress(|download_id, progress| {
        println!("Download {} progress: {}%", download_id, progress * 100.0);
    });

    // Wait for download to complete
    let file_path = file_sharing.wait_for_download(&download_id).await?;
    println!("Download completed: {:?}", file_path);

    Ok(())
}
```

### Multi-Peer Download

```rust
use gigi_file_sharing::FileSharingManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create file sharing manager
    let mut file_sharing = FileSharingManager::new("/path/to/downloads").await?;

    // Download from multiple peers
    let download_id = file_sharing.download_from_multiple(
        vec!["peer-id-1", "peer-id-2", "peer-id-3"],
        "share-code"
    ).await?;

    println!("Multi-peer download started with ID: {}", download_id);

    // Track progress
    file_sharing.on_progress(|download_id, progress| {
        println!("Download {} progress: {}%", download_id, progress * 100.0);
    });

    // Wait for download to complete
    let file_path = file_sharing.wait_for_download(&download_id).await?;
    println!("Multi-peer download completed: {:?}", file_path);

    Ok(())
}
```

### Download Management

```rust
use gigi_file_sharing::FileSharingManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create file sharing manager
    let mut file_sharing = FileSharingManager::new("/path/to/downloads").await?;

    // Start multiple downloads
    let download_ids = vec![];

    for i in 1..=3 {
        let download_id = file_sharing.download(
            format!("peer-id-{}", i),
            "share-code-{}"
        ).await?;
        download_ids.push(download_id);
        println!("Started download: {}", download_id);
    }

    // List active downloads
    let downloads = file_sharing.list_active_downloads().await?;
    println!("Active downloads: {}", downloads.len());

    // Cancel one download
    if !download_ids.is_empty() {
        let download_id_to_cancel = download_ids[0].clone();
        file_sharing.cancel_download(&download_id_to_cancel).await?;
        println!("Cancelled download: {}", download_id_to_cancel);
    }

    // Wait for remaining downloads to complete
    for download_id in download_ids.iter().skip(1) {
        let file_path = file_sharing.wait_for_download(download_id).await?;
        println!("Download completed: {:?}", file_path);
    }

    Ok(())
}
```

## Conclusion

Gigi File Sharing provides efficient, reliable file transfer between peers in the Gigi P2P network, enabling users to share files of any size without relying on centralized servers. By following this guide, you can integrate Gigi File Sharing into your applications, providing secure, decentralized file sharing capabilities for your users.

For more information, see the [API Reference](api/file-sharing-api.md) and [Troubleshooting Guide](guides/troubleshooting-guide.md).
