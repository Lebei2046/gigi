use gigi_file_sharing::{FileSharingManager, CHUNK_SIZE};
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create a temporary file to share
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "Hello, World!")?;
    let file_path = PathBuf::from(temp_file.path());

    // Create a new file sharing manager
    let mut manager = FileSharingManager::new();

    // Share the file
    println!("Sharing file: {:?}", file_path);
    let share_code = manager.share_file(&file_path).await?;
    println!("Share code: {}", share_code);

    // List shared files
    println!("\nShared files:");
    for shared_file in manager.list_shared_files() {
        println!(
            "  - {} ({} bytes)",
            shared_file.info.name, shared_file.info.size
        );
        println!("    Share code: {}", shared_file.share_code);
        println!("    Hash: {}", shared_file.info.hash);
        println!("    Chunks: {}", shared_file.info.chunk_count);
    }

    // Generate a share code for a filename
    let generated_code = manager.generate_share_code("example.txt");
    println!("\nGenerated share code: {}", generated_code);

    // Calculate file hash
    let hash = manager.calculate_file_hash(&file_path)?;
    println!("File hash: {}", hash);

    println!("\nCHUNK_SIZE: {} bytes", CHUNK_SIZE);

    Ok(())
}
