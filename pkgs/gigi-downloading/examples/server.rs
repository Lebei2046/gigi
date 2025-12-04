use clap::Parser;
use gigi_downloading::{FileTransferServer, ServerConfig};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory for recording shared file info
    #[arg(short, long)]
    info_path: PathBuf,

    /// Space-separated file paths to share
    #[arg(short, long)]
    files: Vec<PathBuf>,

    /// Port for listening
    #[arg(short, long, default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let args = Args::parse();

    let config = ServerConfig {
        info_path: args.info_path,
        listen_port: args.port,
    };

    let (mut server, mut event_receiver) = FileTransferServer::new(config).await?;

    // Share the specified files
    for file_path in args.files {
        match server.share_file(&file_path) {
            Ok(file_id) => {
                println!("Shared file: {:?} with ID: {}", file_path, file_id);
            }
            Err(e) => {
                eprintln!("Failed to share file {:?}: {}", file_path, e);
            }
        }
    }

    // Print initial file list
    let files = server.list_files();
    println!("Shared files:");
    for file in files {
        println!("  {} - {} bytes - ID: {}", file.name, file.size, file.id);
    }

    // Start event handling in a separate task
    tokio::spawn(async move {
        use futures::StreamExt;
        while let Some(event) = event_receiver.next().await {
            match event {
                gigi_downloading::FileTransferEvent::FileShared { file_id, info } => {
                    println!("File shared: {} (ID: {})", info.name, file_id);
                }
                gigi_downloading::FileTransferEvent::FileRevoked { file_id } => {
                    println!("File revoked: {}", file_id);
                }
                gigi_downloading::FileTransferEvent::Error(error) => {
                    eprintln!("Error: {}", error);
                }
                _ => {}
            }
        }
    });

    // Start the server
    println!("Server listening on port {}", args.port);
    println!("Press Ctrl+C to stop the server");

    // Run the server event loop in a separate task
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.run().await {
            eprintln!("Server error: {}", e);
        }
    });

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    println!("Shutting down server...");
    server_handle.abort();

    Ok(())
}
