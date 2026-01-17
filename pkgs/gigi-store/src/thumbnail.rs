use anyhow::Result;
use std::path::PathBuf;
use uuid::Uuid;

/// Generate thumbnail from image file
pub async fn generate_thumbnail(
    file_path: &PathBuf,
    thumbnail_dir: &PathBuf,
    max_size: (u32, u32), // (width, height)
    _quality: u8,         // 1-100 (reserved for future use when quality parameter is supported)
) -> Result<String> {
    use tokio::task;

    // Load image (blocking operation, so we spawn a blocking task)
    let img = task::spawn_blocking({
        let file_path = file_path.clone();
        move || image::open(&file_path).map_err(|e| anyhow::anyhow!("Failed to open image: {}", e))
    })
    .await
    .map_err(|e| anyhow::anyhow!("Thumbnail generation task failed: {}", e))??;

    // Resize maintaining aspect ratio
    let thumbnail = img.thumbnail(max_size.0, max_size.1);

    // Generate unique thumbnail filename
    let thumb_filename = format!("thumb_{}.jpg", Uuid::new_v4());
    let thumb_path = thumbnail_dir.join(&thumb_filename);

    // Save as JPEG (blocking operation)
    // Format is determined by file extension (.jpg)
    task::spawn_blocking({
        let thumb_path = thumb_path.clone();
        move || {
            thumbnail
                .save(&thumb_path)
                .map_err(|e| anyhow::anyhow!("Failed to save thumbnail: {}", e))
        }
    })
    .await
    .map_err(|e| anyhow::anyhow!("Thumbnail save task failed: {}", e))??;

    Ok(thumb_filename)
}

/// Check if a file is an image based on extension
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
        let img = DynamicImage::ImageRgb8(
            image::RgbImage::from_raw(10, 10, vec![255u8; 10 * 10 * 3]).unwrap(),
        );
        let test_image_path = temp_dir.path().join("test.jpg");
        img.save(&test_image_path, image::ImageFormat::Jpeg)
            .unwrap();

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
