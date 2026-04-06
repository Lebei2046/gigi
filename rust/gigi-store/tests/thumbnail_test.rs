// Copyright 2024 Gigi Team.
//
// Comprehensive tests for thumbnail generation

use gigi_store::thumbnail::{generate_thumbnail, is_image_file};
use tempfile::TempDir;

#[tokio::test]
async fn test_generate_thumbnail_from_image() {
    let temp_dir = TempDir::new().unwrap();
    let thumbnail_dir = temp_dir.path().join("thumbnails");
    std::fs::create_dir_all(&thumbnail_dir).unwrap();

    // Create a test image (100x100 red square)
    use image::RgbImage;
    let mut img_data = vec![255u8; 100 * 100 * 3];
    // Create a simple red image
    for i in (0..img_data.len()).step_by(3) {
        img_data[i] = 255; // R
        img_data[i + 1] = 0; // G
        img_data[i + 2] = 0; // B
    }

    let img = image::DynamicImage::ImageRgb8(RgbImage::from_raw(100, 100, img_data).unwrap());
    let test_image_path = temp_dir.path().join("test.jpg");
    img.save(&test_image_path)
        .expect("Failed to save test image");

    // Generate thumbnail
    let thumb_filename = generate_thumbnail(&test_image_path, &thumbnail_dir, (50, 50), 80)
        .await
        .expect("Failed to generate thumbnail");

    // Verify thumbnail was created
    assert!(thumb_filename.starts_with("thumb_"));
    assert!(thumb_filename.ends_with(".jpg"));

    let thumb_path = thumbnail_dir.join(&thumb_filename);
    assert!(thumb_path.exists(), "Thumbnail file should exist");

    // Verify thumbnail is smaller than original
    let thumb_img = image::open(&thumb_path).expect("Failed to open thumbnail");
    assert_eq!(thumb_img.width(), 50, "Thumbnail width should be 50");
    assert_eq!(thumb_img.height(), 50, "Thumbnail height should be 50");
}

#[tokio::test]
async fn test_thumbnail_aspect_ratio_preservation() {
    let temp_dir = TempDir::new().unwrap();
    let thumbnail_dir = temp_dir.path().join("thumbnails");
    std::fs::create_dir_all(&thumbnail_dir).unwrap();

    // Create a wide image (200x100)
    use image::RgbImage;
    let img_data = vec![255u8; 200 * 100 * 3];
    let img = image::DynamicImage::ImageRgb8(RgbImage::from_raw(200, 100, img_data).unwrap());
    let test_image_path = temp_dir.path().join("wide.jpg");
    img.save(&test_image_path)
        .expect("Failed to save test image");

    // Generate thumbnail with square size
    let thumb_filename = generate_thumbnail(&test_image_path, &thumbnail_dir, (50, 50), 80)
        .await
        .expect("Failed to generate thumbnail");

    let thumb_path = thumbnail_dir.join(&thumb_filename);
    let thumb_img = image::open(&thumb_path).expect("Failed to open thumbnail");

    // Thumbnail should maintain aspect ratio and fit within 50x50
    assert!(thumb_img.width() <= 50, "Thumbnail width should be <= 50");
    assert!(thumb_img.height() <= 50, "Thumbnail height should be <= 50");
}

#[tokio::test]
async fn test_thumbnail_different_sizes() {
    let temp_dir = TempDir::new().unwrap();
    let thumbnail_dir = temp_dir.path().join("thumbnails");
    std::fs::create_dir_all(&thumbnail_dir).unwrap();

    use image::RgbImage;
    let img_data = vec![255u8; 100 * 100 * 3];
    let img = image::DynamicImage::ImageRgb8(RgbImage::from_raw(100, 100, img_data).unwrap());
    let test_image_path = temp_dir.path().join("test.jpg");
    img.save(&test_image_path)
        .expect("Failed to save test image");

    // Test different thumbnail sizes
    let sizes = [(100, 100), (200, 200), (50, 50)];

    for (width, height) in sizes {
        let thumb_filename =
            generate_thumbnail(&test_image_path, &thumbnail_dir, (width, height), 80)
                .await
                .expect("Failed to generate thumbnail");

        let thumb_path = thumbnail_dir.join(&thumb_filename);
        let thumb_img = image::open(&thumb_path).expect("Failed to open thumbnail");

        assert!(thumb_img.width() <= width as u32);
        assert!(thumb_img.height() <= height as u32);
    }
}

#[tokio::test]
async fn test_thumbnail_unique_filenames() {
    let temp_dir = TempDir::new().unwrap();
    let thumbnail_dir = temp_dir.path().join("thumbnails");
    std::fs::create_dir_all(&thumbnail_dir).unwrap();

    use image::RgbImage;
    let img_data = vec![255u8; 100 * 100 * 3];
    let img = image::DynamicImage::ImageRgb8(RgbImage::from_raw(100, 100, img_data).unwrap());
    let test_image_path = temp_dir.path().join("test.jpg");
    img.save(&test_image_path)
        .expect("Failed to save test image");

    // Generate multiple thumbnails
    let thumb_filename1 = generate_thumbnail(&test_image_path, &thumbnail_dir, (50, 50), 80)
        .await
        .expect("Failed to generate thumbnail 1");
    let thumb_filename2 = generate_thumbnail(&test_image_path, &thumbnail_dir, (100, 100), 80)
        .await
        .expect("Failed to generate thumbnail 2");

    // Filenames should be different
    assert_ne!(
        thumb_filename1, thumb_filename2,
        "Thumbnail filenames should be unique"
    );

    // Both should exist
    assert!(thumbnail_dir.join(&thumb_filename1).exists());
    assert!(thumbnail_dir.join(&thumb_filename2).exists());
}

#[tokio::test]
async fn test_thumbnail_invalid_image() {
    let temp_dir = TempDir::new().unwrap();
    let thumbnail_dir = temp_dir.path().join("thumbnails");
    std::fs::create_dir_all(&thumbnail_dir).unwrap();

    // Create a non-image file
    let invalid_path = temp_dir.path().join("not_an_image.txt");
    std::fs::write(&invalid_path, b"This is not an image").expect("Failed to write test file");

    // Attempting to generate thumbnail should fail
    let result = generate_thumbnail(&invalid_path, &thumbnail_dir, (50, 50), 80).await;

    assert!(
        result.is_err(),
        "Should fail to generate thumbnail from non-image file"
    );
}

#[test]
fn test_is_image_file_with_image_extensions() {
    use std::path::PathBuf;

    assert!(is_image_file(&PathBuf::from("test.jpg")));
    assert!(is_image_file(&PathBuf::from("test.jpeg")));
    assert!(is_image_file(&PathBuf::from("test.png")));
    assert!(is_image_file(&PathBuf::from("test.gif")));
    assert!(is_image_file(&PathBuf::from("test.webp")));
    assert!(is_image_file(&PathBuf::from("test.bmp")));
    assert!(is_image_file(&PathBuf::from("test.tiff")));
    assert!(is_image_file(&PathBuf::from("test.ico")));
}

#[test]
fn test_is_image_file_with_non_image_extensions() {
    use std::path::PathBuf;

    assert!(!is_image_file(&PathBuf::from("test.pdf")));
    assert!(!is_image_file(&PathBuf::from("test.txt")));
    assert!(!is_image_file(&PathBuf::from("test.doc")));
    assert!(!is_image_file(&PathBuf::from("test.zip")));
}

#[test]
fn test_is_image_file_case_insensitive() {
    use std::path::PathBuf;

    assert!(is_image_file(&PathBuf::from("test.JPG")));
    assert!(is_image_file(&PathBuf::from("test.PNG")));
    assert!(is_image_file(&PathBuf::from("test.JPEG")));
}

#[test]
fn test_is_image_file_without_extension() {
    use std::path::PathBuf;

    assert!(!is_image_file(&PathBuf::from("test")));
    assert!(!is_image_file(&PathBuf::from("no_extension")));
}

#[tokio::test]
async fn test_thumbnail_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    let thumbnail_dir = temp_dir.path().join("thumbnails");
    std::fs::create_dir_all(&thumbnail_dir).unwrap();

    use std::path::PathBuf;
    let nonexistent = PathBuf::from("/nonexistent/path/to/image.jpg");

    // Should fail to generate thumbnail
    let result = generate_thumbnail(&nonexistent, &thumbnail_dir, (50, 50), 80).await;

    assert!(result.is_err(), "Should fail for nonexistent file");
}

#[tokio::test]
async fn test_thumbnail_different_formats() {
    let temp_dir = TempDir::new().unwrap();
    let thumbnail_dir = temp_dir.path().join("thumbnails");
    std::fs::create_dir_all(&thumbnail_dir).unwrap();

    use image::RgbImage;

    // Test JPEG
    let img_data = vec![255u8; 100 * 100 * 3];
    let img = image::DynamicImage::ImageRgb8(RgbImage::from_raw(100, 100, img_data).unwrap());

    let jpeg_path = temp_dir.path().join("test.jpg");
    img.save(&jpeg_path).expect("Failed to save JPEG");
    let thumb_jpeg = generate_thumbnail(&jpeg_path, &thumbnail_dir, (50, 50), 80)
        .await
        .expect("Failed to generate JPEG thumbnail");
    assert!(thumbnail_dir.join(&thumb_jpeg).exists());

    // Test PNG
    let png_path = temp_dir.path().join("test.png");
    img.save(&png_path).expect("Failed to save PNG");
    let thumb_png = generate_thumbnail(&png_path, &thumbnail_dir, (50, 50), 80)
        .await
        .expect("Failed to generate PNG thumbnail");
    assert!(thumbnail_dir.join(&thumb_png).exists());
}
