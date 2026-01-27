// Copyright 2024 Gigi Team.
//
// Comprehensive tests for file sharing error types

use gigi_file_sharing::FileSharingError;
use std::path::PathBuf;

#[test]
fn test_error_display_file_not_found() {
    let path = PathBuf::from("/nonexistent/file.txt");
    let error = FileSharingError::FileNotFound(path.clone());

    let error_string = format!("{}", error);

    assert!(error_string.contains("File not found"));
    assert!(error_string.contains("/nonexistent/file.txt"));
}

#[test]
fn test_error_display_invalid_share_code() {
    let error = FileSharingError::InvalidShareCode("invalid123".to_string());
    let error_string = format!("{}", error);

    assert!(error_string.contains("Share code invalid"));
    assert!(error_string.contains("invalid123"));
}

#[test]
fn test_error_display_invalid_uri() {
    let error = FileSharingError::InvalidUri("bad-uri-format".to_string());
    let error_string = format!("{}", error);

    assert!(error_string.contains("Invalid URI"));
    assert!(error_string.contains("bad-uri-format"));
}

#[test]
fn test_error_display_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test file not found");
    let error = FileSharingError::IoError(io_err);
    let error_string = format!("{}", error);

    assert!(error_string.contains("IO error"));
}

#[test]
fn test_error_display_serialization_error() {
    // Create a serialization error through invalid JSON (missing closing brace)
    let json = r#"{"key": "value", "extra": "data""#;

    let ser_err = serde_json::from_str::<serde_json::Value>(json).unwrap_err();

    let error = FileSharingError::SerializationError(ser_err);
    let error_string = format!("{}", error);

    assert!(error_string.contains("Serialization error"));
}

#[test]
fn test_error_debug_file_not_found() {
    let path = PathBuf::from("/test/file.txt");
    let error = FileSharingError::FileNotFound(path);

    let debug_string = format!("{:?}", error);

    assert!(debug_string.contains("FileNotFound"));
}

#[test]
fn test_error_debug_invalid_share_code() {
    let error = FileSharingError::InvalidShareCode("abc123".to_string());

    let debug_string = format!("{:?}", error);

    assert!(debug_string.contains("InvalidShareCode"));
}

#[test]
fn test_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");

    let error: FileSharingError = io_err.into();

    match error {
        FileSharingError::IoError(_) => {} // Success
        _ => panic!("Expected IoError variant"),
    }
}

#[test]
fn test_error_from_serialization_error() {
    // Create a serialization error through JSON parsing
    let json = r#"{"key": "value", }"#; // Trailing comma
    let ser_err = serde_json::from_str::<serde_json::Value>(json).unwrap_err();

    let error: FileSharingError = ser_err.into();

    match error {
        FileSharingError::SerializationError(_) => {} // Success
        _ => panic!("Expected SerializationError variant"),
    }
}

#[test]
fn test_error_file_not_found_path() {
    let path = PathBuf::from("/specific/path.txt");
    let error = FileSharingError::FileNotFound(path.clone());

    match error {
        FileSharingError::FileNotFound(p) => assert_eq!(p, path),
        _ => panic!("Expected FileNotFound variant"),
    }
}

#[test]
fn test_error_invalid_share_code_string() {
    let code = "bad-code-123";
    let error = FileSharingError::InvalidShareCode(code.to_string());

    match error {
        FileSharingError::InvalidShareCode(c) => assert_eq!(c, code),
        _ => panic!("Expected InvalidShareCode variant"),
    }
}

#[test]
fn test_error_invalid_uri_message() {
    let uri = "not://a//valid//uri";
    let error = FileSharingError::InvalidUri(uri.to_string());

    match error {
        FileSharingError::InvalidUri(u) => assert_eq!(u, uri),
        _ => panic!("Expected InvalidUri variant"),
    }
}

#[test]
fn test_error_variants_are_distinct() {
    let file_path = PathBuf::from("/test.txt");
    let error1 = FileSharingError::FileNotFound(file_path);
    let error2 = FileSharingError::InvalidShareCode("code".to_string());

    assert_ne!(format!("{:?}", error1), format!("{:?}", error2));
}

#[test]
fn test_error_send_sync() {
    // FileSharingError should be Send
    fn is_send<T: Send>() {}
    is_send::<FileSharingError>();

    // FileSharingError should be Sync
    fn is_sync<T: Sync>() {}
    is_sync::<FileSharingError>();
}
