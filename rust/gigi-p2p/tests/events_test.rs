//! Event system tests for gigi-p2p
//!
//! Tests all P2pEvent variants and event data structures

use gigi_p2p::{ChunkInfo, FileInfo, P2pEvent};
use libp2p::{Multiaddr, PeerId};
use std::path::PathBuf;

#[test]
fn test_peer_discovered_event() {
    let peer_id = PeerId::random();
    let address: Multiaddr = "/ip4/127.0.0.1/tcp/1234".parse().unwrap();
    let event = P2pEvent::PeerDiscovered {
        peer_id,
        nickname: "Alice".to_string(),
        address,
    };

    match event {
        P2pEvent::PeerDiscovered {
            peer_id: pid,
            nickname,
            ..
        } => {
            assert_eq!(pid, peer_id);
            assert_eq!(nickname, "Alice");
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_peer_expired_event() {
    let peer_id = PeerId::random();
    let event = P2pEvent::PeerExpired {
        peer_id,
        nickname: "Bob".to_string(),
    };

    match event {
        P2pEvent::PeerExpired {
            peer_id: pid,
            nickname,
        } => {
            assert_eq!(pid, peer_id);
            assert_eq!(nickname, "Bob");
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_nickname_updated_event() {
    let peer_id = PeerId::random();
    let event = P2pEvent::NicknameUpdated {
        peer_id,
        nickname: "Alice-Updated".to_string(),
    };

    match event {
        P2pEvent::NicknameUpdated {
            peer_id: pid,
            nickname,
        } => {
            assert_eq!(pid, peer_id);
            assert_eq!(nickname, "Alice-Updated");
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_direct_message_event() {
    let event = P2pEvent::DirectMessage {
        from: PeerId::random(),
        from_nickname: "Alice".to_string(),
        message: "Hello".to_string(),
    };

    match event {
        P2pEvent::DirectMessage {
            from_nickname,
            message,
            ..
        } => {
            assert_eq!(from_nickname, "Alice");
            assert_eq!(message, "Hello");
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_group_message_event() {
    let event = P2pEvent::GroupMessage {
        from: PeerId::random(),
        from_nickname: "Alice".to_string(),
        group: "test-group".to_string(),
        message: "Group hello".to_string(),
    };

    match event {
        P2pEvent::GroupMessage {
            from_nickname,
            group,
            message,
            ..
        } => {
            assert_eq!(from_nickname, "Alice");
            assert_eq!(group, "test-group");
            assert_eq!(message, "Group hello");
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_group_joined_event() {
    let event = P2pEvent::GroupJoined {
        group: "test-group".to_string(),
    };

    match event {
        P2pEvent::GroupJoined { group } => {
            assert_eq!(group, "test-group");
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_group_left_event() {
    let event = P2pEvent::GroupLeft {
        group: "test-group".to_string(),
    };

    match event {
        P2pEvent::GroupLeft { group } => {
            assert_eq!(group, "test-group");
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_file_download_started_event() {
    let event = P2pEvent::FileDownloadStarted {
        from: PeerId::random(),
        from_nickname: "Alice".to_string(),
        filename: "test.txt".to_string(),
        download_id: "dl-1".to_string(),
        share_code: "share-123".to_string(),
    };

    match event {
        P2pEvent::FileDownloadStarted {
            download_id,
            filename,
            from_nickname,
            ..
        } => {
            assert_eq!(download_id, "dl-1");
            assert_eq!(filename, "test.txt");
            assert_eq!(from_nickname, "Alice");
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_file_download_progress_event() {
    let event = P2pEvent::FileDownloadProgress {
        download_id: "dl-1".to_string(),
        filename: "test.txt".to_string(),
        share_code: "share-123".to_string(),
        from_peer_id: PeerId::random(),
        from_nickname: "Alice".to_string(),
        downloaded_chunks: 5,
        total_chunks: 10,
    };

    match event {
        P2pEvent::FileDownloadProgress {
            download_id,
            downloaded_chunks,
            total_chunks,
            ..
        } => {
            assert_eq!(download_id, "dl-1");
            assert_eq!(downloaded_chunks, 5);
            assert_eq!(total_chunks, 10);
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_file_download_completed_event() {
    let event = P2pEvent::FileDownloadCompleted {
        download_id: "dl-1".to_string(),
        filename: "test.txt".to_string(),
        share_code: "share-123".to_string(),
        from_peer_id: PeerId::random(),
        from_nickname: "Alice".to_string(),
        path: PathBuf::from("/downloads/test.txt"),
    };

    match event {
        P2pEvent::FileDownloadCompleted {
            download_id,
            filename,
            path,
            ..
        } => {
            assert_eq!(download_id, "dl-1");
            assert_eq!(filename, "test.txt");
            assert_eq!(path, PathBuf::from("/downloads/test.txt"));
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_file_download_failed_event() {
    let event = P2pEvent::FileDownloadFailed {
        download_id: "dl-1".to_string(),
        filename: "test.txt".to_string(),
        share_code: "share-123".to_string(),
        from_peer_id: PeerId::random(),
        from_nickname: "Alice".to_string(),
        error: "Connection timeout".to_string(),
    };

    match event {
        P2pEvent::FileDownloadFailed {
            download_id,
            filename,
            error,
            ..
        } => {
            assert_eq!(download_id, "dl-1");
            assert_eq!(filename, "test.txt");
            assert_eq!(error, "Connection timeout");
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_listening_on_event() {
    let address: Multiaddr = "/ip4/127.0.0.1/tcp/1234".parse().unwrap();
    let event = P2pEvent::ListeningOn {
        address: address.clone(),
    };

    match event {
        P2pEvent::ListeningOn { address: addr } => {
            assert_eq!(addr, address);
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_connected_event() {
    let peer_id = PeerId::random();
    let event = P2pEvent::Connected {
        peer_id,
        nickname: "Alice".to_string(),
    };

    match event {
        P2pEvent::Connected {
            peer_id: pid,
            nickname,
        } => {
            assert_eq!(pid, peer_id);
            assert_eq!(nickname, "Alice");
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_disconnected_event() {
    let peer_id = PeerId::random();
    let event = P2pEvent::Disconnected {
        peer_id,
        nickname: "Alice".to_string(),
    };

    match event {
        P2pEvent::Disconnected {
            peer_id: pid,
            nickname,
        } => {
            assert_eq!(pid, peer_id);
            assert_eq!(nickname, "Alice");
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_chunk_info() {
    let chunk = ChunkInfo {
        file_id: "file-123".to_string(),
        chunk_index: 0,
        data: vec![1u8, 2u8, 3u8, 4u8],
        hash: "abc123".to_string(),
    };

    assert_eq!(chunk.file_id, "file-123");
    assert_eq!(chunk.chunk_index, 0);
    assert_eq!(chunk.data, vec![1u8, 2u8, 3u8, 4u8]);
    assert_eq!(chunk.hash, "abc123");
}

#[test]
fn test_file_info() {
    let file_info = FileInfo {
        id: "file-123".to_string(),
        name: "test.txt".to_string(),
        size: 1024,
        hash: "abc123def456".to_string(),
        chunk_count: 4,
        created_at: 1234567890,
    };

    assert_eq!(file_info.id, "file-123");
    assert_eq!(file_info.name, "test.txt");
    assert_eq!(file_info.size, 1024);
}

#[test]
fn test_event_variants_count() {
    let peer_id = PeerId::random();
    let address: Multiaddr = "/ip4/127.0.0.1/tcp/1234".parse().unwrap();

    let events = vec![
        P2pEvent::PeerDiscovered {
            peer_id,
            nickname: "Alice".to_string(),
            address: address.clone(),
        },
        P2pEvent::PeerExpired {
            peer_id,
            nickname: "Alice".to_string(),
        },
        P2pEvent::NicknameUpdated {
            peer_id,
            nickname: "Bob".to_string(),
        },
        P2pEvent::DirectMessage {
            from: peer_id,
            from_nickname: "Alice".to_string(),
            message: "Hello".to_string(),
        },
        P2pEvent::GroupJoined {
            group: "group-1".to_string(),
        },
        P2pEvent::GroupLeft {
            group: "group-1".to_string(),
        },
        P2pEvent::ListeningOn {
            address: address.clone(),
        },
        P2pEvent::Connected {
            peer_id,
            nickname: "Alice".to_string(),
        },
        P2pEvent::Disconnected {
            peer_id,
            nickname: "Alice".to_string(),
        },
        P2pEvent::Error("Test error".to_string()),
    ];

    assert_eq!(events.len(), 10);
}
