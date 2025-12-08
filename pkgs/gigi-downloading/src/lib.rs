use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Seek},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::channel::mpsc;
use libp2p::{
    core::Multiaddr,
    request_response::{self, OutboundRequestId, ProtocolSupport, ResponseChannel},
    swarm::{NetworkBehaviour, Swarm},
    PeerId, StreamProtocol,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::fs as tokio_fs;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tracing::warn;
use uuid::Uuid;

pub const CHUNK_SIZE: usize = 256 * 1024; // 256KB chunks for better performance
pub const PROTOCOL_NAME: &str = "/gigi/file-transfer/1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub hash: String,
    pub chunk_count: usize,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    pub file_id: String,
    pub chunk_index: usize,
    pub data: Vec<u8>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    GetFileInfo(String),
    GetChunk(String, usize),
    ListFiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    FileInfo(Option<FileInfo>),
    Chunk(Option<ChunkInfo>),
    FileList(Vec<FileInfo>),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedFile {
    pub info: FileInfo,
    pub path: PathBuf,
    pub revoked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadingFile {
    pub info: FileInfo,
    pub output_path: PathBuf,
    pub temp_path: PathBuf,
    pub downloaded_chunks: HashMap<usize, bool>,
    pub started_at: u64,
    pub next_chunk_to_request: usize,
    pub max_concurrent_requests: usize,
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "ComposedEvent")]
pub struct FileTransferBehaviour {
    request_response: request_response::Behaviour<FileTransferCodec>,
}

#[derive(Debug)]
pub enum ComposedEvent {
    RequestResponse(request_response::Event<Request, Response>),
}

impl From<request_response::Event<Request, Response>> for ComposedEvent {
    fn from(event: request_response::Event<Request, Response>) -> Self {
        ComposedEvent::RequestResponse(event)
    }
}

#[derive(Debug, Clone, Default)]
pub struct FileTransferCodec;

#[async_trait]
impl request_response::Codec for FileTransferCodec {
    type Protocol = StreamProtocol;
    type Request = Request;
    type Response = Response;

    async fn read_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> std::io::Result<Self::Request>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        let mut buf = vec![];
        futures::io::AsyncReadExt::read_to_end(io, &mut buf).await?;
        serde_json::from_slice(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> std::io::Result<Self::Response>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        let mut buf = vec![];
        futures::io::AsyncReadExt::read_to_end(io, &mut buf).await?;
        serde_json::from_slice(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> std::io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        let data = serde_json::to_vec(&req)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        futures::io::AsyncWriteExt::write_all(io, &data).await?;
        futures::io::AsyncWriteExt::close(io).await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> std::io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        let data = serde_json::to_vec(&res)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        futures::io::AsyncWriteExt::write_all(io, &data).await?;
        futures::io::AsyncWriteExt::close(io).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum FileTransferEvent {
    RequestReceived {
        peer: PeerId,
        request: Request,
        channel: ResponseChannel<Response>,
    },
    ResponseReceived {
        peer: PeerId,
        request_id: OutboundRequestId,
        response: Response,
    },
    ChunkReceived {
        peer: PeerId,
        file_id: String,
        chunk_index: usize,
        chunk: ChunkInfo,
    },
    FileShared {
        file_id: String,
        info: FileInfo,
    },
    FileRevoked {
        file_id: String,
    },
    DownloadProgress {
        file_id: String,
        downloaded_chunks: usize,
        total_chunks: usize,
    },
    DownloadCompleted {
        file_id: String,
        path: PathBuf,
    },
    Connected {
        peer_id: PeerId,
    },
    Disconnected {
        peer_id: PeerId,
    },
    Error(String),
}

pub struct FileTransferServer {
    pub swarm: Swarm<FileTransferBehaviour>,
    shared_files: HashMap<String, SharedFile>,
    info_path: PathBuf,
    pub event_sender: mpsc::UnboundedSender<FileTransferEvent>,
}

pub struct FileTransferClient {
    pub swarm: Swarm<FileTransferBehaviour>,
    downloading_files: HashMap<String, DownloadingFile>,
    pending_requests: HashMap<OutboundRequestId, (String, usize)>,
    pub event_sender: mpsc::UnboundedSender<FileTransferEvent>,
    pub server_peer_id: Option<PeerId>,
}

pub struct ServerConfig {
    pub info_path: PathBuf,
    pub listen_port: u16,
}

pub struct ClientConfig {
    pub server_addr: Multiaddr,
}

impl FileTransferServer {
    /// Create a file transfer server with existing swarm
    pub fn with_swarm(
        swarm: Swarm<FileTransferBehaviour>,
        info_path: PathBuf,
    ) -> Result<(Self, mpsc::UnboundedReceiver<FileTransferEvent>)> {
        let (event_sender, event_receiver) = mpsc::unbounded();

        let mut server = Self {
            swarm,
            shared_files: HashMap::new(),
            info_path,
            event_sender,
        };

        server.load_shared_files()?;

        Ok((server, event_receiver))
    }

    /// Create a file transfer behaviour for external swarm creation
    pub fn create_behaviour() -> Result<FileTransferBehaviour> {
        let behaviour = FileTransferBehaviour {
            request_response: request_response::Behaviour::new(
                [(StreamProtocol::new(PROTOCOL_NAME), ProtocolSupport::Full)],
                request_response::Config::default()
                    .with_request_timeout(std::time::Duration::from_secs(60)), // 1 minute per chunk
            ),
        };
        Ok(behaviour)
    }

    pub fn share_file(&mut self, file_path: &Path) -> Result<String> {
        let metadata = fs::metadata(file_path)?;
        let size = metadata.len();
        let name = file_path
            .file_name()
            .ok_or_else(|| anyhow!("Invalid file name"))?
            .to_string_lossy()
            .to_string();

        let chunk_count = (size as usize + CHUNK_SIZE - 1) / CHUNK_SIZE;
        let hash = self.calculate_file_hash(file_path)?;

        // Check if file with same name already exists
        let existing_entry = self
            .shared_files
            .values()
            .find(|shared_file| shared_file.info.name == name && !shared_file.revoked);

        let file_id = if let Some(existing) = existing_entry {
            // Update existing file
            let existing_id = existing.info.id.clone();
            let updated_info = FileInfo {
                id: existing_id.clone(),
                name: name.clone(),
                size,
                hash,
                chunk_count,
                created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            };

            let updated_shared_file = SharedFile {
                info: updated_info.clone(),
                path: file_path.to_path_buf(),
                revoked: false,
            };

            self.shared_files
                .insert(existing_id.clone(), updated_shared_file);
            existing_id
        } else {
            // Create new file entry
            let file_id = Uuid::new_v4().to_string();
            let file_info = FileInfo {
                id: file_id.clone(),
                name: name.clone(),
                size,
                hash,
                chunk_count,
                created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            };

            let shared_file = SharedFile {
                info: file_info.clone(),
                path: file_path.to_path_buf(),
                revoked: false,
            };

            self.shared_files.insert(file_id.clone(), shared_file);
            file_id
        };

        self.save_shared_files()?;

        let file_info = self.shared_files.get(&file_id).unwrap().info.clone();
        let _ = self
            .event_sender
            .unbounded_send(FileTransferEvent::FileShared {
                file_id: file_id.clone(),
                info: file_info,
            });

        Ok(file_id)
    }

    pub fn revoke_file(&mut self, file_id: &str) -> Result<()> {
        if let Some(shared_file) = self.shared_files.get_mut(file_id) {
            shared_file.revoked = true;
            self.save_shared_files()?;

            let _ = self
                .event_sender
                .unbounded_send(FileTransferEvent::FileRevoked {
                    file_id: file_id.to_string(),
                });
            Ok(())
        } else {
            Err(anyhow!("File not found"))
        }
    }

    pub fn list_files(&self) -> Vec<FileInfo> {
        self.shared_files
            .values()
            .filter(|f| !f.revoked)
            .map(|f| f.info.clone())
            .collect()
    }

    /// Handle a single request-response event and return structured events
    pub fn handle_request_response_event(
        &mut self,
        event: request_response::Event<Request, Response>,
    ) -> Result<Vec<FileTransferEvent>> {
        let _events = Vec::<FileTransferEvent>::new();
        match event {
            request_response::Event::Message { message, .. } => {
                match message {
                    request_response::Message::Request {
                        request, channel, ..
                    } => {
                        let response = match request {
                            Request::GetFileInfo(file_id) => {
                                let info = self
                                    .shared_files
                                    .get(&file_id)
                                    .filter(|f| !f.revoked)
                                    .map(|f| f.info.clone());
                                Response::FileInfo(info)
                            }
                            Request::GetChunk(file_id, chunk_index) => {
                                if let Some(shared_file) = self.shared_files.get(&file_id) {
                                    if !shared_file.revoked {
                                        if let Ok(chunk) =
                                            self.read_chunk(&shared_file.path, chunk_index)
                                        {
                                            Response::Chunk(Some(chunk))
                                        } else {
                                            Response::Error("Failed to read chunk".to_string())
                                        }
                                    } else {
                                        Response::Error("File has been revoked".to_string())
                                    }
                                } else {
                                    Response::Chunk(None)
                                }
                            }
                            Request::ListFiles => {
                                let files = self.list_files();
                                Response::FileList(files)
                            }
                        };

                        let _ = self
                            .swarm
                            .behaviour_mut()
                            .request_response
                            .send_response(channel, response);
                    }
                    request_response::Message::Response { .. } => {
                        // Server typically doesn't send requests, so ignore responses
                    }
                }
            }
            request_response::Event::ResponseSent { .. } => {
                // Handle response sent if needed
            }
            request_response::Event::OutboundFailure {
                request_id, error, ..
            } => {
                warn!(
                    "Server outbound failure for request {:?}: {:?}",
                    request_id, error
                );
                // Server doesn't retry failed responses
            }
            request_response::Event::InboundFailure { error, .. } => {
                warn!("Inbound failure: {:?}", error);
            }
        }

        Ok(Vec::new())
    }

    fn read_chunk(&self, file_path: &Path, chunk_index: usize) -> Result<ChunkInfo> {
        let mut file = File::open(file_path)?;
        let offset = chunk_index * CHUNK_SIZE;
        file.seek(std::io::SeekFrom::Start(offset as u64))?;

        let mut buffer = vec![0u8; CHUNK_SIZE];
        let bytes_read = file.read(&mut buffer)?;
        buffer.truncate(bytes_read);

        let hash = blake3::hash(&buffer).to_hex().to_string();

        let file_id = self
            .shared_files
            .iter()
            .find(|(_, f)| f.path == file_path)
            .map(|(id, _)| id.clone())
            .ok_or_else(|| anyhow!("File not found in shared files"))?;

        Ok(ChunkInfo {
            file_id,
            chunk_index,
            data: buffer,
            hash,
        })
    }

    fn calculate_file_hash(&self, file_path: &Path) -> Result<String> {
        let mut file = File::open(file_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    fn load_shared_files(&mut self) -> Result<()> {
        if self.info_path.exists() {
            let content = fs::read_to_string(&self.info_path)?;
            self.shared_files = serde_json::from_str(&content)?;
        }
        Ok(())
    }

    fn save_shared_files(&self) -> Result<()> {
        fs::create_dir_all(
            self.info_path
                .parent()
                .ok_or_else(|| anyhow!("Invalid info path"))?,
        )?;
        let content = serde_json::to_string_pretty(&self.shared_files)?;
        fs::write(&self.info_path, content)?;
        Ok(())
    }
}

impl FileTransferClient {
    /// Create a file transfer client with existing swarm
    pub fn with_swarm(
        swarm: Swarm<FileTransferBehaviour>,
    ) -> Result<(Self, mpsc::UnboundedReceiver<FileTransferEvent>)> {
        let (event_sender, event_receiver) = mpsc::unbounded();

        let client = Self {
            swarm,
            downloading_files: HashMap::new(),
            pending_requests: HashMap::new(),
            event_sender,
            server_peer_id: None,
        };

        Ok((client, event_receiver))
    }

    /// Create a file transfer behaviour for external swarm creation
    pub fn create_behaviour() -> Result<FileTransferBehaviour> {
        let behaviour = FileTransferBehaviour {
            request_response: request_response::Behaviour::new(
                [(StreamProtocol::new(PROTOCOL_NAME), ProtocolSupport::Full)],
                request_response::Config::default()
                    .with_request_timeout(std::time::Duration::from_secs(60)), // 1 minute per chunk
            ),
        };
        Ok(behaviour)
    }

    pub async fn get_file_info(&mut self, file_id: &str) -> Result<()> {
        if let Some(peer) = self.get_connected_peer() {
            let request_id = self
                .swarm
                .behaviour_mut()
                .request_response
                .send_request(&peer, Request::GetFileInfo(file_id.to_string()));
            self.pending_requests
                .insert(request_id, (file_id.to_string(), usize::MAX));
        } else {
            return Err(anyhow!("No connected peers"));
        }
        Ok(())
    }

    pub async fn start_download(&mut self, file_info: FileInfo, output_dir: &Path) -> Result<()> {
        let output_path = output_dir.join(&file_info.name);
        let temp_path = output_path.with_extension("downloading");

        let mut downloaded_chunks = HashMap::new();
        for i in 0..file_info.chunk_count {
            downloaded_chunks.insert(i, false);
        }

        let downloading_file = DownloadingFile {
            info: file_info.clone(),
            output_path: output_path.clone(),
            temp_path: temp_path.clone(),
            downloaded_chunks,
            started_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            next_chunk_to_request: 0,
            max_concurrent_requests: 10,
        };

        self.downloading_files
            .insert(file_info.id.clone(), downloading_file);

        // Create output directory if it doesn't exist
        tokio_fs::create_dir_all(output_dir).await?;

        // Create temp file with expected size
        {
            let temp_file = tokio_fs::File::create(&temp_path).await?;
            temp_file.set_len(file_info.size).await?;
        }

        // Start requesting chunks with a sliding window approach
        let max_concurrent_requests = 10; // Limit concurrent requests
        let mut next_chunk_to_request = 0;

        // Request first batch of chunks
        while next_chunk_to_request < file_info.chunk_count
            && next_chunk_to_request < max_concurrent_requests
        {
            if let Some(peer) = self.get_connected_peer() {
                let request_id = self.swarm.behaviour_mut().request_response.send_request(
                    &peer,
                    Request::GetChunk(file_info.id.clone(), next_chunk_to_request),
                );
                self.pending_requests
                    .insert(request_id, (file_info.id.clone(), next_chunk_to_request));
            }
            next_chunk_to_request += 1;
        }

        // Store the next chunk index to request when chunks complete
        if let Some(downloading_file) = self.downloading_files.get_mut(&file_info.id) {
            downloading_file.next_chunk_to_request = next_chunk_to_request;
            downloading_file.max_concurrent_requests = max_concurrent_requests;
        }

        Ok(())
    }

    /// Handle a single request-response event and return structured events
    pub fn handle_request_response_event(
        &mut self,
        event: request_response::Event<Request, Response>,
    ) -> Result<Vec<FileTransferEvent>> {
        let mut events = Vec::<FileTransferEvent>::new();
        match event {
            request_response::Event::Message { peer, message, .. } => {
                match message {
                    request_response::Message::Response {
                        request_id,
                        response,
                        ..
                    } => {
                        if let Some((file_id, chunk_index)) =
                            self.pending_requests.remove(&request_id)
                        {
                            match response {
                                Response::FileInfo(Some(info)) => {
                                    events.push(FileTransferEvent::ResponseReceived {
                                        peer,
                                        request_id,
                                        response: Response::FileInfo(Some(info)),
                                    });
                                }
                                Response::Chunk(Some(chunk)) => {
                                    // For chunk responses, include the file_id and chunk_index for the application to handle
                                    // Create a special chunk event that includes the metadata
                                    events.push(FileTransferEvent::ChunkReceived {
                                        peer,
                                        file_id,
                                        chunk_index,
                                        chunk,
                                    });
                                }
                                Response::Error(error) => {
                                    events.push(FileTransferEvent::Error(error));
                                }
                                _ => {}
                            }
                        }
                    }
                    request_response::Message::Request { .. } => {
                        // Client typically doesn't handle requests
                    }
                }
            }
            request_response::Event::ResponseSent { .. } => {
                // Handle response sent if needed
            }
            request_response::Event::OutboundFailure {
                request_id, error, ..
            } => {
                warn!(
                    "Client outbound failure for request {:?}: {:?}",
                    request_id, error
                );
            }
            request_response::Event::InboundFailure { error, .. } => {
                warn!("Inbound failure: {:?}", error);
            }
        }

        Ok(events)
    }

    /// Handle chunk received (async method for complex operations)
    pub async fn handle_chunk_received(
        &mut self,
        file_id: String,
        chunk_index: usize,
        chunk: ChunkInfo,
    ) -> Result<Vec<FileTransferEvent>> {
        let mut events = Vec::<FileTransferEvent>::new();
        let mut should_request_next = false;
        let mut next_chunk_to_request = 0;
        let mut download_complete = false;

        if let Some(downloading_file) = self.downloading_files.get_mut(&file_id) {
            // Verify chunk hash
            let calculated_hash = blake3::hash(&chunk.data).to_hex().to_string();
            if calculated_hash != chunk.hash {
                return Err(anyhow!("Chunk hash mismatch"));
            }

            // Write chunk to temp file
            let offset = chunk_index * CHUNK_SIZE;
            let mut temp_file = tokio_fs::OpenOptions::new()
                .write(true)
                .open(&downloading_file.temp_path)
                .await?;
            temp_file
                .seek(std::io::SeekFrom::Start(offset as u64))
                .await?;
            temp_file.write_all(&chunk.data).await?;
            temp_file.flush().await?;

            // Mark chunk as downloaded
            downloading_file.downloaded_chunks.insert(chunk_index, true);

            let downloaded_count = downloading_file
                .downloaded_chunks
                .values()
                .filter(|&&v| v)
                .count();

            let total_chunks = downloading_file.info.chunk_count;

            events.push(FileTransferEvent::DownloadProgress {
                file_id: file_id.clone(),
                downloaded_chunks: downloaded_count,
                total_chunks,
            });

            // Request next chunk if available (sliding window)
            should_request_next = downloading_file.next_chunk_to_request < total_chunks;
            next_chunk_to_request = downloading_file.next_chunk_to_request;

            // Check if download is complete
            download_complete = downloaded_count == total_chunks;
        }

        if should_request_next {
            if let Some(peer) = self.get_connected_peer() {
                let request_id = self.swarm.behaviour_mut().request_response.send_request(
                    &peer,
                    Request::GetChunk(file_id.clone(), next_chunk_to_request),
                );
                self.pending_requests
                    .insert(request_id, (file_id.clone(), next_chunk_to_request));

                // Update next chunk index
                if let Some(df) = self.downloading_files.get_mut(&file_id) {
                    df.next_chunk_to_request += 1;
                }
            }
        }

        // Check if download is complete
        if download_complete {
            if let Some(downloading_file) = self.downloading_files.remove(&file_id) {
                // Verify file hash
                let file_hash = self
                    .calculate_file_hash(&downloading_file.temp_path)
                    .await?;
                if file_hash != downloading_file.info.hash {
                    return Err(anyhow!("File hash verification failed"));
                }

                // Rename temp file to final name
                tokio_fs::rename(&downloading_file.temp_path, &downloading_file.output_path)
                    .await?;

                events.push(FileTransferEvent::DownloadCompleted {
                    file_id: file_id.to_string(),
                    path: downloading_file.output_path,
                });
            }
        }

        Ok(events)
    }

    async fn calculate_file_hash(&self, file_path: &Path) -> Result<String> {
        let mut file = tokio_fs::File::open(file_path).await?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    fn get_connected_peer(&self) -> Option<PeerId> {
        self.server_peer_id
            .or_else(|| self.swarm.connected_peers().next().copied())
    }
}
