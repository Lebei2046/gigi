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
    chunk_reader: Option<super::file_sharing::FileChunkReader>,
    request_id_to_download: HashMap<String, String>, // request_id (as string) -> download_id mapping
}

impl DownloadManager {
    /// Create a new download manager
    pub fn new(output_directory: PathBuf) -> Self {
        Self {
            active_downloads: HashMap::new(),
            download_share_codes: HashMap::new(),
            downloading_files: HashMap::new(),
            output_directory,
            chunk_reader: None,
            request_id_to_download: HashMap::new(),
        }
    }

    /// Set the chunk reader callback for URI-based files
    pub fn set_chunk_reader(&mut self, reader: super::file_sharing::FileChunkReader) {
        self.chunk_reader = Some(reader);
    }

    /// Start tracking a new download
    pub fn start_download(
        &mut self,
        peer_id: libp2p::PeerId,
        from_nickname: String,
        share_code: String,
        filename: Option<String>,
    ) -> String {
        // Only remove COMPLETED downloads with same share_code before creating new one
        // This allows parallel downloads of the same file, but prevents reuse of old completed entries
        let to_remove: Vec<String> = self
            .active_downloads
            .values()
            .filter(|d| d.share_code == share_code && d.completed)
            .map(|d| d.download_id.clone())
            .collect();
        for dl_id in to_remove {
            self.active_downloads.remove(&dl_id);
        }

        // Create unique download_id for this specific download
        // Use nanos for uniqueness when multiple downloads start at same time
        let download_id = format!(
            "pending_{}_{}",
            share_code,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
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
        // Keep the original download_id - don't create a new one
        // This ensures the frontend mapping remains valid

        // Get existing download to preserve share_code and started_at
        let share_code = self
            .get_share_code_for_download(download_id)
            .unwrap_or_else(|| download_id.to_string());
        let started_at = self
            .active_downloads
            .get(download_id)
            .map(|d| d.started_at)
            .unwrap_or_else(std::time::Instant::now);

        // Store share code mapping
        self.download_share_codes
            .insert(share_code.clone(), download_id.to_string());

        // Update the active download entry in place
        let active_download = ActiveDownload {
            download_id: download_id.to_string(),
            filename,
            share_code,
            from_peer_id,
            from_nickname,
            total_chunks,
            downloaded_chunks: 0,
            started_at,
            completed: false,
            failed: false,
            error_message: None,
            final_path: None,
        };

        self.active_downloads
            .insert(download_id.to_string(), active_download);

        Ok(download_id.to_string())
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

    /// Associate a request_id with a download_id
    pub fn map_request_to_download(&mut self, request_id: String, download_id: String) {
        self.request_id_to_download.insert(request_id, download_id);
    }

    /// Get download_id by request_id
    pub fn get_download_by_request_id(&self, request_id: &str) -> Option<String> {
        self.request_id_to_download.get(request_id).cloned()
    }

    /// Clean up request_id to download_id mapping
    pub fn cleanup_request_mapping(&mut self, request_id: &str) {
        self.request_id_to_download.remove(request_id);
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
    pub fn start_download_file(
        &mut self,
        _peer_id: libp2p::PeerId,
        info: FileInfo,
        download_id: Option<&str>,
    ) -> Result<()> {
        // Find available filename
        let filename = self.find_available_filename(&info.name);
        let output_path = self.output_directory.join(&filename);

        // Use download_id for temp path to ensure uniqueness when same file is downloaded multiple times
        // If download_id is provided, use it; otherwise fall back to info.id with timestamp
        let temp_path = if let Some(dl_id) = download_id {
            // Extract the unique part from download_id (e.g., "dl_..." or "pending_...")
            // Use the download_id directly to ensure unique temp paths
            self.output_directory.join(format!("{}.downloading", dl_id))
        } else {
            // Fallback: use info.id with timestamp for uniqueness
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            self.output_directory
                .join(format!("{}_{}.downloading", info.id, timestamp))
        };

        // Create downloading file entry
        let downloading_file = DownloadingFile {
            info: info.clone(),
            output_path: output_path.clone(),
            temp_path: temp_path.clone(),
            downloaded_chunks: HashMap::new(),
        };

        // Use download_id as key instead of info.id to support parallel downloads of the same file
        let key = download_id.unwrap_or(&info.id);
        self.downloading_files
            .insert(key.to_string(), downloading_file);

        Ok(())
    }

    /// Get downloading file by download ID
    pub fn get_downloading_file(&self, download_id: &str) -> Option<&DownloadingFile> {
        self.downloading_files.get(download_id)
    }

    /// Get downloading file by download ID (mutable)
    pub fn get_downloading_file_mut(&mut self, download_id: &str) -> Option<&mut DownloadingFile> {
        self.downloading_files.get_mut(download_id)
    }

    /// Remove downloading file
    pub fn remove_downloading_file(&mut self, download_id: &str) -> Option<DownloadingFile> {
        self.downloading_files.remove(download_id)
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
        file_path: &crate::events::FilePath,
        chunk_index: usize,
        file_id: &str,
    ) -> Result<crate::events::ChunkInfo> {
        use crate::events::{ChunkInfo, FilePath};
        use gigi_file_sharing::CHUNK_SIZE;

        let offset = chunk_index
            .checked_mul(CHUNK_SIZE)
            .ok_or_else(|| anyhow::anyhow!("Chunk index overflow: {}", chunk_index))?;

        match file_path {
            FilePath::Path(path) => {
                // Regular file - use std::fs
                let mut file = std::fs::File::open(path)?;
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
            FilePath::Url(_url) => {
                // Content URI or file:// URI - use callback
                if let Some(reader) = &self.chunk_reader {
                    match reader(file_path, offset as u64, CHUNK_SIZE) {
                        Ok(data) => {
                            let hash = self.calculate_chunk_hash(&data);
                            Ok(ChunkInfo {
                                file_id: file_id.to_string(),
                                chunk_index,
                                data,
                                hash,
                            })
                        }
                        Err(e) => Err(anyhow::anyhow!("Failed to read chunk from URI: {}", e)),
                    }
                } else {
                    Err(anyhow::anyhow!("No chunk reader configured for URIs"))
                }
            }
        }
    }

    /// Process a received chunk - handles verification, storage, and progress tracking
    pub fn process_received_chunk(
        &mut self,
        download_id: &str,
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
                .get_downloading_file(download_id)
                .ok_or_else(|| anyhow::anyhow!("Download not found: {}", download_id))?;
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
            .get_downloading_file_mut(download_id)
            .ok_or_else(|| anyhow::anyhow!("Download not found: {}", download_id))?;

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
            .checked_mul(gigi_file_sharing::CHUNK_SIZE)
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
        download_id: &str,
        max_concurrent_requests: usize,
    ) -> Option<Vec<usize>> {
        let downloading_file = self.get_downloading_file(download_id)?;

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
    pub fn mark_chunks_requested(
        &mut self,
        download_id: &str,
        chunk_indices: &[usize],
    ) -> Result<()> {
        let downloading_file = self
            .get_downloading_file_mut(download_id)
            .ok_or_else(|| anyhow::anyhow!("Download not found: {}", download_id))?;

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
