// File utility functions for handling images, content URIs, and file operations

use base64::Engine;
use std::path::PathBuf;
use tracing::info;

/// Helper function to check if file is an image based on extension
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

/// Helper function to check if data is an image using magic bytes
pub fn is_image_data(data: &[u8]) -> bool {
    data.len() >= 4
        && (
            data.starts_with(&[0xFF, 0xD8, 0xFF]) || // JPEG
            data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) || // PNG
            data.starts_with(&[0x47, 0x49, 0x46]) || // GIF
            data.starts_with(b"WEBP") || // WEBP
            data.starts_with(b"BM")
            // BMP
        )
}

/// Helper function to convert file data to base64 (only for images)
pub fn convert_to_base64_if_image(data: &[u8]) -> Option<String> {
    if is_image_data(data) {
        let encoded = base64::engine::general_purpose::STANDARD.encode(data);
        info!(
            "✅ Successfully converted image to base64, size: {} bytes",
            data.len()
        );
        Some(encoded)
    } else {
        info!("⚠️ File is not an image based on magic bytes, skipping base64 conversion");
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

    /// Helper function to read content URI on Android
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

    /// Extract filename from content URI
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

    /// Helper function to save content URI to app directory
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

        info!("📝 Saving content URI to: {:?}", save_path);
        // Save image data to file
        fs::write(&save_path, image_data).map_err(|e| format!("Failed to save file: {}", e))?;

        info!("✅ Saved {} bytes to disk", image_data.len());
        Ok(save_path)
    }
}
