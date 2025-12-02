//! Gigi File Transfer Library
//!
//! This library provides file transfer capabilities using libp2p's request-response protocol.
//! It allows peers to transfer files between each other over the network.

use futures::prelude::*;
use libp2p::{
    core::upgrade,
    identity::Keypair,
    noise,
    request_response::{self, Config, ProtocolSupport},
    swarm::SwarmEvent,
    tcp, yamux, Multiaddr, PeerId, Swarm, SwarmBuilder, Transport,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc;

/// Error types for file transfer operations
#[derive(Debug, Error)]
pub enum TransferError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid file data: {0}")]
    InvalidFileData(String),

    #[error("Transfer timeout")]
    Timeout,

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Result type for file transfer operations
pub type Result<T> = std::result::Result<T, TransferError>;

/// File metadata for transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub filename: String,
    pub size: u64,
    pub hash: String, // SHA-256 hash
    pub mime_type: Option<String>,
}

impl FileMetadata {
    /// Create new file metadata
    pub fn new(filename: String, size: u64, hash: String) -> Self {
        Self {
            filename,
            size,
            hash,
            mime_type: None,
        }
    }

    /// Create metadata with MIME type
    pub fn with_mime_type(mut self, mime_type: String) -> Self {
        self.mime_type = Some(mime_type);
        self
    }
}

/// File data chunk for transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunk {
    pub metadata: FileMetadata,
    pub chunk_index: u32,
    pub total_chunks: u32,
    pub data: Vec<u8>, // Base64 encoded
    pub is_last: bool,
}

/// File transfer requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferRequest {
    /// Request file list from peer
    ListFiles,
    /// Request specific file by filename
    RequestFile { filename: String },
    /// Request file chunk
    RequestChunk { filename: String, chunk_index: u32 },
    /// Send file to peer
    SendFile { chunk: FileChunk },
}

/// File transfer responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferResponse {
    /// List of available files
    FileList { files: Vec<FileMetadata> },
    /// File metadata (response to file request)
    FileMetadata { metadata: FileMetadata },
    /// File chunk
    FileChunk { chunk: FileChunk },
    /// Error response
    Error { message: String },
    /// Success acknowledgment
    Success { message: String },
}

/// File transfer manager
pub struct TransferManager {
    swarm: Swarm<request_response::Behaviour<TransferCodec>>,
    available_files: HashMap<String, (FileMetadata, Vec<u8>)>, // filename -> (metadata, data)
    pending_transfers: HashMap<String, (FileMetadata, Vec<FileChunk>)>, // filename -> (metadata, chunks)
    event_sender: Option<mpsc::UnboundedSender<TransferEvent>>,
}

/// Custom codec for file transfer protocol
#[derive(Debug, Clone, Default)]
pub struct TransferCodec;

#[async_trait::async_trait]
impl request_response::Codec for TransferCodec {
    type Protocol = std::borrow::Cow<'static, str>;
    type Request = TransferRequest;
    type Response = TransferResponse;

    async fn read_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> std::io::Result<Self::Request>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        let mut buf = vec![0; 4096];
        let n = io.read(&mut buf).await?;
        buf.truncate(n);
        serde_json::from_slice(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> std::io::Result<Self::Response>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        let mut buf = vec![0; 4096];
        let n = io.read(&mut buf).await?;
        buf.truncate(n);
        serde_json::from_slice(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> std::io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        let data = serde_json::to_vec(&req)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        io.write_all(&data).await?;
        io.close().await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> std::io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        let data = serde_json::to_vec(&res)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        io.write_all(&data).await?;
        io.close().await?;
        Ok(())
    }
}

/// Transfer events
#[derive(Debug, Clone)]
pub enum TransferEvent {
    /// File received successfully
    FileReceived {
        filename: String,
        metadata: FileMetadata,
    },
    /// File chunk received
    ChunkReceived {
        filename: String,
        chunk_index: u32,
        total_chunks: u32,
    },
    /// Transfer started
    TransferStarted { filename: String, peer_id: PeerId },
    /// Transfer completed
    TransferCompleted { filename: String, peer_id: PeerId },
    /// Transfer failed
    TransferFailed { filename: String, error: String },
    /// Peer connected
    PeerConnected { peer_id: PeerId },
    /// Network event
    NetworkEvent { message: String },
}

impl std::fmt::Debug for TransferManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransferManager")
            .field("available_files", &self.available_files.len())
            .field("pending_transfers", &self.pending_transfers.len())
            .field("event_sender", &self.event_sender.is_some())
            .finish()
    }
}

impl TransferManager {
    /// Create a new transfer manager
    pub async fn new() -> Result<Self> {
        let keypair = Keypair::generate_ed25519();
        let _peer_id = PeerId::from(keypair.public());

        // Create request-response behavior
        let behaviour = request_response::Behaviour::new(
            [(
                std::borrow::Cow::Borrowed("/file-transfer/1.0.0"),
                ProtocolSupport::Full,
            )],
            Config::default(),
        );

        // Create swarm using the correct API
        let swarm = SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_other_transport(|_keypair| {
                tcp::tokio::Transport::default()
                    .upgrade(upgrade::Version::V1)
                    .authenticate(
                        noise::Config::new(&_keypair)
                            .expect("Signing libp2p-noise static DH keypair failed."),
                    )
                    .multiplex(yamux::Config::default())
                    .boxed()
            })
            .expect("Failed to create transport")
            .with_behaviour(|_keypair| behaviour)
            .expect("Failed to create behaviour")
            .with_swarm_config(|c| {
                c.with_idle_connection_timeout(std::time::Duration::from_secs(60))
            })
            .build();

        Ok(Self {
            swarm,
            available_files: HashMap::new(),
            pending_transfers: HashMap::new(),
            event_sender: None,
        })
    }

    /// Create a new transfer manager with event channel
    pub async fn new_with_events() -> Result<(Self, mpsc::UnboundedReceiver<TransferEvent>)> {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut manager = Self::new().await?;
        manager.event_sender = Some(tx);
        Ok((manager, rx))
    }

    /// Get local peer ID
    pub fn local_peer_id(&self) -> PeerId {
        *self.swarm.local_peer_id()
    }

    /// Start listening on the given address
    pub fn start_listening(&mut self, addr: Multiaddr) -> Result<()> {
        self.swarm
            .listen_on(addr)
            .map_err(|e| TransferError::NetworkError(e.to_string()))?;
        Ok(())
    }

    /// Add a file to be available for transfer
    pub fn add_file(&mut self, filename: String, data: Vec<u8>) -> Result<FileMetadata> {
        let size = data.len() as u64;
        let hash = format!("{:x}", Sha256::digest(&data));

        let metadata = FileMetadata::new(filename.clone(), size, hash);
        self.available_files
            .insert(filename, (metadata.clone(), data));

        Ok(metadata)
    }

    /// Remove a file from available transfers
    pub fn remove_file(&mut self, filename: &str) -> bool {
        self.available_files.remove(filename).is_some()
    }

    /// Get list of available files
    pub fn get_available_files(&self) -> Vec<FileMetadata> {
        self.available_files
            .values()
            .map(|(metadata, _)| metadata.clone())
            .collect()
    }

    /// Connect to a peer
    pub fn connect(&mut self, addr: Multiaddr) -> Result<()> {
        self.swarm
            .dial(addr)
            .map_err(|e| TransferError::ConnectionFailed(e.to_string()))?;
        Ok(())
    }

    /// Request file list from a peer
    pub fn request_file_list(
        &mut self,
        peer_id: PeerId,
    ) -> Result<request_response::OutboundRequestId> {
        let request_id = self
            .swarm
            .behaviour_mut()
            .send_request(&peer_id, TransferRequest::ListFiles);
        Ok(request_id)
    }

    /// Request a file from a peer
    pub fn request_file(
        &mut self,
        peer_id: PeerId,
        filename: String,
    ) -> Result<request_response::OutboundRequestId> {
        let request_id = self
            .swarm
            .behaviour_mut()
            .send_request(&peer_id, TransferRequest::RequestFile { filename });
        Ok(request_id)
    }

    /// Send a file chunk to a peer
    pub fn send_file_chunk(
        &mut self,
        peer_id: PeerId,
        chunk: FileChunk,
    ) -> Result<request_response::OutboundRequestId> {
        let request_id = self
            .swarm
            .behaviour_mut()
            .send_request(&peer_id, TransferRequest::SendFile { chunk });
        Ok(request_id)
    }

    /// Send a complete file to a peer (splits into chunks)
    pub fn send_file(
        &mut self,
        peer_id: PeerId,
        filename: String,
        data: Vec<u8>,
        chunk_size: usize,
    ) -> Result<Vec<request_response::OutboundRequestId>> {
        let metadata = self.available_files.get(&filename).map(|(m, _)| m.clone());
        if let Some(metadata) = metadata {
            let mut request_ids = Vec::new();
            let total_chunks = (data.len() as f64 / chunk_size as f64).ceil() as u32;

            for (i, chunk_data) in data.chunks(chunk_size).enumerate() {
                let chunk = FileChunk {
                    metadata: metadata.clone(),
                    chunk_index: i as u32,
                    total_chunks,
                    data: chunk_data.to_vec(),
                    is_last: i == total_chunks as usize - 1,
                };

                let request_id = self.send_file_chunk(peer_id, chunk)?;
                request_ids.push(request_id);
            }

            Ok(request_ids)
        } else {
            Err(TransferError::FileNotFound(filename))
        }
    }

    /// Handle swarm events and return transfer events
    pub async fn handle_swarm_events(&mut self) -> Vec<TransferEvent> {
        use futures::StreamExt;
        let mut events = Vec::new();

        // Use timeout to make it non-blocking
        use tokio::time::timeout;
        match timeout(Duration::from_millis(100), self.swarm.next()).await {
            Ok(Some(event)) => {
                match event {
                    SwarmEvent::Behaviour(request_response::Event::Message {
                        message,
                        peer,
                        ..
                    }) => {
                        match message {
                            request_response::Message::Request {
                                request, channel, ..
                            } => {
                                // Handle incoming request
                                let response = self.handle_incoming_request(request, &peer);
                                let _ = self.swarm.behaviour_mut().send_response(channel, response);
                            }
                            request_response::Message::Response { response, .. } => {
                                // Handle incoming response
                                self.handle_incoming_response(response, &peer, &mut events);
                            }
                        }
                    }
                    SwarmEvent::NewListenAddr { address, .. } => {
                        events.push(TransferEvent::NetworkEvent {
                            message: format!("Listening on: {}", address),
                        });
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        events.push(TransferEvent::PeerConnected { peer_id });
                    }
                    _ => {}
                }
            }
            Ok(None) => {
                // Stream ended
            }
            Err(_) => {
                // Timeout, no events this time
            }
        }

        // Send events to channel if available
        if let Some(ref tx) = self.event_sender {
            for event in &events {
                let _ = tx.send(event.clone());
            }
        }

        events
    }

    /// Handle incoming request and return response
    fn handle_incoming_request(
        &mut self,
        request: TransferRequest,
        peer_id: &PeerId,
    ) -> TransferResponse {
        match request {
            TransferRequest::ListFiles => {
                let files = self.get_available_files();
                TransferResponse::FileList { files }
            }
            TransferRequest::RequestFile { filename } => {
                if let Some((metadata, data)) = self.available_files.get(&filename) {
                    // Create a single chunk for the entire file (for simplicity)
                    let chunk = FileChunk {
                        metadata: metadata.clone(),
                        chunk_index: 0,
                        total_chunks: 1,
                        data: data.clone(),
                        is_last: true,
                    };

                    // Send the file as a chunk in the response
                    TransferResponse::FileChunk { chunk }
                } else {
                    TransferResponse::Error {
                        message: format!("File not found: {}", filename),
                    }
                }
            }
            TransferRequest::SendFile { chunk } => {
                let chunk_index = chunk.chunk_index;
                self.handle_received_chunk(chunk.clone(), peer_id);
                TransferResponse::Success {
                    message: format!("Chunk {} received", chunk_index),
                }
            }
            _ => TransferResponse::Error {
                message: "Unsupported request".to_string(),
            },
        }
    }

    /// Handle incoming response
    fn handle_incoming_response(
        &mut self,
        response: TransferResponse,
        peer_id: &PeerId,
        events: &mut Vec<TransferEvent>,
    ) {
        match response {
            TransferResponse::FileList { files } => {
                // Handle file list response
                events.push(TransferEvent::NetworkEvent {
                    message: format!(
                        "Received file list with {} files from {}",
                        files.len(),
                        peer_id
                    ),
                });
            }
            TransferResponse::FileMetadata { metadata } => {
                // Handle file metadata response
                events.push(TransferEvent::TransferStarted {
                    filename: metadata.filename.clone(),
                    peer_id: *peer_id,
                });
            }
            TransferResponse::FileChunk { chunk } => {
                let filename = chunk.metadata.filename.clone();
                let chunk_index = chunk.chunk_index;
                let total_chunks = chunk.total_chunks;
                self.handle_received_chunk(chunk.clone(), peer_id);
                events.push(TransferEvent::ChunkReceived {
                    filename,
                    chunk_index,
                    total_chunks,
                });
            }
            TransferResponse::Error { message } => {
                events.push(TransferEvent::TransferFailed {
                    filename: "unknown".to_string(),
                    error: message,
                });
            }
            TransferResponse::Success { message } => {
                events.push(TransferEvent::NetworkEvent {
                    message: format!("Success: {}", message),
                });
            }
        }
    }

    /// Handle received file chunk
    fn handle_received_chunk(&mut self, chunk: FileChunk, peer_id: &PeerId) {
        let filename = chunk.metadata.filename.clone();

        // Get or create pending transfer
        let transfer = self
            .pending_transfers
            .entry(filename.clone())
            .or_insert_with(|| (chunk.metadata.clone(), Vec::new()));

        transfer.1.push(chunk.clone());

        // Check if transfer is complete
        if chunk.is_last && transfer.1.len() == chunk.total_chunks as usize {
            // Sort chunks by index
            transfer.1.sort_by_key(|c| c.chunk_index);

            // Combine chunks into complete file
            let mut file_data = Vec::new();
            for chunk in &transfer.1 {
                file_data.extend_from_slice(&chunk.data);
            }

            // Verify file integrity
            let hash = format!("{:x}", Sha256::digest(&file_data));
            if hash == chunk.metadata.hash {
                // Add to available files
                self.available_files.insert(
                    filename.clone(),
                    (chunk.metadata.clone(), file_data.clone()),
                );

                // Send event
                if let Some(ref tx) = self.event_sender {
                    let _ = tx.send(TransferEvent::FileReceived {
                        filename: filename.clone(),
                        metadata: chunk.metadata.clone(),
                    });
                    let _ = tx.send(TransferEvent::TransferCompleted {
                        filename: filename.clone(),
                        peer_id: *peer_id,
                    });
                }
            } else {
                let filename_clone = filename.clone();
                if let Some(ref tx) = self.event_sender {
                    let _ = tx.send(TransferEvent::TransferFailed {
                        filename: filename_clone,
                        error: "File integrity check failed".to_string(),
                    });
                }
            }

            // Remove from pending transfers
            self.pending_transfers.remove(&filename);
        }
    }

    /// Get pending transfers
    pub fn get_pending_transfers(&self) -> HashMap<String, (FileMetadata, Vec<FileChunk>)> {
        self.pending_transfers.clone()
    }

    /// Get received file data
    pub fn get_file_data(&self, filename: &str) -> Option<Vec<u8>> {
        self.available_files
            .get(filename)
            .map(|(_, data)| data.clone())
    }

    /// Get the current listening addresses
    pub fn listeners(&self) -> Vec<Multiaddr> {
        self.swarm.listeners().cloned().collect()
    }
}

impl Default for TransferManager {
    fn default() -> Self {
        let keypair = Keypair::generate_ed25519();
        let _peer_id = PeerId::from(keypair.public());

        let behaviour = request_response::Behaviour::new(
            [(
                std::borrow::Cow::Borrowed("/file-transfer/1.0.0"),
                ProtocolSupport::Full,
            )],
            Config::default(),
        );

        let swarm = SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_other_transport(|_keypair| {
                tcp::tokio::Transport::default()
                    .upgrade(upgrade::Version::V1)
                    .authenticate(
                        noise::Config::new(&_keypair)
                            .expect("Signing libp2p-noise static DH keypair failed."),
                    )
                    .multiplex(yamux::Config::default())
                    .boxed()
            })
            .expect("Failed to create transport")
            .with_behaviour(|_keypair| behaviour)
            .expect("Failed to create behaviour")
            .build();

        Self {
            swarm,
            available_files: HashMap::new(),
            pending_transfers: HashMap::new(),
            event_sender: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_metadata_creation() {
        let metadata = FileMetadata::new("test.txt".to_string(), 1024, "abc123".to_string());
        assert_eq!(metadata.filename, "test.txt");
        assert_eq!(metadata.size, 1024);
        assert_eq!(metadata.hash, "abc123");
    }

    #[test]
    fn test_file_chunk_creation() {
        let metadata = FileMetadata::new("test.txt".to_string(), 1024, "abc123".to_string());
        let chunk = FileChunk {
            metadata: metadata.clone(),
            chunk_index: 0,
            total_chunks: 1,
            data: vec![1, 2, 3, 4],
            is_last: true,
        };

        assert_eq!(chunk.chunk_index, 0);
        assert_eq!(chunk.total_chunks, 1);
        assert!(chunk.is_last);
    }

    #[tokio::test]
    async fn test_transfer_manager_creation() {
        let manager = TransferManager::new().await.unwrap();
        let peer_id = manager.local_peer_id();
        assert!(!peer_id.to_string().is_empty());
    }

    #[tokio::test]
    async fn test_add_file() {
        let mut manager = TransferManager::new().await.unwrap();
        let data = b"Hello, World!".to_vec();
        let metadata = manager
            .add_file("test.txt".to_string(), data.clone())
            .unwrap();

        assert_eq!(metadata.filename, "test.txt");
        assert_eq!(metadata.size, data.len() as u64);

        let files = manager.get_available_files();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename, "test.txt");
    }
}
