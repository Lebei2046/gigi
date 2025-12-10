//! Network functionality for Gigi Messaging
//! 
//! This module provides network layer abstractions for P2P communications.

use crate::{Error, models::*};
use std::collections::HashMap;

/// Network manager for P2P operations
pub struct NetworkManager {
    peers: HashMap<String, PeerInfo>,
    subscriptions: HashMap<String, Vec<String>>,
}

/// Information about a connected peer
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub id: String,
    pub address: String,
    pub last_seen: u64,
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
            subscriptions: HashMap::new(),
        }
    }

    /// Add a peer to the network
    pub fn add_peer(&mut self, peer_info: PeerInfo) {
        self.peers.insert(peer_info.id.clone(), peer_info);
    }

    /// Remove a peer from the network
    pub fn remove_peer(&mut self, peer_id: &str) -> Option<PeerInfo> {
        self.peers.remove(peer_id)
    }

    /// Get all connected peers
    pub fn get_peers(&self) -> Vec<String> {
        self.peers.keys().cloned().collect()
    }

    /// Subscribe to a topic
    pub fn subscribe(&mut self, topic: String, peer_id: String) {
        self.subscriptions
            .entry(topic.clone())
            .or_insert_with(Vec::new)
            .push(peer_id);
    }

    /// Unsubscribe from a topic
    pub fn unsubscribe(&mut self, topic: &str, peer_id: &str) {
        if let Some(subscribers) = self.subscriptions.get_mut(topic) {
            subscribers.retain(|p| p != peer_id);
        }
    }

    /// Get subscribers for a topic
    pub fn get_subscribers(&self, topic: &str) -> Vec<String> {
        self.subscriptions
            .get(topic)
            .map(|subs| subs.clone())
            .unwrap_or_default()
    }
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_manager() {
        let mut manager = NetworkManager::new();
        
        // Test adding peers
        let peer = PeerInfo {
            id: "peer1".to_string(),
            address: "/ip4/127.0.0.1/tcp/8080".to_string(),
            last_seen: 123456789,
        };
        
        manager.add_peer(peer);
        assert_eq!(manager.get_peers().len(), 1);
        
        // Test subscription
        manager.subscribe("topic1".to_string(), "peer1".to_string());
        let subscribers = manager.get_subscribers("topic1");
        assert_eq!(subscribers.len(), 1);
        assert_eq!(subscribers[0], "peer1");
        
        // Test removal
        let removed = manager.remove_peer("peer1");
        assert!(removed.is_some());
        assert_eq!(manager.get_peers().len(), 0);
    }
}