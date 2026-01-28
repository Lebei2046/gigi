// Unit tests for the tauri-plugin-gigi crate.
//
// This module contains tests for various components of the plugin including
// file utilities, error handling, data models, and command functions.

// Import file utils tests directly into lib
// These are inline tests that will be part of the library tests

#[cfg(test)]
mod tests {
    // File utility tests
    use crate::file_utils::{convert_to_base64_if_image, is_image_data, is_image_file};

    #[test]
    fn test_is_image_file_valid_extensions() {
        assert!(is_image_file("photo.jpg"));
        assert!(is_image_file("photo.jpeg"));
        assert!(is_image_file("image.png"));
        assert!(is_image_file("animation.gif"));
        assert!(is_image_file("picture.webp"));
        assert!(is_image_file("bitmap.bmp"));
    }

    #[test]
    fn test_is_image_file_uppercase_extensions() {
        assert!(is_image_file("photo.JPG"));
        assert!(is_image_file("photo.PNG"));
        assert!(is_image_file("image.GIF"));
        assert!(is_image_file("picture.WEBP"));
    }

    #[test]
    fn test_is_image_file_non_image() {
        assert!(!is_image_file("document.pdf"));
        assert!(!is_image_file("video.mp4"));
        assert!(!is_image_file("archive.zip"));
        assert!(!is_image_file("code.rs"));
    }

    #[test]
    fn test_is_image_data_jpeg() {
        let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
        assert!(is_image_data(&jpeg_data));
    }

    #[test]
    fn test_is_image_data_png() {
        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert!(is_image_data(&png_data));
    }

    #[test]
    fn test_is_image_data_gif() {
        let gif_data = vec![0x47, 0x49, 0x46, 0x38, 0x37, 0x61];
        assert!(is_image_data(&gif_data));
    }

    #[test]
    fn test_is_image_data_webp() {
        // WEBP format: "RIFF" (4 bytes) + size (4 bytes) + "WEBP" (4 bytes)
        // "WEBP" is at bytes 8-11
        let mut webp_data = vec![0x52, 0x49, 0x46, 0x46]; // "RIFF"
        webp_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // size
        webp_data.extend_from_slice(b"WEBP"); // "WEBP" marker at bytes 8-11
        assert!(is_image_data(&webp_data));

        // Also test minimal valid WEBP
        let short_webp = b"RIFF\x00\x00\x00\x00WEBP";
        assert!(is_image_data(short_webp));
    }

    #[test]
    fn test_is_image_data_bmp() {
        let bmp_data = vec![0x42, 0x4D, 0x36, 0x00, 0x00, 0x00];
        assert!(is_image_data(&bmp_data));
    }

    #[test]
    fn test_is_image_data_non_image() {
        let pdf_data = b"%PDF-1.4".to_vec();
        assert!(!is_image_data(&pdf_data));

        let mp4_data = vec![0x00, 0x00, 0x00, 0x20, 0x66, 0x74, 0x79, 0x70];
        assert!(!is_image_data(&mp4_data));
    }

    #[test]
    fn test_convert_to_base64_if_image_valid() {
        let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
        let result = convert_to_base64_if_image(&jpeg_data);
        assert!(result.is_some());
        assert!(result.unwrap().len() > jpeg_data.len());
    }

    #[test]
    fn test_convert_to_base64_if_image_non_image() {
        let pdf_data = b"%PDF-1.4".to_vec();
        let result = convert_to_base64_if_image(&pdf_data);
        assert!(result.is_none());
    }

    // Error type tests
    use crate::error::{Error, Result};

    #[test]
    fn test_error_display() {
        let err = Error::Io("test error".to_string());
        assert_eq!(err.to_string(), "IO error: test error");

        let err = Error::P2pNotInitialized;
        assert_eq!(err.to_string(), "P2P client not initialized");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_error_from_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid");
        assert!(json_err.is_err());
        let err: Error = json_err.unwrap_err().into();
        assert!(matches!(err, Error::SerdeJson(_)));
    }

    #[test]
    fn test_result_type() {
        let success: Result<()> = Ok(());
        assert!(success.is_ok());

        let failure: Result<()> = Err(Error::P2pNotInitialized);
        assert!(failure.is_err());
    }

    // Model tests
    use crate::models::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.nickname, "Anonymous");
        assert_eq!(config.auto_accept_files, false);
        assert_eq!(config.max_concurrent_downloads, 3);
        assert_eq!(config.port, 0);
    }

    #[test]
    fn test_plugin_state_new() {
        let state = PluginState::new();
        // Just verify it creates without panicking
        let _ = state.p2p_client;
        let _ = state.config;
        let _ = state.active_downloads;
    }

    #[test]
    fn test_peer_creation() {
        let peer = Peer {
            id: "12D3KooW...".to_string(),
            nickname: "TestPeer".to_string(),
            capabilities: vec!["messaging".to_string()],
        };
        assert_eq!(peer.nickname, "TestPeer");
        assert_eq!(peer.capabilities.len(), 1);
    }

    #[test]
    fn test_message_creation() {
        let msg = Message {
            id: "msg-1".to_string(),
            from_peer_id: "peer-1".to_string(),
            from_nickname: "Alice".to_string(),
            content: "Hello".to_string(),
            timestamp: 123456,
        };
        assert_eq!(msg.content, "Hello");
        assert_eq!(msg.timestamp, 123456);
    }

    #[test]
    fn test_group_message_creation() {
        let msg = GroupMessage {
            id: "msg-2".to_string(),
            group_id: "group-1".to_string(),
            from_peer_id: "peer-1".to_string(),
            from_nickname: "Bob".to_string(),
            content: "Hi".to_string(),
            timestamp: 123456,
        };
        assert_eq!(msg.group_id, "group-1");
    }

    #[test]
    fn test_file_info_creation() {
        let info = FileInfo {
            id: "file-1".to_string(),
            name: "test.jpg".to_string(),
            size: 1024,
            mime_type: "image/jpeg".to_string(),
            peer_id: "peer-1".to_string(),
        };
        assert_eq!(info.size, 1024);
        assert_eq!(info.mime_type, "image/jpeg");
    }

    #[test]
    fn test_download_progress_creation() {
        let progress = DownloadProgress {
            download_id: "dl-1".to_string(),
            progress: 50.0,
            speed: 1024,
        };
        assert_eq!(progress.progress, 50.0);
        assert_eq!(progress.speed, 1024);
    }

    #[test]
    fn test_file_send_target() {
        let direct = FileSendTarget::Direct("peer-1");
        assert!(matches!(direct, FileSendTarget::Direct(_)));

        let group = FileSendTarget::Group("group-1");
        assert!(matches!(group, FileSendTarget::Group(_)));
    }

    // Command tests
    use crate::commands::peer::try_get_peer_id;

    #[test]
    fn test_try_get_peer_id_valid_key() {
        let key = vec![
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C,
            0x1D, 0x1E, 0x1F, 0x20,
        ];
        let result = try_get_peer_id(key);
        assert!(result.is_ok());
        let peer_id = result.unwrap();
        assert!(!peer_id.is_empty());
        assert!(peer_id.starts_with("12D3KooW"));
    }

    #[test]
    fn test_try_get_peer_id_invalid_key() {
        let key = vec![0x01, 0x02, 0x03];
        let result = try_get_peer_id(key);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_get_peer_id_empty_key() {
        let key = vec![];
        let result = try_get_peer_id(key);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_get_peer_id_consistency() {
        let key = vec![
            0xAA, 0xBB, 0xCC, 0xDD, 0xAA, 0xBB, 0xCC, 0xDD, 0xAA, 0xBB, 0xCC, 0xDD, 0xAA, 0xBB,
            0xCC, 0xDD, 0xAA, 0xBB, 0xCC, 0xDD, 0xAA, 0xBB, 0xCC, 0xDD, 0xAA, 0xBB, 0xCC, 0xDD,
            0xAA, 0xBB, 0xCC, 0xDD,
        ];
        let result1 = try_get_peer_id(key.clone());
        let result2 = try_get_peer_id(key);
        assert_eq!(result1.is_ok(), result2.is_ok());
        if result1.is_ok() && result2.is_ok() {
            assert_eq!(result1.unwrap(), result2.unwrap());
        }
    }

    // Serialization tests
    #[test]
    fn test_config_serialization() {
        let config = Config {
            nickname: "Test".to_string(),
            auto_accept_files: true,
            download_folder: "/path".to_string(),
            max_concurrent_downloads: 5,
            port: 12345,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(config.nickname, deserialized.nickname);
        assert_eq!(
            config.max_concurrent_downloads,
            deserialized.max_concurrent_downloads
        );
    }

    #[test]
    fn test_peer_serialization() {
        let peer = Peer {
            id: "12D3KooW...".to_string(),
            nickname: "Test".to_string(),
            capabilities: vec!["messaging".to_string(), "file_transfer".to_string()],
        };

        let json = serde_json::to_string(&peer).unwrap();
        let deserialized: Peer = serde_json::from_str(&json).unwrap();

        assert_eq!(peer.id, deserialized.id);
        assert_eq!(peer.nickname, deserialized.nickname);
        assert_eq!(peer.capabilities.len(), deserialized.capabilities.len());
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message {
            id: "msg-1".to_string(),
            from_peer_id: "peer-1".to_string(),
            from_nickname: "Alice".to_string(),
            content: "Hello World".to_string(),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();

        assert_eq!(msg.id, deserialized.id);
        assert_eq!(msg.content, deserialized.content);
    }

    // Clone and Send tests
    #[test]
    fn test_config_clone() {
        let config = Config::default();
        let cloned = config.clone();
        assert_eq!(config.nickname, cloned.nickname);
    }

    #[test]
    fn test_message_clone() {
        let msg = Message {
            id: "msg-1".to_string(),
            from_peer_id: "peer-1".to_string(),
            from_nickname: "Alice".to_string(),
            content: "Hello".to_string(),
            timestamp: 123456,
        };
        let cloned = msg.clone();
        assert_eq!(msg.id, cloned.id);
    }

    #[test]
    fn test_plugin_state_clone() {
        let state = PluginState::new();
        let cloned = state.clone();
        // Both should share the same Arc references
        let _ = state.p2p_client;
        let _ = cloned.p2p_client;
    }

    // Edge case tests
    #[test]
    fn test_empty_string_paths() {
        assert!(!is_image_file(""));
    }

    #[test]
    fn test_path_with_dots() {
        assert!(is_image_file("path/to/file.jpg"));
        assert!(!is_image_file("path/to/file.txt"));
    }

    #[test]
    fn test_config_with_empty_values() {
        let config = Config {
            nickname: String::new(),
            auto_accept_files: false,
            download_folder: String::new(),
            max_concurrent_downloads: 0,
            port: 0,
        };
        assert_eq!(config.nickname, "");
        assert_eq!(config.max_concurrent_downloads, 0);
    }

    #[test]
    fn test_download_progress_bounds() {
        let progress = DownloadProgress {
            download_id: "dl-1".to_string(),
            progress: 0.0,
            speed: 0,
        };
        assert_eq!(progress.progress, 0.0);

        let progress = DownloadProgress {
            download_id: "dl-2".to_string(),
            progress: 100.0,
            speed: 9999999,
        };
        assert_eq!(progress.progress, 100.0);
        assert_eq!(progress.speed, 9999999);
    }
}
