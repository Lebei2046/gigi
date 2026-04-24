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
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

use crate::behaviour::UnifiedBehaviour;
use crate::error::P2pError;
use crate::events::{P2pEvent, PeerInfo};

/// Peer management functionality
///
/// Maintains peer state including discovery, connection status, and metadata.
/// Supports bidirectional lookups: peer ID ↔ nickname.
/// Uses LRU cache for unconnected peers to prevent memory leaks.
pub struct PeerManager {
    /// Map of peer ID to peer info (for connected peers only)
    peers: HashMap<PeerId, PeerInfo>,
    /// Map of nickname to peer ID for reverse lookup (for connected peers only)
    nickname_to_peer: HashMap<String, PeerId>,
    /// LRU cache for unconnected peers (limited to prevent memory leaks)
    unconnected_peers: LruCache<PeerId, PeerInfo>,
}

impl PeerManager {
    /// Create a new peer manager with empty peer tables
    pub fn new() -> Self {
        // Limit unconnected peers to 1000 to prevent memory leaks
        let capacity = NonZeroUsize::new(1000).unwrap();
        Self {
            peers: HashMap::new(),
            nickname_to_peer: HashMap::new(),
            unconnected_peers: LruCache::new(capacity),
        }
    }

    /// Handle peer discovery from gigi-dns
    ///
    /// When a peer is discovered via mDNS:
    /// 1. Check if peer is already tracked
    /// 2. If new, add to LRU cache (unconnected peers) or main map (connected peers)
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
        // Check if peer is already in connected peers
        if !self.peers.contains_key(&peer_id) {
            // Check if peer is already in unconnected cache
            if !self.unconnected_peers.contains(&peer_id) {
                let nickname = nickname.to_string();

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

                // Store in LRU cache for unconnected peers
                self.unconnected_peers.put(peer_id, peer_info);

                // Attempt to dial the discovered peer
                if let Err(e) = swarm.dial(addr.clone()) {
                    gigi_logging::warn!("Failed to dial discovered peer {}: {}", peer_id, e);
                } else {
                    gigi_logging::info!("Dialing discovered peer: {}", peer_id);
                }
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
        // First check connected peers
        if let Some(peer) = self.peers.get(peer_id) {
            return Ok(peer.nickname.clone());
        }

        // Then check unconnected peers
        if let Some(peer) = self.unconnected_peers.peek(peer_id) {
            return Ok(peer.nickname.clone());
        }

        Err(P2pError::PeerNotFound(*peer_id).into())
    }

    /// Get peer info
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<&PeerInfo> {
        // First check connected peers
        if let Some(peer) = self.peers.get(peer_id) {
            return Some(peer);
        }

        // Then check unconnected peers
        self.unconnected_peers.peek(peer_id)
    }

    /// Get peer info by nickname
    pub fn get_peer_id_by_nickname(&self, nickname: &str) -> Option<PeerId> {
        // First check connected peers
        if let Some(peer_id) = self.nickname_to_peer.get(nickname) {
            return Some(*peer_id);
        }

        // Then check unconnected peers
        for (peer_id, peer) in &self.unconnected_peers {
            if peer.nickname == nickname {
                return Some(*peer_id);
            }
        }

        None
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
        let mut all_peers: Vec<&PeerInfo> = self.peers.values().collect();
        all_peers.extend(self.unconnected_peers.iter().map(|(_, peer)| peer));
        all_peers
    }

    /// Handle peer connection established
    pub fn handle_connection_established(
        &mut self,
        peer_id: PeerId,
        event_sender: &mut futures::channel::mpsc::UnboundedSender<P2pEvent>,
    ) {
        // Check if peer is in unconnected cache and move to connected peers
        if let Some(mut peer) = self.unconnected_peers.pop(&peer_id) {
            peer.connected = true;
            peer.last_seen = Instant::now();

            let nickname = peer.nickname.clone();
            self.nickname_to_peer.insert(nickname.clone(), peer_id);
            self.peers.insert(peer_id, peer);

            let _ = event_sender.unbounded_send(P2pEvent::Connected { peer_id, nickname });
        } else if let Some(peer) = self.peers.get_mut(&peer_id) {
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
        self.peers.len() + self.unconnected_peers.len()
    }

    /// Get connected peers count
    pub fn connected_peers_count(&self) -> usize {
        self.peers.values().filter(|p| p.connected).count()
    }

    /// Cleanup old unconnected peers that haven't been seen recently
    /// This is called periodically to free up memory in the LRU cache
    #[allow(dead_code)] // Available for periodic cleanup
    pub fn cleanup_old_peers(&mut self, max_age: Duration) {
        let now = Instant::now();
        let mut peers_to_remove = Vec::new();

        // Collect peers older than max_age
        for (&peer_id, peer_info) in self.unconnected_peers.iter() {
            if now.duration_since(peer_info.last_seen) > max_age {
                peers_to_remove.push(peer_id);
            }
        }

        // Remove old peers
        for peer_id in peers_to_remove {
            if let Some(peer) = self.unconnected_peers.pop(&peer_id) {
                gigi_logging::debug!("Removing old unconnected peer: {}", peer.nickname);
            }
        }

        // Clean up nickname_to_peer for disconnected peers
        let connected_peer_ids: std::collections::HashSet<PeerId> =
            self.peers.keys().copied().collect();
        self.nickname_to_peer
            .retain(|_, peer_id| connected_peer_ids.contains(peer_id));
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
