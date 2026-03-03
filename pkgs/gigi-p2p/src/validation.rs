//! Input validation utilities for P2P operations
//!
//! This module provides validation functions for all user inputs to prevent
//! security issues such as injection attacks, DoS attacks, and resource exhaustion.

use crate::P2pError;
use libp2p::PeerId;
use std::path::Path;

const MAX_NICKNAME_LENGTH: usize = 64;
const MAX_MESSAGE_LENGTH: usize = 100_000; // 100KB
const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024 * 1024; // 5GB
const MAX_GROUP_NAME_LENGTH: usize = 128;
const MAX_SHARE_CODE_LENGTH: usize = 256;
const MAX_URI_LENGTH: usize = 2048;

/// Validate a nickname
///
/// Ensures nickname is not empty, not too long, and contains only safe characters.
///
/// # Arguments
/// * `nickname` - The nickname to validate
///
/// # Returns
/// Ok if valid, Err(P2pError) if invalid
pub fn validate_nickname(nickname: &str) -> Result<(), P2pError> {
    if nickname.is_empty() {
        return Err(P2pError::InvalidInput("Nickname cannot be empty".into()));
    }

    if nickname.len() > MAX_NICKNAME_LENGTH {
        return Err(P2pError::InvalidInput(format!(
            "Nickname too long (max {} characters)",
            MAX_NICKNAME_LENGTH
        )));
    }

    // Only allow alphanumeric, spaces, underscores, and hyphens
    if !nickname
        .chars()
        .all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '_' || c == '-')
    {
        return Err(P2pError::InvalidInput(
            "Nickname contains invalid characters".into(),
        ));
    }

    Ok(())
}

/// Validate a message
///
/// Ensures message is not too large to prevent DoS attacks.
///
/// # Arguments
/// * `message` - The message to validate
///
/// # Returns
/// Ok if valid, Err(P2pError) if invalid
pub fn validate_message(message: &str) -> Result<(), P2pError> {
    if message.len() > MAX_MESSAGE_LENGTH {
        return Err(P2pError::InvalidInput(format!(
            "Message too long (max {} characters)",
            MAX_MESSAGE_LENGTH
        )));
    }

    // Check for potential XSS patterns
    let dangerous_patterns = ["<script", "javascript:", "onerror=", "onload=", "data:"];
    let lower = message.to_lowercase();
    for pattern in &dangerous_patterns {
        if lower.contains(pattern) {
            return Err(P2pError::InvalidInput(format!(
                "Message contains potentially dangerous content: {}",
                pattern
            )));
        }
    }

    Ok(())
}

/// Validate a file size
///
/// Ensures file is not too large to prevent DoS attacks.
///
/// # Arguments
/// * `size` - The file size in bytes
///
/// # Returns
/// Ok if valid, Err(P2pError) if invalid
pub fn validate_file_size(size: u64) -> Result<(), P2pError> {
    if size > MAX_FILE_SIZE {
        return Err(P2pError::InvalidInput(format!(
            "File too large (max {} GB)",
            MAX_FILE_SIZE / (1024 * 1024 * 1024)
        )));
    }

    Ok(())
}

/// Validate a group name
///
/// Ensures group name is valid and safe.
///
/// # Arguments
/// * `group_name` - The group name to validate
///
/// # Returns
/// Ok if valid, Err(P2pError) if invalid
pub fn validate_group_name(group_name: &str) -> Result<(), P2pError> {
    if group_name.is_empty() {
        return Err(P2pError::InvalidInput("Group name cannot be empty".into()));
    }

    if group_name.len() > MAX_GROUP_NAME_LENGTH {
        return Err(P2pError::InvalidInput(format!(
            "Group name too long (max {} characters)",
            MAX_GROUP_NAME_LENGTH
        )));
    }

    // Only allow alphanumeric, spaces, underscores, and hyphens
    if !group_name
        .chars()
        .all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '_' || c == '-')
    {
        return Err(P2pError::InvalidInput(
            "Group name contains invalid characters".into(),
        ));
    }

    Ok(())
}

/// Validate a share code
///
/// Ensures share code format is valid.
///
/// # Arguments
/// * `share_code` - The share code to validate
///
/// # Returns
/// Ok if valid, Err(P2pError) if invalid
pub fn validate_share_code(share_code: &str) -> Result<(), P2pError> {
    if share_code.is_empty() {
        return Err(P2pError::InvalidInput("Share code cannot be empty".into()));
    }

    if share_code.len() > MAX_SHARE_CODE_LENGTH {
        return Err(P2pError::InvalidInput(format!(
            "Share code too long (max {} characters)",
            MAX_SHARE_CODE_LENGTH
        )));
    }

    // Only allow alphanumeric and some special characters
    if !share_code
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(P2pError::InvalidInput(
            "Share code contains invalid characters".into(),
        ));
    }

    Ok(())
}

/// Validate a URI
///
/// Ensures URI format is valid and safe.
///
/// # Arguments
/// * `uri` - The URI to validate
///
/// # Returns
/// Ok if valid, Err(P2pError) if invalid
pub fn validate_uri(uri: &str) -> Result<(), P2pError> {
    if uri.is_empty() {
        return Err(P2pError::InvalidInput("URI cannot be empty".into()));
    }

    if uri.len() > MAX_URI_LENGTH {
        return Err(P2pError::InvalidInput(format!(
            "URI too long (max {} characters)",
            MAX_URI_LENGTH
        )));
    }

    // Check for dangerous URI schemes
    let dangerous_schemes = ["file:///", "javascript:", "vbscript:", "data:"];
    let lower = uri.to_lowercase();
    for scheme in &dangerous_schemes {
        if lower.starts_with(scheme) {
            return Err(P2pError::InvalidInput(format!(
                "URI with dangerous scheme not allowed: {}",
                scheme
            )));
        }
    }

    Ok(())
}

/// Validate a peer ID string
///
/// Ensures peer ID format is valid.
///
/// # Arguments
/// * `peer_id_str` - The peer ID string to validate
///
/// # Returns
/// Ok(PeerId) if valid, Err(P2pError) if invalid
pub fn validate_peer_id(peer_id_str: &str) -> Result<PeerId, P2pError> {
    if peer_id_str.len() > 256 {
        return Err(P2pError::InvalidInput("Peer ID too long".into()));
    }

    // Try to parse as libp2p PeerId
    peer_id_str
        .parse::<PeerId>()
        .map_err(|_| P2pError::InvalidInput("Invalid peer ID format".into()))
}

/// Validate a file path
///
/// Ensures file path is safe and doesn't attempt directory traversal.
///
/// # Arguments
/// * `path` - The file path to validate
///
/// # Returns
/// Ok if valid, Err(P2pError) if invalid
pub fn validate_file_path(path: &Path) -> Result<(), P2pError> {
    let path_str = path.to_string_lossy();

    // Check for path traversal attempts
    if path_str.contains("..") || path_str.contains("~") {
        return Err(P2pError::InvalidInput("Path traversal not allowed".into()));
    }

    // Check for absolute paths (only relative paths allowed)
    if path.is_absolute() {
        return Err(P2pError::InvalidInput("Absolute paths not allowed".into()));
    }

    // Check for suspicious patterns
    if path_str.contains("etc/") || path_str.contains("proc/") {
        return Err(P2pError::InvalidInput(
            "Access to system directories not allowed".into(),
        ));
    }

    Ok(())
}

/// Sanitize a string for safe display
///
/// Removes or escapes potentially dangerous characters.
///
/// # Arguments
/// * `input` - The string to sanitize
///
/// # Returns
/// Sanitized string
pub fn sanitize_string(input: &str) -> String {
    input
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('&', "&amp;")
}
