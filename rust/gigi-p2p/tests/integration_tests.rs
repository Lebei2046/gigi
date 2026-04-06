use gigi_p2p::P2pClient;
use std::path::PathBuf;

#[tokio::test]
async fn test_p2p_client_creation() {
    let keypair = libp2p::identity::Keypair::generate_ed25519();

    let result = P2pClient::new(keypair, "test_peer".to_string(), PathBuf::from("/tmp"));
    assert!(result.is_ok());

    let (_client, _receiver) = result.unwrap();
}

#[tokio::test]
async fn test_peer_nickname() {
    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let (client, _receiver) =
        P2pClient::new(keypair, "test_nickname".to_string(), PathBuf::from("/tmp")).unwrap();

    // Test that client was created successfully
    let peers = client.list_peers();
    assert!(peers.is_empty()); // Should be empty initially
}

#[tokio::test]
async fn test_start_listening() {
    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let (mut client, _receiver) =
        P2pClient::new(keypair, "listener".to_string(), PathBuf::from("/tmp")).unwrap();

    let addr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
    let result = client.start_listening(addr);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_group_management() {
    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let (mut client, _receiver) =
        P2pClient::new(keypair, "group_test".to_string(), PathBuf::from("/tmp")).unwrap();

    // Test joining a group
    let result = client.join_group("test-group");
    assert!(result.is_ok());

    // Test leaving a group
    let result = client.leave_group("test-group");
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_file_sharing() {
    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let (client, _receiver) =
        P2pClient::new(keypair, "file_sharer".to_string(), PathBuf::from("/tmp")).unwrap();

    // Test listing shared files (should be empty initially)
    let files = client.list_shared_files();
    assert!(files.is_empty());
}

#[tokio::test]
async fn test_peer_listing() {
    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let (client, _receiver) =
        P2pClient::new(keypair, "peer_lister".to_string(), PathBuf::from("/tmp")).unwrap();

    // Test listing peers (should be empty initially)
    let peers = client.list_peers();
    assert!(peers.is_empty());
}
