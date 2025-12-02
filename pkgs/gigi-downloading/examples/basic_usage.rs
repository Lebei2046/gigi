//! Basic example of using gigi-downloading for file transfers

use gigi_downloading::TransferManager;
use libp2p::Multiaddr;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Gigi File Transfer Example");

    println!("🚀 Starting Gigi File Transfer Example");

    // Create two transfer managers to simulate two peers
    let (mut manager1, mut receiver1) = TransferManager::new_with_events().await?;
    let (mut manager2, mut receiver2) = TransferManager::new_with_events().await?;

    // Get peer IDs
    let peer1_id = manager1.local_peer_id();
    let peer2_id = manager2.local_peer_id();

    println!("📍 Peer 1 ID: {}", peer1_id);
    println!("📍 Peer 2 ID: {}", peer2_id);

    // Start listening on different addresses
    let addr1: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse()?;
    let addr2: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse()?;

    manager1.start_listening(addr1.clone())?;
    manager2.start_listening(addr2.clone())?;

    // Wait a bit for listeners to start
    sleep(Duration::from_secs(1)).await;

    // For now, we'll just simulate the connection
    // In a real scenario, you would get the actual listening addresses
    println!("🌐 Peers are listening on dynamic addresses");
    println!("🔗 In a real scenario, peers would connect to each other");

    // Handle events for both peers
    let mut event_count = 0;
    while event_count < 10 {
        // Handle peer 1 events
        let events1 = manager1.handle_swarm_events().await;
        for event in events1 {
            println!("📡 Peer 1 Event: {:?}", event);
            event_count += 1;
        }

        // Handle peer 2 events
        let events2 = manager2.handle_swarm_events().await;
        for event in events2 {
            println!("📡 Peer 2 Event: {:?}", event);
            event_count += 1;
        }

        // Also check receiver events
        while let Ok(event) = receiver1.try_recv() {
            println!("📨 Peer 1 Channel Event: {:?}", event);
            event_count += 1;
        }

        while let Ok(event) = receiver2.try_recv() {
            println!("📨 Peer 2 Channel Event: {:?}", event);
            event_count += 1;
        }

        sleep(Duration::from_millis(100)).await;
    }

    // Add a file to peer 1
    let file_data = b"Hello, this is a test file for gigi-downloading!".to_vec();
    let metadata = manager1.add_file("test.txt".to_string(), file_data.clone())?;
    println!(
        "📁 Added file to peer 1: {} ({} bytes)",
        metadata.filename, metadata.size
    );

    // Request file list (would work when peers are connected)
    println!("📋 File list requests would work when peers are connected");

    // Handle more events
    for _ in 0..50 {
        let events1 = manager1.handle_swarm_events().await;
        for event in events1 {
            println!("📡 Peer 1 Event: {:?}", event);
        }

        let events2 = manager2.handle_swarm_events().await;
        for event in events2 {
            println!("📡 Peer 2 Event: {:?}", event);
        }

        sleep(Duration::from_millis(100)).await;
    }

    println!("✅ Example completed successfully!");
    Ok(())
}
