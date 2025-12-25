//! Network behaviour and protocols

use blake3::Hasher;
use libp2p::{
    gossipsub::{self, MessageAuthenticity, MessageId, ValidationMode},
    mdns::{self},
    request_response::{self},
    swarm::NetworkBehaviour,
};
use serde::{Deserialize, Serialize};

/// Nickname exchange messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NicknameRequest {
    AnnounceNickname { nickname: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NicknameResponse {
    Ack { nickname: String },
    Error(String),
}

/// Direct messaging messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DirectMessage {
    Text {
        message: String,
    },
    FileShare {
        share_code: String,
        filename: String,
        file_size: u64,
        file_type: String,
    },
    ShareGroup {
        group_id: String,
        group_name: String,
        inviter_nickname: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DirectResponse {
    Ack,
    Error(String),
}

/// File transfer messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileTransferRequest {
    GetFileInfo(String),
    GetChunk(String, usize),
    ListFiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileTransferResponse {
    FileInfo(Option<super::events::FileInfo>),
    Chunk(Option<super::events::ChunkInfo>),
    FileList(Vec<super::events::FileInfo>),
    Error(String),
}

/// Unified network behaviour combining all protocols
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "UnifiedEvent")]
pub struct UnifiedBehaviour {
    pub mdns: mdns::tokio::Behaviour,
    pub nickname: request_response::cbor::Behaviour<NicknameRequest, NicknameResponse>,
    pub direct_msg: request_response::cbor::Behaviour<DirectMessage, DirectResponse>,
    pub gossipsub: gossipsub::Behaviour,
    pub file_transfer: request_response::cbor::Behaviour<FileTransferRequest, FileTransferResponse>,
}

/// Unified event from network behaviour
#[derive(Debug)]
pub enum UnifiedEvent {
    Mdns(mdns::Event),
    Nickname(request_response::Event<NicknameRequest, NicknameResponse>),
    DirectMessage(request_response::Event<DirectMessage, DirectResponse>),
    Gossipsub(gossipsub::Event),
    FileTransfer(request_response::Event<FileTransferRequest, FileTransferResponse>),
}

impl From<mdns::Event> for UnifiedEvent {
    fn from(event: mdns::Event) -> Self {
        Self::Mdns(event)
    }
}

impl From<request_response::Event<NicknameRequest, NicknameResponse>> for UnifiedEvent {
    fn from(event: request_response::Event<NicknameRequest, NicknameResponse>) -> Self {
        Self::Nickname(event)
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

impl From<request_response::Event<FileTransferRequest, FileTransferResponse>> for UnifiedEvent {
    fn from(event: request_response::Event<FileTransferRequest, FileTransferResponse>) -> Self {
        Self::FileTransfer(event)
    }
}

/// Create gossipsub configuration
pub fn create_gossipsub_config(
    _keypair: &libp2p::identity::Keypair,
) -> Result<gossipsub::Config, Box<dyn std::error::Error>> {
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(std::time::Duration::from_secs(10))
        .validation_mode(ValidationMode::Strict)
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
