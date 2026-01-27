//! Types for file sharing operations
//!
//! This module defines the core data structures used throughout the file sharing system,
//! including file path representations and file metadata.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

/// File path representation supporting both filesystem paths and URIs
///
/// This enum enables the file sharing system to work across different platforms:
///
/// # Variants
///
/// ## `Url(Url)`
/// Used on mobile platforms where files are accessed via content URIs:
/// - **Android**: `content://com.android.providers.media.documents/document/...`
/// - **iOS**: `file:///private/var/mobile/Containers/Data/...`
///
/// Requires a `FileChunkReader` callback to read file chunks.
///
/// ## `Path(PathBuf)`
/// Used on desktop platforms (Linux, macOS, Windows):
/// - Direct filesystem access
/// - Standard path operations
/// - No special callback needed
///
/// # Example
///
/// ```rust,no_run
/// use gigi_file_sharing::FilePath;
/// use std::path::PathBuf;
/// use url::Url;
///
/// // Desktop
/// let desktop_path = FilePath::Path(PathBuf::from("/home/user/file.txt"));
///
/// // Mobile (Android)
/// let android_uri = FilePath::Url(
///     Url::parse("content://com.android.../document").unwrap()
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FilePath {
    /// Android content:// or iOS file:// URIs
    Url(Url),
    /// Regular filesystem paths (desktop platforms)
    Path(PathBuf),
}

/// File metadata and sharing information
///
/// Contains all information needed to track and transfer a file.
///
/// # Fields
///
/// - `id`: Unique identifier (typically the share code)
/// - `name`: Display filename for the user
/// - `size`: Total file size in bytes
/// - `hash`: SHA256 hash for integrity verification
/// - `chunk_count`: Number of chunks (ceil(size / CHUNK_SIZE))
/// - `created_at`: Unix timestamp (seconds since epoch)
///
/// # Example
///
/// ```rust,no_run
/// use gigi_file_sharing::FileInfo;
///
/// let info = FileInfo {
///     id: "a1b2c3d4".to_string(),
///     name: "document.pdf".to_string(),
///     size: 1024 * 1024,  // 1MB
///     hash: "3b4c5e8b5f2...".to_string(),
///     chunk_count: 4,
///     created_at: 1640995200,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// Unique file identifier (typically the share code)
    pub id: String,
    /// Display filename
    pub name: String,
    /// File size in bytes
    pub size: u64,
    /// SHA256 hash for integrity verification (64 hex characters)
    pub hash: String,
    /// Number of chunks for chunked transfer
    pub chunk_count: usize,
    /// Creation timestamp (Unix epoch seconds)
    pub created_at: u64,
}

/// Complete shared file record
///
/// Combines file metadata with path and share information.
///
/// # Fields
///
/// - `info`: File metadata (FileInfo)
/// - `path`: File location (filesystem path or URI)
/// - `share_code`: Unique code for sharing
/// - `revoked`: Whether sharing has been revoked
///
/// # Lifecycle
///
/// ```text
/// Created (revoked = false)
///     ↓
/// Shared with peer
///     ↓
/// Downloaded by recipient
///     ↓
/// Unshared (revoked = true)
/// ```
///
/// # Example
///
/// ```rust,no_run
/// use gigi_file_sharing::{FileInfo, FilePath, SharedFile};
/// use std::path::PathBuf;
///
/// let shared_file = SharedFile {
///     info: FileInfo {
///         id: "test".to_string(),
///         name: "file.txt".to_string(),
///         size: 512,
///         hash: "abc123".to_string(),
///         chunk_count: 1,
///         created_at: 1640995200,
///     },
///     path: FilePath::Path(PathBuf::from("/path/to/file.pdf")),
///     share_code: "a1b2c3d4".to_string(),
///     revoked: false,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedFile {
    /// File metadata
    pub info: FileInfo,
    /// File location (path or URI)
    pub path: FilePath,
    /// Share code for this file
    pub share_code: String,
    /// Whether file sharing has been revoked
    pub revoked: bool,
}
