//! File sharing functionality
//!
//! This crate provides file sharing management capabilities including:
//! - File sharing with chunked transfer support
//! - Support for filesystem paths and URIs
//! - Persistent storage via gigi-store
//! - File hash calculation
//! - Share code generation

pub mod error;
pub mod types;

// Re-export types for convenience
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

use crate::error::FileSharingError;
use gigi_store::FileSharingStore;

/// Constants for chunked file sharing
pub const CHUNK_SIZE: usize = 256 * 1024; // 256KB chunks for better performance

/// Callback type for reading file chunks from URIs
pub type FileChunkReader = Arc<dyn Fn(&FilePath, u64, usize) -> Result<Vec<u8>> + Send + Sync>;

/// File sharing manager
pub struct FileSharingManager {
    pub shared_files: HashMap<String, SharedFile>,
    chunk_reader: Option<FileChunkReader>,
    file_sharing_store: Option<Arc<FileSharingStore>>,
}

impl FileSharingManager {
    pub fn new() -> Self {
        Self {
            shared_files: HashMap::new(),
            chunk_reader: None,
            file_sharing_store: None,
        }
    }

    pub fn with_store(mut self, store: Arc<FileSharingStore>) -> Self {
        self.file_sharing_store = Some(store);
        self
    }

    /// Set the chunk reader callback for URI-based files
    pub fn set_chunk_reader(&mut self, reader: FileChunkReader) {
        self.chunk_reader = Some(reader);
    }

    /// Generate share code for file
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

    /// Share a file
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

    /// Share a content URI (Android content:// or iOS file://)
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

    /// List shared files
    pub fn list_shared_files(&self) -> Vec<&SharedFile> {
        self.shared_files.values().collect()
    }

    /// Unshare a file by share code
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

    /// Save shared file to gigi-store (if available)
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

    /// Load shared files from gigi-store (if available)
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

    /// Update thumbnail path for a shared file
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

    /// Get thumbnail path for a shared file
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
