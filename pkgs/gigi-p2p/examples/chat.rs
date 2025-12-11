use clap::Parser;
use futures::StreamExt;
use gigi_p2p::{P2pClient, P2pEvent};
use std::io::{self, Write};
use std::path::PathBuf;
use tokio::fs;

/// Gigi P2P Chat - Simple and clean terminal chat
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port for listening (0 for random available port)
    #[arg(short, long, default_value = "0")]
    port: u16,

    /// Nickname for the peer
    #[arg(short, long, default_value = "anonymous")]
    nickname: String,

    /// Directory for saving files
    #[arg(short, long, default_value = "downloads")]
    output: PathBuf,

    /// File for recording shared file information
    #[arg(long, default_value = "shared.json")]
    shared: PathBuf,
}

fn show_help() {
    println!("\n📖 Available Commands:");
    println!("  ┌─────────────────────────────────────────────────┐");
    println!("  │  help, ?, h              Show this help          │");
    println!("  │  peers, p                 List connected peers    │");
    println!("  │  send <nick> <msg>       Send direct message     │");
    println!("  │  send-image <nick> <path> Send image file        │");
    println!("  │  join <group>            Join a group            │");
    println!("  │  leave <group>           Leave a group           │");
    println!("  │  send-group <grp> <msg>  Send to group           │");
    println!("  │  send-group-image <grp> <path> Send group image   │");
    println!("  │  share <path>            Share a file            │");
    println!("  │  unshare <code>          Unshare a file          │");
    println!("  │  files, f                List shared files       │");
    println!("  │  download <nick> <code>  Download shared file    │");
    println!("  │  quit, exit, q           Exit the chat           │");
    println!("  └─────────────────────────────────────────────────┘");
    println!("\n💡 Tips:");
    println!("  • Commands can be abbreviated (e.g., 's' for 'send')");
    println!("  • Use Ctrl+C to force quit");
    println!("  • Files are automatically saved to downloads/");
}

async fn handle_p2p_event(event: P2pEvent, output_dir: &PathBuf, _client: &P2pClient) {
    match event {
        P2pEvent::PeerDiscovered {
            peer_id,
            nickname,
            address,
        } => {
            println!(
                "🔍 New peer discovered: {} ({}) at {}",
                nickname, peer_id, address
            );
        }
        P2pEvent::PeerExpired { peer_id, nickname } => {
            println!("👋 Peer disconnected: {} ({})", nickname, peer_id);
        }
        P2pEvent::NicknameUpdated { peer_id, nickname } => {
            println!("📝 Nickname updated: {} ({})", nickname, peer_id);
        }
        P2pEvent::DirectMessage {
            from,
            from_nickname,
            message,
        } => {
            println!("💬 {} ({}): {}", from_nickname, from, message);
        }
        P2pEvent::DirectImageMessage {
            from,
            from_nickname,
            filename,
            data,
        } => {
            let file_path = output_dir.join(&filename);
            match fs::write(&file_path, &data).await {
                Ok(()) => {
                    println!(
                        "🖼️ {} ({}) sent image: {} (saved to {})",
                        from_nickname,
                        from,
                        filename,
                        file_path.display()
                    );
                }
                Err(e) => {
                    println!(
                        "❌ Failed to save image from {} ({}): {}",
                        from_nickname, from, e
                    );
                }
            }
        }
        P2pEvent::GroupMessage {
            from,
            from_nickname,
            group,
            message,
        } => {
            // Check if this is an image message with download command
            if message.contains("🖼️") && message.starts_with("/download") {
                let parts: Vec<&str> = message.split_whitespace().collect();
                if parts.len() >= 2 {
                    let share_code = parts[1];
                    println!(
                        "🖼️ [{}/{}]: {} ({}) sent image, downloading...",
                        group, from_nickname, from, from_nickname
                    );

                    // Download the file (need to use a mutable reference)
                    // For now, we'll just print the command for the user to execute manually
                    println!(
                        "💡 To download this image, run: download {} {}",
                        from_nickname, share_code
                    );
                    return;
                }
            }

            println!(
                "📢 [{}/{}]: {} ({}): {}",
                group, from_nickname, from, from_nickname, message
            );
        }
        P2pEvent::GroupImageMessage {
            from,
            from_nickname,
            group,
            filename,
            data,
            message,
        } => {
            // Check if this is the new format with share code (data is empty)
            // or legacy format with embedded image data
            if data.is_empty() {
                // New format - extract share code from message and show download hint
                if message.contains("🖼️") && message.starts_with("/download") {
                    let parts: Vec<&str> = message.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let share_code = parts[1];
                        println!(
                            "🖼️ [{}/{}]: {} ({}) sent image: {} - Use 'download {} {}' to get it",
                            group,
                            from_nickname,
                            from,
                            from_nickname,
                            filename,
                            from_nickname,
                            share_code
                        );
                    } else {
                        println!(
                            "🖼️ [{}/{}]: {} ({}) sent image: {}",
                            group, from_nickname, from, from_nickname, filename
                        );
                    }
                } else {
                    println!(
                        "🖼️ [{}/{}]: {} ({}) sent image: {}",
                        group, from_nickname, from, from_nickname, filename
                    );
                }
            } else {
                // Legacy mode - save directly embedded image data
                let file_path = output_dir.join(&filename);
                match fs::write(&file_path, &data).await {
                    Ok(()) => {
                        println!(
                            "🖼️ [{}/{}]: {} ({}) sent image: {} (saved to {})",
                            group,
                            from_nickname,
                            from,
                            from_nickname,
                            filename,
                            file_path.display()
                        );
                    }
                    Err(e) => {
                        println!(
                            "❌ Failed to save group image from {} ({}): {}",
                            from_nickname, from, e
                        );
                    }
                }
            }
        }
        P2pEvent::GroupJoined { group } => {
            println!("✅ Joined group: {}", group);
        }
        P2pEvent::GroupLeft { group } => {
            println!("🚪 Left group: {}", group);
        }
        P2pEvent::FileShared { file_id, info } => {
            println!(
                "📎 File shared: {} (ID: {}) - {} bytes",
                info.name, file_id, info.size
            );
        }
        P2pEvent::FileInfoReceived { from, info } => {
            println!(
                "📋 File info received from {}: {} ({} chunks)",
                from, info.name, info.chunk_count
            );
        }
        P2pEvent::ChunkReceived {
            from,
            file_id,
            chunk_index,
            chunk: _,
        } => {
            println!(
                "🧩 Received chunk {} for file {} from {}",
                chunk_index, file_id, from
            );
        }
        P2pEvent::FileDownloadStarted {
            from,
            from_nickname,
            filename,
        } => {
            println!(
                "⬇️ Downloading {} from {} ({})...",
                filename, from_nickname, from
            );
        }
        P2pEvent::FileDownloadProgress {
            file_id,
            downloaded_chunks,
            total_chunks,
        } => {
            let progress = (downloaded_chunks as f32 / total_chunks as f32) * 100.0;
            println!(
                "📊 Download progress for {}: {:.1}% ({}/{})",
                file_id, progress, downloaded_chunks, total_chunks
            );
        }
        P2pEvent::FileDownloadCompleted { file_id, path } => {
            println!(
                "✅ Download completed: {} saved to {}",
                file_id,
                path.display()
            );
        }
        P2pEvent::ListeningOn { address } => {
            println!("🎯 Listening on: {}", address);
        }
        P2pEvent::Connected { peer_id, nickname } => {
            println!("🔗 Connected to: {} ({})", nickname, peer_id);
        }
        P2pEvent::Disconnected { peer_id, nickname } => {
            println!("❌ Disconnected from: {} ({})", nickname, peer_id);
        }
        P2pEvent::FileShareRequest {
            from,
            from_nickname,
            share_code,
            filename,
            size,
        } => {
            println!(
                "📎 {} ({}) wants to share file: {} ({} bytes) [Code: {}]",
                from_nickname, from, filename, size, share_code
            );
        }
        P2pEvent::FileRevoked { file_id } => {
            println!("🚫 File revoked: {}", file_id);
        }
        P2pEvent::FileListReceived { from, files } => {
            println!("📋 File list received from {}:", from);
            for file in files {
                println!("  - {} ({} bytes)", file.name, file.size);
            }
        }
        P2pEvent::FileDownloadFailed { file_id, error } => {
            println!("❌ Download failed for {}: {}", file_id, error);
        }
        P2pEvent::Error(err) => {
            println!("❌ Error: {}", err);
        }
    }
}

async fn process_command(input: &str, client: &mut P2pClient) -> bool {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return true;
    }

    match parts[0] {
        "help" | "?" | "h" => {
            show_help();
        }
        "peers" | "p" => {
            println!("\n👥 Connected Peers:");
            let peers = client.list_peers();
            if peers.is_empty() {
                println!("  No peers found. Make sure others are on the same network.");
            } else {
                for (i, peer) in peers.iter().enumerate() {
                    println!("  {}. {} ({})", i + 1, peer.nickname, peer.peer_id);
                }
            }
        }
        "send" | "s" => {
            if parts.len() < 3 {
                println!("❌ Usage: send <nickname> <message>");
            } else {
                let nickname = parts[1];
                let message = parts[2..].join(" ");
                match client.send_direct_message(nickname, message) {
                    Ok(()) => println!("✅ Message sent to {}", nickname),
                    Err(e) => println!("❌ Failed to send: {}", e),
                }
            }
        }
        "send-image" | "si" => {
            if parts.len() < 3 {
                println!("❌ Usage: send-image <nickname> <image-path>");
            } else {
                let nickname = parts[1];
                let image_path = parts[2];
                match client.send_direct_image(nickname, &PathBuf::from(image_path)) {
                    Ok(()) => println!("✅ Image sent to {}", nickname),
                    Err(e) => println!("❌ Failed to send image: {}", e),
                }
            }
        }
        "join" | "j" => {
            if parts.len() < 2 {
                println!("❌ Usage: join <group>");
            } else {
                let group = parts[1];
                match client.join_group(group) {
                    Ok(()) => println!("✅ Joined group: {}", group),
                    Err(e) => println!("❌ Failed to join group: {}", e),
                }
            }
        }
        "leave" | "l" => {
            if parts.len() < 2 {
                println!("❌ Usage: leave <group>");
            } else {
                let group = parts[1];
                match client.leave_group(group) {
                    Ok(()) => println!("✅ Left group: {}", group),
                    Err(e) => println!("❌ Failed to leave group: {}", e),
                }
            }
        }
        "send-group" | "sg" => {
            if parts.len() < 3 {
                println!("❌ Usage: send-group <group> <message>");
            } else {
                let group = parts[1];
                let message = parts[2..].join(" ");
                match client.send_group_message(group, message) {
                    Ok(()) => println!("✅ Message sent to group: {}", group),
                    Err(e) => println!("❌ Failed to send to group: {}", e),
                }
            }
        }
        "send-group-image" | "sgi" => {
            if parts.len() < 3 {
                println!("❌ Usage: send-group-image <group> <image-path>");
            } else {
                let group = parts[1];
                let image_path = parts[2];
                match client
                    .send_group_image(group, &PathBuf::from(image_path))
                    .await
                {
                    Ok(()) => println!("✅ Image sent to group: {}", group),
                    Err(e) => println!("❌ Failed to send image to group: {}", e),
                }
            }
        }
        "share" | "sh" => {
            if parts.len() < 2 {
                println!("❌ Usage: share <file-path>");
            } else {
                let file_path = parts[1];
                match client.share_file(&PathBuf::from(file_path)).await {
                    Ok(share_code) => println!("✅ File shared with code: {}", share_code),
                    Err(e) => println!("❌ Failed to share file: {}", e),
                }
            }
        }
        "unshare" | "ush" => {
            if parts.len() < 2 {
                println!("❌ Usage: unshare <share-code>");
            } else {
                let share_code = parts[1];
                match client.unshare_file(share_code) {
                    Ok(()) => println!("✅ File with code {} is no longer shared", share_code),
                    Err(e) => println!("❌ Failed to unshare file: {}", e),
                }
            }
        }
        "files" | "f" => {
            println!("\n📁 Shared files:");
            let files = client.list_shared_files();
            if files.is_empty() {
                println!("  No files shared yet.");
            } else {
                for (i, file) in files.iter().enumerate() {
                    println!(
                        "  {}. {} ({}) - {} bytes",
                        i + 1,
                        file.info.name,
                        file.share_code,
                        file.info.size
                    );
                }
            }
        }
        "download" | "d" => {
            if parts.len() < 3 {
                println!("❌ Usage: download <nickname> <share-code>");
            } else {
                let nickname = parts[1];
                let share_code = parts[2];
                match client.download_file(nickname, share_code) {
                    Ok(()) => println!(
                        "✅ Download started from {} with code {}",
                        nickname, share_code
                    ),
                    Err(e) => println!("❌ Failed to start download: {}", e),
                }
            }
        }
        "quit" | "exit" | "q" => {
            println!("👋 Goodbye!");
            return false;
        }
        _ => {
            if input.starts_with('/') {
                println!(
                    "❌ Unknown command: {}. Type 'help' for available commands.",
                    parts[0]
                );
            } else {
                println!("💬 You: {}", input);
            }
        }
    }
    true
}

fn display_welcome(nickname: &str, port: u16, output_dir: &PathBuf, shared_file: &PathBuf) {
    println!("🎯 Gigi P2P Chat - {}", nickname);
    if port == 0 {
        println!("📡 Port: Random (assigned by OS)");
    } else {
        println!("📡 Port: {}", port);
    }
    println!("💾 Downloads: {:?}", output_dir);
    println!("📋 Shared files: {:?}", shared_file);
    println!("🚀 Starting up...\n");

    println!("Welcome to Gigi P2P Chat! Type 'help' for commands.");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\n💬 Enter commands or messages. Use 'help' to see all options.");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize logging with minimal output
    tracing_subscriber::fmt()
        .with_env_filter("error") // Only show errors
        .init();

    // Create output directory
    fs::create_dir_all(&args.output).await?;

    // Create P2P client
    let (mut client, mut event_receiver) = P2pClient::new_with_config(
        libp2p::identity::Keypair::generate_ed25519(),
        args.nickname.clone(),
        args.output.clone(),
        args.shared.clone(),
    )?;

    // Start listening
    let listen_addr = format!("/ip4/0.0.0.0/tcp/{}", args.port).parse()?;
    client.start_listening(listen_addr)?;

    display_welcome(&args.nickname, args.port, &args.output, &args.shared);

    println!("🌐 Local Peer ID: {}", client.local_peer_id());
    println!("🔍 Starting mDNS discovery...");

    // Spawn a simple stdin reader that sends messages via a channel
    let (stdin_sender, mut stdin_receiver) = tokio::sync::mpsc::unbounded_channel::<String>();
    let stdin_handle = tokio::spawn(async move {
        use tokio::io::{AsyncBufReadExt, BufReader};
        let mut stdin = BufReader::new(tokio::io::stdin());
        let mut line = String::new();

        loop {
            line.clear();
            match stdin.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let line = line.trim().to_string();
                    if !line.is_empty() && stdin_sender.send(line).is_err() {
                        break; // Channel closed
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Main loop for handling swarm events and user input
    let mut running = true;
    while running {
        print!("> ");
        io::stdout().flush().unwrap();

        tokio::select! {
            // Handle swarm events through the client
            swarm_event = client.swarm.select_next_some() => {
                if let Err(e) = client.handle_event(swarm_event) {
                    println!("❌ Error handling swarm event: {}", e);
                }
            }

            // Handle P2P events from the event receiver
            Some(event) = event_receiver.next() => {
                handle_p2p_event(event, &args.output, &client).await;
            }

            // Handle stdin input
            Some(input) = stdin_receiver.recv() => {
                if !process_command(&input, &mut client).await {
                    running = false;
                }
            }

            // Handle Ctrl+C
            _ = tokio::signal::ctrl_c() => {
                println!("\n👋 Goodbye!");
                running = false;
            }
        }
    }

    // Clean up
    stdin_handle.abort();

    Ok(())
}
