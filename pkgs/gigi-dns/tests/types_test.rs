// Copyright 2024 Gigi Team.
//
// Tests for Gigi DNS types

use gigi_dns::types::*;
use libp2p::{Multiaddr, PeerId};
use std::time::{Duration, Instant};

#[test]
fn test_gigidnsconfig_validate_default() {
    let config = GigiDnsConfig::default();
    assert!(config.validate().is_ok());
}

#[test]
fn test_gigidnsconfig_validate_nickname_empty() {
    let mut config = GigiDnsConfig::default();
    config.nickname = String::new();
    assert!(config.validate().is_err());
}

#[test]
fn test_gigidnsconfig_validate_nickname_too_long() {
    let mut config = GigiDnsConfig::default();
    config.nickname = "a".repeat(65);
    assert!(config.validate().is_err());
}

#[test]
fn test_gigidnsconfig_validate_nickname_max_length() {
    let mut config = GigiDnsConfig::default();
    config.nickname = "a".repeat(64);
    assert!(config.validate().is_ok());
}

#[test]
fn test_gigidnsconfig_validate_ttl_too_short() {
    let mut config = GigiDnsConfig::default();
    config.ttl = Duration::from_secs(59);
    assert!(config.validate().is_err());
}

#[test]
fn test_gigidnsconfig_validate_ttl_too_long() {
    let mut config = GigiDnsConfig::default();
    config.ttl = Duration::from_secs(24 * 60 * 60 + 1);
    assert!(config.validate().is_err());
}

#[test]
fn test_gigidnsconfig_validate_ttl_boundaries() {
    let mut config = GigiDnsConfig::default();
    config.ttl = Duration::from_secs(60);
    assert!(config.validate().is_ok());

    config.ttl = Duration::from_secs(24 * 60 * 60);
    assert!(config.validate().is_ok());
}

#[test]
fn test_gigidnsconfig_validate_query_interval_too_short() {
    let mut config = GigiDnsConfig::default();
    config.query_interval = Duration::from_secs(4);
    assert!(config.validate().is_err());
}

#[test]
fn test_gigidnsconfig_validate_query_interval_too_long() {
    let mut config = GigiDnsConfig::default();
    config.query_interval = Duration::from_secs(60 * 60 + 1);
    assert!(config.validate().is_err());
}

#[test]
fn test_gigidnsconfig_validate_announce_interval_too_short() {
    let mut config = GigiDnsConfig::default();
    config.announce_interval = Duration::from_secs(4);
    assert!(config.validate().is_err());
}

#[test]
fn test_gigidnsconfig_validate_announce_interval_too_long() {
    let mut config = GigiDnsConfig::default();
    config.announce_interval = Duration::from_secs(10 * 60 + 1);
    assert!(config.validate().is_err());
}

#[test]
fn test_gigidnsconfig_validate_cleanup_interval_too_short() {
    let mut config = GigiDnsConfig::default();
    config.cleanup_interval = Duration::from_secs(9);
    assert!(config.validate().is_err());
}

#[test]
fn test_gigidnsconfig_validate_cleanup_interval_too_long() {
    let mut config = GigiDnsConfig::default();
    config.cleanup_interval = Duration::from_secs(5 * 60 + 1);
    assert!(config.validate().is_err());
}

#[test]
fn test_gigidnsrecord_encode_basic() {
    let record = GigiDnsRecord {
        peer_id: "12D3KooW...".to_string(),
        nickname: "Alice".to_string(),
        addr: "/ip4/192.168.1.10/tcp/7174".to_string(),
        capabilities: String::new(),
        metadata: String::new(),
    };

    let encoded = record.encode().unwrap();
    assert!(encoded.contains("peer_id=12D3KooW..."));
    assert!(encoded.contains("nickname=Alice"));
    assert!(encoded.contains("addr=/ip4/192.168.1.10/tcp/7174"));
}

#[test]
fn test_gigidnsrecord_encode_with_capabilities() {
    let record = GigiDnsRecord {
        peer_id: "12D3KooW...".to_string(),
        nickname: "Alice".to_string(),
        addr: "/ip4/192.168.1.10/tcp/7174".to_string(),
        capabilities: "file-sharing,chat".to_string(),
        metadata: String::new(),
    };

    let encoded = record.encode().unwrap();
    assert!(encoded.contains("caps=file-sharing,chat"));
}

#[test]
fn test_gigidnsrecord_encode_with_metadata() {
    let record = GigiDnsRecord {
        peer_id: "12D3KooW...".to_string(),
        nickname: "Alice".to_string(),
        addr: "/ip4/192.168.1.10/tcp/7174".to_string(),
        capabilities: String::new(),
        metadata: "version:1.0,os:linux".to_string(),
    };

    let encoded = record.encode().unwrap();
    assert!(encoded.contains("meta=version:1.0,os:linux"));
}

#[test]
fn test_gigidnsrecord_encode_too_long() {
    let mut metadata = String::new();
    for _ in 0..5000 {
        metadata.push('a');
    }

    let record = GigiDnsRecord {
        peer_id: "12D3KooW...".to_string(),
        nickname: "Alice".to_string(),
        addr: "/ip4/192.168.1.10/tcp/7174".to_string(),
        capabilities: String::new(),
        metadata,
    };

    assert!(record.encode().is_err());
}

#[test]
fn test_gigidnsrecord_decode_basic() {
    let input = "peer_id=12D3KooW... nickname=Alice addr=/ip4/192.168.1.10/tcp/7174";
    let record = GigiDnsRecord::decode(input).unwrap();

    assert_eq!(record.peer_id, "12D3KooW...");
    assert_eq!(record.nickname, "Alice");
    assert_eq!(record.addr, "/ip4/192.168.1.10/tcp/7174");
    assert!(record.capabilities.is_empty());
    assert!(record.metadata.is_empty());
}

#[test]
fn test_gigidnsrecord_decode_with_capabilities() {
    let input =
        "peer_id=12D3KooW... nickname=Alice addr=/ip4/192.168.1.10/tcp/7174 caps=file-sharing,chat";
    let record = GigiDnsRecord::decode(input).unwrap();

    assert_eq!(record.capabilities, "file-sharing,chat");
}

#[test]
fn test_gigidnsrecord_decode_with_metadata() {
    let input = "peer_id=12D3KooW... nickname=Alice addr=/ip4/192.168.1.10/tcp/7174 meta=version:1.0,os:linux";
    let record = GigiDnsRecord::decode(input).unwrap();

    assert_eq!(record.metadata, "version:1.0,os:linux");
}

#[test]
fn test_gigidnsrecord_decode_missing_peer_id() {
    let input = "nickname=Alice addr=/ip4/192.168.1.10/tcp/7174";
    assert!(GigiDnsRecord::decode(input).is_err());
}

#[test]
fn test_gigidnsrecord_decode_missing_nickname() {
    let input = "peer_id=12D3KooW... addr=/ip4/192.168.1.10/tcp/7174";
    assert!(GigiDnsRecord::decode(input).is_err());
}

#[test]
fn test_gigidnsrecord_decode_missing_addr() {
    let input = "peer_id=12D3KooW... nickname=Alice";
    assert!(GigiDnsRecord::decode(input).is_err());
}

#[test]
fn test_gigidnsrecord_roundtrip() {
    let original = GigiDnsRecord {
        peer_id: "12D3KooW...".to_string(),
        nickname: "Alice".to_string(),
        addr: "/ip4/192.168.1.10/tcp/7174".to_string(),
        capabilities: "file-sharing,chat".to_string(),
        metadata: "version:1.0,os:linux".to_string(),
    };

    let encoded = original.encode().unwrap();
    let decoded = GigiDnsRecord::decode(&encoded).unwrap();

    assert_eq!(decoded.peer_id, original.peer_id);
    assert_eq!(decoded.nickname, original.nickname);
    assert_eq!(decoded.addr, original.addr);
    assert_eq!(decoded.capabilities, original.capabilities);
    assert_eq!(decoded.metadata, original.metadata);
}

#[test]
fn test_constants() {
    assert_eq!(
        IPV4_MDNS_MULTICAST_ADDRESS,
        std::net::Ipv4Addr::new(224, 0, 0, 251)
    );
    assert_eq!(
        IPV6_MDNS_MULTICAST_ADDRESS,
        std::net::Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0xFB)
    );
    assert_eq!(GIGI_DNS_PORT, 7173);
}

#[test]
fn test_gigipeerinfo_creation() {
    let peer_id = PeerId::random();
    let multiaddr: Multiaddr = "/ip4/192.168.1.10/tcp/7174".parse().unwrap();

    let info = GigiPeerInfo {
        peer_id,
        nickname: "Alice".to_string(),
        multiaddr: multiaddr.clone(),
        capabilities: vec!["file-sharing".to_string()],
        metadata: {
            let mut map = std::collections::HashMap::new();
            map.insert("version".to_string(), "1.0".to_string());
            map
        },
        discovered_at: Instant::now(),
        expires_at: Instant::now() + Duration::from_secs(3600),
    };

    assert_eq!(info.peer_id, peer_id);
    assert_eq!(info.nickname, "Alice");
    assert_eq!(info.multiaddr, multiaddr);
    assert_eq!(info.capabilities.len(), 1);
    assert!(info.metadata.contains_key("version"));
}

#[test]
fn test_offline_reason_variants() {
    let ttl_expired = OfflineReason::TtlExpired;
    let health_check_failed = OfflineReason::HealthCheckFailed;

    assert!(ttl_expired == OfflineReason::TtlExpired);
    assert!(health_check_failed == OfflineReason::HealthCheckFailed);
    assert!(ttl_expired != health_check_failed);
}
