//! Download management functionality for mobile apps

use anyhow::Result;
use sha2::Digest;
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::events::{ActiveDownload, FileInfo};

/// Downloading file information
#[derive(Debug, Clone)]
pub struct DownloadingFile {
    pub info: FileInfo,
    pub output_path: PathBuf,
    pub temp_path: PathBuf,
    pub downloaded_chunks: HashMap<usize, bool>,
}

/// Download management functionality
pub struct DownloadManager {
    active_downloads: HashMap<String, ActiveDownload>,
    download_share_codes: HashMap<String, String>, // download_id -> share_code mapping
    downloading_files: HashMap<String, DownloadingFile>,
    output_directory: PathBuf,
}

impl DownloadManager {
    /// Create a new download manager
    pub fn new(output_directory: PathBuf) -> Self {
        Self {
            active_downloads: HashMap::new(),
            download_share_codes: HashMap::new(),
            downloading_files: HashMap::new(),
            output_directory,
        }
    }

    /// Start tracking a new download
    pub fn start_download(
        &mut self,
        peer_id: libp2p::PeerId,
        from_nickname: String,
        share_code: String,
        filename: Option<String>,
    ) -> String {
        // Create unique download_id for this specific download
        let download_id = format!(
            "pending_{}_{}",
            share_code,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );

        let active_download = ActiveDownload {
            download_id: download_id.clone(),
            filename: filename.unwrap_or_else(|| "Loading...".to_string()), // Will be updated when file info arrives
            share_code: share_code.clone(),
            from_peer_id: peer_id,
            from_nickname,
            total_chunks: 0,
            downloaded_chunks: 0,
            started_at: Instant::now(),
            completed: false,
            failed: false,
            error_message: None,
            final_path: None,
        };

        self.active_downloads
            .insert(download_id.clone(), active_download);

        download_id
    }

    /// Update download when file info is received
    pub fn update_download_with_file_info(
        &mut self,
        download_id: &str,
        filename: String,
        total_chunks: usize,
        from_peer_id: libp2p::PeerId,
        from_nickname: String,
    ) -> Result<String> {
        // Create proper download_id for this specific download
        let final_download_id = format!(
            "dl_{}_{}_{}",
            filename,
            from_peer_id,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );

        // Store share code mapping - map share_code to final_download_id for easier lookup
        if let Some(active_download) = self.active_downloads.get(download_id) {
            self.download_share_codes.insert(
                active_download.share_code.clone(),
                final_download_id.clone(),
            );
        }

        // Update or create active download entry
        let active_download = ActiveDownload {
            download_id: final_download_id.clone(),
            filename: filename.clone(),
            share_code: self
                .get_share_code_for_download(download_id)
                .unwrap_or_else(|| download_id.to_string()),
            from_peer_id,
            from_nickname,
            total_chunks,
            downloaded_chunks: 0,
            started_at: std::time::Instant::now(),
            completed: false,
            failed: false,
            error_message: None,
            final_path: None,
        };

        self.active_downloads
            .insert(final_download_id.clone(), active_download);

        // Remove the old pending download
        self.active_downloads.remove(download_id);

        Ok(final_download_id)
    }

    /// Update download progress
    pub fn update_download_progress(&mut self, download_id: &str, downloaded_chunks: usize) {
        if let Some(active_download) = self.active_downloads.get_mut(download_id) {
            active_download.downloaded_chunks = downloaded_chunks;
        }
    }

    /// Mark download as completed
    pub fn complete_download(
        &mut self,
        download_id: &str,
        final_path: std::path::PathBuf,
    ) -> Option<ActiveDownload> {
        if let Some(mut active_download) = self.active_downloads.remove(download_id) {
            active_download.completed = true;
            active_download.final_path = Some(final_path);
            Some(active_download)
        } else {
            None
        }
    }

    /// Mark download as failed
    pub fn fail_download(
        &mut self,
        download_id: &str,
        error_message: String,
    ) -> Option<ActiveDownload> {
        if let Some(mut active_download) = self.active_downloads.remove(download_id) {
            active_download.failed = true;
            active_download.error_message = Some(error_message);
            Some(active_download)
        } else {
            None
        }
    }

    /// Get all active downloads
    pub fn get_active_downloads(&self) -> Vec<&ActiveDownload> {
        self.active_downloads.values().collect()
    }

    /// Get active download by download_id
    pub fn get_active_download(&self, download_id: &str) -> Option<&ActiveDownload> {
        self.active_downloads.get(download_id)
    }

    /// Get active download by share code
    pub fn get_download_by_share_code(&self, share_code: &str) -> Option<&ActiveDownload> {
        self.active_downloads
            .values()
            .find(|download| download.share_code == share_code)
    }

    /// Remove completed or failed downloads (cleanup)
    pub fn cleanup_downloads(&mut self) {
        self.active_downloads
            .retain(|_, download| !download.completed && !download.failed);
    }

    /// Get downloads from a specific peer
    pub fn get_downloads_from_peer(&self, peer_nickname: &str) -> Vec<&ActiveDownload> {
        self.active_downloads
            .values()
            .filter(|download| download.from_nickname == peer_nickname)
            .collect()
    }

    /// Get recent downloads (useful for UI history)
    pub fn get_recent_downloads(&self, limit: usize) -> Vec<&ActiveDownload> {
        let mut downloads: Vec<&ActiveDownload> = self
            .active_downloads
            .values()
            .filter(|download| download.completed || download.failed)
            .collect();

        // Sort by started time (most recent first)
        downloads.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        downloads.truncate(limit);
        downloads
    }

    /// Helper to find share code for a download
    pub fn find_share_code_for_file(&self, file_id: &str) -> Option<String> {
        // First check if we have it mapped
        if let Some(share_code) = self.download_share_codes.get(file_id) {
            return Some(share_code.clone());
        }

        // Look for pending downloads with this file_id pattern
        self.active_downloads
            .values()
            .find(|download| {
                download.download_id.contains(file_id)
                    || download.download_id.starts_with("pending_")
            })
            .map(|download| download.share_code.clone())
    }

    /// Helper to find download_id by file_id (share_code)
    pub fn find_download_id_by_file_id(&self, file_id: &str) -> Option<String> {
        // First check if we have a direct mapping from share_code to download_id
        if let Some(_download_id) = self.download_share_codes.get(file_id) {
            // Now find the actual download entry that corresponds to this share_code
            for download in self.active_downloads.values() {
                if download.share_code == file_id {
                    return Some(download.download_id.clone());
                }
            }
        }

        // Look through active downloads to find the one with this share_code
        self.active_downloads
            .values()
            .find(|download| {
                download.share_code == file_id || download.download_id.contains(file_id)
            })
            .map(|download| download.download_id.clone())
    }

    /// Get download info for events
    pub fn get_download_info_for_event(
        &self,
        download_id: &Option<String>,
    ) -> (String, String, String, String, libp2p::PeerId) {
        if let Some(download_id) = download_id {
            if let Some(active_download) = self.active_downloads.get(download_id) {
                (
                    active_download.download_id.clone(),
                    active_download.filename.clone(),
                    active_download.share_code.clone(),
                    active_download.from_nickname.clone(),
                    active_download.from_peer_id,
                )
            } else {
                (
                    download_id.clone(),
                    "Unknown".to_string(),
                    "Unknown".to_string(),
                    "Unknown".to_string(),
                    libp2p::PeerId::random(),
                )
            }
        } else {
            (
                "unknown".to_string(),
                "Unknown".to_string(),
                "Unknown".to_string(),
                "Unknown".to_string(),
                libp2p::PeerId::random(),
            )
        }
    }

    /// Get share code for a download
    pub fn get_share_code_for_download(&self, download_id: &str) -> Option<String> {
        if let Some(share_code) = self.download_share_codes.get(download_id) {
            Some(share_code.clone())
        } else if let Some(active_download) = self.active_downloads.get(download_id) {
            Some(active_download.share_code.clone())
        } else {
            None
        }
    }

    // ===== File System and Download Management Methods =====

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
    pub fn start_download_file(&mut self, _peer_id: libp2p::PeerId, info: FileInfo) -> Result<()> {
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
        };

        self.downloading_files
            .insert(info.id.clone(), downloading_file);

        Ok(())
    }

    /// Get downloading file by file ID
    pub fn get_downloading_file(&self, file_id: &str) -> Option<&DownloadingFile> {
        self.downloading_files.get(file_id)
    }

    /// Get downloading file by file ID (mutable)
    pub fn get_downloading_file_mut(&mut self, file_id: &str) -> Option<&mut DownloadingFile> {
        self.downloading_files.get_mut(file_id)
    }

    /// Remove downloading file
    pub fn remove_downloading_file(&mut self, file_id: &str) -> Option<DownloadingFile> {
        self.downloading_files.remove(file_id)
    }

    /// Calculate SHA256 hash of a file
    pub fn calculate_file_hash(&self, file_path: &Path) -> Result<String> {
        let mut file = std::fs::File::open(file_path)?;
        let mut hasher = sha2::Sha256::new();
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

    /// Read a chunk from a shared file (for serving downloads to others)
    pub fn read_chunk(
        &self,
        file_path: &Path,
        chunk_index: usize,
        file_id: &str,
    ) -> Result<crate::events::ChunkInfo> {
        use crate::events::ChunkInfo;

        let mut file = std::fs::File::open(file_path)?;
        let offset = chunk_index
            .checked_mul(crate::client::file_sharing::CHUNK_SIZE)
            .ok_or_else(|| anyhow::anyhow!("Chunk index overflow: {}", chunk_index))?;
        file.seek(std::io::SeekFrom::Start(offset as u64))?;

        let mut buffer = vec![0u8; crate::client::file_sharing::CHUNK_SIZE];
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

    /// Process a received chunk - handles verification, storage, and progress tracking
    pub fn process_received_chunk(
        &mut self,
        file_id: &str,
        chunk_index: usize,
        chunk: &crate::events::ChunkInfo,
    ) -> Result<ChunkProcessResult> {
        // Verify chunk hash
        let calculated_hash = self.calculate_chunk_hash(&chunk.data);
        if calculated_hash != chunk.hash {
            return Ok(ChunkProcessResult::HashMismatch);
        }

        // Get downloading file info and extract needed data before borrowing
        let (temp_path, output_path, expected_hash, total_chunks) = {
            let downloading_file = self
                .get_downloading_file(file_id)
                .ok_or_else(|| anyhow::anyhow!("File not found in downloads: {}", file_id))?;
            (
                downloading_file.temp_path.clone(),
                downloading_file.output_path.clone(),
                downloading_file.info.hash.clone(),
                downloading_file.info.chunk_count,
            )
        };

        // Write chunk to temp file
        if let Err(e) = self.write_chunk_to_file(&temp_path, chunk_index, &chunk.data) {
            return Ok(ChunkProcessResult::WriteFailed(e.to_string()));
        }

        // Now get mutable reference to update progress
        let downloading_file = self
            .get_downloading_file_mut(file_id)
            .ok_or_else(|| anyhow::anyhow!("File not found in downloads: {}", file_id))?;

        // Mark chunk as downloaded
        downloading_file.downloaded_chunks.insert(chunk_index, true);

        // Calculate progress
        let downloaded_count = downloading_file
            .downloaded_chunks
            .values()
            .filter(|&&v| v)
            .count();

        // Check if download is complete
        let is_complete = downloaded_count >= total_chunks;

        Ok(ChunkProcessResult::Success {
            downloaded_count,
            total_chunks,
            is_complete,
            output_path,
            temp_path,
            expected_hash,
        })
    }

    /// Write chunk data to file at specific offset
    fn write_chunk_to_file(&self, temp_path: &Path, chunk_index: usize, data: &[u8]) -> Result<()> {
        use std::io::{Seek, Write};

        let offset = chunk_index
            .checked_mul(crate::client::file_sharing::CHUNK_SIZE)
            .ok_or_else(|| anyhow::anyhow!("Chunk index overflow: {}", chunk_index))?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(temp_path)?;

        file.seek(std::io::SeekFrom::Start(offset as u64))?;
        file.write_all(data)?;
        file.flush()?;
        Ok(())
    }

    /// Get next chunks to request for maintaining optimal concurrency
    pub fn get_next_chunks_to_request(
        &self,
        file_id: &str,
        max_concurrent_requests: usize,
    ) -> Option<Vec<usize>> {
        let downloading_file = self.get_downloading_file(file_id)?;

        let downloaded_count = downloading_file
            .downloaded_chunks
            .values()
            .filter(|&&v| v)
            .count();
        let total_chunks = downloading_file.info.chunk_count;

        // Calculate how many chunks we should have requested by now
        let chunks_we_should_have_requested =
            std::cmp::min(downloaded_count + max_concurrent_requests, total_chunks);
        let chunks_already_requested = downloading_file.downloaded_chunks.len();

        // Request more chunks if needed
        if chunks_already_requested < chunks_we_should_have_requested {
            let requests_to_make = chunks_we_should_have_requested - chunks_already_requested;
            let mut next_chunks = Vec::new();

            for next_chunk in 0..total_chunks {
                if !downloading_file.downloaded_chunks.contains_key(&next_chunk) {
                    next_chunks.push(next_chunk);
                    if next_chunks.len() >= requests_to_make {
                        break;
                    }
                }
            }

            if !next_chunks.is_empty() {
                return Some(next_chunks);
            }
        }

        None
    }

    /// Mark chunks as requested (not downloaded)
    pub fn mark_chunks_requested(&mut self, file_id: &str, chunk_indices: &[usize]) -> Result<()> {
        let downloading_file = self
            .get_downloading_file_mut(file_id)
            .ok_or_else(|| anyhow::anyhow!("File not found in downloads: {}", file_id))?;

        for &chunk_index in chunk_indices {
            downloading_file
                .downloaded_chunks
                .insert(chunk_index, false);
        }

        Ok(())
    }
}

/// Result of processing a received chunk
#[derive(Debug)]
pub enum ChunkProcessResult {
    Success {
        downloaded_count: usize,
        total_chunks: usize,
        is_complete: bool,
        output_path: PathBuf,
        temp_path: PathBuf,
        expected_hash: String,
    },
    HashMismatch,
    WriteFailed(String),
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new(std::path::PathBuf::from("./downloads"))
    }
}
