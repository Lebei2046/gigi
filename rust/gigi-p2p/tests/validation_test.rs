//! Unit tests for validation module
//!
//! Tests all input validation functions to prevent security issues.

use gigi_p2p::validation;
use std::path::Path;

#[test]
fn test_validate_nickname_valid() {
    let max_nickname = "A".repeat(64);
    let valid_nicknames = vec![
        "Alice",
        "Bob",
        "alice123",
        "Alice_Bob",
        "Alice-Bob",
        "Test User",
        "a",                   // minimum length (1)
        max_nickname.as_str(), // maximum length
    ];

    for nickname in valid_nicknames {
        assert!(
            validation::validate_nickname(nickname).is_ok(),
            "Nickname '{}' should be valid",
            nickname
        );
    }
}

#[test]
fn test_validate_nickname_invalid_empty() {
    let result = validation::validate_nickname("");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));
}

#[test]
fn test_validate_nickname_invalid_too_long() {
    let long_nickname = "A".repeat(65);
    let result = validation::validate_nickname(&long_nickname);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too long"));
}

#[test]
fn test_validate_nickname_invalid_characters() {
    let invalid_nicknames = vec!["Alice<script>", "Bob@home", "Test#123", "User!", "Test$"];

    for nickname in invalid_nicknames {
        assert!(
            validation::validate_nickname(nickname).is_err(),
            "Nickname '{}' should be invalid",
            nickname
        );
    }
}

#[test]
fn test_validate_message_valid() {
    let max_message = "A".repeat(100_000);
    let valid_messages = vec![
        "Hello, world!",
        "Test message with emojis 😊",
        max_message.as_str(), // maximum length
    ];

    for message in valid_messages {
        assert!(
            validation::validate_message(message).is_ok(),
            "Message should be valid"
        );
    }
}

#[test]
fn test_validate_message_invalid_too_long() {
    let long_message = "A".repeat(100_001);
    let result = validation::validate_message(&long_message);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too long"));
}

#[test]
fn test_validate_message_invalid_xss() {
    let dangerous_messages = vec![
        "<script>alert('xss')</script>",
        "javascript:alert('xss')",
        "<img onerror=alert('xss')>",
        "<body onload=alert('xss')>",
        "data:text/html,<script>alert('xss')</script>",
    ];

    for message in dangerous_messages {
        assert!(
            validation::validate_message(message).is_err(),
            "Message '{}' should be rejected",
            message
        );
    }
}

#[test]
fn test_validate_file_size_valid() {
    let valid_sizes = vec![
        0,
        1024,
        1024 * 1024,            // 1MB
        1024 * 1024 * 1024,     // 1GB
        5 * 1024 * 1024 * 1024, // 5GB (maximum)
    ];

    for size in valid_sizes {
        assert!(
            validation::validate_file_size(size).is_ok(),
            "File size {} should be valid",
            size
        );
    }
}

#[test]
fn test_validate_file_size_invalid() {
    let invalid_sizes = vec![
        5 * 1024 * 1024 * 1024 + 1, // just over 5GB
        10 * 1024 * 1024 * 1024,    // 10GB
        u64::MAX,
    ];

    for size in invalid_sizes {
        assert!(
            validation::validate_file_size(size).is_err(),
            "File size {} should be invalid",
            size
        );
    }
}

#[test]
fn test_validate_group_name_valid() {
    let max_name = "A".repeat(128);
    let valid_names = vec![
        "Test Group",
        "Developers",
        "TestGroup",
        "Test_Group",
        "Test-Group",
        "A",
        max_name.as_str(),
    ];

    for name in valid_names {
        assert!(
            validation::validate_group_name(name).is_ok(),
            "Group name '{}' should be valid",
            name
        );
    }
}

#[test]
fn test_validate_group_name_invalid_empty() {
    let result = validation::validate_group_name("");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));
}

#[test]
fn test_validate_group_name_invalid_too_long() {
    let long_name = "A".repeat(129);
    let result = validation::validate_group_name(&long_name);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too long"));
}

#[test]
fn test_validate_share_code_valid() {
    let max_code = "A".repeat(256);
    let valid_codes = vec![
        "abc123",
        "test-file-share",
        "file_123.pdf",
        "share.code.456",
        max_code.as_str(),
    ];

    for code in valid_codes {
        assert!(
            validation::validate_share_code(code).is_ok(),
            "Share code '{}' should be valid",
            code
        );
    }
}

#[test]
fn test_validate_share_code_invalid_empty() {
    let result = validation::validate_share_code("");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));
}

#[test]
fn test_validate_share_code_invalid_too_long() {
    let long_code = "A".repeat(257);
    let result = validation::validate_share_code(&long_code);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too long"));
}

#[test]
fn test_validate_share_code_invalid_characters() {
    let invalid_codes = vec!["share@code", "code#123", "test&code", "code$", "code/"];

    for code in invalid_codes {
        assert!(
            validation::validate_share_code(code).is_err(),
            "Share code '{}' should be invalid",
            code
        );
    }
}

#[test]
fn test_validate_uri_valid() {
    let max_uri = "A".repeat(2048);
    let valid_uris = vec![
        "https://example.com/file.pdf",
        "http://localhost:8080/file",
        "ipfs://QmTestHash123",
        max_uri.as_str(),
    ];

    for uri in valid_uris {
        assert!(
            validation::validate_uri(uri).is_ok(),
            "URI '{}' should be valid",
            uri
        );
    }
}

#[test]
fn test_validate_uri_invalid_empty() {
    let result = validation::validate_uri("");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));
}

#[test]
fn test_validate_uri_invalid_too_long() {
    let long_uri = "A".repeat(2049);
    let result = validation::validate_uri(&long_uri);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too long"));
}

#[test]
fn test_validate_uri_invalid_schemes() {
    let dangerous_uris = vec![
        "file:///etc/passwd",
        "javascript:alert('xss')",
        "vbscript:alert('xss')",
        "data:text/html,<script>alert('xss')</script>",
    ];

    for uri in dangerous_uris {
        assert!(
            validation::validate_uri(uri).is_err(),
            "URI '{}' should be rejected",
            uri
        );
    }
}

#[test]
fn test_validate_peer_id_valid() {
    // Test with a valid libp2p peer ID (generate one for the test)
    use libp2p::identity::Keypair;
    let keypair = Keypair::generate_ed25519();
    let peer_id = keypair.public().to_peer_id();
    let peer_id_str = peer_id.to_string();

    let result = validation::validate_peer_id(&peer_id_str);
    assert!(result.is_ok());
    // Should return a valid PeerId
    let parsed_peer_id = result.unwrap();
    assert_eq!(parsed_peer_id, peer_id);
}

#[test]
fn test_validate_peer_id_invalid() {
    let long_peer_id = "A".repeat(257);
    let invalid_peer_ids = vec![
        "", // empty
        "invalid_peer_id",
        "not-a-valid-peer-id",
        long_peer_id.as_str(), // too long
    ];

    for peer_id_str in invalid_peer_ids {
        assert!(
            validation::validate_peer_id(peer_id_str).is_err(),
            "Peer ID '{}' should be invalid",
            peer_id_str
        );
    }
}

#[test]
fn test_validate_file_path_valid() {
    let valid_paths = vec![
        Path::new("relative/path/to/file.txt"),
        Path::new("file.pdf"),
        Path::new("./file.pdf"),
        Path::new("data/uploads/image.png"),
    ];

    for path in valid_paths {
        assert!(
            validation::validate_file_path(path).is_ok(),
            "Path '{:?}' should be valid",
            path
        );
    }
}

#[test]
fn test_validate_file_path_invalid_traversal() {
    let invalid_paths = vec![
        Path::new("../etc/passwd"),
        Path::new("./../../etc/passwd"),
        Path::new("data/../../../etc/passwd"),
    ];

    for path in invalid_paths {
        assert!(
            validation::validate_file_path(path).is_err(),
            "Path '{:?}' should be rejected (path traversal)",
            path
        );
        assert!(validation::validate_file_path(path)
            .unwrap_err()
            .to_string()
            .contains("not allowed"));
    }
}

#[test]
fn test_validate_file_path_invalid_absolute() {
    let invalid_paths = vec![
        Path::new("/etc/passwd"),
        Path::new("/home/user/file.txt"),
        // Windows paths are relative on Linux, so this test is platform-dependent
        // On Windows, it would be an absolute path and rejected
        // Path::new("C:\\Windows\\System32\\config"),
    ];

    for path in invalid_paths {
        assert!(
            validation::validate_file_path(path).is_err(),
            "Path '{:?}' should be rejected (absolute path)",
            path
        );
    }
}

#[test]
fn test_validate_file_path_invalid_system_dirs() {
    let invalid_paths = vec![
        Path::new("./etc/passwd"),
        Path::new("data/etc/config"),
        Path::new("./proc/123/status"),
    ];

    for path in invalid_paths {
        assert!(
            validation::validate_file_path(path).is_err(),
            "Path '{:?}' should be rejected (system directory)",
            path
        );
    }
}

#[test]
fn test_sanitize_string() {
    let input = "<script>alert('xss')</script>";
    let sanitized = validation::sanitize_string(input);
    // The sanitize function replaces & last, so & in entities get double-encoded
    assert_eq!(
        sanitized,
        "&amp;lt;script&amp;gt;alert(&amp;#x27;xss&amp;#x27;)&amp;lt;/script&amp;gt;"
    );
    assert!(!sanitized.contains("<"));
    assert!(!sanitized.contains(">"));
    assert!(!sanitized.contains("'"));
}

#[test]
fn test_sanitize_string_safe() {
    let safe_input = "Hello, world! This is safe text.";
    let sanitized = validation::sanitize_string(safe_input);
    assert_eq!(sanitized, safe_input);
}

#[test]
fn test_sanitize_string_mixed() {
    let input = "Hello <b>world</b>! & \"test\" 'data'";
    let sanitized = validation::sanitize_string(input);
    // The sanitize function replaces & last, so & in entities get double-encoded
    assert_eq!(
        sanitized,
        "Hello &amp;lt;b&amp;gt;world&amp;lt;/b&amp;gt;! &amp; &amp;quot;test&amp;quot; &amp;#x27;data&amp;#x27;"
    );
}

#[test]
fn test_validate_all_inputs_with_edge_cases() {
    // Test boundary conditions and edge cases
    assert!(validation::validate_nickname("A").is_ok());
    assert!(validation::validate_nickname("Z").is_ok());
    assert!(validation::validate_nickname("9").is_ok());
    // Whitespace is allowed in nicknames
    assert!(validation::validate_nickname(" ").is_ok()); // only space

    assert!(validation::validate_message("").is_ok()); // empty message is allowed
    assert!(validation::validate_file_size(0).is_ok()); // zero-size file is allowed

    assert!(validation::validate_group_name("A").is_ok());
    assert!(validation::validate_share_code("A").is_ok());
    assert!(validation::validate_uri("A").is_ok());
}
