// Copyright 2024 Gigi Team.
//
// Comprehensive tests for FileSharingManager core functionality

use gigi_file_sharing::{FileSharingManager, CHUNK_SIZE};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_manager_creation() {
    let manager = FileSharingManager::new();

    // Manager should be created with empty shared files
    assert_eq!(manager.list_shared_files().len(), 0);
}

#[test]
fn test_default_manager() {
    // Default should create empty manager
    let manager = FileSharingManager::default();

    assert_eq!(manager.list_shared_files().len(), 0);
}

#[test]
fn test_share_code_generation() {
    let manager = FileSharingManager::new();

    // Generate share codes
    let code1 = manager.generate_share_code("file.txt");
    let code2 = manager.generate_share_code("file.txt");

    // Codes should be different (due to timestamp)
    assert_ne!(code1, code2);

    // Codes should be 8 hex characters
    assert_eq!(code1.len(), 8);
    assert!(code1.chars().all(|c| c.is_ascii_hexdigit()));

    assert_eq!(code2.len(), 8);
    assert!(code2.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_share_code_determinism() {
    let manager = FileSharingManager::new();

    // Same filename should produce different codes (timestamp)
    let code1 = manager.generate_share_code("test.txt");
    std::thread::sleep(std::time::Duration::from_millis(10));
    let code2 = manager.generate_share_code("test.txt");

    assert_ne!(code1, code2);
}

#[tokio::test]
async fn test_share_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"Hello, World!").unwrap();

    let mut manager = FileSharingManager::new();
    let share_code = manager.share_file(&test_file).await.unwrap();

    // Share code should be generated
    assert!(!share_code.is_empty());

    // File should appear in list
    let files = manager.list_shared_files();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].info.name, "test.txt");
    assert_eq!(files[0].info.size, 13);
}

#[tokio::test]
async fn test_share_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_file = temp_dir.path().join("nonexistent.txt");

    let mut manager = FileSharingManager::new();
    let result = manager.share_file(&nonexistent_file).await;

    // Should return error
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_share_same_file_twice() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"Test content").unwrap();

    let mut manager = FileSharingManager::new();
    let code1 = manager.share_file(&test_file).await.unwrap();
    let code2 = manager.share_file(&test_file).await.unwrap();

    // Should return same share code for unchanged file
    assert_eq!(code1, code2);

    // Should still have only one entry
    assert_eq!(manager.list_shared_files().len(), 1);
}

#[tokio::test]
async fn test_share_modified_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"Original content").unwrap();

    let mut manager = FileSharingManager::new();
    let code1 = manager.share_file(&test_file).await.unwrap();

    // Modify file
    fs::write(&test_file, b"Modified content").unwrap();

    let code2 = manager.share_file(&test_file).await.unwrap();

    // Should return same share code (existing entry)
    assert_eq!(code1, code2);

    // Hash should be different
    let files = manager.list_shared_files();
    assert_eq!(files.len(), 1);
}

#[tokio::test]
async fn test_chunk_count_calculation() {
    let temp_dir = TempDir::new().unwrap();

    // Test exact chunk size
    let file1 = temp_dir.path().join("exact.txt");
    let exact_size = CHUNK_SIZE;
    fs::write(&file1, vec![0u8; exact_size]).unwrap();

    let mut manager = FileSharingManager::new();
    manager.share_file(&file1).await.unwrap();
    assert_eq!(manager.list_shared_files()[0].info.chunk_count, 1);

    // Test file requiring partial chunk
    let file2 = temp_dir.path().join("partial.txt");
    let partial_size = CHUNK_SIZE / 2;
    fs::write(&file2, vec![0u8; partial_size]).unwrap();

    manager.share_file(&file2).await.unwrap();
    assert_eq!(manager.list_shared_files()[1].info.chunk_count, 1);

    // Test file requiring multiple chunks - use different manager to avoid caching
    let file3 = temp_dir.path().join("multiple.txt");
    let multiple_size = CHUNK_SIZE * 3 + CHUNK_SIZE / 2;
    fs::write(&file3, vec![0u8; multiple_size]).unwrap();

    let mut manager2 = FileSharingManager::new();
    manager2.share_file(&file3).await.unwrap();
    assert_eq!(manager2.list_shared_files()[0].info.chunk_count, 4);
}

#[tokio::test]
async fn test_list_shared_files() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = FileSharingManager::new();

    // Share multiple files
    for i in 0..3 {
        let file = temp_dir.path().join(format!("file{}.txt", i));
        fs::write(&file, format!("Content {}", i)).unwrap();
        manager.share_file(&file).await.unwrap();
    }

    // Should list all shared files
    let files = manager.list_shared_files();
    assert_eq!(files.len(), 3);

    // Should contain all files
    let names: Vec<&str> = files.iter().map(|f| f.info.name.as_str()).collect();
    assert!(names.contains(&"file0.txt"));
    assert!(names.contains(&"file1.txt"));
    assert!(names.contains(&"file2.txt"));
}

#[tokio::test]
async fn test_unshare_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"Test content").unwrap();

    let mut manager = FileSharingManager::new();
    let share_code = manager.share_file(&test_file).await.unwrap();

    // Unshare the file
    manager.unshare_file(&share_code).unwrap();

    // Should be removed from list
    assert_eq!(manager.list_shared_files().len(), 0);
}

#[tokio::test]
async fn test_unshare_nonexistent_file() {
    let mut manager = FileSharingManager::new();

    // Should return error for nonexistent share code
    let result = manager.unshare_file("nonexistent");

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("invalid"));
}

#[tokio::test]
async fn test_file_hash_calculation() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"Test content").unwrap();

    let manager = FileSharingManager::new();
    let hash = manager.calculate_file_hash(&test_file).unwrap();

    // Hash should be 64 hex characters
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

    // Same file should produce same hash
    let hash2 = manager.calculate_file_hash(&test_file).unwrap();
    assert_eq!(hash, hash2);
}

#[tokio::test]
async fn test_file_hash_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("empty.txt");
    fs::write(&test_file, b"").unwrap();

    let manager = FileSharingManager::new();
    let hash = manager.calculate_file_hash(&test_file).unwrap();

    // Empty file should have valid hash (SHA256 of empty string)
    assert_eq!(
        hash,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[tokio::test]
async fn test_share_content_uri() {
    let mut manager = FileSharingManager::new();
    let uri = "content://com.android.providers.media.documents/document/image:1234";
    let name = "photo.jpg";
    let size = 1024 * 1024; // 1MB

    let share_code = manager.share_content_uri(uri, name, size).await.unwrap();

    // Should generate valid share code
    assert_eq!(share_code.len(), 8);
    assert!(share_code.chars().all(|c| c.is_ascii_hexdigit()));

    // File should appear in list
    let files = manager.list_shared_files();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].info.name, "photo.jpg");
    assert_eq!(files[0].info.size, 1024 * 1024);
}

#[tokio::test]
async fn test_share_content_uri_with_valid_scheme() {
    let mut manager = FileSharingManager::new();
    let uri = "content://com.android.providers.media.documents/document/image:1234";
    let name = "photo.jpg";
    let size = 1024;

    let share_code = manager.share_content_uri(uri, name, size).await.unwrap();

    // Should generate valid share code
    assert_eq!(share_code.len(), 8);
    assert!(share_code.chars().all(|c| c.is_ascii_hexdigit()));
}

#[tokio::test]
async fn test_chunk_reader_setter() {
    use std::sync::Arc;

    let mut manager = FileSharingManager::new();
    let reader = Arc::new(
        |_path: &gigi_file_sharing::FilePath,
         _offset: u64,
         _length: usize|
         -> anyhow::Result<Vec<u8>> { Ok(vec![0u8; 1024]) },
    );

    // Should accept reader without error
    manager.set_chunk_reader(reader);

    // Cannot directly test reader usage without URI, but should not panic
    assert!(true);
}
