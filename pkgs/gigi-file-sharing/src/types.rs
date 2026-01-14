//! Types for file sharing operations

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

/// File path representation - supports both filesystem paths and URIs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FilePath {
    Url(Url),      // Android content:// or iOS file:// URIs
    Path(PathBuf), // Regular filesystem paths
}

/// File information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub hash: String,
    pub chunk_count: usize,
    pub created_at: u64,
}

/// File sharing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedFile {
    pub info: FileInfo,
    pub path: FilePath,
    pub share_code: String,
    pub revoked: bool,
}
