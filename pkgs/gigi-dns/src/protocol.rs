// Copyright 2024 Gigi Team.
//
// Gigi DNS Protocol - Core logic

use crate::types::*;
use libp2p::{Multiaddr, PeerId};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

pub struct GigiDnsProtocol {
    config: GigiDnsConfig,
    local_peer_id: PeerId,
    discovered_peers: HashMap<PeerId, GigiPeerInfo>,
    pending_queries: HashMap<u16, Instant>,
    next_transaction_id: u16,
    listen_addresses: Vec<Multiaddr>,
}

impl GigiDnsProtocol {
    pub fn new(local_peer_id: PeerId, config: GigiDnsConfig) -> Self {
        Self {
            config,
            local_peer_id,
            discovered_peers: HashMap::new(),
            pending_queries: HashMap::new(),
            next_transaction_id: rand::random(),
            listen_addresses: Vec::new(),
        }
    }

    pub fn build_query(&mut self) -> Vec<u8> {
        let transaction_id = self.next_transaction_id;
        self.next_transaction_id = self.next_transaction_id.wrapping_add(1);

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
            append_u16(&mut packet, txt_value.len() as u16);
            append_u8(&mut packet, txt_value.len() as u8);
            packet.extend_from_slice(txt_value.as_bytes());

            packets.push(packet);
        }

        Ok(packets)
    }

    pub fn handle_packet(&mut self, packet: &[u8]) -> Result<Option<GigiDnsEvent>, String> {
        if packet.len() < 12 {
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
                return Err("Invalid record length".to_string());
            }

            if record_type == 0x0010 {
                let txt_data = &packet[pos + 1..pos + rdlength];

                match String::from_utf8(txt_data.to_vec()) {
                    Ok(txt_str) => match GigiDnsRecord::decode(&txt_str) {
                        Ok(record) => {
                            return Ok(Some(self.process_discovered_peer(record, ttl)?));
                        }
                        Err(e) => {
                            return Err(format!("Failed to decode record: {}", e));
                        }
                    },
                    Err(_) => {
                        return Err("Invalid UTF-8 in TXT record".to_string());
                    }
                }
            }

            pos += rdlength;
        }

        Ok(None)
    }

    pub fn is_query(&self, packet: &[u8]) -> bool {
        if packet.len() < 12 {
            return false;
        }
        let flags = parse_u16(&packet[2..4]);
        flags & 0x8000 == 0
    }

    fn process_discovered_peer(
        &mut self,
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

        if let Some(old_info) = self.discovered_peers.get(&peer_id) {
            if old_info.nickname != new_info.nickname || old_info.multiaddr != new_info.multiaddr {
                let old_info = old_info.clone();
                self.discovered_peers.insert(peer_id, new_info.clone());
                return Ok(GigiDnsEvent::Updated {
                    peer_id,
                    old_info,
                    new_info,
                });
            }
        }

        self.discovered_peers.insert(peer_id, new_info.clone());
        Ok(GigiDnsEvent::Discovered(new_info))
    }

    pub fn update_listen_addresses(&mut self, addresses: Vec<Multiaddr>) {
        self.listen_addresses = addresses;
    }

    pub fn update_nickname(&mut self, nickname: String) {
        self.config.nickname = nickname;
    }

    pub fn get_discovered_peers(&self) -> Vec<&GigiPeerInfo> {
        self.discovered_peers.values().collect()
    }

    pub fn find_peer_by_nickname(&self, nickname: &str) -> Option<&GigiPeerInfo> {
        self.discovered_peers
            .values()
            .find(|p| p.nickname == nickname)
    }

    pub fn find_peer_by_id(&self, peer_id: &PeerId) -> Option<&GigiPeerInfo> {
        self.discovered_peers.get(peer_id)
    }

    pub fn cleanup_expired(&mut self) -> Vec<GigiDnsEvent> {
        let now = Instant::now();
        let mut events = Vec::new();

        let expired: Vec<_> = self
            .discovered_peers
            .iter()
            .filter(|(_, info)| info.expires_at <= now)
            .map(|(peer_id, info)| (*peer_id, info.clone()))
            .collect();

        for (peer_id, info) in expired {
            self.discovered_peers.remove(&peer_id);
            events.push(GigiDnsEvent::Expired { peer_id, info });
        }

        events
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
