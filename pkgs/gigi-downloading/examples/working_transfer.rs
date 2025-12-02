//! Working file transfer example
//!
//! Demonstrates successful file transfer between two peers

use gigi_downloading::{TransferEvent, TransferManager};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Working File Transfer Example");

    // Create sender and receiver
    let (mut sender, _) = TransferManager::new_with_events().await?;
    let (mut receiver, mut receiver_rx) = TransferManager::new_with_events().await?;

    println!("📍 Sender: {}", sender.local_peer_id());
    println!("📍 Receiver: {}", receiver.local_peer_id());

    // Start listening
    sender.start_listening("/ip4/127.0.0.1/tcp/0".parse()?)?;
    receiver.start_listening("/ip4/127.0.0.1/tcp/0".parse()?)?;

    // Wait for listeners to start
    for _ in 0..20 {
        sender.handle_swarm_events().await;
        receiver.handle_swarm_events().await;
        sleep(Duration::from_millis(50)).await;
    }

    // Get receiver address and connect
    let receiver_addrs = receiver.listeners();
    let receiver_addr = &receiver_addrs[0];
    println!("🌐 Connecting to: {}", receiver_addr);

    sender.connect(receiver_addr.clone())?;

    // Wait for connection
    for _ in 0..30 {
        sender.handle_swarm_events().await;
        receiver.handle_swarm_events().await;

        // Check for connection events
        while let Ok(event) = receiver_rx.try_recv() {
            if matches!(event, TransferEvent::PeerConnected { .. }) {
                println!("✅ Connected!");
            }
        }

        sleep(Duration::from_millis(50)).await;
    }

    // Add file to sender
    let filename = "message.txt";
    let content = "Hello from Gigi File Transfer! 🚀".as_bytes().to_vec();
    let metadata = sender.add_file(filename.to_string(), content.clone())?;
    println!(
        "📁 Added file: {} ({} bytes)",
        metadata.filename, metadata.size
    );

    // Request file list
    println!("📋 Requesting file list...");
    let _request_id = receiver.request_file_list(sender.local_peer_id())?;

    // Wait for file list response
    for _ in 0..20 {
        receiver.handle_swarm_events().await;
        sleep(Duration::from_millis(50)).await;
    }

    // Request the file
    println!("📥 Requesting file: {}", filename);
    let _request_id = receiver.request_file(sender.local_peer_id(), filename.to_string())?;

    // Process events to complete the transfer
    let mut transfer_complete = false;
    for _i in 0..50 {
        let _sender_events = sender.handle_swarm_events().await;
        let _receiver_events = receiver.handle_swarm_events().await;

        // Check receiver channel events for transfer completion
        while let Ok(event) = receiver_rx.try_recv() {
            match &event {
                TransferEvent::ChunkReceived {
                    filename: fname, ..
                } => {
                    println!("📦 Received chunk for: {}", fname);
                }
                TransferEvent::FileReceived {
                    filename: fname, ..
                } => {
                    println!("📁 File received: {}", fname);
                }
                TransferEvent::TransferCompleted {
                    filename: fname, ..
                } => {
                    println!("✅ Transfer completed: {}", fname);
                    transfer_complete = true;
                }
                _ => {}
            }
        }

        if transfer_complete {
            break;
        }

        sleep(Duration::from_millis(100)).await;
    }

    // Show final results
    println!("\n📋 Results:");
    println!("📤 Sender files: {}", sender.get_available_files().len());
    println!(
        "📥 Receiver files: {}",
        receiver.get_available_files().len()
    );

    // Show received file content
    for file in receiver.get_available_files() {
        let data = receiver.get_file_data(&file.filename).unwrap_or_default();
        let content = String::from_utf8_lossy(&data);
        println!("📄 Received: {} -> \"{}\"", file.filename, content);
    }

    if transfer_complete {
        println!("🎉 File transfer successful!");
    } else {
        println!("⚠️  Transfer incomplete");
    }

    Ok(())
}
