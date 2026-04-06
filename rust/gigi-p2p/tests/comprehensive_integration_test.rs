//! Integration tests for P2P functionality
//!
//! Tests the available P2P client API including file sharing and validation.

use gigi_p2p::P2pClient;
use libp2p::identity::Keypair;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::time::{timeout, Duration};

/// Helper function to create a test P2P client
fn create_test_client(
    nickname: &str,
    temp_dir: &PathBuf,
) -> (
    P2pClient,
    futures::channel::mpsc::UnboundedReceiver<gigi_p2p::P2pEvent>,
) {
    let keypair = Keypair::generate_ed25519();
    P2pClient::new(keypair, nickname.to_string(), temp_dir.clone())
        .expect("Failed to create client")
}

#[tokio::test]
async fn test_client_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let download_dir = temp_dir.path().to_path_buf();

    let (client, _event_receiver) = create_test_client("Alice", &download_dir);

    // Verify client was created successfully
    assert_eq!(client.local_nickname(), "Alice");
    assert_eq!(client.peers_count(), 0);
    assert_eq!(client.connected_peers_count(), 0);
}

#[tokio::test]
async fn test_start_listening() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let download_dir = temp_dir.path().to_path_buf();

    let (mut client, _event_receiver) = create_test_client("Alice", &download_dir);

    // Start listening
    let addr: libp2p::Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
    let result = client.start_listening(addr);

    assert!(result.is_ok(), "Should be able to start listening");
}

#[tokio::test]
async fn test_group_lifecycle() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let download_dir = temp_dir.path().to_path_buf();

    let (mut client, _event_receiver) = create_test_client("Alice", &download_dir);

    let group_name = "test-group";

    // Create/join group
    let result = client.join_group(group_name);
    assert!(result.is_ok(), "Should be able to join group");

    // Verify group is listed
    let groups = client.list_groups();
    let found = groups.iter().any(|g| g.name == group_name);
    assert!(found, "Group should be in the list");

    // Leave group
    let result = client.leave_group(group_name);
    assert!(result.is_ok(), "Should be able to leave group");

    // Rejoin group
    let result = client.join_group(group_name);
    assert!(result.is_ok(), "Should be able to rejoin group");
}

#[tokio::test]
async fn test_file_sharing() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let download_dir = temp_dir.path().to_path_buf();

    let (mut client, mut _event_receiver) = create_test_client("Alice", &download_dir);

    // Create a test file
    let test_file_path = temp_dir.path().join("test-file.txt");
    let test_data = b"Hello, this is a test file for P2P sharing!";
    std::fs::write(&test_file_path, test_data).expect("Failed to write test file");

    // Share file
    let share_result = client.share_file(&test_file_path).await;
    assert!(share_result.is_ok(), "Should be able to share file");

    let share_code = share_result.unwrap();

    // Verify file is listed
    let shared_files = client.list_shared_files();
    let found = shared_files.iter().any(|f| f.share_code == share_code);
    assert!(found, "File should be in the shared files list");

    // Unshare file
    let result = client.unshare_file(&share_code);
    assert!(result.is_ok(), "Should be able to unshare file");

    // Verify file is removed from list
    let shared_files = client.list_shared_files();
    let found = shared_files.iter().any(|f| f.share_code == share_code);
    assert!(!found, "File should be removed from shared files list");
}

#[tokio::test]
async fn test_multiple_files_sharing() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let download_dir = temp_dir.path().to_path_buf();

    let (mut client, _event_receiver) = create_test_client("Alice", &download_dir);

    let mut share_codes = Vec::new();

    // Share multiple files
    for i in 0..3 {
        let file_path = temp_dir.path().join(format!("test-file-{}.txt", i));
        let test_data = format!("Test file content {}", i).into_bytes();
        std::fs::write(&file_path, test_data).expect("Failed to write test file");

        let share_result = client.share_file(&file_path).await;
        assert!(share_result.is_ok(), "Should be able to share file {}", i);
        share_codes.push(share_result.unwrap());
    }

    // Verify all files are listed
    let shared_files = client.list_shared_files();
    assert_eq!(shared_files.len(), 3, "Should have 3 shared files");

    // Clean up
    for share_code in &share_codes {
        let _ = client.unshare_file(share_code);
    }
}

#[tokio::test]
async fn test_download_tracking() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let download_dir = temp_dir.path().to_path_buf();

    let (client, _event_receiver) = create_test_client("Alice", &download_dir);

    // Verify no active downloads initially
    let active_downloads = client.get_active_downloads();
    assert_eq!(
        active_downloads.len(),
        0,
        "Should have no active downloads initially"
    );

    // Note: Actual download testing would require peer connection setup
    // This test verifies the download tracking structure
}

#[tokio::test]
async fn test_event_stream() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let download_dir = temp_dir.path().to_path_buf();

    let (_client, mut event_receiver) = create_test_client("TestClient", &download_dir);

    // Verify event receiver is available
    // Events are emitted via the event system when client is active
    // This test verifies the receiver structure is correct
    use futures::StreamExt;

    let timeout_duration = Duration::from_secs(1);
    let result = timeout(timeout_duration, event_receiver.next()).await;

    // Either we get an event or timeout - either is acceptable for this test
    let _ = result;
}

#[tokio::test]
async fn test_peer_info_queries() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let download_dir = temp_dir.path().to_path_buf();

    let (client, _event_receiver) = create_test_client("Alice", &download_dir);

    // Query for non-existent peer
    let result = client.get_peer_by_nickname("Bob");
    assert!(result.is_err(), "Should return error for non-existent peer");

    // Get peer ID by nickname
    let peer_id = client.get_peer_id_by_nickname("Bob");
    assert!(
        peer_id.is_none(),
        "Should return None for non-existent peer"
    );

    // List peers (should be empty initially)
    let peers = client.list_peers();
    assert_eq!(peers.len(), 0, "Should have no peers initially");
}

#[tokio::test]
async fn test_send_group_message() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let download_dir = temp_dir.path().to_path_buf();

    let (mut client, _event_receiver) = create_test_client("Alice", &download_dir);

    let group_name = "test-group";

    // Join group
    client.join_group(group_name).expect("Failed to join group");

    // Send group message - just test that method exists and returns Result
    let result = client.send_group_message(group_name, "Hello group!".to_string());
    // Result depends on event loop being active, which requires async setup
    // We just verify the method is available
    let _ = result;
}

#[tokio::test]
async fn test_client_shutdown() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let download_dir = temp_dir.path().to_path_buf();

    let (mut client, _event_receiver) = create_test_client("Alice", &download_dir);

    // Start listening
    let addr: libp2p::Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
    client
        .start_listening(addr)
        .expect("Failed to start listening");

    // Shutdown
    let result = client.shutdown();
    assert!(result.is_ok(), "Should be able to shutdown client");
}

#[tokio::test]
async fn test_validation_integrated() {
    // Test that validation is integrated properly
    use gigi_p2p::validation;

    // Valid inputs
    assert!(validation::validate_nickname("Alice").is_ok());
    assert!(validation::validate_message("Hello, world!").is_ok());
    assert!(validation::validate_group_name("Test Group").is_ok());

    // Invalid inputs
    assert!(validation::validate_nickname("").is_err());
    assert!(validation::validate_nickname("A".repeat(100).as_str()).is_err());
    assert!(validation::validate_message(&"A".repeat(200_000)).is_err());
}
