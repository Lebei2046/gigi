// Copyright 2024 Gigi Team.
//
// Gigi DNS Types

use libp2p::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr},
    time::{Duration, Instant},
};

pub const IPV4_MDNS_MULTICAST_ADDRESS: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
pub const IPV6_MDNS_MULTICAST_ADDRESS: Ipv6Addr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0xFB);
pub const GIGI_DNS_PORT: u16 = 7173;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GigiDnsConfig {
    pub nickname: String,
    pub ttl: Duration,
    pub query_interval: Duration,
    pub announce_interval: Duration,
    pub cleanup_interval: Duration,
    pub enable_ipv6: bool,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub use_localhost: bool, // Use localhost unicast instead of multicast for testing
}

impl Default for GigiDnsConfig {
    fn default() -> Self {
        let config = Self {
            nickname: "Anonymous".to_string(),
            ttl: Duration::from_secs(6 * 60),
            query_interval: Duration::from_secs(5 * 60),
            announce_interval: Duration::from_secs(15),
            cleanup_interval: Duration::from_secs(30),
            enable_ipv6: false,
            capabilities: Vec::new(),
            metadata: HashMap::new(),
            use_localhost: false,
        };
        config.validate().expect("Default config should be valid");
        config
    }
}

impl GigiDnsConfig {
    const MIN_TTL: Duration = Duration::from_secs(60);
    const MAX_TTL: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours
    const MIN_QUERY_INTERVAL: Duration = Duration::from_secs(5);
    const MAX_QUERY_INTERVAL: Duration = Duration::from_secs(60 * 60); // 1 hour
    const MIN_ANNOUNCE_INTERVAL: Duration = Duration::from_secs(5);
    const MAX_ANNOUNCE_INTERVAL: Duration = Duration::from_secs(10 * 60); // 10 minutes
    const MIN_CLEANUP_INTERVAL: Duration = Duration::from_secs(10);
    const MAX_CLEANUP_INTERVAL: Duration = Duration::from_secs(5 * 60); // 5 minutes
    const MAX_NICKNAME_LENGTH: usize = 64;

    pub fn validate(&self) -> Result<(), String> {
        // Validate nickname
        if self.nickname.is_empty() {
            return Err("Nickname cannot be empty".to_string());
        }
        if self.nickname.len() > Self::MAX_NICKNAME_LENGTH {
            return Err(format!(
                "Nickname too long: {} chars (max: {})",
                self.nickname.len(),
                Self::MAX_NICKNAME_LENGTH
            ));
        }

        // Validate TTL
        if self.ttl < Self::MIN_TTL {
            return Err(format!(
                "TTL too short: {:?} (min: {:?})",
                self.ttl,
                Self::MIN_TTL
            ));
        }
        if self.ttl > Self::MAX_TTL {
            return Err(format!(
                "TTL too long: {:?} (max: {:?})",
                self.ttl,
                Self::MAX_TTL
            ));
        }

        // Validate query_interval
        if self.query_interval < Self::MIN_QUERY_INTERVAL {
            return Err(format!(
                "Query interval too short: {:?} (min: {:?})",
                self.query_interval,
                Self::MIN_QUERY_INTERVAL
            ));
        }
        if self.query_interval > Self::MAX_QUERY_INTERVAL {
            return Err(format!(
                "Query interval too long: {:?} (max: {:?})",
                self.query_interval,
                Self::MAX_QUERY_INTERVAL
            ));
        }

        // Validate announce_interval
        if self.announce_interval < Self::MIN_ANNOUNCE_INTERVAL {
            return Err(format!(
                "Announce interval too short: {:?} (min: {:?})",
                self.announce_interval,
                Self::MIN_ANNOUNCE_INTERVAL
            ));
        }
        if self.announce_interval > Self::MAX_ANNOUNCE_INTERVAL {
            return Err(format!(
                "Announce interval too long: {:?} (max: {:?})",
                self.announce_interval,
                Self::MAX_ANNOUNCE_INTERVAL
            ));
        }

        // Validate cleanup_interval
        if self.cleanup_interval < Self::MIN_CLEANUP_INTERVAL {
            return Err(format!(
                "Cleanup interval too short: {:?} (min: {:?})",
                self.cleanup_interval,
                Self::MIN_CLEANUP_INTERVAL
            ));
        }
        if self.cleanup_interval > Self::MAX_CLEANUP_INTERVAL {
            return Err(format!(
                "Cleanup interval too long: {:?} (max: {:?})",
                self.cleanup_interval,
                Self::MAX_CLEANUP_INTERVAL
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct GigiPeerInfo {
    pub peer_id: PeerId,
    pub nickname: String,
    pub multiaddr: Multiaddr,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub discovered_at: Instant,
    pub expires_at: Instant,
}

#[derive(Debug, Clone)]
pub enum GigiDnsEvent {
    Discovered(GigiPeerInfo),
    Updated {
        peer_id: PeerId,
        old_info: GigiPeerInfo,
        new_info: GigiPeerInfo,
    },
    Expired {
        peer_id: PeerId,
        info: GigiPeerInfo,
    },
    Offline {
        peer_id: PeerId,
        info: GigiPeerInfo,
        reason: OfflineReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineReason {
    TtlExpired,
    HealthCheckFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GigiDnsRecord {
    pub peer_id: String,
    pub nickname: String,
    pub addr: String,
    pub capabilities: String,
    pub metadata: String,
}

impl GigiDnsRecord {
    pub const MAX_TXT_LENGTH: usize = 4096; // Increased to support IPv6 and longer peer_ids

    pub fn encode(&self) -> Result<String, String> {
        let mut parts = Vec::new();
        parts.push(format!("peer_id={}", self.peer_id));
        parts.push(format!("nickname={}", self.nickname));
        parts.push(format!("addr={}", self.addr));

        if !self.capabilities.is_empty() {
            parts.push(format!("caps={}", self.capabilities));
        }

        if !self.metadata.is_empty() {
            parts.push(format!("meta={}", self.metadata));
        }

        let encoded = parts.join(" ");

        if encoded.len() > Self::MAX_TXT_LENGTH {
            return Err(format!(
                "Record too long: {} bytes (max: {})",
                encoded.len(),
                Self::MAX_TXT_LENGTH
            ));
        }

        Ok(encoded)
    }

    pub fn decode(input: &str) -> Result<Self, String> {
        let mut peer_id = None;
        let mut nickname = None;
        let mut addr = None;
        let mut capabilities = String::new();
        let mut metadata = String::new();

        for pair in input.split(' ') {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().ok_or("Missing key")?;
            let value = parts.next().unwrap_or_default();

            match key {
                "peer_id" => peer_id = Some(value.to_string()),
                "nickname" => nickname = Some(value.to_string()),
                "addr" => addr = Some(value.to_string()),
                "caps" => capabilities = value.to_string(),
                "meta" => metadata = value.to_string(),
                _ => {}
            }
        }

        let peer_id = peer_id.ok_or("Missing peer_id")?;
        let nickname = nickname.ok_or("Missing nickname")?;
        let addr = addr.ok_or("Missing addr")?;

        Ok(GigiDnsRecord {
            peer_id,
            nickname,
            addr,
            capabilities,
            metadata,
        })
    }
}
