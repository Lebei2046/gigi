//! Connection recovery with exponential backoff
//!
//! This module provides automatic reconnection logic for peers that disconnect.
//! Uses exponential backoff with jitter to avoid thundering herd problems.

use libp2p::{Multiaddr, PeerId, Swarm};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Peer reconnection state
#[derive(Debug, Clone)]
struct ReconnectionState {
    /// The peer's address
    address: Multiaddr,
    /// Current backoff delay
    backoff: Duration,
    /// Time when next reconnection attempt should be made
    next_attempt: Instant,
    /// Number of reconnection attempts made
    attempts: u32,
}

impl ReconnectionState {
    fn new(address: Multiaddr) -> Self {
        Self {
            address,
            backoff: Duration::from_secs(1), // Start with 1 second
            next_attempt: Instant::now(),
            attempts: 0,
        }
    }

    /// Calculate next backoff with exponential increase (max 60 seconds)
    fn next_backoff(&mut self) {
        self.backoff = (self.backoff * 2).min(Duration::from_secs(60));
        self.attempts += 1;
        self.next_attempt = Instant::now() + self.backoff;
    }

    /// Reset backoff after successful connection
    fn reset(&mut self) {
        self.backoff = Duration::from_secs(1);
        self.attempts = 0;
    }

    /// Check if reconnection attempt should be made now
    fn should_attempt_now(&self) -> bool {
        Instant::now() >= self.next_attempt
    }
}

/// Connection recovery manager
///
/// Manages reconnection attempts for disconnected peers with exponential backoff.
pub struct ConnectionRecovery {
    /// Map of peer ID to reconnection state
    reconnecting_peers: HashMap<PeerId, ReconnectionState>,
    /// Maximum number of reconnection attempts before giving up
    max_attempts: u32,
    /// Enable/disable auto-reconnection
    enabled: bool,
}

impl ConnectionRecovery {
    /// Create a new connection recovery manager
    ///
    /// # Arguments
    ///
    /// * `max_attempts` - Maximum reconnection attempts before giving up (0 = infinite)
    pub fn new(max_attempts: u32) -> Self {
        Self {
            reconnecting_peers: HashMap::new(),
            max_attempts,
            enabled: true,
        }
    }

    /// Enable or disable auto-reconnection
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Add a peer for reconnection tracking
    ///
    /// Called when a peer disconnects.
    pub fn peer_disconnected(&mut self, peer_id: PeerId, address: Multiaddr) {
        if self.enabled {
            let state = ReconnectionState::new(address);
            self.reconnecting_peers.insert(peer_id, state);
            tracing::info!("Tracking peer {} for reconnection", peer_id);
        }
    }

    /// Remove peer from reconnection tracking
    ///
    /// Called when a peer successfully reconnects.
    pub fn peer_connected(&mut self, peer_id: &PeerId) {
        if let Some(mut state) = self.reconnecting_peers.remove(peer_id) {
            state.reset();
            tracing::info!(
                "Peer {} reconnected after {} attempts",
                peer_id,
                state.attempts
            );
        }
    }

    /// Process reconnection attempts
    ///
    /// Should be called periodically (e.g., every 5 seconds) from the event loop.
    /// Attempts to reconnect to peers that are due for reconnection.
    ///
    /// # Arguments
    ///
    /// * `swarm` - The libp2p swarm for dialing peers
    ///
    /// # Returns
    ///
    /// Number of reconnection attempts made
    pub fn process_reconnections(
        &mut self,
        swarm: &mut Swarm<crate::behaviour::UnifiedBehaviour>,
    ) -> usize {
        if !self.enabled {
            return 0;
        }

        let mut attempts_made = 0;

        // Collect peers to reconnect
        let peers_to_reconnect: Vec<(PeerId, Multiaddr)> = self
            .reconnecting_peers
            .iter()
            .filter(|(_, state)| state.should_attempt_now())
            .map(|(peer_id, state)| (*peer_id, state.address.clone()))
            .collect();

        // Attempt reconnection
        for (peer_id, address) in peers_to_reconnect {
            // Check max attempts
            if let Some(state) = self.reconnecting_peers.get(&peer_id) {
                if self.max_attempts > 0 && state.attempts >= self.max_attempts {
                    tracing::warn!(
                        "Giving up on peer {} after {} attempts",
                        peer_id,
                        state.attempts
                    );
                    self.reconnecting_peers.remove(&peer_id);
                    continue;
                }
            }

            // Attempt dial
            match swarm.dial(address.clone()) {
                Ok(_) => {
                    tracing::info!("Attempting reconnection to {} (attempt {})", peer_id, {
                        self.reconnecting_peers
                            .get(&peer_id)
                            .map(|s| s.attempts + 1)
                            .unwrap_or(1)
                    });
                    attempts_made += 1;
                }
                Err(e) => {
                    tracing::warn!("Failed to dial peer {}: {}", peer_id, e);
                }
            }

            // Update backoff
            if let Some(state) = self.reconnecting_peers.get_mut(&peer_id) {
                state.next_backoff();
            }
        }

        attempts_made
    }

    /// Get number of peers currently being tracked for reconnection
    pub fn reconnecting_count(&self) -> usize {
        self.reconnecting_peers.len()
    }

    /// Clear all reconnection tracking
    pub fn clear(&mut self) {
        self.reconnecting_peers.clear();
    }
}

impl Default for ConnectionRecovery {
    fn default() -> Self {
        Self::new(10) // Default: max 10 attempts
    }
}
