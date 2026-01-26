// Copyright 2024 Gigi Team.
//
// Gigi DNS Protocol - Core logic

use crate::types::*;
use libp2p::{Multiaddr, PeerId};
use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

pub struct GigiDnsProtocol {
    config: GigiDnsConfig,
    local_peer_id: PeerId,
    pending_queries: HashMap<u16, Instant>,
    next_transaction_id: u32,
    listen_addresses: Vec<Multiaddr>,
    // Error rate limiting: track recent errors
    recent_errors: VecDeque<Instant>,
}

impl GigiDnsProtocol {
    pub fn new(local_peer_id: PeerId, config: GigiDnsConfig) -> Self {
        Self {
            config,
            local_peer_id,
            pending_queries: HashMap::new(),
            next_transaction_id: rand::random::<u16>() as u32,
            listen_addresses: Vec::new(),
            recent_errors: VecDeque::new(),
        }
    }

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

            let txt_value = record.encode()?;

            let mut packet = Vec::new();

            append_u16(&mut packet, rand::random());
            append_u16(&mut packet, 0x8400);
            append_u16(&mut packet, 0x0000);
            append_u16(&mut packet, 0x0001);
            append_u16(&mut packet, 0x0000);
            append_u16(&mut packet, 0x0000);

            append_qname(&mut packet, crate::SERVICE_NAME);
            append_u16(&mut packet, 0x0010);
            append_u16(&mut packet, 0x0001);
            append_u32(&mut packet, self.config.ttl.as_secs() as u32);

            // DNS TXT record format: RDLENGTH (2 bytes) + TXT-DATA (length-prefixed strings)
            // TXT-DATA consists of one or more <character-string>, each prefixed with length byte (u8)
            // For long strings, we need to split into multiple character-strings (max 255 bytes each)
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

    pub fn handle_packet(&mut self, packet: &[u8]) -> Result<Option<GigiDnsEvent>, String> {
        // Rate limiting: if too many errors recently, silently drop packet
        if self.is_rate_limited() {
            return Ok(None);
        }

        if packet.len() < 12 {
            self.record_error();
            return Err("Packet too short".to_string());
        }

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

        for _ in 0..answers_count {
            while pos < packet.len() && packet[pos] != 0 {
                let len = packet[pos] as usize;
                pos += 1 + len;
            }
            pos += 1;

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

            if pos + rdlength > packet.len() {
                self.record_error();
                return Err("Invalid record length".to_string());
            }

            if record_type == 0x0010 {
                // DNS TXT record: RDLENGTH bytes of TXT-DATA
                // TXT-DATA consists of one or more <character-string>
                // Each character-string: 1 length byte (u8) + up to 255 bytes of data
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

    fn record_error(&mut self) {
        const MAX_ERROR_HISTORY: usize = 20;
        let now = Instant::now();
        self.recent_errors.push_back(now);

        // Cleanup old error records
        while self.recent_errors.len() > MAX_ERROR_HISTORY {
            self.recent_errors.pop_front();
        }
    }

    pub fn is_query(&self, packet: &[u8]) -> bool {
        if packet.len() < 12 {
            return false;
        }
        let flags = parse_u16(&packet[2..4]);
        flags & 0x8000 == 0
    }

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

    pub fn update_listen_addresses(&mut self, addresses: Vec<Multiaddr>) {
        self.listen_addresses = addresses;
    }

    pub fn update_nickname(&mut self, nickname: String) {
        self.config.nickname = nickname;
    }

    pub fn get_discovered_peers(&self) -> Vec<&GigiPeerInfo> {
        Vec::new() // State managed by behaviour
    }

    pub fn find_peer_by_nickname(&self, _nickname: &str) -> Option<&GigiPeerInfo> {
        None // State managed by behaviour
    }

    pub fn find_peer_by_id(&self, _peer_id: &PeerId) -> Option<&GigiPeerInfo> {
        None // State managed by behaviour
    }

    pub fn cleanup_expired(&mut self) -> Vec<GigiDnsEvent> {
        // Cleanup expired pending queries (older than 30 seconds)
        let timeout = Duration::from_secs(30);
        let now = Instant::now();
        self.pending_queries
            .retain(|_, timestamp| now.duration_since(*timestamp) < timeout);

        Vec::new() // State managed by behaviour
    }
}

fn append_u16(packet: &mut Vec<u8>, value: u16) {
    packet.push((value >> 8) as u8);
    packet.push((value & 0xFF) as u8);
}

fn append_u32(packet: &mut Vec<u8>, value: u32) {
    packet.push((value >> 24) as u8);
    packet.push((value >> 16) as u8);
    packet.push((value >> 8) as u8);
    packet.push((value & 0xFF) as u8);
}

fn append_u8(packet: &mut Vec<u8>, value: u8) {
    packet.push(value);
}

fn append_qname(packet: &mut Vec<u8>, name: &[u8]) {
    let parts = name.split(|&b| b == b'.');

    for part in parts {
        if !part.is_empty() {
            packet.push(part.len() as u8);
            packet.extend_from_slice(part);
        }
    }

    packet.push(0);
}

fn parse_u16(data: &[u8]) -> u16 {
    ((data[0] as u16) << 8) | (data[1] as u16)
}

fn parse_u32(data: &[u8]) -> u32 {
    ((data[0] as u32) << 24) | ((data[1] as u32) << 16) | ((data[2] as u32) << 8) | (data[3] as u32)
}
