//! File utility functions for handling images, content URIs, and file operations.
//!
//! This module provides helper functions for:
//! - Detecting and validating image files
//! - Converting image data to base64 encoding
//! - Handling Android content URIs for file access

use base64::Engine;
use std::path::PathBuf;
use tracing::info;

/// Checks if a file is an image based on its file extension.
///
/// This function examines the file extension and returns `true` if it matches
/// a known image format (jpg, jpeg, png, gif, webp, bmp).
///
/// # Arguments
///
/// * `file_path` - Path to the file to check
///
/// # Returns
///
/// `true` if the file has an image extension, `false` otherwise
///
/// # Example
///
/// ```
/// # use tauri_plugin_gigi::file_utils::is_image_file;
/// assert!(is_image_file("photo.jpg"));
/// assert!(is_image_file("image.png"));
/// assert!(!is_image_file("document.pdf"));
/// ```
pub fn is_image_file(file_path: &str) -> bool {
    let path = PathBuf::from(file_path);
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_lowercase().as_str(),
                "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp"
            )
        })
        .unwrap_or(false)
}

/// Checks if binary data represents an image using magic bytes.
///
/// This function examines the first few bytes of the data to determine if it
/// matches the magic byte signatures of common image formats. This is more
/// reliable than checking file extensions alone.
///
/// # Supported Formats
///
/// - JPEG: `0xFF 0xD8 0xFF`
/// - PNG: `0x89 0x50 0x4E 0x47`
/// - GIF: `0x47 0x49 0x46`
/// - WEBP: `WEBP` (at bytes 8-11)
/// - BMP: `0x42 0x4D` ("BM")
///
/// # Arguments
///
/// * `data` - The binary data to check
///
/// # Returns
///
/// `true` if the data starts with a known image signature, `false` otherwise
///
/// # Example
///
/// ```
/// # use tauri_plugin_gigi::file_utils::is_image_data;
/// let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
/// assert!(is_image_data(&jpeg_data));
/// ```
pub fn is_image_data(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }

    // Check for image signatures
    data.starts_with(&[0xFF, 0xD8, 0xFF]) || // JPEG
        data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) || // PNG
        data.starts_with(&[0x47, 0x49, 0x46]) || // GIF
        (data.len() >= 12 && &data[8..12] == b"WEBP") || // WEBP (at bytes 8-11)
        data.starts_with(b"BM") // BMP
}

/// Converts image data to base64 encoding if it represents an image.
///
/// This function first checks if the data is an image using magic bytes, then
/// converts it to a base64 string if it is. Returns `None` if the data is not
/// recognized as an image.
///
/// # Arguments
///
/// * `data` - The binary data to potentially convert
///
/// # Returns
///
/// - `Some(String)` - Base64 encoded string if data is an image
/// - `None` - If data is not an image
///
/// # Example
///
/// ```
/// # use tauri_plugin_gigi::file_utils::convert_to_base64_if_image;
/// let image_data = vec![0xFF, 0xD8, 0xFF, 0xE0];
/// let base64 = convert_to_base64_if_image(&image_data);
/// assert!(base64.is_some());
/// ```
pub fn convert_to_base64_if_image(data: &[u8]) -> Option<String> {
    if is_image_data(data) {
        let encoded = base64::engine::general_purpose::STANDARD.encode(data);
        info!(
            "Successfully converted image to base64, size: {} bytes",
            data.len()
        );
        Some(encoded)
    } else {
        info!("File is not an image based on magic bytes, skipping base64 conversion");
        None
    }
}

// Android-specific content URI handling
#[cfg(target_os = "android")]
pub mod android {
    use percent_encoding::percent_decode_str;
    use std::path::PathBuf;
    use tauri::AppHandle;
    use tauri_plugin_android_fs::{AndroidFsExt, FileAccessMode, FileUri};
    use tauri_plugin_fs::FilePath;

    /// Reads a content URI on Android.
    ///
    /// Android uses content URIs (content://) to access files from other apps.
    /// This function handles the conversion from URI string to actual file data.
    ///
    /// # Arguments
    ///
    /// * `app` - Reference to the Tauri AppHandle
    /// * `uri_str` - The content URI string to read
    ///
    /// # Returns
    ///
    /// A `Result` containing the file data as a byte vector or an error message
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let uri = "content://com.android.externalstorage.documents/document/primary:photo.jpg";
    /// let data = read_content_uri(&app_handle, uri)?;
    /// ```
    pub fn read_content_uri(app: &AppHandle, uri_str: &str) -> Result<Vec<u8>, String> {
        let android_api = app.android_fs();

        // Convert URI string to FileUri via FilePath::Url
        let url = tauri::Url::parse(uri_str)
            .map_err(|e| format!("Failed to parse content URI: {}", e))?;
        let file_uri = FileUri::from(FilePath::Url(url));

        // Open the file and read its contents
        match android_api.open_file(&file_uri, FileAccessMode::Read) {
            Ok(mut file) => {
                let mut buffer = Vec::new();
                use std::io::Read;
                file.read_to_end(&mut buffer)
                    .map_err(|e| format!("Failed to read content URI data: {}", e))?;
                Ok(buffer)
            }
            Err(e) => Err(format!("Failed to open content URI: {}", e)),
        }
    }

    /// Extracts a filename from a content URI.
    ///
    /// Android content URIs can contain encoded filenames in various formats.
    /// This function attempts to extract a meaningful filename from the URI.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The content URI string
    /// * `prefix` - A prefix to use if the filename cannot be extracted
    ///
    /// # Returns
    ///
    /// The extracted filename, or a generated name if extraction fails
    ///
    /// # Behavior
    ///
    /// 1. Attempts to extract the display name from the URI (after `=` sign)
    /// 2. Falls back to extracting from the URI path
    /// 3. As a last resort, generates a hash-based filename using the prefix
    pub fn extract_filename_from_uri(file_path: &str, prefix: &str) -> String {
        if let Some(display_name) = file_path.split('=').last().and_then(|s| {
            let decoded = percent_decode_str(s).decode_utf8().ok()?;
            let name = decoded.split('/').last()?.to_string();
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        }) {
            display_name
        } else {
            // Fallback: extract from URI path
            file_path
                .split('/')
                .last()
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    // Final fallback: use hash
                    let uri_hash = blake3::hash(file_path.as_bytes()).to_hex();
                    format!("{}_{}", prefix, &uri_hash[..16])
                })
        }
    }

    /// Saves content URI data to the app directory.
    ///
    /// This function reads data from a content URI, detects the file type,
    /// generates an appropriate filename, and saves it to the download directory.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The content URI string
    /// * `image_data` - The binary data to save
    /// * `download_dir` - The directory to save the file in
    /// * `prefix` - A prefix for the filename if needed
    ///
    /// # Returns
    ///
    /// A `Result` containing the full path to the saved file or an error message
    ///
    /// # File Type Detection
    ///
    /// The function examines the binary data to determine the appropriate file
    /// extension:
    /// - JPEG: `0xFF 0xD8 0xFF` → `.jpg`
    /// - PNG: `0x89 0x50 0x4E 0x47` → `.png`
    /// - GIF: `0x47 0x49 0x46` → `.gif`
    /// - WEBP: `WEBP` or `RIFF` → `.webp`
    /// - BMP: `0x42 0x4D` → `.bmp`
    /// - Video signatures: `.mp4`
    /// - Otherwise: Uses filename extension or `.dat` as fallback
    pub fn save_content_uri_to_app_dir(
        file_path: &str,
        image_data: &[u8],
        download_dir: &PathBuf,
        prefix: &str,
    ) -> Result<PathBuf, String> {
        use std::fs;

        let filename_from_uri = extract_filename_from_uri(file_path, prefix);

        // Detect file type from data and get appropriate extension
        let file_ext = if image_data.len() >= 8 {
            // Check common signatures
            if image_data.starts_with(&[0xFF, 0xD8, 0xFF]) {
                ".jpg"
            } else if image_data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                ".png"
            } else if image_data.starts_with(&[0x47, 0x49, 0x46]) {
                ".gif"
            } else if image_data.starts_with(b"WEBP")
                || image_data.starts_with(&[0x52, 0x49, 0x46, 0x46])
            {
                ".webp"
            } else if image_data.starts_with(b"BM") {
                ".bmp"
            } else if image_data.starts_with(&[0x00, 0x00, 0x00])
                || image_data.starts_with(&[0x1A, 0x45, 0xDF, 0xA3])
            {
                // Video signatures: MP4/AVI or WebM
                ".mp4"
            } else {
                // If filename already has extension, use it; otherwise use .dat
                if filename_from_uri.contains('.') {
                    ""
                } else {
                    ".dat"
                }
            }
        } else {
            // For small files, trust the filename extension
            ""
        };

        // Combine filename and extension
        let filename = if !file_ext.is_empty() && !filename_from_uri.contains('.') {
            format!("{}{}", filename_from_uri, file_ext)
        } else {
            filename_from_uri
        };

        let save_path = download_dir.join(&filename);

        info!("Saving content URI to: {:?}", save_path);
        // Save image data to file
        fs::write(&save_path, image_data).map_err(|e| format!("Failed to save file: {}", e))?;

        info!("Saved {} bytes to disk", image_data.len());
        Ok(save_path)
    }
}
