pub mod event_handler;
pub mod file_sharing;
pub mod p2p_client;

// Internal modules (not part of public API)
mod download_manager;
mod group_manager;
mod peer_manager;

pub use file_sharing::{FileChunkReader, FileSharingManager, CHUNK_SIZE};
pub use p2p_client::P2pClient;
