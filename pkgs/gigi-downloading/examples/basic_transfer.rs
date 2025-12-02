//! Basic file transfer example
//!
//! This example demonstrates how to use the gigi-downloading library
//! to transfer files between two peers.

use gigi_downloading::{TransferEvent, TransferManager};
use std::env;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Parse command line arguments
    let (mode, filename) = if args.len() >= 3 {
        (args[1].clone(), args[2].clone())
    } else {
        eprintln!("Usage: {} <sender|receiver> [filename]", args[0]);
        eprintln!("Examples:");
        eprintln!("  {} sender test.txt", args[0]);
        eprintln!("  {} receiver", args[0]);
        return Ok(());
    };

    println!("🚀 Starting Gigi File Transfer Example");
    println!("📝 Mode: {}", mode);

    // Create transfer manager with event channel
    let (mut manager, mut event_rx) = TransferManager::new_with_events().await?;

    // Start listening on a local address
    println!("🔍 Starting file transfer service...");
    let listen_addr: libp2p::Multiaddr = "/ip4/0.0.0.0/tcp/0".parse().unwrap();
    println!("📍 Attempting to listen on: {}", listen_addr);
    manager.start_listening(listen_addr)?;

    if mode == "sender" {
        // Sender mode: create and share a file
        println!("📁 Creating test file: {}", filename);
        let file_data = format!(
            "This is a test file named '{}'.\nCreated by Gigi File Transfer.\nTimestamp: {}",
            filename,
            chrono::Utc::now().to_rfc3339()
        );

        let metadata = manager.add_file(filename.clone(), file_data.into_bytes())?;
        println!("✅ File added to transfer list:");
        println!("   - Name: {}", metadata.filename);
        println!("   - Size: {} bytes", metadata.size);
        println!("   - Hash: {}", metadata.hash);

        println!("\n📋 Available files:");
        for file in manager.get_available_files() {
            println!("   - {} ({} bytes)", file.filename, file.size);
        }
    }

    println!("\n📡 Listening for events for 60 seconds...");
    let mut event_count = 0;
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < Duration::from_secs(60) {
        tokio::select! {
            // Handle swarm events
            events = manager.handle_swarm_events() => {
                if !events.is_empty() {
                    for event in events {
                        handle_transfer_event(event, &mut event_count);
                    }
                }
            }
            // Handle channel events
            Some(event) = event_rx.recv() => {
                handle_transfer_event(event, &mut event_count);
            }
            _ = sleep(Duration::from_millis(500)) => {
                // Periodic status update
                if event_count > 0 && event_count % 20 == 0 {
                    println!("📊 Status: {} events processed", event_count);

                    if mode == "sender" {
                        println!("📁 Available files: {}", manager.get_available_files().len());
                    } else {
                        println!("⏳ Pending transfers: {}", manager.get_pending_transfers().len());
                    }
                }
            }
        }
    }

    // Show final status
    println!("\n📋 Final Status:");

    if mode == "sender" {
        println!("📁 Files available for transfer:");
        for file in manager.get_available_files() {
            println!("   - {} ({} bytes)", file.filename, file.size);
        }
    } else {
        println!("📁 Received files:");
        for file in manager.get_available_files() {
            let data = manager.get_file_data(&file.filename).unwrap_or_default();
            let preview = String::from_utf8_lossy(&data[..data.len().min(100)]);
            println!(
                "   - {} ({} bytes): \"{}\"",
                file.filename, file.size, preview
            );
        }

        println!("⏳ Pending transfers:");
        for (_filename, (metadata, chunks)) in manager.get_pending_transfers() {
            println!(
                "   - {} ({} chunks received)",
                metadata.filename,
                chunks.len()
            );
        }
    }

    println!("✨ Example completed!");
    Ok(())
}

fn handle_transfer_event(event: TransferEvent, event_count: &mut i32) {
    match event {
        TransferEvent::FileReceived { filename, metadata } => {
            println!("📥 File received: {} ({} bytes)", filename, metadata.size);
        }
        TransferEvent::ChunkReceived {
            filename,
            chunk_index,
            total_chunks,
        } => {
            println!(
                "📦 Chunk received: {} - {}/{}",
                filename,
                chunk_index + 1,
                total_chunks
            );
        }
        TransferEvent::TransferStarted { filename, peer_id } => {
            println!("🚀 Transfer started: {} from {}", filename, peer_id);
        }
        TransferEvent::TransferCompleted { filename, peer_id } => {
            println!("✅ Transfer completed: {} from {}", filename, peer_id);
        }
        TransferEvent::TransferFailed { filename, error } => {
            println!("❌ Transfer failed: {} - {}", filename, error);
        }
        TransferEvent::PeerConnected { peer_id } => {
            println!("🤝 Peer connected: {}", peer_id);
        }
        TransferEvent::NetworkEvent { message } => {
            println!("🌐 Network: {}", message);
        }
    }
    *event_count += 1;
}
