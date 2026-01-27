//! Peer management functionality
//!
//! This module manages peer state including:
//! - Peer discovery and tracking
//! - Bidirectional mapping between peer IDs and nicknames
//! - Connection state management
//! - Automatic dialing of discovered peers
//!
//! # Data Structures
//!
//! - **peers**: HashMap<PeerId, PeerInfo> - Lookup by peer ID
//! - **nickname_to_peer**: HashMap<String, PeerId> - Lookup by nickname
//!
//! This dual mapping enables efficient lookups from either direction.

use anyhow::Result;
use libp2p::{multiaddr::Multiaddr, PeerId, Swarm};
use std::collections::HashMap;
use std::time::Instant;

use crate::behaviour::UnifiedBehaviour;
use crate::error::P2pError;
use crate::events::{P2pEvent, PeerInfo};

/// Peer management functionality
///
/// Maintains peer state including discovery, connection status, and metadata.
/// Supports bidirectional lookups: peer ID ↔ nickname.
pub struct PeerManager {
    /// Map of peer ID to peer info
    peers: HashMap<PeerId, PeerInfo>,
    /// Map of nickname to peer ID for reverse lookup
    nickname_to_peer: HashMap<String, PeerId>,
}

impl PeerManager {
    /// Create a new peer manager with empty peer tables
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
            nickname_to_peer: HashMap::new(),
        }
    }

    /// Handle peer discovery from gigi-dns
    ///
    /// When a peer is discovered via mDNS:
    /// 1. Check if peer is already tracked
    /// 2. If new, add to both maps (peer_id ↔ nickname)
    /// 3. Emit PeerDiscovered event
    /// 4. Attempt to dial peer for connection
    ///
    /// # Arguments
    ///
    /// - `peer_id`: Peer's libp2p PeerId
    /// - `addr`: Multiaddr of the peer
    /// - `swarm`: Swarm for dialing the peer
    /// - `nickname`: Peer's nickname (from gigi-dns)
    /// - `event_sender`: Channel for emitting P2pEvents
    pub fn handle_peer_discovered(
        &mut self,
        peer_id: PeerId,
        addr: Multiaddr,
        swarm: &mut Swarm<UnifiedBehaviour>,
        nickname: &str,
        event_sender: &mut futures::channel::mpsc::UnboundedSender<P2pEvent>,
    ) -> Result<()> {
        if !self.peers.contains_key(&peer_id) {
            // Nickname is now provided by gigi-dns, no need to request it
            let nickname = nickname.to_string();

            self.nickname_to_peer.insert(nickname.clone(), peer_id);

            let _ = event_sender.unbounded_send(P2pEvent::PeerDiscovered {
                peer_id,
                nickname: nickname.clone(),
                address: addr.clone(),
            });

            let peer_info = PeerInfo {
                peer_id,
                nickname,
                addresses: vec![addr.clone()],
                last_seen: Instant::now(),
                connected: false,
            };

            self.peers.insert(peer_id, peer_info);

            // Attempt to dial the discovered peer
            if let Err(e) = swarm.dial(addr.clone()) {
                tracing::warn!("Failed to dial discovered peer {}: {}", peer_id, e);
            } else {
                tracing::info!("Dialing discovered peer: {}", peer_id);
            }
        }
        Ok(())
    }

    /// Handle peer expiration
    pub fn handle_peer_expired(
        &mut self,
        peer_id: PeerId,
        event_sender: &mut futures::channel::mpsc::UnboundedSender<P2pEvent>,
    ) -> Result<()> {
        if let Some(peer) = self.peers.remove(&peer_id) {
            self.nickname_to_peer.remove(&peer.nickname);

            let _ = event_sender.unbounded_send(P2pEvent::PeerExpired {
                peer_id,
                nickname: peer.nickname,
            });
        }
        Ok(())
    }

    /// Update peer nickname
    pub fn update_peer_nickname(
        &mut self,
        peer_id: PeerId,
        nickname: String,
        event_sender: &mut futures::channel::mpsc::UnboundedSender<P2pEvent>,
    ) {
        if let Some(peer) = self.peers.get_mut(&peer_id) {
            let old_nickname = peer.nickname.clone();
            if old_nickname != nickname {
                self.nickname_to_peer.remove(&old_nickname);
                self.nickname_to_peer.insert(nickname.clone(), peer_id);
                peer.nickname = nickname.clone();

                let _ =
                    event_sender.unbounded_send(P2pEvent::NicknameUpdated { peer_id, nickname });
            }
        }
    }

    /// Get peer by nickname
    pub fn get_peer_by_nickname(&self, nickname: &str) -> Result<&PeerInfo> {
        let peer_id = *self
            .nickname_to_peer
            .get(nickname)
            .ok_or_else(|| P2pError::NicknameNotFound(nickname.to_string()))?;
        self.peers
            .get(&peer_id)
            .ok_or_else(|| P2pError::PeerNotFound(peer_id).into())
    }

    /// Get peer nickname
    pub fn get_peer_nickname(&self, peer_id: &PeerId) -> Result<String> {
        self.peers
            .get(peer_id)
            .map(|p| p.nickname.clone())
            .ok_or_else(|| P2pError::PeerNotFound(*peer_id).into())
    }

    /// Get peer info
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<&PeerInfo> {
        self.peers.get(peer_id)
    }

    /// Get peer info by nickname
    pub fn get_peer_id_by_nickname(&self, nickname: &str) -> Option<PeerId> {
        self.nickname_to_peer.get(nickname).copied()
    }

    /// Remove a peer from the peer list
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        if let Some(peer) = self.peers.remove(peer_id) {
            self.nickname_to_peer.remove(&peer.nickname);
        }
    }

    /// Gracefully shutdown all peers and notify
    pub fn shutdown(
        &mut self,
        event_sender: &mut futures::channel::mpsc::UnboundedSender<P2pEvent>,
    ) -> Result<()> {
        // Close all connections and notify peers
        let connected_peers: Vec<PeerId> = self.peers.keys().copied().collect();
        for peer_id in connected_peers {
            if let Some(peer) = self.peers.remove(&peer_id) {
                self.nickname_to_peer.remove(&peer.nickname);
                let _ = event_sender.unbounded_send(P2pEvent::Disconnected {
                    peer_id,
                    nickname: peer.nickname.clone(),
                });
            }
        }

        Ok(())
    }

    /// List all discovered peers
    pub fn list_peers(&self) -> Vec<&PeerInfo> {
        self.peers.values().collect()
    }

    /// Handle peer connection established
    pub fn handle_connection_established(
        &mut self,
        peer_id: PeerId,
        event_sender: &mut futures::channel::mpsc::UnboundedSender<P2pEvent>,
    ) {
        if let Some(peer) = self.peers.get_mut(&peer_id) {
            peer.connected = true;
            peer.last_seen = Instant::now();

            let _ = event_sender.unbounded_send(P2pEvent::Connected {
                peer_id,
                nickname: peer.nickname.clone(),
            });
        }
    }

    /// Handle peer connection closed
    pub fn handle_connection_closed(
        &mut self,
        peer_id: PeerId,
        event_sender: &mut futures::channel::mpsc::UnboundedSender<P2pEvent>,
    ) {
        if let Some(peer) = self.peers.remove(&peer_id) {
            self.nickname_to_peer.remove(&peer.nickname);

            let _ = event_sender.unbounded_send(P2pEvent::Disconnected {
                peer_id,
                nickname: peer.nickname.clone(),
            });
        }
    }

    /// Get peers count
    pub fn peers_count(&self) -> usize {
        self.peers.len()
    }

    /// Get connected peers count
    pub fn connected_peers_count(&self) -> usize {
        self.peers.values().filter(|p| p.connected).count()
    }

    /// Get all connected peers
    pub fn get_connected_peers(&self) -> Vec<&PeerInfo> {
        self.peers.values().filter(|p| p.connected).collect()
    }

    /// Check if a peer is connected
    #[allow(dead_code)]
    pub fn is_connected(&self, peer_id: &PeerId) -> bool {
        self.peers
            .get(peer_id)
            .map(|p| p.connected)
            .unwrap_or(false)
    }
}

impl Default for PeerManager {
    fn default() -> Self {
        Self::new()
    }
}
