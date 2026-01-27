// Copyright 2024 Gigi Team.
//
// Integration tests for Gigi DNS

use gigi_dns::protocol::GigiDnsProtocol;
use gigi_dns::types::*;
use libp2p::{Multiaddr, PeerId};
use std::time::Duration;

#[test]
fn test_query_response_flow() {
    let peer_id1 = PeerId::random();
    let mut config1 = GigiDnsConfig::default();
    config1.nickname = "Peer1".to_string();
    config1.capabilities = vec!["file-sharing".to_string()];

    let mut peer1 = GigiDnsProtocol::new(peer_id1, config1);
    let addr1: Multiaddr = "/ip4/192.168.1.10/tcp/7174".parse().unwrap();
    peer1.update_listen_addresses(vec![addr1.clone()]);

    // Peer 1 creates a query
    let query = peer1.build_query();
    assert!(!query.is_empty());

    // Peer 1 responds to query
    let responses = peer1.build_response().unwrap();
    assert_eq!(responses.len(), 1);
    assert!(!responses[0].is_empty());

    // Parse the response packet
    let response = &responses[0];

    // Verify it's a response packet
    let flags = ((response[2] as u16) << 8) | (response[3] as u16);
    assert_ne!(flags & 0x8000, 0);

    // Verify it has one answer
    let ancount = ((response[6] as u16) << 8) | (response[7] as u16);
    assert_eq!(ancount, 1);
}

#[test]
fn test_peer_discovery_from_response() {
    let peer_id1 = PeerId::random();
    let mut config1 = GigiDnsConfig::default();
    config1.nickname = "Alice".to_string();
    config1.capabilities = vec!["file-sharing".to_string(), "chat".to_string()];
    config1
        .metadata
        .insert("version".to_string(), "1.0".to_string());

    let mut peer1 = GigiDnsProtocol::new(peer_id1, config1);
    let addr1: Multiaddr = "/ip4/192.168.1.10/tcp/7174".parse().unwrap();
    peer1.update_listen_addresses(vec![addr1.clone()]);

    // Peer 1 creates a response
    let responses = peer1.build_response().unwrap();
    let response = &responses[0];

    // Peer 2 processes the response
    let peer_id2 = PeerId::random();
    let config2 = GigiDnsConfig::default();
    let mut peer2 = GigiDnsProtocol::new(peer_id2, config2);

    let result = peer2.handle_packet(response);
    assert!(result.is_ok());

    // Should discover peer 1
    if let Some(GigiDnsEvent::Discovered(info)) = result.unwrap() {
        assert_eq!(info.peer_id, peer_id1);
        assert_eq!(info.nickname, "Alice");
        assert_eq!(info.multiaddr, addr1);
        assert_eq!(info.capabilities.len(), 2);
        assert!(info.capabilities.contains(&"file-sharing".to_string()));
        assert!(info.capabilities.contains(&"chat".to_string()));
        assert_eq!(info.metadata.get("version"), Some(&"1.0".to_string()));
    } else {
        panic!("Expected Discovered event");
    }
}

#[test]
fn test_self_discovery_ignored() {
    let peer_id = PeerId::random();
    let mut config = GigiDnsConfig::default();
    config.nickname = "Self".to_string();

    let mut protocol = GigiDnsProtocol::new(peer_id, config);
    let addr: Multiaddr = "/ip4/192.168.1.10/tcp/7174".parse().unwrap();
    protocol.update_listen_addresses(vec![addr]);

    // Create a response
    let responses = protocol.build_response().unwrap();
    let response = &responses[0];

    // Process our own response (should be ignored)
    let result = protocol.handle_packet(response);
    // Self-discovery returns an Err with "Self-discovery" message, not Ok(None)
    assert!(result.is_err() || result.unwrap().is_none());
}

#[test]
fn test_multiple_addresses_response() {
    let peer_id = PeerId::random();
    let mut config = GigiDnsConfig::default();
    config.nickname = "MultiAddrPeer".to_string();

    let mut protocol = GigiDnsProtocol::new(peer_id, config);
    let addr1: Multiaddr = "/ip4/192.168.1.10/tcp/7174".parse().unwrap();
    let addr2: Multiaddr = "/ip4/192.168.1.10/tcp/7175".parse().unwrap();
    let addr3: Multiaddr = "/ip6/::1/tcp/7176".parse().unwrap();
    protocol.update_listen_addresses(vec![addr1, addr2, addr3]);

    // Should create one response per address
    let responses = protocol.build_response().unwrap();
    assert_eq!(responses.len(), 3);

    // Each response should be a valid DNS packet
    for response in &responses {
        assert!(response.len() >= 12);
        let flags = ((response[2] as u16) << 8) | (response[3] as u16);
        assert_ne!(flags & 0x8000, 0);
    }
}

#[test]
fn test_capability_and_metadata_encoding() {
    let peer_id = PeerId::random();
    let mut config = GigiDnsConfig::default();
    config.nickname = "TestPeer".to_string();
    config.capabilities = vec![
        "file-sharing".to_string(),
        "chat".to_string(),
        "video".to_string(),
    ];
    config
        .metadata
        .insert("version".to_string(), "2.0".to_string());
    config
        .metadata
        .insert("os".to_string(), "linux".to_string());

    let mut protocol = GigiDnsProtocol::new(peer_id, config);
    let addr: Multiaddr = "/ip4/192.168.1.10/tcp/7174".parse().unwrap();
    protocol.update_listen_addresses(vec![addr]);

    // Build response
    let responses = protocol.build_response().unwrap();
    let response = &responses[0];

    // Another peer processes the response
    let peer_id2 = PeerId::random();
    let config2 = GigiDnsConfig::default();
    let mut peer2 = GigiDnsProtocol::new(peer_id2, config2);

    let result = peer2.handle_packet(response);
    assert!(result.is_ok());

    if let Some(GigiDnsEvent::Discovered(info)) = result.unwrap() {
        // Verify all capabilities are present
        assert_eq!(info.capabilities.len(), 3);
        assert!(info.capabilities.contains(&"file-sharing".to_string()));
        assert!(info.capabilities.contains(&"chat".to_string()));
        assert!(info.capabilities.contains(&"video".to_string()));

        // Verify all metadata is present
        assert_eq!(info.metadata.len(), 2);
        assert_eq!(info.metadata.get("version"), Some(&"2.0".to_string()));
        assert_eq!(info.metadata.get("os"), Some(&"linux".to_string()));
    } else {
        panic!("Expected Discovered event");
    }
}

#[test]
fn test_ttl_based_expiration() {
    let peer_id = PeerId::random();
    let mut config = GigiDnsConfig::default();
    config.nickname = "TestPeer".to_string();
    config.ttl = Duration::from_secs(3600); // 1 hour TTL

    let mut protocol = GigiDnsProtocol::new(peer_id, config);
    let addr: Multiaddr = "/ip4/192.168.1.10/tcp/7174".parse().unwrap();
    protocol.update_listen_addresses(vec![addr]);

    // Build response
    let responses = protocol.build_response().unwrap();
    let response = &responses[0];

    // Another peer processes the response
    let peer_id2 = PeerId::random();
    let config2 = GigiDnsConfig::default();
    let mut peer2 = GigiDnsProtocol::new(peer_id2, config2);

    let result = peer2.handle_packet(response);
    assert!(result.is_ok());

    if let Some(GigiDnsEvent::Discovered(info)) = result.unwrap() {
        // Verify expiration is set correctly
        let now = std::time::Instant::now();
        let expires_in = info.expires_at.duration_since(now).as_secs();

        // Should expire approximately in 1 hour
        assert!(expires_in >= 3590 && expires_in <= 3610);
    } else {
        panic!("Expected Discovered event");
    }
}

#[test]
fn test_empty_capabilities_and_metadata() {
    let peer_id = PeerId::random();
    let mut config = GigiDnsConfig::default();
    config.nickname = "MinimalPeer".to_string();

    let mut protocol = GigiDnsProtocol::new(peer_id, config);
    let addr: Multiaddr = "/ip4/192.168.1.10/tcp/7174".parse().unwrap();
    protocol.update_listen_addresses(vec![addr]);

    // Build response
    let responses = protocol.build_response().unwrap();
    let response = &responses[0];

    // Another peer processes the response
    let peer_id2 = PeerId::random();
    let config2 = GigiDnsConfig::default();
    let mut peer2 = GigiDnsProtocol::new(peer_id2, config2);

    let result = peer2.handle_packet(response);
    assert!(result.is_ok());

    if let Some(GigiDnsEvent::Discovered(info)) = result.unwrap() {
        // Should have empty capabilities and metadata
        assert!(info.capabilities.is_empty());
        assert!(info.metadata.is_empty());
    } else {
        panic!("Expected Discovered event");
    }
}

#[test]
fn test_long_nickname() {
    let peer_id = PeerId::random();
    let mut config = GigiDnsConfig::default();
    config.nickname = "A".repeat(64); // Max length

    let mut protocol = GigiDnsProtocol::new(peer_id, config);
    let addr: Multiaddr = "/ip4/192.168.1.10/tcp/7174".parse().unwrap();
    protocol.update_listen_addresses(vec![addr]);

    // Build response should succeed
    let result = protocol.build_response();
    assert!(result.is_ok());
}

#[test]
fn test_ipv6_address() {
    let peer_id = PeerId::random();
    let mut config = GigiDnsConfig::default();
    config.nickname = "IPv6Peer".to_string();

    let mut protocol = GigiDnsProtocol::new(peer_id, config);
    let addr: Multiaddr = "/ip6/fe80::1/tcp/7174".parse().unwrap();
    protocol.update_listen_addresses(vec![addr.clone()]);

    // Build response
    let responses = protocol.build_response().unwrap();
    let response = &responses[0];

    // Another peer processes the response
    let peer_id2 = PeerId::random();
    let config2 = GigiDnsConfig::default();
    let mut peer2 = GigiDnsProtocol::new(peer_id2, config2);

    let result = peer2.handle_packet(response);
    assert!(result.is_ok());

    if let Some(GigiDnsEvent::Discovered(info)) = result.unwrap() {
        assert_eq!(info.multiaddr, addr);
    } else {
        panic!("Expected Discovered event");
    }
}

#[test]
fn test_query_response_roundtrip() {
    // Create two peers
    let peer_id1 = PeerId::random();
    let mut config1 = GigiDnsConfig::default();
    config1.nickname = "Peer1".to_string();

    let peer_id2 = PeerId::random();
    let mut config2 = GigiDnsConfig::default();
    config2.nickname = "Peer2".to_string();

    let mut protocol1 = GigiDnsProtocol::new(peer_id1, config1);
    let mut protocol2 = GigiDnsProtocol::new(peer_id2, config2);

    // Peer 1 sets up addresses
    let addr1: Multiaddr = "/ip4/192.168.1.10/tcp/7174".parse().unwrap();
    protocol1.update_listen_addresses(vec![addr1]);

    // Peer 2 sets up addresses
    let addr2: Multiaddr = "/ip4/192.168.1.11/tcp/7174".parse().unwrap();
    protocol2.update_listen_addresses(vec![addr2]);

    // Peer 1 sends query
    let _query = protocol1.build_query();

    // Peer 2 receives query and sends response
    let responses = protocol2.build_response().unwrap();

    // Peer 1 processes response from peer 2
    for response in responses {
        let result = protocol1.handle_packet(&response);
        assert!(result.is_ok());

        if let Some(GigiDnsEvent::Discovered(info)) = result.unwrap() {
            assert_eq!(info.peer_id, peer_id2);
            assert_eq!(info.nickname, "Peer2");
        }
    }
}
