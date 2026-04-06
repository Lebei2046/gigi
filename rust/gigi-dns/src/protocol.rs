// Copyright 2024 Gigi Team.
//
// Gigi DNS Protocol - Core logic
//
// This module implements the Gigi DNS protocol, which is based on DNS (RFC 1035) with custom TXT records.
// The protocol uses DNS-like queries and responses over UDP multicast for local network peer discovery.
//
// Key features:
// - DNS query/response format for compatibility with existing DNS infrastructure
// - TXT records to encode peer information (peer_id, nickname, multiaddr, capabilities, metadata)
// - Rate limiting to prevent DoS attacks
// - Transaction ID tracking to match responses with queries
//
// Protocol flow:
// 1. Peers periodically send DNS queries to multicast address (224.0.0.251:7173)
// 2. Peers respond with DNS TXT records containing their peer information
// 3. Discovered peers are cached with TTL-based expiration
// 4. Rate limiting prevents excessive response generation

use crate::types::*;
use libp2p::{Multiaddr, PeerId};
use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

/// Gigi DNS protocol handler
///
/// This struct manages the core DNS protocol logic including:
/// - Building DNS queries to discover peers
/// - Building DNS responses with peer information
/// - Parsing received DNS packets and extracting peer information
/// - Rate limiting to prevent abuse
/// - Tracking pending queries and cleanup
pub struct GigiDnsProtocol {
    /// Configuration for this DNS instance (nickname, TTL, intervals, etc.)
    config: GigiDnsConfig,
    /// Our local peer ID (used to skip self-discovery)
    local_peer_id: PeerId,
    /// Map of transaction IDs to query timestamps (for tracking pending queries)
    pending_queries: HashMap<u16, Instant>,
    /// Next transaction ID counter (u32 to prevent wraparound issues)
    next_transaction_id: u32,
    /// List of libp2p listen addresses (included in responses)
    listen_addresses: Vec<Multiaddr>,
    /// Error rate limiting: track recent errors to detect and prevent DoS attacks
    recent_errors: VecDeque<Instant>,
}

impl GigiDnsProtocol {
    /// Creates a new GigiDnsProtocol instance
    ///
    /// # Arguments
    /// * `local_peer_id` - Our own libp2p peer ID
    /// * `config` - Configuration for DNS behavior (TTL, intervals, etc.)
    ///
    /// # Returns
    /// A new GigiDnsProtocol instance initialized with random transaction ID
    pub fn new(local_peer_id: PeerId, config: GigiDnsConfig) -> Self {
        Self {
            config,
            local_peer_id,
            pending_queries: HashMap::new(),
            // Start with random transaction ID to avoid conflicts
            next_transaction_id: rand::random::<u16>() as u32,
            listen_addresses: Vec::new(),
            recent_errors: VecDeque::new(),
        }
    }

    /// Builds a DNS query packet for peer discovery
    ///
    /// The query follows DNS format (RFC 1035):
    /// - Header: Transaction ID, Flags (query=0x0000), QDCOUNT=1, others=0
    /// - Question: QNAME="_gigi-dns._udp.local", QTYPE=0x000C (PTR), QCLASS=0x0001 (IN)
    ///
    /// # Returns
    /// Raw bytes of the DNS query packet
    pub fn build_query(&mut self) -> Vec<u8> {
        let transaction_id = (self.next_transaction_id % 65536) as u16;
        self.next_transaction_id += 1;

        let mut packet = Vec::with_capacity(64);

        append_u16(&mut packet, transaction_id);
        append_u16(&mut packet, 0x0000);
        append_u16(&mut packet, 0x0001);
        append_u16(&mut packet, 0x0000);
        append_u16(&mut packet, 0x0000);
        append_u16(&mut packet, 0x0000);

        append_qname(&mut packet, crate::SERVICE_NAME);
        append_u16(&mut packet, 0x000C);
        append_u16(&mut packet, 0x0001);

        self.pending_queries.insert(transaction_id, Instant::now());

        packet
    }

    /// Builds DNS response packets containing peer information
    ///
    /// One response packet is created for each listen address. Each packet contains
    /// a TXT record with peer information (peer_id, nickname, multiaddr, capabilities, metadata).
    ///
    /// The response follows DNS format (RFC 1035):
    /// - Header: Transaction ID, Flags (response=0x8400), ANCOUNT=1, others=0
    /// - Answer: QNAME, TYPE=0x0010 (TXT), CLASS=0x0001 (IN), TTL, RDLENGTH, TXT-DATA
    ///
    /// TXT record encoding (RFC 1035 section 3.3.14):
    /// - TXT-DATA consists of one or more <character-string>
    /// - Each <character-string> is prefixed with a length byte (u8)
    /// - Maximum length per <character-string> is 255 bytes
    /// - For longer data, split into multiple <character-string>s
    ///
    /// # Returns
    /// - `Ok(Vec<Vec<u8>>)` - One DNS response packet per listen address
    /// - `Err(String)` - If no listen addresses are available
    pub fn build_response(&self) -> Result<Vec<Vec<u8>>, String> {
        if self.listen_addresses.is_empty() {
            return Err("No listen addresses available".to_string());
        }

        let mut packets = Vec::new();

        for addr in &self.listen_addresses {
            let record = GigiDnsRecord {
                peer_id: self.local_peer_id.to_string(),
                nickname: self.config.nickname.clone(),
                addr: addr.to_string(),
                capabilities: self.config.capabilities.join(","),
                metadata: self
                    .config
                    .metadata
                    .iter()
                    .map(|(k, v)| format!("{}:{}", k, v))
                    .collect::<Vec<_>>()
                    .join(","),
            };

            // Encode peer information into DNS TXT record format
            let txt_value = record.encode()?;

            // Build DNS response packet
            let mut packet = Vec::new();

            // DNS Header
            append_u16(&mut packet, rand::random()); // Transaction ID (random for responses)
            append_u16(&mut packet, 0x8400); // Flags: Response, Authoritative answer, Recursion available
            append_u16(&mut packet, 0x0000); // QDCOUNT: 0 questions
            append_u16(&mut packet, 0x0001); // ANCOUNT: 1 answer
            append_u16(&mut packet, 0x0000); // NSCOUNT: 0 authority records
            append_u16(&mut packet, 0x0000); // ARCOUNT: 0 additional records

            // Answer section: QNAME
            append_qname(&mut packet, crate::SERVICE_NAME);
            append_u16(&mut packet, 0x0010); // TYPE: TXT (16)
            append_u16(&mut packet, 0x0001); // CLASS: IN (1)
            append_u32(&mut packet, self.config.ttl.as_secs() as u32); // TTL

            // DNS TXT record format: RDLENGTH (2 bytes) + TXT-DATA (length-prefixed strings)
            // TXT-DATA consists of one or more <character-string>, each prefixed with length byte (u8)
            // For long strings, we need to split into multiple character-strings (max 255 bytes each)
            // This is critical: if we don't split properly, data longer than 255 bytes will be truncated
            let txt_data = txt_value.as_bytes();

            // Calculate total RDLENGTH: length of all character-strings
            // Each character-string: 1 length byte + up to 255 data bytes
            let mut rdlength_calculated = 0;
            let mut pos = 0;
            while pos < txt_data.len() {
                let chunk_size = (txt_data.len() - pos).min(255);
                rdlength_calculated += 1 + chunk_size; // 1 length byte + chunk data
                pos += chunk_size;
            }

            append_u16(&mut packet, rdlength_calculated as u16);

            // Append TXT-DATA as multiple character-strings (max 255 bytes each)
            let mut pos = 0;
            while pos < txt_data.len() {
                let chunk_size = (txt_data.len() - pos).min(255);
                append_u8(&mut packet, chunk_size as u8);
                packet.extend_from_slice(&txt_data[pos..pos + chunk_size]);
                pos += chunk_size;
            }

            packets.push(packet);
        }

        Ok(packets)
    }

    /// Handles an incoming DNS packet and extracts peer information if present
    ///
    /// This method parses DNS packets following RFC 1035 and extracts TXT records
    /// containing peer information. It handles both query packets (for responding)
    /// and response packets (for discovering peers).
    ///
    /// # Arguments
    /// * `packet` - Raw DNS packet bytes
    ///
    /// # Returns
    /// - `Ok(Some(GigiDnsEvent))` - A peer was discovered
    /// - `Ok(None)` - Packet processed but no peer discovered (e.g., query packet)
    /// - `Err(String)` - Invalid packet format (also triggers rate limiting)
    ///
    /// # Rate Limiting
    /// If too many errors occur (more than 10 per minute), the protocol will
    /// silently drop packets to prevent resource exhaustion attacks.
    pub fn handle_packet(&mut self, packet: &[u8]) -> Result<Option<GigiDnsEvent>, String> {
        // Rate limiting: if too many errors recently, silently drop packet
        if self.is_rate_limited() {
            return Ok(None);
        }

        // Basic packet validation
        if packet.len() < 12 {
            self.record_error();
            return Err("Packet too short".to_string());
        }

        // Parse DNS header
        let transaction_id = parse_u16(&packet[0..2]);
        let flags = parse_u16(&packet[2..4]);
        let is_response = flags & 0x8000 != 0;

        if is_response {
            // Handle response packet
            // Remove pending query if exists, but still process the response
            // This allows us to handle announcements from other peers
            self.pending_queries.remove(&transaction_id);
        } else {
            // Handle query packet - return None, will be handled by behaviour to send response
            return Ok(None);
        }

        let answers_count = parse_u16(&packet[6..8]);

        if answers_count == 0 {
            return Ok(None);
        }

        let mut pos = 12;

        // Parse answer section
        for _ in 0..answers_count {
            // Skip QNAME (variable length, terminated by 0 byte)
            while pos < packet.len() && packet[pos] != 0 {
                let len = packet[pos] as usize;
                pos += 1 + len;
            }
            pos += 1; // Skip null terminator

            // Validate we have enough bytes for answer header (TYPE, CLASS, TTL, RDLENGTH)
            if pos + 10 > packet.len() {
                self.record_error();
                break;
            }

            let record_type = parse_u16(&packet[pos..pos + 2]);
            pos += 2;
            let _record_class = parse_u16(&packet[pos..pos + 2]);
            pos += 2;
            let ttl = parse_u32(&packet[pos..pos + 4]);
            pos += 4;
            let rdlength = parse_u16(&packet[pos..pos + 2]) as usize;
            pos += 2;

            // Validate RDLENGTH doesn't exceed packet bounds
            if pos + rdlength > packet.len() {
                self.record_error();
                return Err("Invalid record length".to_string());
            }

            // Process TXT record (TYPE = 0x0010)
            if record_type == 0x0010 {
                // DNS TXT record: RDLENGTH bytes of TXT-DATA
                // TXT-DATA consists of one or more <character-string>
                // Each character-string: 1 length byte (u8) + up to 255 bytes of data
                // We must reassemble all character-strings to get the full TXT data
                let mut txt_data = Vec::new();
                let mut rdlength_pos = pos;
                let rdlength_end = pos + rdlength;

                while rdlength_pos < rdlength_end {
                    if rdlength_pos >= packet.len() {
                        self.record_error();
                        return Err("Invalid TXT record format".to_string());
                    }

                    let chunk_len = packet[rdlength_pos] as usize;
                    rdlength_pos += 1;

                    if rdlength_pos + chunk_len > rdlength_end {
                        self.record_error();
                        return Err("Invalid TXT chunk length".to_string());
                    }

                    txt_data.extend_from_slice(&packet[rdlength_pos..rdlength_pos + chunk_len]);
                    rdlength_pos += chunk_len;
                }

                match String::from_utf8(txt_data) {
                    Ok(txt_str) => match GigiDnsRecord::decode(&txt_str) {
                        Ok(record) => {
                            return Ok(Some(self.process_discovered_peer(record, ttl)?));
                        }
                        Err(e) => {
                            self.record_error();
                            return Err(format!("Failed to decode record: {}", e));
                        }
                    },
                    Err(_) => {
                        self.record_error();
                        return Err("Invalid UTF-8 in TXT record".to_string());
                    }
                }
            }

            pos += rdlength;
        }

        Ok(None)
    }

    /// Checks if the protocol is currently rate-limited due to excessive errors
    ///
    /// Rate limiting prevents resource exhaustion attacks by silently dropping
    /// packets when too many parse errors have occurred recently.
    ///
    /// # Returns
    /// `true` if rate limited (should drop packet), `false` otherwise
    fn is_rate_limited(&self) -> bool {
        const MAX_ERRORS_PER_MINUTE: usize = 10;
        const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);

        let now = Instant::now();
        let recent_count = self
            .recent_errors
            .iter()
            .filter(|&&timestamp| now.duration_since(timestamp) < RATE_LIMIT_WINDOW)
            .count();

        recent_count >= MAX_ERRORS_PER_MINUTE
    }

    /// Records a parse error for rate limiting purposes
    ///
    /// Errors older than 60 seconds are automatically cleaned up.
    fn record_error(&mut self) {
        const MAX_ERROR_HISTORY: usize = 20;
        let now = Instant::now();
        self.recent_errors.push_back(now);

        // Cleanup old error records (keep only last MAX_ERROR_HISTORY)
        while self.recent_errors.len() > MAX_ERROR_HISTORY {
            self.recent_errors.pop_front();
        }
    }

    /// Checks if a DNS packet is a query (not a response)
    ///
    /// DNS packets have a QR flag in the header: 0 = query, 1 = response
    ///
    /// # Returns
    /// `true` if the packet is a query, `false` otherwise
    pub fn is_query(&self, packet: &[u8]) -> bool {
        if packet.len() < 12 {
            return false;
        }
        let flags = parse_u16(&packet[2..4]);
        flags & 0x8000 == 0
    }

    /// Processes a discovered peer from a DNS TXT record
    ///
    /// Parses peer information from a GigiDnsRecord and creates a Discovered event.
    /// Also validates that the peer is not ourselves.
    ///
    /// # Arguments
    /// * `record` - The parsed DNS record containing peer information
    /// * `ttl` - The TTL from the DNS record (used to calculate expiration)
    ///
    /// # Returns
    /// - `Ok(GigiDnsEvent::Discovered)` - Valid peer discovered
    /// - `Err(String)` - Invalid peer ID, self-discovery, or invalid multiaddr
    fn process_discovered_peer(
        &self,
        record: GigiDnsRecord,
        ttl: u32,
    ) -> Result<GigiDnsEvent, String> {
        let peer_id: PeerId = record
            .peer_id
            .parse()
            .map_err(|e| format!("Invalid PeerId: {}", e))?;

        // Skip if discovered self
        if peer_id == self.local_peer_id {
            tracing::debug!("Ignoring self-discovery");
            return Err("Self-discovery".to_string());
        }

        let multiaddr: Multiaddr = record
            .addr
            .parse()
            .map_err(|e| format!("Invalid Multiaddr: {}", e))?;

        let now = Instant::now();
        let expires_at = now + Duration::from_secs(ttl as u64);

        let capabilities = if record.capabilities.is_empty() {
            Vec::new()
        } else {
            record
                .capabilities
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };

        let mut metadata = HashMap::new();
        if !record.metadata.is_empty() {
            for pair in record.metadata.split(',') {
                let mut parts = pair.splitn(2, ':');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    metadata.insert(key.to_string(), value.to_string());
                }
            }
        }

        let new_info = GigiPeerInfo {
            peer_id,
            nickname: record.nickname.clone(),
            multiaddr,
            capabilities,
            metadata,
            discovered_at: now,
            expires_at,
        };

        // Just return Discovered event, behaviour will manage state
        Ok(GigiDnsEvent::Discovered(new_info))
    }

    /// Updates the list of listen addresses to advertise in responses
    ///
    /// Called when the libp2p swarm's listen addresses change.
    ///
    /// # Arguments
    /// * `addresses` - New list of libp2p multiaddrs
    pub fn update_listen_addresses(&mut self, addresses: Vec<Multiaddr>) {
        self.listen_addresses = addresses;
    }

    /// Updates the nickname advertised in DNS responses
    ///
    /// # Arguments
    /// * `nickname` - New nickname string
    pub fn update_nickname(&mut self, nickname: String) {
        self.config.nickname = nickname;
    }

    /// Returns list of discovered peers
    ///
    /// Note: This is a no-op as peer state is managed by GigiDnsBehaviour
    pub fn get_discovered_peers(&self) -> Vec<&GigiPeerInfo> {
        Vec::new() // State managed by behaviour
    }

    /// Finds a peer by nickname
    ///
    /// Note: This is a no-op as peer state is managed by GigiDnsBehaviour
    pub fn find_peer_by_nickname(&self, _nickname: &str) -> Option<&GigiPeerInfo> {
        None // State managed by behaviour
    }

    /// Finds a peer by peer ID
    ///
    /// Note: This is a no-op as peer state is managed by GigiDnsBehaviour
    pub fn find_peer_by_id(&self, _peer_id: &PeerId) -> Option<&GigiPeerInfo> {
        None // State managed by behaviour
    }

    /// Cleanup expired pending queries and expired peers
    ///
    /// Removes pending queries older than 30 seconds.
    /// Peer expiration is handled by GigiDnsBehaviour.
    ///
    /// # Returns
    /// Vector of expired peer events (always empty, managed by behaviour)
    pub fn cleanup_expired(&mut self) -> Vec<GigiDnsEvent> {
        // Cleanup expired pending queries (older than 30 seconds)
        let timeout = Duration::from_secs(30);
        let now = Instant::now();
        self.pending_queries
            .retain(|_, timestamp| now.duration_since(*timestamp) < timeout);

        Vec::new() // State managed by behaviour
    }
}

// DNS utility functions for packet encoding/decoding

/// Appends a 16-bit big-endian integer to a packet buffer
fn append_u16(packet: &mut Vec<u8>, value: u16) {
    packet.push((value >> 8) as u8);
    packet.push((value & 0xFF) as u8);
}

/// Appends a 32-bit big-endian integer to a packet buffer
fn append_u32(packet: &mut Vec<u8>, value: u32) {
    packet.push((value >> 24) as u8);
    packet.push((value >> 16) as u8);
    packet.push((value >> 8) as u8);
    packet.push((value & 0xFF) as u8);
}

/// Appends an 8-bit integer to a packet buffer
fn append_u8(packet: &mut Vec<u8>, value: u8) {
    packet.push(value);
}

/// Appends a DNS QNAME (domain name) to a packet buffer
///
/// DNS QNAME format (RFC 1035 section 4.1.2):
/// - Each label is prefixed with a length byte
/// - Labels are concatenated
/// - Terminated by a null byte (0)
/// - Example: "example.com" -> [7, 'e', 'x', 'a', 'm', 'p', 'l', 'e', 3, 'c', 'o', 'm', 0]
fn append_qname(packet: &mut Vec<u8>, name: &[u8]) {
    let parts = name.split(|&b| b == b'.');

    for part in parts {
        if !part.is_empty() {
            packet.push(part.len() as u8);
            packet.extend_from_slice(part);
        }
    }

    packet.push(0); // Null terminator
}

/// Parses a 16-bit big-endian integer from data
fn parse_u16(data: &[u8]) -> u16 {
    ((data[0] as u16) << 8) | (data[1] as u16)
}

/// Parses a 32-bit big-endian integer from data
fn parse_u32(data: &[u8]) -> u32 {
    ((data[0] as u32) << 24) | ((data[1] as u32) << 16) | ((data[2] as u32) << 8) | (data[3] as u32)
}
