//! File sharing functionality

use anyhow::Result;
use blake3::Hasher;
use libp2p::PeerId;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::fs;
use tracing::info;

use crate::error::P2pError;
use crate::events::{ChunkInfo, FileInfo, SharedFile};

/// Constants for chunked file sharing
pub const CHUNK_SIZE: usize = 256 * 1024; // 256KB chunks for better performance

/// Downloading file information
#[derive(Debug, Clone)]
pub struct DownloadingFile {
    pub info: FileInfo,
    pub output_path: PathBuf,
    pub temp_path: PathBuf,
    pub downloaded_chunks: HashMap<usize, bool>,
    pub started_at: Instant,
    pub next_chunk_to_request: usize,
    pub max_concurrent_requests: usize,
    pub peer_id: PeerId,
}

/// File sharing manager
pub struct FileSharingManager {
    pub shared_files: HashMap<String, SharedFile>,
    pub downloading_files: HashMap<String, DownloadingFile>,
    pub output_directory: PathBuf,
    pub shared_file_path: PathBuf,
}

impl FileSharingManager {
    pub fn new(output_directory: PathBuf, shared_file_path: PathBuf) -> Self {
        Self {
            shared_files: HashMap::new(),
            downloading_files: HashMap::new(),
            output_directory,
            shared_file_path,
        }
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

    /// Calculate SHA256 hash of a file
    pub fn calculate_file_hash(&self, file_path: &Path) -> Result<String> {
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

    /// Calculate hash of data chunk
    pub fn calculate_chunk_hash(&self, data: &[u8]) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        hasher.finalize().to_hex().to_string()
    }

    /// Read a chunk from a file
    pub fn read_chunk(
        &self,
        file_path: &Path,
        chunk_index: usize,
        file_id: &str,
    ) -> Result<ChunkInfo> {
        let mut file = std::fs::File::open(file_path)?;
        let offset = chunk_index * CHUNK_SIZE;
        file.seek(std::io::SeekFrom::Start(offset as u64))?;

        let mut buffer = vec![0u8; CHUNK_SIZE];
        let bytes_read = file.read(&mut buffer)?;
        buffer.truncate(bytes_read);

        let hash = self.calculate_chunk_hash(&buffer);

        Ok(ChunkInfo {
            file_id: file_id.to_string(),
            chunk_index,
            data: buffer,
            hash,
        })
    }

    /// Share a file
    pub async fn share_file(&mut self, file_path: &Path) -> Result<String> {
        let path = file_path
            .canonicalize()
            .map_err(|_| P2pError::FileNotFound(file_path.to_path_buf()))?;

        let metadata = fs::metadata(&path).await?;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| P2pError::FileNotFound(path.clone()))?
            .to_string();

        // Calculate file hash
        let hash = self.calculate_file_hash(&path)?;

        // Check if file is already shared
        if let Some((existing_share_code, existing_shared_file)) = self
            .shared_files
            .iter()
            .find(|(_, shared_file)| shared_file.path == path)
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
                    chunk_count: (metadata.len() as usize + CHUNK_SIZE - 1) / CHUNK_SIZE,
                    created_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs(),
                };

                let share_code = existing_share_code.clone();
                let updated_shared_file = SharedFile {
                    info: updated_info,
                    path: path.clone(),
                    share_code: share_code.clone(),
                    revoked: false,
                };

                self.shared_files
                    .insert(share_code.clone(), updated_shared_file);
                self.save_shared_files()?;

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
        let chunk_count = (metadata.len() as usize + CHUNK_SIZE - 1) / CHUNK_SIZE;

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
            path: path.clone(),
            share_code: share_code.clone(),
            revoked: false,
        };

        self.shared_files.insert(share_code.clone(), shared_file);

        // Save to persistent storage
        self.save_shared_files()?;

        info!(
            "Shared file '{}' (hash: {}) with code: {}",
            filename,
            &hash[..8],
            share_code
        );

        Ok(share_code)
    }

    /// List shared files
    pub fn list_shared_files(&self) -> Vec<&SharedFile> {
        self.shared_files.values().collect()
    }

    /// Unshare a file by share code
    pub fn unshare_file(&mut self, share_code: &str) -> Result<()> {
        if let Some(shared_file) = self.shared_files.remove(share_code) {
            // Save updated shared files
            self.save_shared_files()?;

            info!(
                "Unshared file '{}' with share code: {}",
                shared_file.info.name, share_code
            );

            Ok(())
        } else {
            Err(P2pError::InvalidShareCode(share_code.to_string()).into())
        }
    }

    /// Load shared files from persistent storage
    pub fn load_shared_files(&mut self) -> Result<()> {
        // For relative paths, use current working directory, not persistent_dir
        let shared_files_path = if self.shared_file_path.is_absolute() {
            self.shared_file_path.clone()
        } else {
            std::env::current_dir()?.join(&self.shared_file_path)
        };

        if shared_files_path.exists() {
            let content = std::fs::read_to_string(&shared_files_path)?;

            // Try to deserialize as new format first
            let loaded_files: Result<HashMap<String, SharedFile>, _> =
                serde_json::from_str(&content);

            let loaded_files = match loaded_files {
                Ok(files) => files,
                Err(_) => {
                    // Try to deserialize as old format and migrate
                    #[derive(Debug, Deserialize)]
                    struct OldSharedFile {
                        path: PathBuf,
                        filename: String,
                        size: u64,
                        share_code: String,
                        hash: String,
                        created_at: String,
                    }

                    let old_files: HashMap<String, OldSharedFile> = serde_json::from_str(&content)?;
                    old_files
                        .into_iter()
                        .map(|(share_code, old_file)| {
                            // Parse the created_at timestamp
                            use chrono::{DateTime, Utc};
                            let created_at = old_file
                                .created_at
                                .parse::<DateTime<Utc>>()
                                .unwrap_or_else(|_| Utc::now())
                                .timestamp() as u64;

                            let shared_file = SharedFile {
                                info: FileInfo {
                                    id: share_code.clone(),
                                    name: old_file.filename,
                                    size: old_file.size,
                                    hash: old_file.hash,
                                    chunk_count: ((old_file.size + CHUNK_SIZE as u64 - 1)
                                        / CHUNK_SIZE as u64)
                                        as usize,
                                    created_at,
                                },
                                path: old_file.path,
                                share_code: old_file.share_code,
                                revoked: false,
                            };
                            (share_code, shared_file)
                        })
                        .collect()
                }
            };

            // Only load files that still exist on disk
            for (share_code, shared_file) in loaded_files {
                if shared_file.path.exists() {
                    self.shared_files.insert(share_code, shared_file);
                }
            }

            info!(
                "Loaded {} shared files from storage",
                self.shared_files.len()
            );
        }

        Ok(())
    }

    /// Save shared files to persistent storage
    pub fn save_shared_files(&self) -> Result<()> {
        let shared_files_path = if self.shared_file_path.is_absolute() {
            self.shared_file_path
                .parent()
                .map(|p| std::fs::create_dir_all(p))
                .transpose()?
                .unwrap_or(());
            self.shared_file_path.clone()
        } else {
            // For relative paths, use current working directory
            let path = std::env::current_dir()?.join(&self.shared_file_path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            path
        };

        let content = serde_json::to_string_pretty(&self.shared_files)?;
        std::fs::write(&shared_files_path, content)?;

        Ok(())
    }

    /// Find available filename (append number if exists)
    pub fn find_available_filename(&self, filename: &str) -> String {
        let path = self.output_directory.join(filename);

        if !path.exists() {
            return filename.to_string();
        }

        // Extract name and extension
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        for i in 1..1000 {
            let candidate = if extension.is_empty() {
                format!("{}_{}", stem, i)
            } else {
                format!("{}_{}.{}", stem, i, extension)
            };

            if !self.output_directory.join(&candidate).exists() {
                return candidate;
            }
        }

        // Fallback to timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!("{}_{}.{}", stem, timestamp, extension)
    }

    /// Start downloading a file after receiving file info
    pub fn start_download(&mut self, peer_id: PeerId, info: FileInfo) -> Result<()> {
        // Find available filename
        let filename = self.find_available_filename(&info.name);
        let output_path = self.output_directory.join(&filename);
        let temp_path = self
            .output_directory
            .join(format!("{}.downloading", info.id));

        // Create downloading file entry
        let downloading_file = DownloadingFile {
            info: info.clone(),
            output_path: output_path.clone(),
            temp_path: temp_path.clone(),
            downloaded_chunks: HashMap::new(),
            started_at: Instant::now(),
            next_chunk_to_request: 0,
            max_concurrent_requests: 5, // Allow 5 concurrent chunk requests
            peer_id,
        };

        self.downloading_files
            .insert(info.id.clone(), downloading_file);

        Ok(())
    }
}
