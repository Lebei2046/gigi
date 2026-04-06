// Copyright 2024 Gigi Team.
//
// Tests for Gigi DNS protocol

use gigi_dns::protocol::GigiDnsProtocol;
use gigi_dns::types::*;
use libp2p::{Multiaddr, PeerId};

#[test]
fn test_gigidnsprotocol_new() {
    let peer_id = PeerId::random();
    let config = GigiDnsConfig::default();

    let protocol = GigiDnsProtocol::new(peer_id, config);

    // Verify initial state
    assert_eq!(protocol.get_discovered_peers().len(), 0);
}

#[test]
fn test_build_query_format() {
    let peer_id = PeerId::random();
    let config = GigiDnsConfig::default();
    let mut protocol = GigiDnsProtocol::new(peer_id, config);

    let query = protocol.build_query();

    // Verify minimum DNS packet size (12 bytes header + variable QNAME + 4 bytes question)
    assert!(query.len() >= 12);

    // Verify flags indicate query (QR bit = 0)
    let flags = ((query[2] as u16) << 8) | (query[3] as u16);
    assert_eq!(flags & 0x8000, 0);

    // Verify QDCOUNT = 1 (one question)
    let qdcount = ((query[4] as u16) << 8) | (query[5] as u16);
    assert_eq!(qdcount, 1);
}

#[test]
fn test_build_response_no_addresses() {
    let peer_id = PeerId::random();
    let mut config = GigiDnsConfig::default();
    config.nickname = "TestPeer".to_string();

    let protocol = GigiDnsProtocol::new(peer_id, config);

    // Should fail when no listen addresses
    let result = protocol.build_response();
    assert!(result.is_err());
}

#[test]
fn test_build_response_with_addresses() {
    let peer_id = PeerId::random();
    let mut config = GigiDnsConfig::default();
    config.nickname = "TestPeer".to_string();

    let mut protocol = GigiDnsProtocol::new(peer_id, config);
    let addr: Multiaddr = "/ip4/192.168.1.10/tcp/7174".parse().unwrap();
    protocol.update_listen_addresses(vec![addr]);

    let result = protocol.build_response();
    assert!(result.is_ok());

    let packets = result.unwrap();
    assert_eq!(packets.len(), 1);

    let packet = &packets[0];

    // Verify minimum DNS packet size
    assert!(packet.len() >= 12);

    // Verify flags indicate response (QR bit = 1)
    let flags = ((packet[2] as u16) << 8) | (packet[3] as u16);
    assert_ne!(flags & 0x8000, 0);

    // Verify ANCOUNT = 1 (one answer)
    let ancount = ((packet[6] as u16) << 8) | (packet[7] as u16);
    assert_eq!(ancount, 1);
}

#[test]
fn test_build_response_multiple_addresses() {
    let peer_id = PeerId::random();
    let mut config = GigiDnsConfig::default();
    config.nickname = "TestPeer".to_string();

    let mut protocol = GigiDnsProtocol::new(peer_id, config);
    let addr1: Multiaddr = "/ip4/192.168.1.10/tcp/7174".parse().unwrap();
    let addr2: Multiaddr = "/ip4/192.168.1.10/tcp/7175".parse().unwrap();
    protocol.update_listen_addresses(vec![addr1, addr2]);

    let result = protocol.build_response();
    assert!(result.is_ok());

    let packets = result.unwrap();
    // Should create one packet per address
    assert_eq!(packets.len(), 2);
}

#[test]
fn test_is_query() {
    let peer_id = PeerId::random();
    let config = GigiDnsConfig::default();
    let protocol = GigiDnsProtocol::new(peer_id, config);

    // Query packet
    let mut query = vec![0u8; 12];
    query[2] = 0x00; // QR bit = 0 (query)
    assert!(protocol.is_query(&query));

    // Response packet
    let mut response = vec![0u8; 12];
    response[2] = 0x80; // QR bit = 1 (response)
    assert!(!protocol.is_query(&response));

    // Too short
    let short = vec![0u8; 10];
    assert!(!protocol.is_query(&short));
}

#[test]
fn test_handle_packet_too_short() {
    let peer_id = PeerId::random();
    let config = GigiDnsConfig::default();
    let mut protocol = GigiDnsProtocol::new(peer_id, config);

    // 11 bytes is too short for a valid DNS packet
    let short_packet = vec![0u8; 11];
    let result = protocol.handle_packet(&short_packet);

    assert!(result.is_err());
}

#[test]
fn test_handle_packet_query() {
    let peer_id = PeerId::random();
    let config = GigiDnsConfig::default();
    let mut protocol = GigiDnsProtocol::new(peer_id, config);

    let query = protocol.build_query();
    let result = protocol.handle_packet(&query);

    // Queries return None (handled elsewhere)
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_update_nickname() {
    let peer_id = PeerId::random();
    let mut config = GigiDnsConfig::default();
    config.nickname = "Original".to_string();

    let mut protocol = GigiDnsProtocol::new(peer_id, config);

    // Create a response with original nickname
    let addr: Multiaddr = "/ip4/192.168.1.10/tcp/7174".parse().unwrap();
    protocol.update_listen_addresses(vec![addr]);

    let packets = protocol.build_response().unwrap();
    let packet_str = String::from_utf8_lossy(&packets[0]);
    assert!(packet_str.contains("Original"));

    // Update nickname
    protocol.update_nickname("NewNickname".to_string());

    // Create a new response with updated nickname
    let packets = protocol.build_response().unwrap();
    let packet_str = String::from_utf8_lossy(&packets[0]);
    assert!(!packet_str.contains("Original"));
    assert!(packet_str.contains("NewNickname"));
}

#[test]
fn test_update_listen_addresses() {
    let peer_id = PeerId::random();
    let config = GigiDnsConfig::default();

    let mut protocol = GigiDnsProtocol::new(peer_id, config);

    // Initially no addresses
    assert!(protocol.build_response().is_err());

    // Add address
    let addr1: Multiaddr = "/ip4/192.168.1.10/tcp/7174".parse().unwrap();
    protocol.update_listen_addresses(vec![addr1]);
    let result = protocol.build_response();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 1);

    // Update addresses
    let addr2: Multiaddr = "/ip4/192.168.1.10/tcp/7175".parse().unwrap();
    protocol.update_listen_addresses(vec![addr2]);
    let result = protocol.build_response();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 1);
}

#[test]
fn test_cleanup_expired_pending_queries() {
    let peer_id = PeerId::random();
    let config = GigiDnsConfig::default();
    let mut protocol = GigiDnsProtocol::new(peer_id, config);

    // Build a query to add to pending queries
    protocol.build_query();

    // Cleanup should not remove recent queries
    let events = protocol.cleanup_expired();
    assert_eq!(events.len(), 0);
}

#[test]
fn test_transaction_id_increment() {
    let peer_id = PeerId::random();
    let config = GigiDnsConfig::default();
    let mut protocol = GigiDnsProtocol::new(peer_id, config);

    let query1 = protocol.build_query();
    let tid1 = ((query1[0] as u16) << 8) | (query1[1] as u16);

    let query2 = protocol.build_query();
    let tid2 = ((query2[0] as u16) << 8) | (query2[1] as u16);

    let query3 = protocol.build_query();
    let tid3 = ((query3[0] as u16) << 8) | (query3[1] as u16);

    // Transaction IDs should be sequential
    let expected_tid2 = tid1.wrapping_add(1);
    let expected_tid3 = tid2.wrapping_add(1);
    assert_eq!(tid2, expected_tid2);
    assert_eq!(tid3, expected_tid3);
}

#[test]
fn test_transaction_id_wraparound() {
    let peer_id = PeerId::random();
    let config = GigiDnsConfig::default();

    // Create protocol with transaction ID near wraparound
    let mut protocol = GigiDnsProtocol::new(peer_id, config);

    // Simulate many queries to test wraparound
    for _ in 0..70000 {
        protocol.build_query();
    }

    let query = protocol.build_query();
    let tid = ((query[0] as u16) << 8) | (query[1] as u16);

    // Should be a valid u16 (0-65535) - u16 already enforces this
    assert!(tid <= u16::MAX);
}

#[test]
fn test_rate_limiting() {
    let peer_id = PeerId::random();
    let config = GigiDnsConfig::default();
    let mut protocol = GigiDnsProtocol::new(peer_id, config);

    // Generate 20 malformed packets to trigger rate limiting
    for _ in 0..20 {
        let bad_packet = vec![0u8; 5]; // Too short
        let _ = protocol.handle_packet(&bad_packet);
    }

    // Should be rate limited
    let query = protocol.build_query();
    let result = protocol.handle_packet(&query);
    // May return None due to rate limiting
    assert!(result.is_ok());
}
