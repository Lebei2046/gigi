//! # Gigi File Sharing
//!
//! This crate provides file sharing management capabilities for P2P file transfers in the gigi application.
//!
//! ## Architecture
//!
//! The file sharing system is designed around chunked file transfers to handle large files efficiently
//! and support resumable downloads. It provides:
//!
//! - **Chunked Transfer Support**: Files are split into 256KB chunks for reliable P2P transfer
//! - **Dual Path Support**: Works with both filesystem paths (desktop) and URIs (mobile platforms)
//! - **Persistent Storage**: Integrates with gigi-store for metadata persistence
//! - **File Hashing**: Uses SHA256 for integrity verification
//! - **Share Codes**: BLAKE3-based unique identifiers for file sharing
//!
//! ## File Sharing Flow
//!
//! ```text
//! 1. Share Request
//!    ↓
//! 2. Calculate file hash (SHA256)
//!    ↓
//! 3. Generate share code (BLAKE3 hash of filename + timestamp)
//!    ↓
//! 4. Calculate chunk count based on file size
//!    ↓
//! 5. Store metadata (in-memory and persistent storage)
//!    ↓
//! 6. Return share code to sender
//!    ↓
//! 7. Recipient uses share code to request chunks
//! ```
//!
//! ## Chunked Transfer
//!
//! Files are split into chunks to enable:
//! - **Resumable Downloads**: Only failed chunks need to be retransmitted
//! - **Parallel Transfer**: Multiple chunks can be transferred concurrently
//! - **Memory Efficiency**: Small chunk size (256KB) prevents memory pressure
//! - **Progress Tracking**: Granular progress reporting
//!
//! Chunk calculation formula:
//! ```text
//! chunk_count = ceil(file_size / CHUNK_SIZE)
//!
//! Example for 1MB file:
//! chunk_count = ceil(1,048,576 / 262,144) = 4 chunks
//! ```
//!
//! ## Platform Support
//!
//! ### Desktop (Linux, macOS, Windows)
//! - Uses filesystem paths (PathBuf)
//! - Direct file access for reading chunks
//!
//! ### Mobile (Android, iOS)
//! - Uses content URIs (content://, file://)
//! - Requires platform-specific FileChunkReader callback
//! - Example Android URI: `content://com.android.providers.media.documents/document/image:1234`
//!
//! ## Share Code Generation
//!
//! Share codes are generated using BLAKE3 hash combining:
//! 1. Filename (as bytes)
//! 2. Current timestamp (nanoseconds since UNIX epoch)
//!
//! The first 8 hex characters are used as the share code, providing:
//! - Sufficient uniqueness (4.29 billion possible values)
//! - Short, shareable format
//! - Collision resistance through timestamp
//!
//! ## Persistence
//!
//! When configured with a `FileSharingStore`, the manager persists:
//! - Share code mappings
//! - File metadata (name, size, hash, chunk count)
//! - Thumbnail paths (for preview images)
//! - Revocation status
//!
//! Persistence is asynchronous and doesn't block the main thread.
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use gigi_file_sharing::{FileSharingManager, CHUNK_SIZE};
//! use gigi_store::FileSharingStore;
//! use std::sync::Arc;
//!
//! // Create manager
//! let mut manager = FileSharingManager::new();
//!
//! // Optional: Attach persistent storage
//! let store = Arc::new(FileSharingStore::new(db_connection).await?);
//! manager = manager.with_store(store);
//!
//! // Share a filesystem file
//! let share_code = manager.share_file(&PathBuf::from("document.pdf")).await?;
//! println!("Share code: {}", share_code);
//!
//! // Share a content URI (mobile)
//! let share_code = manager.share_content_uri(
//!     "content://com.android.../document",
//!     "photo.jpg",
//!     1024 * 1024  // 1MB
//! ).await?;
//!
//! // List all shared files
//! for file in manager.list_shared_files() {
//!     println!("{}: {} ({} bytes)",
//!         file.share_code,
//!         file.info.name,
//!         file.info.size
//!     );
//! }
//!
//! // Unshare a file
//! manager.unshare_file(&share_code)?;
//! ```
//!
//! ## Thread Safety
//!
//! - `FileSharingManager` is not thread-safe internally
//! - Use external synchronization if sharing files from multiple threads
//! - `FileSharingStore` operations are wrapped in Arc for thread-safe access

pub mod error;
pub mod types;

// Re-export types for convenience
pub use error::FileSharingError;
pub use types::{FileInfo, FilePath, SharedFile};

use anyhow::Result;
use blake3::Hasher;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tracing::info;
use url::Url;

use gigi_store::FileSharingStore;

/// Size of each file chunk in bytes (256KB)
///
/// This chunk size balances:
/// - **Network Efficiency**: Small enough for reliable transmission
/// - **Overhead**: Not too many chunks for large files
/// - **Memory**: Fits comfortably in memory buffers
///
/// Examples:
/// - 1KB file: 1 chunk
/// - 1MB file: 4 chunks
/// - 100MB file: 400 chunks
/// - 1GB file: 4096 chunks
pub const CHUNK_SIZE: usize = 256 * 1024;

/// Callback type for reading file chunks from URI-based files
///
/// This is used on mobile platforms where files are accessed through content URIs
/// rather than direct filesystem paths. The callback receives:
///
/// - `&FilePath`: The file path (will be a URI variant)
/// - `u64`: The starting byte offset for the chunk
/// - `usize`: The number of bytes to read (chunk size)
///
/// # Example Implementation
///
/// ```rust,no_run
/// use std::collections::HashMap;
/// use gigi_file_sharing::{FileSharingManager, FilePath};
///
/// let mut file_cache: HashMap<String, Vec<u8>> = HashMap::new();
///
/// let reader = |path: &FilePath, offset: u64, length: usize| -> Result<Vec<u8>> {
///     if let FilePath::Url(uri) = path {
///         // Read chunk from content URI using platform API
///         let data = read_from_content_uri(uri, offset, length)?;
///         Ok(data)
///     } else {
///         Err(anyhow::anyhow!("Expected URI path"))
///     }
/// };
///
/// let mut manager = FileSharingManager::new();
/// manager.set_chunk_reader(reader.into());
/// ```
pub type FileChunkReader = Arc<dyn Fn(&FilePath, u64, usize) -> Result<Vec<u8>> + Send + Sync>;

/// File sharing manager
///
/// Manages file sharing operations including:
/// - Sharing files (filesystem paths and content URIs)
/// - Tracking shared files in memory
/// - Persisting metadata to gigi-store
/// - Unsharing files
///
/// # Fields
///
/// - `shared_files`: In-memory mapping of share_code → SharedFile
/// - `chunk_reader`: Optional callback for reading chunks from URIs
/// - `file_sharing_store`: Optional persistent storage backend
///
/// # Example
///
/// ```rust,no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use gigi_file_sharing::FileSharingManager;
/// use std::path::PathBuf;
///
/// let mut manager = FileSharingManager::new();
/// let share_code = manager.share_file(&PathBuf::from("file.txt")).await?;
/// println!("Share code: {}", share_code);
/// # Ok(())
/// # }
/// ```
pub struct FileSharingManager {
    /// In-memory registry of shared files, keyed by share code
    pub shared_files: HashMap<String, SharedFile>,
    /// Callback for reading chunks from content URIs (mobile platforms)
    chunk_reader: Option<FileChunkReader>,
    /// Persistent storage backend (optional, from gigi-store)
    file_sharing_store: Option<Arc<FileSharingStore>>,
}

impl FileSharingManager {
    /// Create a new file sharing manager
    ///
    /// # Returns
    ///
    /// A new `FileSharingManager` instance with:
    /// - Empty shared files registry
    /// - No chunk reader configured
    /// - No persistent storage
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use gigi_file_sharing::FileSharingManager;
    ///
    /// let manager = FileSharingManager::new();
    /// ```
    pub fn new() -> Self {
        Self {
            shared_files: HashMap::new(),
            chunk_reader: None,
            file_sharing_store: None,
        }
    }

    /// Attach a persistent storage backend
    ///
    /// # Arguments
    ///
    /// * `store` - An Arc-wrapped FileSharingStore for persisting metadata
    ///
    /// # Returns
    ///
    /// Self (builder pattern for chaining)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use gigi_file_sharing::FileSharingManager;
    /// use gigi_store::FileSharingStore;
    /// use std::sync::Arc;
    ///
    /// # let store = Arc::new(FileSharingStore::new(db_connection).await?);
    /// let manager = FileSharingManager::new().with_store(store);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_store(mut self, store: Arc<FileSharingStore>) -> Self {
        self.file_sharing_store = Some(store);
        self
    }

    /// Set the chunk reader callback for URI-based files
    ///
    /// # Arguments
    ///
    /// * `reader` - A callback that can read file chunks from content URIs
    ///
    /// # Platform Notes
    ///
    /// This is primarily used on:
    /// - **Android**: For content:// URIs from MediaStore
    /// - **iOS**: For file:// URIs from file pickers
    ///
    /// Desktop platforms typically use filesystem paths and don't need this callback.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use gigi_file_sharing::FileSharingManager;
    /// use std::sync::Arc;
    ///
    /// let reader = Arc::new(|path, offset, length| {
    ///     // Read chunk from content URI
    ///     Ok(vec![0u8; length])
    /// });
    ///
    /// let mut manager = FileSharingManager::new();
    /// manager.set_chunk_reader(reader);
    /// ```
    pub fn set_chunk_reader(&mut self, reader: FileChunkReader) {
        self.chunk_reader = Some(reader);
    }

    /// Generate a unique share code for a file
    ///
    /// # Arguments
    ///
    /// * `filename` - The name of the file being shared
    ///
    /// # Returns
    ///
    /// An 8-character hexadecimal share code
    ///
    /// # Algorithm
    ///
    /// Uses BLAKE3 hash combining:
    /// 1. Filename bytes (provides determinism for same file names)
    /// 2. Current timestamp in nanoseconds (ensures uniqueness)
    ///
    /// The first 8 hex characters (32 bits) are used as the share code.
    ///
    /// # Collision Probability
    ///
    /// With 8 hex characters (32 bits):
    /// - 4,294,967,296 possible values
    /// - ~0.000023% collision chance with 1M simultaneous shares
    /// - Timestamp adds additional uniqueness
    ///
    /// # Example
    ///
    /// ```text
    /// Filename: "document.pdf"
    /// Timestamp: 16409952001234567890 ns
    /// BLAKE3 Hash: "a1b2c3d4e5f67890..."
    /// Share Code: "a1b2c3d4"
    /// ```
    pub fn generate_share_code(&self, filename: &str) -> String {
        let mut hasher = Hasher::new();
        hasher.update(filename.as_bytes());
        hasher.update(
            &std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_le_bytes(),
        );

        format!("{}", hasher.finalize().to_hex())[..8].to_string()
    }

    /// Share a file from the filesystem
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file to share
    ///
    /// # Returns
    ///
    /// The share code for the file
    ///
    /// # Process
    ///
    /// 1. Canonicalize the path (falls back on error for URIs)
    /// 2. Verify file exists and is accessible
    /// 3. Extract filename from path
    /// 4. Calculate SHA256 hash of the entire file
    /// 5. Check if file is already shared:
    ///    - If unchanged hash: Return existing share code
    ///    - If changed: Update metadata with new hash
    ///    - If new: Create new share entry
    /// 6. Calculate chunk count based on file size
    /// 7. Save metadata to persistent storage (if configured)
    /// 8. Return share code
    ///
    /// # Errors
    ///
    /// - `FileNotFound`: If the file doesn't exist
    /// - `IoError`: If file cannot be read
    /// - `SerializationError`: If metadata cannot be serialized
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use gigi_file_sharing::FileSharingManager;
    /// use std::path::PathBuf;
    ///
    /// let mut manager = FileSharingManager::new();
    /// let code = manager.share_file(&PathBuf::from("document.pdf")).await?;
    /// println!("Share code: {}", code);
    /// ```
    pub async fn share_file(&mut self, file_path: &Path) -> Result<String> {
        // Try canonicalize, but fall back to original path if it fails (Android content URIs)
        let path = file_path
            .canonicalize()
            .unwrap_or_else(|_| file_path.to_path_buf());

        // Verify file exists and is accessible
        if !path.exists() {
            return Err(FileSharingError::FileNotFound(path.clone()).into());
        }

        let metadata = fs::metadata(&path).await?;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| FileSharingError::FileNotFound(path.clone()))?
            .to_string();

        // Calculate file hash
        let hash = self.calculate_file_hash(&path)?;

        // Check if file is already shared
        if let Some((existing_share_code, existing_shared_file)) =
            self.shared_files
                .iter()
                .find(|(_, shared_file)| match &shared_file.path {
                    FilePath::Path(existing_path) => existing_path == &path,
                    _ => false,
                })
        {
            // File already shared, check if it has changed
            if existing_shared_file.info.hash == hash {
                // File unchanged, return existing share code
                info!(
                    "File '{}' already shared with code: {} (unchanged)",
                    filename, existing_share_code
                );
                return Ok(existing_share_code.clone());
            } else {
                // File changed, update the existing entry
                let updated_info = FileInfo {
                    id: existing_shared_file.info.id.clone(),
                    name: filename.clone(),
                    size: metadata.len(),
                    hash: hash.clone(),
                    chunk_count: (metadata.len() / CHUNK_SIZE as u64) as usize
                        + if metadata.len() % CHUNK_SIZE as u64 != 0 {
                            1
                        } else {
                            0
                        },
                    created_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs(),
                };

                let share_code = existing_share_code.clone();
                let updated_shared_file = SharedFile {
                    info: updated_info,
                    path: FilePath::Path(path.clone()),
                    share_code: share_code.clone(),
                    revoked: false,
                };

                self.shared_files
                    .insert(share_code.clone(), updated_shared_file);

                info!(
                    "Updated file '{}' (hash: {}) with existing code: {}",
                    filename,
                    &hash[..8],
                    share_code
                );
                return Ok(share_code);
            }
        }

        // New file, create new entry
        let share_code = self.generate_share_code(&filename);
        let file_id = share_code.clone();

        // Calculate chunk count
        let chunk_count = (metadata.len() / CHUNK_SIZE as u64) as usize
            + if metadata.len() % CHUNK_SIZE as u64 != 0 {
                1
            } else {
                0
            };

        // Create FileInfo
        let file_info = FileInfo {
            id: file_id.clone(),
            name: filename.clone(),
            size: metadata.len(),
            hash: hash.clone(),
            chunk_count,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        };

        let shared_file = SharedFile {
            info: file_info,
            path: FilePath::Path(path.clone()),
            share_code: share_code.clone(),
            revoked: false,
        };

        self.shared_files
            .insert(share_code.clone(), shared_file.clone());

        // Save to persistent storage
        self.save_to_store(&share_code, &shared_file).await?;

        info!(
            "Shared file '{}' (hash: {}) with code: {}",
            filename,
            &hash[..8],
            share_code
        );

        Ok(share_code)
    }

    /// Share a file from a content URI
    ///
    /// # Arguments
    ///
    /// * `uri` - Content URI (e.g., `content://com.android...`)
    /// * `name` - Display name for the file
    /// * `size` - Total file size in bytes
    ///
    /// # Returns
    ///
    /// The share code for the content
    ///
    /// # Platform Usage
    ///
    /// **Android**:
    /// - URIs from MediaStore: `content://com.android.providers.media.documents/document/...`
    /// - URIs from FilePicker: `content://com.android.externalstorage.documents/document/...`
    ///
    /// **iOS**:
    /// - URIs from UIDocumentPicker: `file:///private/var/mobile/Containers/Data/...`
    ///
    /// # Limitations
    ///
    /// - File hash is not calculated (empty string) since we can't read the file
    /// - Hash should be calculated by the recipient or stored separately
    /// - Chunks must be read via the `FileChunkReader` callback
    ///
    /// # Example (Android)
    ///
    /// ```rust,no_run
    /// use gigi_file_sharing::FileSharingManager;
    ///
    /// let uri = "content://com.android.providers.media.documents/document/image:1234";
    /// let name = "photo.jpg";
    /// let size = 1024 * 1024; // 1MB
    ///
    /// let code = manager.share_content_uri(uri, name, size).await?;
    /// ```
    pub async fn share_content_uri(&mut self, uri: &str, name: &str, size: u64) -> Result<String> {
        let url = Url::parse(uri)
            .map_err(|e: url::ParseError| FileSharingError::InvalidUri(e.to_string()))?;
        let share_code = self.generate_share_code(name);

        let file_id = share_code.clone();

        // Calculate chunk count
        let chunk_count =
            (size / CHUNK_SIZE as u64) as usize + if size % CHUNK_SIZE as u64 != 0 { 1 } else { 0 };

        // Create FileInfo
        let file_info = FileInfo {
            id: file_id.clone(),
            name: name.to_string(),
            size,
            hash: String::new(), // Will be calculated by the caller if needed
            chunk_count,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        };

        let shared_file = SharedFile {
            info: file_info,
            path: FilePath::Url(url),
            share_code: share_code.clone(),
            revoked: false,
        };

        self.shared_files.insert(share_code.clone(), shared_file);

        // Save to persistent storage
        self.save_to_store(&share_code, &self.shared_files[&share_code])
            .await?;

        info!("Shared content URI '{}' with code: {}", name, share_code);

        Ok(share_code)
    }

    /// List all currently shared files
    ///
    /// # Returns
    ///
    /// A vector of references to all shared files
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use gigi_file_sharing::FileSharingManager;
    ///
    /// let manager = FileSharingManager::new();
    /// // ... share some files ...
    ///
    /// for file in manager.list_shared_files() {
    ///     println!("{}: {} ({} bytes, {} chunks)",
    ///         file.share_code,
    ///         file.info.name,
    ///         file.info.size,
    ///         file.info.chunk_count
    ///     );
    /// }
    /// ```
    pub fn list_shared_files(&self) -> Vec<&SharedFile> {
        self.shared_files.values().collect()
    }

    /// Unshare (revoke access to) a file
    ///
    /// # Arguments
    ///
    /// * `share_code` - The share code to revoke
    ///
    /// # Returns
    ///
    /// `Ok(())` if successfully unshared
    ///
    /// # Errors
    ///
    /// - `InvalidShareCode`: If the share code doesn't exist
    ///
    /// # Notes
    ///
    /// This only removes the file from the sharing registry.
    /// It does NOT:
    /// - Delete the actual file from disk
    /// - Stop ongoing transfers
    /// - Notify recipients that the file is no longer available
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use gigi_file_sharing::FileSharingManager;
    ///
    /// let mut manager = FileSharingManager::new();
    /// let code = manager.share_file(&PathBuf::from("secret.pdf")).await?;
    ///
    /// // Later, revoke access
    /// manager.unshare_file(&code)?;
    /// ```
    pub fn unshare_file(&mut self, share_code: &str) -> Result<()> {
        if let Some(shared_file) = self.shared_files.remove(share_code) {
            info!(
                "Unshared file '{}' with share code: {}",
                shared_file.info.name, share_code
            );

            Ok(())
        } else {
            Err(FileSharingError::InvalidShareCode(share_code.to_string()).into())
        }
    }

    /// Calculate SHA256 hash of a file
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file to hash
    ///
    /// # Returns
    ///
    /// Hexadecimal string of the SHA256 hash
    ///
    /// # Algorithm
    ///
    /// Uses SHA256 for:
    /// - **Security**: Cryptographically secure
    /// - **Performance**: Fast computation on modern CPUs
    /// - **Uniqueness**: 256-bit output provides virtually no collisions
    /// - **Integrity**: Recipients can verify file wasn't corrupted
    ///
    /// # Buffer Size
    ///
    /// Uses 8KB buffer for:
    /// - Good I/O performance (matches typical block sizes)
    /// - Low memory usage (even on embedded devices)
    ///
    /// # Example
    ///
    /// ```text
    /// File: "document.pdf"
    /// SHA256: "3b4c5e8b5f2a1c9d5e0f7a6b3c8d5e2f1a9c4d8e6f7a0b1c2d3e4f5a6"
    /// ```
    pub fn calculate_file_hash(&self, file_path: &Path) -> Result<String> {
        use sha2::{Digest, Sha256};

        let mut file = std::fs::File::open(file_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Save shared file metadata to persistent storage
    ///
    /// # Arguments
    ///
    /// * `share_code` - The share code for the file
    /// * `shared_file` - The shared file metadata
    ///
    /// # Returns
    ///
    /// `Ok(())` on success (even if store is not configured)
    ///
    /// # Notes
    ///
    /// - This is a **fire-and-forget** operation
    /// - Spawns a background task to avoid blocking
    /// - Errors are logged but not propagated to caller
    /// - Safe to call even if no store is configured
    ///
    /// # Async Task
    ///
    /// The save operation runs in a spawned task to:
    /// - Avoid blocking the main async runtime
    /// - Allow the caller to continue immediately
    /// - Handle storage I/O independently
    async fn save_to_store(&self, share_code: &str, shared_file: &SharedFile) -> Result<()> {
        if let Some(store) = &self.file_sharing_store {
            let file_path = match &shared_file.path {
                FilePath::Path(p) => p.to_string_lossy().to_string(),
                FilePath::Url(u) => u.to_string(),
            };

            let info = gigi_store::SharedFileInfo::new(
                share_code.to_string(),
                shared_file.info.name.clone(),
                file_path,
                shared_file.info.size,
                shared_file.info.hash.clone(),
                shared_file.info.chunk_count,
                shared_file.info.created_at as i64,
            );

            let store_clone = Arc::clone(store);
            tokio::task::spawn(async move {
                if let Err(e) = store_clone.store_shared_file(&info).await {
                    tracing::error!("Failed to save shared file to store: {}", e);
                }
            });
        }
        Ok(())
    }

    /// Load shared files from persistent storage
    ///
    /// # Returns
    ///
    /// `Ok(())` on success (even if store is not configured)
    ///
    /// # Process
    ///
    /// 1. Retrieve all shared files from gigi-store
    /// 2. For each file:
    ///    - Check if file still exists on disk
    ///    - If exists: Add to in-memory registry
    ///    - If doesn't exist: Skip (orphaned entry)
    /// 3. Log summary of loaded files
    ///
    /// # Error Handling
    ///
    /// - Continues loading even if some files fail
    /// - Errors are logged via tracing
    /// - Orphaned entries (missing files) are silently skipped
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use gigi_file_sharing::FileSharingManager;
    ///
    /// let mut manager = FileSharingManager::new();
    /// manager.load_from_store().await?;
    /// println!("Loaded {} files", manager.list_shared_files().len());
    /// ```
    pub async fn load_from_store(&mut self) -> Result<()> {
        if let Some(store) = &self.file_sharing_store {
            let files = store.list_shared_files().await?;
            for file_info in files {
                let file_path = PathBuf::from(&file_info.file_path);
                // Only load files that still exist
                if file_path.exists() {
                    let shared_file = SharedFile {
                        info: FileInfo {
                            id: file_info.share_code.clone(),
                            name: file_info.file_name.clone(),
                            size: file_info.file_size,
                            hash: file_info.hash.clone(),
                            chunk_count: file_info.chunk_count,
                            created_at: file_info.created_at as u64,
                        },
                        path: FilePath::Path(file_path),
                        share_code: file_info.share_code.clone(),
                        revoked: file_info.revoked,
                    };
                    self.shared_files.insert(file_info.share_code, shared_file);
                }
            }
            info!(
                "Loaded {} shared files from gigi-store",
                self.shared_files.len()
            );
        }
        Ok(())
    }

    /// Update the thumbnail path for a shared file
    ///
    /// # Arguments
    ///
    /// * `share_code` - The share code of the file
    /// * `thumbnail_path` - Path to the generated thumbnail
    ///
    /// # Returns
    ///
    /// `Ok(())` on success
    ///
    /// # Usage
    ///
    /// Called after generating a thumbnail for a shared image/video:
    /// ```rust,no_run
    /// use gigi_file_sharing::FileSharingManager;
    /// use gigi_store::thumbnail::generate_thumbnail;
    ///
    /// let thumb = generate_thumbnail(&file_path, &thumb_dir, (200, 200), 80).await?;
    /// manager.update_thumbnail_path(&share_code, &thumb).await?;
    /// ```
    pub async fn update_thumbnail_path(
        &self,
        share_code: &str,
        thumbnail_path: &str,
    ) -> Result<()> {
        if let Some(store) = &self.file_sharing_store {
            store
                .update_thumbnail_path(share_code, thumbnail_path)
                .await?;
            info!("Updated thumbnail path for share_code: {}", share_code);
        }
        Ok(())
    }

    /// Get the thumbnail path for a shared file
    ///
    /// # Arguments
    ///
    /// * `share_code` - The share code of the file
    ///
    /// # Returns
    ///
    /// - `Ok(Some(path))` if thumbnail exists
    /// - `Ok(None)` if no thumbnail (or no store configured)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use gigi_file_sharing::FileSharingManager;
    ///
    /// let manager = FileSharingManager::new();
    /// if let Some(thumb_path) = manager.get_thumbnail_path(&share_code).await? {
    ///     println!("Thumbnail at: {}", thumb_path);
    /// }
    /// ```
    pub async fn get_thumbnail_path(&self, share_code: &str) -> Result<Option<String>> {
        if let Some(store) = &self.file_sharing_store {
            Ok(store.get_thumbnail_path(share_code).await?)
        } else {
            Ok(None)
        }
    }
}

impl Default for FileSharingManager {
    fn default() -> Self {
        Self::new()
    }
}
