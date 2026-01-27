//! Network behaviour and protocols
//!
//! This module defines the unified network behaviour that combines multiple libp2p protocols:
//! - **gigi-dns**: Peer discovery with nicknames and metadata
//! - **Direct Messaging**: Request-response protocol for 1-to-1 communication
//! - **GossipSub**: Pub-sub protocol for group messaging
//! - **File Sharing**: Request-response protocol for file chunk transfer
//!
//! # Protocol Details
//!
//! ## Direct Messaging (`/direct/1.0.0`)
//!
//! Simple request-response protocol for direct peer communication:
//!
//! ```text
//! Request                          Response
//! ─────────                        ─────────
//! DirectMessage::Text {           DirectResponse::Ack
//!     message: String
//! }
//!
//! DirectMessage::FileShare {
//!     share_code: String,
//!     filename: String,
//!     file_size: u64,
//!     file_type: String
//! }                           DirectResponse::Ack
//!
//! DirectMessage::ShareGroup {
//!     group_id: String,
//!     group_name: String,
//!     inviter_nickname: String
//! }                           DirectResponse::Ack
//! ```
//!
//! ## File Sharing (`/file/1.0.0`)
//!
//! Pull-based protocol for chunked file transfer:
//!
//! ```text
//! Request                          Response
//! ─────────                        ─────────
//! GetFileInfo(share_code)       FileInfo(Option<FileInfo>)
//!
//! GetChunk(share_code, index)   Chunk(Option<ChunkInfo>)
//!                                 or Chunk(None) if chunk unavailable
//!
//! ListFiles                    FileList(Vec<FileInfo>)
//!                                 or Error(String)
//! ```
//!
//! ## GossipSub Configuration
//!
//! The GossipSub behaviour uses:
//! - **Blake3** for message deduplication (via `message_id_fn`)
//! - **Signed** messages (via `MessageAuthenticity::Signed`)
//! - **Strict validation** to prevent message flood attacks
//! - **10-second heartbeat** for mesh maintenance

use blake3::Hasher;
use gigi_dns::GigiDnsBehaviour;
use libp2p::{
    gossipsub::{self, MessageAuthenticity, MessageId, ValidationMode},
    request_response::{self},
    swarm::NetworkBehaviour,
};
use serde::{Deserialize, Serialize};

/// Direct messaging messages
///
/// Messages sent via the `/direct/1.0.0` protocol for 1-to-1 peer communication.
///
/// # Message Types
///
/// - **Text**: Plain text message
/// - **FileShare**: Announce a file share code to a peer
/// - **ShareGroup**: Invite a peer to join a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DirectMessage {
    Text {
        message: String,
    },
    /// File share announcement with share code and metadata
    /// The receiver should use the share_code to initiate download via `download_file()`
    FileShare {
        share_code: String,
        filename: String,
        file_size: u64,
        file_type: String,
    },
    /// Group invitation with group ID and name
    /// The receiver can use group_id to join via `join_group()`
    ShareGroup {
        group_id: String,
        group_name: String,
        inviter_nickname: String,
    },
}

/// Direct messaging response
///
/// Simple acknowledgement or error response for direct messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DirectResponse {
    /// Message was successfully received
    Ack,
    /// Message processing failed
    Error(String),
}

/// File sharing request messages
///
/// Requests sent via `/file/1.0.0` protocol for file chunk transfer.
///
/// This is a **pull-based** protocol:
/// 1. Sender announces share code via direct/group message
/// 2. Receiver requests file metadata using `GetFileInfo`
/// 3. Receiver requests chunks on-demand using `GetChunk`
/// 4. Receiver can parallelize chunk requests for better performance
///
/// # Request Types
///
/// - **GetFileInfo**: Get file metadata (name, size, hash, chunk count)
/// - **GetChunk**: Get specific chunk (0 to chunk_count-1)
/// - **ListFiles**: Get list of all shared files (for browsing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileSharingRequest {
    /// Request file metadata by share code
    /// Returns FileInfo with chunk count, size, hash, etc.
    GetFileInfo(String),

    /// Request specific chunk by share code and index
    /// Returns ChunkInfo with chunk data and Blake3 hash
    /// Index must be 0 to chunk_count-1
    GetChunk(String, usize),

    /// Request list of all shared files
    /// Returns FileList with all shared files
    ListFiles,
}

/// File sharing response messages
///
/// Responses to file sharing requests.
///
/// # Response Types
///
/// - **FileInfo**: File metadata or None if share code invalid
/// - **Chunk**: Chunk data with hash or None if chunk unavailable
/// - **FileList**: All shared files or error if listing fails
/// - **Error**: General error message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileSharingResponse {
    /// File metadata with chunk count, size, SHA256 hash, MIME type
    /// Returns None if share code is invalid or file not shared
    FileInfo(Option<super::events::FileInfo>),

    /// Chunk data with Blake3 hash for integrity verification
    /// Returns None if chunk index out of range or file not available
    /// Chunks are 256KB each (see `CHUNK_SIZE` constant)
    Chunk(Option<super::events::ChunkInfo>),

    /// List of all shared files
    /// Returns Vec<FileInfo> or Error if listing fails
    FileList(Vec<super::events::FileInfo>),

    /// General error message
    Error(String),
}

/// Unified network behaviour combining all protocols
///
/// Combines four libp2p behaviours into a single NetworkBehaviour implementation.
/// This allows the swarm to handle all protocol events through a unified event stream.
///
/// # Behaviours
///
/// - **gigi_dns**: mDNS-based discovery with nicknames and metadata
/// - **direct_msg**: Request-response for 1-to-1 messaging
/// - **gossipsub**: Pub-sub for group messaging
/// - **file_sharing**: Request-response for chunked file transfer
///
/// # Event Handling
///
/// Each behaviour emits events that are converted to `UnifiedEvent` via `From` implementations.
/// The swarm then forwards these events to the event handler for processing.
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "UnifiedEvent")]
pub struct UnifiedBehaviour {
    /// mDNS-based discovery with nicknames and capabilities
    pub gigi_dns: GigiDnsBehaviour,

    /// Request-response for direct peer communication
    pub direct_msg: request_response::cbor::Behaviour<DirectMessage, DirectResponse>,

    /// Pub-sub for group messaging
    pub gossipsub: gossipsub::Behaviour,

    /// Request-response for chunked file transfer
    pub file_sharing: request_response::cbor::Behaviour<FileSharingRequest, FileSharingResponse>,
}

/// Unified event from network behaviour
///
/// Enum containing all possible events from the four behaviours.
/// Used by the event handler to delegate to specialized handlers.
///
/// # Event Categories
///
/// - **GigiDns**: Peer discovery events (Discovered, Updated, Expired, Offline)
/// - **DirectMessage**: Direct messaging events (requests, responses, failures)
/// - **Gossipsub**: Group messaging events (subscribed, published, etc.)
/// - **FileSharing**: File transfer events (requests, responses, failures)
#[derive(Debug)]
pub enum UnifiedEvent {
    GigiDns(gigi_dns::GigiDnsEvent),
    DirectMessage(request_response::Event<DirectMessage, DirectResponse>),
    Gossipsub(gossipsub::Event),
    FileSharing(request_response::Event<FileSharingRequest, FileSharingResponse>),
}

impl From<gigi_dns::GigiDnsEvent> for UnifiedEvent {
    fn from(event: gigi_dns::GigiDnsEvent) -> Self {
        Self::GigiDns(event)
    }
}

impl From<request_response::Event<DirectMessage, DirectResponse>> for UnifiedEvent {
    fn from(event: request_response::Event<DirectMessage, DirectResponse>) -> Self {
        Self::DirectMessage(event)
    }
}

impl From<gossipsub::Event> for UnifiedEvent {
    fn from(event: gossipsub::Event) -> Self {
        Self::Gossipsub(event)
    }
}

impl From<request_response::Event<FileSharingRequest, FileSharingResponse>> for UnifiedEvent {
    fn from(event: request_response::Event<FileSharingRequest, FileSharingResponse>) -> Self {
        Self::FileSharing(event)
    }
}

/// Create gossipsub configuration
///
/// Creates a GossipSub configuration optimized for group messaging.
///
/// # Configuration Details
///
/// - **Heartbeat Interval**: 10 seconds - balances mesh maintenance vs. overhead
/// - **Validation Mode**: Strict - ensures only valid messages are forwarded
/// - **Message ID**: Blake3 hash of message content for deduplication
///
/// # Message ID Function
///
/// Uses Blake3 hash of message data to generate message IDs.
/// This ensures that duplicate messages (with same content) are not propagated,
/// regardless of which peer published them.
///
/// # Arguments
///
/// * `_keypair` - Keypair (not used in config but kept for future extensions)
///
/// # Returns
///
/// * `Ok(Config)` - GossipSub configuration
pub fn create_gossipsub_config(
    _keypair: &libp2p::identity::Keypair,
) -> Result<gossipsub::Config, Box<dyn std::error::Error>> {
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        // Send heartbeat every 10 seconds to maintain mesh
        .heartbeat_interval(std::time::Duration::from_secs(10))
        // Strict validation ensures only valid messages are forwarded
        .validation_mode(ValidationMode::Strict)
        // Use Blake3 hash for message deduplication
        .message_id_fn(|message| {
            let mut hasher = Hasher::new();
            hasher.update(&message.data);
            MessageId::from(hasher.finalize().as_bytes())
        })
        .build()
        .expect("Valid config");

    Ok(gossipsub_config)
}

/// Create gossipsub behaviour
///
/// Creates a GossipSub behaviour with signed messages for authentication.
///
/// # Authentication
///
/// Uses `MessageAuthenticity::Signed(keypair)` to sign all published messages.
/// Peers can verify that messages came from the claimed sender by verifying
/// the signature using the sender's public key.
///
/// # Arguments
///
/// * `keypair` - Ed25519 keypair for signing messages
/// * `config` - GossipSub configuration from `create_gossipsub_config()`
///
/// # Returns
///
/// * `Ok(Behaviour)` - GossipSub behaviour ready for use
/// * `Err(anyhow::Error)` - Failed to create behaviour
pub fn create_gossipsub_behaviour(
    keypair: libp2p::identity::Keypair,
    config: gossipsub::Config,
) -> Result<gossipsub::Behaviour, anyhow::Error> {
    match gossipsub::Behaviour::new(MessageAuthenticity::Signed(keypair), config) {
        Ok(behaviour) => Ok(behaviour),
        Err(e) => Err(anyhow::anyhow!(
            "Failed to create gossipsub behaviour: {}",
            e
        )),
    }
}
