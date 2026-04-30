// Additional tests for GigiDnsProtocol

import { describe, it, expect, beforeEach, vi } from 'vitest';
import type { PeerId } from '@libp2p/interface';
import { GigiDnsProtocol } from '../protocol';
import { defaultGigiDnsConfig, GigiDnsRecord } from '../types';
import { multiaddr } from '@multiformats/multiaddr';

// Mock peerIdFromString
vi.mock('@libp2p/peer-id', () => ({
  peerIdFromString: vi.fn(),
}));

import { peerIdFromString } from '@libp2p/peer-id';
const mockPeerIdFromString = vi.mocked(peerIdFromString);

describe('GigiDnsProtocol Coverage Tests', () => {
  let protocol: GigiDnsProtocol;
  let localPeerId: { toString: () => string; toBytes: () => Uint8Array };

  beforeEach(() => {
    // Mock peer ID
    localPeerId = {
      toString: () => '12D3KooWBob',
      toBytes: () => new Uint8Array([1, 2, 3]),
    };
    protocol = new GigiDnsProtocol(
      localPeerId as unknown as PeerId,
      defaultGigiDnsConfig
    );
  });

  it('should handle handlePacket with rate limiting', () => {
    // Mock the isRateLimited method to return true
    const originalIsRateLimited = (protocol as any).isRateLimited;
    (protocol as any).isRateLimited = vi.fn().mockReturnValue(true);

    // Create a mock packet
    const packet = new Uint8Array(12);
    const view = new DataView(packet.buffer);
    view.setUint16(0, 1234, false); // Transaction ID
    view.setUint16(2, 0x8000, false); // Response flag

    // Test rate limiting
    const result = protocol.handlePacket(packet);
    expect(result.success).toBe(true);
    expect(result.result).toBeNull();

    // Restore original method
    (protocol as any).isRateLimited = originalIsRateLimited;
  });

  it('should handle handlePacket with invalid source address', () => {
    // Create a mock packet
    const packet = new Uint8Array(12);
    const view = new DataView(packet.buffer);
    view.setUint16(0, 1234, false); // Transaction ID
    view.setUint16(2, 0x8000, false); // Response flag

    // Test with invalid source address
    const result = protocol.handlePacket(packet, 'invalid-ip');
    expect(result.success).toBe(false);
    expect(result.result).toBe('Invalid source address');
  });

  it('should handle handlePacket with invalid DNS header', () => {
    // Create a mock packet with invalid flags
    const packet = new Uint8Array(12);
    const view = new DataView(packet.buffer);
    view.setUint16(0, 1234, false); // Transaction ID
    view.setUint16(2, 0xffff, false); // Invalid flags

    // Test with invalid header
    const result = protocol.handlePacket(packet);
    expect(result.success).toBe(false);
    expect(result.result).toBe('Invalid DNS header flags');
  });

  it('should handle handlePacket with too many answers', () => {
    // Create a mock packet with too many answers
    const packet = new Uint8Array(12);
    const view = new DataView(packet.buffer);
    view.setUint16(0, 1234, false); // Transaction ID
    view.setUint16(2, 0x8000, false); // Response flag
    view.setUint16(6, 11, false); // 11 answers (too many)

    // Test with too many answers
    const result = protocol.handlePacket(packet);
    expect(result.success).toBe(false);
    expect(result.result).toBe('Too many answers in packet');
  });

  it('should handle handlePacket with invalid TXT record format', () => {
    // Create a mock packet with TXT record
    const packet = new Uint8Array(50);
    const view = new DataView(packet.buffer);
    view.setUint16(0, 1234, false); // Transaction ID
    view.setUint16(2, 0x8000, false); // Response flag
    view.setUint16(4, 0, false); // No questions
    view.setUint16(6, 1, false); // 1 answer

    // Set up answer section
    let pos = 12;
    packet[pos] = 0; // Empty QNAME
    pos += 1;
    view.setUint16(pos, 0x0010, false); // TYPE: TXT
    pos += 2;
    view.setUint16(pos, 0x0001, false); // CLASS: IN
    pos += 2;
    view.setUint32(pos, 30, false); // TTL
    pos += 4;
    view.setUint16(pos, 5, false); // RDLENGTH: 5
    pos += 2;
    packet[pos] = 10; // Invalid chunk length (exceeds RDLENGTH)

    // Test with invalid TXT record
    const result = protocol.handlePacket(packet);
    expect(result.success).toBe(false);
  });

  it('should handle processDiscoveredPeer with self-discovery', () => {
    // Create a record with our own peer ID
    const record: GigiDnsRecord = {
      peerId: '12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3',
      nickname: 'TestPeer',
      addr: '/ip4/127.0.0.1/tcp/1234',
      capabilities: '',
      metadata: '',
    };

    // Mock peerIdFromString to return our local peer ID
    mockPeerIdFromString.mockReturnValue(localPeerId as any);

    // Test self-discovery
    const result = (protocol as any).processDiscoveredPeer(record, 30);
    expect(result.success).toBe(false);
    expect(result.result).toBe('Self-discovery');
  });

  it('should handle processDiscoveredPeer with no nickname', () => {
    // Create a record with no nickname
    const record: GigiDnsRecord = {
      peerId: '12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3',
      nickname: '',
      addr: '/ip4/127.0.0.1/tcp/1234',
      capabilities: '',
      metadata: '',
    };

    // Mock peerIdFromString
    mockPeerIdFromString.mockReturnValue({
      toString: () => '12D3KooWOther',
    } as unknown as PeerId);

    // Test no nickname
    const result = (protocol as any).processDiscoveredPeer(record, 30);
    expect(result.success).toBe(false);
    expect(result.result).toBe('No nickname provided');
  });

  it('should handle processDiscoveredPeer with nickname that looks like peer ID', () => {
    // Create a record with nickname that looks like peer ID
    const record: GigiDnsRecord = {
      peerId: '12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3',
      nickname: '12D3KooWPeerId',
      addr: '/ip4/127.0.0.1/tcp/1234',
      capabilities: '',
      metadata: '',
    };

    // Mock peerIdFromString
    mockPeerIdFromString.mockReturnValue({
      toString: () => '12D3KooWOther',
    } as unknown as PeerId);

    // Test nickname that looks like peer ID
    const result = (protocol as any).processDiscoveredPeer(record, 30);
    expect(result.success).toBe(false);
    expect(result.result).toBe('Nickname looks like a peer ID');
  });

  it('should handle processDiscoveredPeer with invalid multiaddr', () => {
    // Create a record with invalid multiaddr
    const record: GigiDnsRecord = {
      peerId: '12D3KooWOther',
      nickname: 'TestPeer',
      addr: 'invalid-multiaddr',
      capabilities: '',
      metadata: '',
    };

    // Test invalid multiaddr
    const result = (protocol as any).processDiscoveredPeer(record, 30);
    expect(result.success).toBe(false);
  });

  it('should handle recordError method', () => {
    // Call recordError multiple times
    for (let i = 0; i < 25; i++) {
      (protocol as any).recordError();
    }

    // The recentErrors array should be limited to MAX_ERROR_HISTORY
    expect((protocol as any).recentErrors.length).toBeLessThanOrEqual(20);
  });

  it('should handle cleanupExpired method', () => {
    // Add some old pending queries
    const oldTimestamp = Date.now() - 60000; // 1 minute ago
    (protocol as any).pendingQueries.set(1, oldTimestamp);
    (protocol as any).pendingQueries.set(2, oldTimestamp);
    (protocol as any).pendingQueries.set(3, Date.now()); // Recent query

    // Cleanup expired queries
    protocol.cleanupExpired();

    // Only the recent query should remain
    expect((protocol as any).pendingQueries.size).toBe(1);
    expect((protocol as any).pendingQueries.has(3)).toBe(true);
  });

  it('should handle isValidIpAddress method', () => {
    const validIpv4 = '192.168.1.1';
    const validIpv6 = '2001:0db8:85a3:0000:0000:8a2e:0370:7334';
    const invalidIp = 'invalid-ip';

    expect((protocol as any).isValidIpAddress(validIpv4)).toBe(true);
    expect((protocol as any).isValidIpAddress(validIpv6)).toBe(true);
    expect((protocol as any).isValidIpAddress(invalidIp)).toBe(false);
  });

  it('should handle isValidDnsHeader method', () => {
    // Test valid response header
    expect((protocol as any).isValidDnsHeader(0x8000, true)).toBe(true);

    // Test valid query header
    expect((protocol as any).isValidDnsHeader(0x0000, false)).toBe(true);

    // Test invalid response header (QR bit not set)
    expect((protocol as any).isValidDnsHeader(0x0000, true)).toBe(false);

    // Test invalid query header (QR bit set)
    expect((protocol as any).isValidDnsHeader(0x8000, false)).toBe(false);
  });

  it('should handle appendQname method', () => {
    const packet = new Uint8Array(64);
    const serviceName = Buffer.from('_gigi-dns._udp.local');
    const pos = (protocol as any).appendQname(packet, 0, serviceName);

    // The method should return a position greater than 0
    expect(pos).toBeGreaterThan(0);
  });

  it('should handle isQuery method with invalid packet', () => {
    // Test with too short packet
    const shortPacket = new Uint8Array(10);
    expect(protocol.isQuery(shortPacket)).toBe(false);
  });

  it('should handle updateNickname with valid nickname', () => {
    const result = protocol.updateNickname('NewNickname');
    expect(result.success).toBe(true);
    expect(result.result).toBeNull();
  });

  it('should handle updateNickname with empty nickname', () => {
    const result = protocol.updateNickname('');
    expect(result.success).toBe(false);
    expect(result.result).toBe('Nickname cannot be empty');
  });

  it('should handle updateNickname with too long nickname', () => {
    const longNickname = 'a'.repeat(65);
    const result = protocol.updateNickname(longNickname);
    expect(result.success).toBe(false);
    expect(result.result).toBe(`Nickname too long: 65 chars (max: 64)`);
  });

  it('should handle updateListenAddresses method', () => {
    const addr = multiaddr('/ip4/192.168.1.100/tcp/8000');
    protocol.updateListenAddresses([addr]);
    expect((protocol as any).listenAddresses).toEqual([addr]);
  });

  it('should handle buildResponse with no listen addresses', () => {
    protocol.updateListenAddresses([]);
    const result = protocol.buildResponse(12345);
    expect(result.success).toBe(false);
    expect(result.result).toBe('No listen addresses available');
  });

  it('should handle buildQuery method', () => {
    const query = protocol.buildQuery();
    expect(query).toBeInstanceOf(Uint8Array);
    expect(query.length).toBeGreaterThan(0);
  });
});
