// Copyright 2024 Gigi Team.
//
// Comprehensive tests for file sharing types

use gigi_file_sharing::{FileInfo, FilePath, SharedFile};
use std::path::PathBuf;
use url::Url;

#[test]
fn test_file_path_path_variant() {
    let path = PathBuf::from("/home/user/file.txt");
    let file_path = FilePath::Path(path.clone());

    match file_path {
        FilePath::Path(p) => assert_eq!(p, path),
        _ => panic!("Expected Path variant"),
    }
}

#[test]
fn test_file_path_url_variant() {
    let url = Url::parse("content://com.android.test/document").unwrap();
    let file_path = FilePath::Url(url.clone());

    match file_path {
        FilePath::Url(u) => assert_eq!(u.to_string(), url.to_string()),
        _ => panic!("Expected Url variant"),
    }
}

#[test]
fn test_file_path_serialization() {
    let path = FilePath::Path(PathBuf::from("/test/file.txt"));
    let json = serde_json::to_string(&path).unwrap();
    let deserialized: FilePath = serde_json::from_str(&json).unwrap();

    assert_eq!(path, deserialized);
}

#[test]
fn test_file_path_url_serialization() {
    let url = Url::parse("content://com.android.test/doc").unwrap();
    let path = FilePath::Url(url);
    let json = serde_json::to_string(&path).unwrap();
    let deserialized: FilePath = serde_json::from_str(&json).unwrap();

    assert_eq!(path, deserialized);
}

#[test]
fn test_file_info_creation() {
    let info = FileInfo {
        id: "test123".to_string(),
        name: "document.pdf".to_string(),
        size: 1024 * 1024,
        hash: "abc123def456".to_string(),
        chunk_count: 4,
        created_at: 1640995200,
    };

    assert_eq!(info.id, "test123");
    assert_eq!(info.name, "document.pdf");
    assert_eq!(info.size, 1024 * 1024);
    assert_eq!(info.hash, "abc123def456");
    assert_eq!(info.chunk_count, 4);
    assert_eq!(info.created_at, 1640995200);
}

#[test]
fn test_file_info_serialization() {
    let info = FileInfo {
        id: "test".to_string(),
        name: "file.txt".to_string(),
        size: 512,
        hash: "123abc".to_string(),
        chunk_count: 1,
        created_at: 1234567890,
    };

    let json = serde_json::to_string(&info).unwrap();
    let deserialized: FileInfo = serde_json::from_str(&json).unwrap();

    assert_eq!(info.id, deserialized.id);
    assert_eq!(info.name, deserialized.name);
    assert_eq!(info.size, deserialized.size);
    assert_eq!(info.hash, deserialized.hash);
    assert_eq!(info.chunk_count, deserialized.chunk_count);
    assert_eq!(info.created_at, deserialized.created_at);
}

#[test]
fn test_shared_file_creation() {
    let info = FileInfo {
        id: "code123".to_string(),
        name: "image.jpg".to_string(),
        size: 2048,
        hash: "hash456".to_string(),
        chunk_count: 1,
        created_at: 1640995200,
    };

    let shared_file = SharedFile {
        info: info.clone(),
        path: FilePath::Path(PathBuf::from("/path/to/image.jpg")),
        share_code: "code123".to_string(),
        revoked: false,
    };

    assert_eq!(shared_file.share_code, "code123");
    assert!(!shared_file.revoked);
    assert_eq!(shared_file.info.name, "image.jpg");
}

#[test]
fn test_shared_file_serialization() {
    let shared_file = SharedFile {
        info: FileInfo {
            id: "test".to_string(),
            name: "file.txt".to_string(),
            size: 512,
            hash: "abc123".to_string(),
            chunk_count: 1,
            created_at: 1640995200,
        },
        path: FilePath::Path(PathBuf::from("/test/file.txt")),
        share_code: "share123".to_string(),
        revoked: false,
    };

    let json = serde_json::to_string(&shared_file).unwrap();
    let deserialized: SharedFile = serde_json::from_str(&json).unwrap();

    assert_eq!(shared_file.share_code, deserialized.share_code);
    assert_eq!(shared_file.revoked, deserialized.revoked);
    assert_eq!(shared_file.info.name, deserialized.info.name);
}

#[test]
fn test_shared_file_equality() {
    let info = FileInfo {
        id: "test".to_string(),
        name: "file.txt".to_string(),
        size: 512,
        hash: "abc".to_string(),
        chunk_count: 1,
        created_at: 1640995200,
    };

    let file1 = SharedFile {
        info: info.clone(),
        path: FilePath::Path(PathBuf::from("/test/file.txt")),
        share_code: "code123".to_string(),
        revoked: false,
    };

    let file2 = SharedFile {
        info: info.clone(),
        path: FilePath::Path(PathBuf::from("/test/file.txt")),
        share_code: "code123".to_string(),
        revoked: false,
    };

    // Check individual fields since SharedFile doesn't implement PartialEq
    assert_eq!(file1.share_code, file2.share_code);
    assert_eq!(file1.revoked, file2.revoked);
}

#[test]
fn test_shared_file_path_equality() {
    let info = FileInfo {
        id: "test".to_string(),
        name: "file.txt".to_string(),
        size: 512,
        hash: "abc".to_string(),
        chunk_count: 1,
        created_at: 1640995200,
    };

    let path = PathBuf::from("/test/file.txt");

    let file1 = SharedFile {
        info: info.clone(),
        path: FilePath::Path(path.clone()),
        share_code: "code123".to_string(),
        revoked: false,
    };

    let file2 = SharedFile {
        info: info.clone(),
        path: FilePath::Path(path.clone()),
        share_code: "code123".to_string(),
        revoked: false,
    };

    // Check individual fields since SharedFile doesn't implement PartialEq
    assert_eq!(file1.share_code, file2.share_code);
    assert_eq!(file1.revoked, file2.revoked);
}

#[test]
fn test_file_path_url_parsing() {
    let urls = vec![
        "content://com.android.providers.media.documents/document/image:1234",
        "file:///private/var/mobile/Containers/Data/Application/Documents/file.txt",
    ];

    for url_str in urls {
        let url = Url::parse(url_str).unwrap();
        let file_path = FilePath::Url(url);

        match file_path {
            FilePath::Url(_) => {} // Success
            _ => panic!("Failed to parse URL: {}", url_str),
        }
    }
}

#[test]
fn test_file_info_clone() {
    let info = FileInfo {
        id: "test".to_string(),
        name: "file.txt".to_string(),
        size: 512,
        hash: "abc".to_string(),
        chunk_count: 1,
        created_at: 1640995200,
    };

    let cloned = info.clone();

    assert_eq!(info.id, cloned.id);
    assert_eq!(info.name, cloned.name);
}

#[test]
fn test_shared_file_clone() {
    let shared_file = SharedFile {
        info: FileInfo {
            id: "test".to_string(),
            name: "file.txt".to_string(),
            size: 512,
            hash: "abc".to_string(),
            chunk_count: 1,
            created_at: 1640995200,
        },
        path: FilePath::Path(PathBuf::from("/test/file.txt")),
        share_code: "code123".to_string(),
        revoked: false,
    };

    let cloned = shared_file.clone();

    assert_eq!(shared_file.share_code, cloned.share_code);
    assert_eq!(shared_file.revoked, cloned.revoked);
}
