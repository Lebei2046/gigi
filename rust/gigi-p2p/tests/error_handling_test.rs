//! Error handling tests for gigi-p2p
//!
//! Tests all error variants and error handling scenarios

use gigi_p2p::P2pError;
use libp2p::PeerId;
use std::path::PathBuf;

#[test]
fn test_peer_not_found_error() {
    let peer_id = PeerId::random();
    let error = P2pError::PeerNotFound(peer_id);

    assert!(matches!(error, P2pError::PeerNotFound(_)));
    assert!(error.to_string().contains("Peer not found"));
    assert!(error.to_string().contains(&peer_id.to_string()));
}

#[test]
fn test_nickname_not_found_error() {
    let nickname = "Alice".to_string();
    let error = P2pError::NicknameNotFound(nickname.clone());

    assert!(matches!(error, P2pError::NicknameNotFound(_)));
    assert!(error.to_string().contains("Nickname not found"));
    assert!(error.to_string().contains("Alice"));
}

#[test]
fn test_group_not_found_error() {
    let group_name = "test-group".to_string();
    let error = P2pError::GroupNotFound(group_name.clone());

    assert!(matches!(error, P2pError::GroupNotFound(_)));
    assert!(error.to_string().contains("Group not found"));
    assert!(error.to_string().contains("test-group"));
}

#[test]
fn test_file_not_found_error() {
    let path = PathBuf::from("/nonexistent/file.txt");
    let error = P2pError::FileNotFound(path.clone());

    assert!(matches!(error, P2pError::FileNotFound(_)));
    assert!(error.to_string().contains("File not found"));
    assert!(error.to_string().contains("/nonexistent/file.txt"));
}

#[test]
fn test_invalid_share_code_error() {
    let share_code = "invalid-share-code".to_string();
    let error = P2pError::InvalidShareCode(share_code.clone());

    assert!(matches!(error, P2pError::InvalidShareCode(_)));
    assert!(error.to_string().contains("Share code invalid"));
    assert!(error.to_string().contains("invalid-share-code"));
}

#[test]
fn test_invalid_uri_error() {
    let uri = "content://invalid/path".to_string();
    let error = P2pError::InvalidUri(uri.clone());

    assert!(matches!(error, P2pError::InvalidUri(_)));
    assert!(error.to_string().contains("Invalid URI"));
    assert!(error.to_string().contains("content://invalid/path"));
}

#[test]
fn test_network_error() {
    let msg = "Connection timeout".to_string();
    let error = P2pError::NetworkError(msg.clone());

    assert!(matches!(error, P2pError::NetworkError(_)));
    assert!(error.to_string().contains("Network error"));
    assert!(error.to_string().contains("Connection timeout"));
}

#[test]
fn test_io_error() {
    use std::io;
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let error = P2pError::IoError(io_error);

    assert!(matches!(error, P2pError::IoError(_)));
    assert!(error.to_string().contains("IO error"));
}

#[test]
fn test_serialization_error() {
    use serde_json;
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();

    let error = P2pError::SerializationError(json_error);

    assert!(matches!(error, P2pError::SerializationError(_)));
    assert!(error.to_string().contains("Serialization error"));
}

#[test]
fn test_timeout_error() {
    let msg = "Operation timed out after 30s".to_string();
    let error = P2pError::Timeout(msg.clone());

    assert!(matches!(error, P2pError::Timeout(_)));
    assert!(error.to_string().contains("Timeout"));
    assert!(error.to_string().contains("Operation timed out after 30s"));
}

#[test]
fn test_message_send_error() {
    let msg = "Peer not connected".to_string();
    let error = P2pError::MessageSendError(msg.clone());

    assert!(matches!(error, P2pError::MessageSendError(_)));
    assert!(error.to_string().contains("Message send error"));
    assert!(error.to_string().contains("Peer not connected"));
}

#[test]
fn test_persistence_not_enabled_error() {
    let error = P2pError::PersistenceNotEnabled;

    assert!(matches!(error, P2pError::PersistenceNotEnabled));
    assert!(error
        .to_string()
        .contains("Message persistence is not enabled"));
}

#[test]
fn test_error_debug_format() {
    let peer_id = PeerId::random();
    let error = P2pError::PeerNotFound(peer_id);

    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("PeerNotFound"));
}

#[test]
fn test_multiple_error_variants() {
    let errors = vec![
        P2pError::PeerNotFound(PeerId::random()),
        P2pError::NicknameNotFound("Bob".to_string()),
        P2pError::GroupNotFound("group-1".to_string()),
        P2pError::FileNotFound(PathBuf::from("/tmp/test.txt")),
        P2pError::InvalidShareCode("invalid".to_string()),
        P2pError::InvalidUri("file://test".to_string()),
        P2pError::NetworkError("network fail".to_string()),
        P2pError::Timeout("timeout".to_string()),
        P2pError::MessageSendError("send fail".to_string()),
        P2pError::PersistenceNotEnabled,
    ];

    assert_eq!(errors.len(), 10);

    // Verify each error has a string representation
    for error in errors {
        let error_str = error.to_string();
        assert!(!error_str.is_empty());
    }
}
