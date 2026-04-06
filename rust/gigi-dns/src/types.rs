// Copyright 2024 Gigi Team.
//
// Gigi DNS Types
//
// This module defines all types used by the Gigi DNS protocol, including:
// - Configuration constants and structures
// - Peer information
// - DNS events
// - DNS record encoding/decoding

use libp2p::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr},
    time::{Duration, Instant},
};

/// IPv4 multicast address for mDNS (224.0.0.251)
/// This is the standard mDNS multicast address used by Apple Bonjour
pub const IPV4_MDNS_MULTICAST_ADDRESS: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);

/// IPv6 multicast address for mDNS (FF02::FB)
/// This is the IPv6 link-local multicast address for mDNS
pub const IPV6_MDNS_MULTICAST_ADDRESS: Ipv6Addr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0xFB);

/// UDP port for Gigi DNS protocol (7173)
/// This port is used for all Gigi DNS multicast communication
pub const GIGI_DNS_PORT: u16 = 7173;

/// Configuration for Gigi DNS behavior
///
/// This struct contains all configurable parameters for the Gigi DNS protocol.
/// All parameters have validation constraints enforced by the `validate()` method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GigiDnsConfig {
    /// Human-readable nickname for this peer (max 64 characters)
    pub nickname: String,
    /// Time-to-live for DNS records (min: 60s, max: 24h)
    pub ttl: Duration,
    /// Interval between discovery queries (min: 5s, max: 1h)
    pub query_interval: Duration,
    /// Interval between announcements (min: 5s, max: 10min)
    pub announce_interval: Duration,
    /// Interval for cleanup operations (min: 10s, max: 5min)
    pub cleanup_interval: Duration,
    /// Enable IPv6 multicast (disabled by default)
    pub enable_ipv6: bool,
    /// List of capabilities this peer provides (e.g., "file-sharing", "chat")
    pub capabilities: Vec<String>,
    /// Optional metadata key-value pairs for additional peer information
    pub metadata: HashMap<String, String>,
    /// Use localhost unicast instead of multicast for testing
    pub use_localhost: bool,
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
    /// Minimum allowed TTL: 60 seconds
    const MIN_TTL: Duration = Duration::from_secs(60);
    /// Maximum allowed TTL: 24 hours
    const MAX_TTL: Duration = Duration::from_secs(24 * 60 * 60);
    /// Minimum allowed query interval: 5 seconds
    const MIN_QUERY_INTERVAL: Duration = Duration::from_secs(5);
    /// Maximum allowed query interval: 1 hour
    const MAX_QUERY_INTERVAL: Duration = Duration::from_secs(60 * 60);
    /// Minimum allowed announce interval: 5 seconds
    const MIN_ANNOUNCE_INTERVAL: Duration = Duration::from_secs(5);
    /// Maximum allowed announce interval: 10 minutes
    const MAX_ANNOUNCE_INTERVAL: Duration = Duration::from_secs(10 * 60);
    /// Minimum allowed cleanup interval: 10 seconds
    const MIN_CLEANUP_INTERVAL: Duration = Duration::from_secs(10);
    /// Maximum allowed cleanup interval: 5 minutes
    const MAX_CLEANUP_INTERVAL: Duration = Duration::from_secs(5 * 60);
    /// Maximum nickname length: 64 characters
    const MAX_NICKNAME_LENGTH: usize = 64;

    /// Validates the configuration parameters
    ///
    /// Ensures all parameters are within acceptable ranges to prevent
    /// misconfiguration that could lead to network issues or resource exhaustion.
    ///
    /// # Returns
    /// - `Ok(())` - Configuration is valid
    /// - `Err(String)` - Describes which parameter is invalid and why
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

/// Information about a discovered peer
///
/// Contains all information advertised by a peer via DNS TXT records,
/// along with timestamps for discovery and expiration.
#[derive(Debug, Clone)]
pub struct GigiPeerInfo {
    /// libp2p peer ID
    pub peer_id: PeerId,
    /// Human-readable nickname
    pub nickname: String,
    /// libp2p multiaddress for connecting to this peer
    pub multiaddr: Multiaddr,
    /// List of capabilities provided by this peer
    pub capabilities: Vec<String>,
    /// Additional metadata key-value pairs
    pub metadata: HashMap<String, String>,
    /// When this peer was first discovered
    pub discovered_at: Instant,
    /// When this peer's information expires (based on DNS TTL)
    pub expires_at: Instant,
}

/// Events emitted by the Gigi DNS behavior
///
/// These events inform the application about peer lifecycle changes.
#[derive(Debug, Clone)]
pub enum GigiDnsEvent {
    /// A new peer was discovered
    Discovered(GigiPeerInfo),
    /// An existing peer's information was updated
    Updated {
        peer_id: PeerId,
        old_info: GigiPeerInfo,
        new_info: GigiPeerInfo,
    },
    /// A peer's information expired (not seen recently)
    Expired { peer_id: PeerId, info: GigiPeerInfo },
    /// A peer went offline (determined via health check)
    Offline {
        peer_id: PeerId,
        info: GigiPeerInfo,
        reason: OfflineReason,
    },
}

/// Reasons why a peer might go offline
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineReason {
    /// Peer's DNS record TTL expired without renewal
    TtlExpired,
    /// Health check to peer failed
    HealthCheckFailed,
}

/// DNS record format for Gigi peer information
///
/// This struct represents the data encoded in DNS TXT records.
/// The encoding format is: "peer_id=<id> nickname=<name> addr=<addr> caps=<caps> meta=<metadata>"
///
/// Example:
/// ```text
/// peer_id=12D3KooW... nickname=Alice addr=/ip4/192.168.1.10/tcp/7174 caps=file-sharing,chat meta=version:1.0
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GigiDnsRecord {
    /// libp2p peer ID as a string
    pub peer_id: String,
    /// Human-readable nickname
    pub nickname: String,
    /// libp2p multiaddress as a string
    pub addr: String,
    /// Comma-separated list of capabilities
    pub capabilities: String,
    /// Comma-separated key:value pairs for metadata
    pub metadata: String,
}

impl GigiDnsRecord {
    /// Maximum length of encoded TXT record in bytes
    ///
    /// Increased to 4096 to support IPv6 addresses and longer peer IDs.
    /// This is well below typical UDP packet limits (~65KB).
    pub const MAX_TXT_LENGTH: usize = 4096;

    /// Encodes the record into a string suitable for DNS TXT records
    ///
    /// Format: "peer_id=<id> nickname=<name> addr=<addr> caps=<caps> meta=<metadata>"
    ///
    /// Caps and metadata are optional and omitted if empty.
    ///
    /// # Returns
    /// - `Ok(String)` - Encoded record
    /// - `Err(String)` - If encoded length exceeds MAX_TXT_LENGTH
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

    /// Decodes a string into a GigiDnsRecord
    ///
    /// Parses the format: "peer_id=<id> nickname=<name> addr=<addr> caps=<caps> meta=<metadata>"
    ///
    /// # Arguments
    /// * `input` - The encoded record string
    ///
    /// # Returns
    /// - `Ok(GigiDnsRecord)` - Successfully decoded record
    /// - `Err(String)` - Missing required fields (peer_id, nickname, addr)
    pub fn decode(input: &str) -> Result<Self, String> {
        let mut peer_id = None;
        let mut nickname = None;
        let mut addr = None;
        let mut capabilities = String::new();
        let mut metadata = String::new();

        // Parse key=value pairs
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

        // Validate required fields
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
