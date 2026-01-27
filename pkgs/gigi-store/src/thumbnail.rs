use anyhow::Result;
use std::path::PathBuf;
use uuid::Uuid;

/// Generate thumbnail from image file
///
/// This function loads an image, resizes it to fit within the specified
/// maximum dimensions while preserving aspect ratio, and saves it as a JPEG.
///
/// # Arguments
/// * `file_path` - Path to the source image file
/// * `thumbnail_dir` - Directory where thumbnails will be saved
/// * `max_size` - Maximum (width, height) for the thumbnail
/// * `_quality` - JPEG quality parameter (1-100, currently reserved for future use)
///
/// # Returns
/// - `Ok(String)` - Filename of the generated thumbnail
/// - `Err(anyhow::Error)` - If image loading or saving fails
///
/// # Notes
/// - Image loading is a blocking operation, so it runs in a separate blocking task
/// - The thumbnail is always saved as JPEG format
/// - A unique filename is generated using UUID v4
/// - Thumbnail dimensions will not exceed `max_size` but may be smaller to maintain aspect ratio
pub async fn generate_thumbnail(
    file_path: &PathBuf,
    thumbnail_dir: &PathBuf,
    max_size: (u32, u32), // (width, height)
    _quality: u8,         // 1-100 (reserved for future use when quality parameter is supported)
) -> Result<String> {
    use tokio::task;

    // Load image (blocking operation, so we spawn a blocking task)
    // This prevents blocking the async runtime
    let img = task::spawn_blocking({
        let file_path = file_path.clone();
        move || image::open(&file_path).map_err(|e| anyhow::anyhow!("Failed to open image: {}", e))
    })
    .await
    .map_err(|e| anyhow::anyhow!("Thumbnail generation task failed: {}", e))??;

    // Resize maintaining aspect ratio
    // The thumbnail() method fits the image within the specified bounds
    // while preserving the original aspect ratio
    let thumbnail = img.thumbnail(max_size.0, max_size.1);

    // Generate unique thumbnail filename
    // Using UUID ensures no collisions even for the same source image
    let thumb_filename = format!("thumb_{}.jpg", Uuid::new_v4());
    let thumb_path = thumbnail_dir.join(&thumb_filename);

    // Save as JPEG (blocking operation)
    // Format is determined by file extension (.jpg)
    // Quality parameter is currently reserved for future use
    task::spawn_blocking({
        let thumb_path = thumb_path.clone();
        move || {
            thumbnail
                .into_rgb8()
                .save(&thumb_path)
                .map_err(|e| anyhow::anyhow!("Failed to save thumbnail: {}", e))
        }
    })
    .await
    .map_err(|e| anyhow::anyhow!("Thumbnail save task failed: {}", e))??;

    Ok(thumb_filename)
}

/// Check if a file is an image based on extension
///
/// # Supported Image Formats
/// - JPEG (.jpg, .jpeg)
/// - PNG (.png)
/// - GIF (.gif)
/// - WebP (.webp)
/// - Bitmap (.bmp)
/// - TIFF (.tiff)
/// - Icon (.ico)
///
/// # Arguments
/// * `file_path` - Path to file to check
///
/// # Returns
/// `true` if the file extension indicates an image, `false` otherwise
///
/// # Notes
/// - Check is case-insensitive
/// - Only checks file extension, does not validate file contents
pub fn is_image_file(file_path: &PathBuf) -> bool {
    if let Some(ext) = file_path.extension() {
        match ext.to_str().map(|s| s.to_lowercase()).as_deref() {
            Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("webp") | Some("bmp")
            | Some("tiff") | Some("ico") => true,
            _ => false,
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_thumbnail_generation() {
        let temp_dir = TempDir::new().unwrap();
        let thumbnail_dir = temp_dir.path().join("thumbnails");
        std::fs::create_dir_all(&thumbnail_dir).unwrap();

        // Create a test image (10x10 red square)
        use image::{self, DynamicImage, RgbImage};
        let img_data = vec![255u8; 10 * 10 * 3];
        let img = DynamicImage::ImageRgb8(RgbImage::from_raw(10, 10, img_data).unwrap());
        let test_image_path = temp_dir.path().join("test.jpg");
        img.into_rgb8().save(&test_image_path).unwrap();

        let thumb_filename = generate_thumbnail(&test_image_path, &thumbnail_dir, (200, 200), 70)
            .await
            .unwrap();
        assert!(thumb_filename.starts_with("thumb_"));
        assert!(thumbnail_dir.join(&thumb_filename).exists());
    }

    #[test]
    fn test_is_image_file() {
        assert!(is_image_file(&PathBuf::from("test.jpg")));
        assert!(is_image_file(&PathBuf::from("test.png")));
        assert!(is_image_file(&PathBuf::from("test.JPG")));
        assert!(!is_image_file(&PathBuf::from("test.pdf")));
        assert!(!is_image_file(&PathBuf::from("test.txt")));
    }
}
