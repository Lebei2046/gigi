// Tests for Gigi DNS protocol

import { describe, it, expect, beforeEach } from 'vitest';
import type { PeerId } from '@libp2p/interface';
import { GigiDnsProtocol } from '../protocol';
import { defaultGigiDnsConfig } from '../types';
import { multiaddr } from '@multiformats/multiaddr';

describe('GigiDnsProtocol', () => {
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

  it('should create a valid instance', () => {
    expect(protocol).toBeInstanceOf(GigiDnsProtocol);
  });

  it('should build a valid DNS query packet', () => {
    const query = protocol.buildQuery();
    expect(query).toBeInstanceOf(Uint8Array);
    expect(query.length).toBeGreaterThan(0);
  });

  it('should build a valid DNS response packet', () => {
    const addr = multiaddr('/ip4/192.168.1.100/tcp/8000');
    protocol.updateListenAddresses([addr]);

    const response = protocol.buildResponse();
    expect(response.success).toBe(true);
    expect(Array.isArray(response.result)).toBe(true);
    expect(response.result.length).toBe(1);
  });

  it('should handle empty listen addresses', () => {
    protocol.updateListenAddresses([]);
    const response = protocol.buildResponse();
    expect(response.success).toBe(false);
    expect(response.result).toBe('No listen addresses available');
  });

  it('should update listen addresses', () => {
    const addr1 = multiaddr('/ip4/192.168.1.100/tcp/8000');
    const addr2 = multiaddr('/ip4/192.168.1.101/tcp/8001');

    protocol.updateListenAddresses([addr1, addr2]);
    const response = protocol.buildResponse();
    expect(response.success).toBe(true);
    expect(Array.isArray(response.result)).toBe(true);
    expect(response.result.length).toBe(2);
  });

  it('should update nickname', () => {
    const result = protocol.updateNickname('NewNickname');
    expect(result.success).toBe(true);
    expect(result.result).toBeNull();
  });

  it('should validate nickname length', () => {
    const result = protocol.updateNickname('a'.repeat(65));
    expect(result.success).toBe(false);
    expect(result.result).toBe('Nickname too long: 65 chars (max: 64)');
  });

  it('should validate empty nickname', () => {
    const result = protocol.updateNickname('');
    expect(result.success).toBe(false);
    expect(result.result).toBe('Nickname cannot be empty');
  });

  it('should cleanup expired queries', () => {
    // Build a query to create a pending query
    protocol.buildQuery();

    // Just verify the method runs without errors
    expect(() => protocol.cleanupExpired()).not.toThrow();
  });

  it('should detect if a packet is a query', () => {
    const query = protocol.buildQuery();
    expect(protocol.isQuery(query)).toBe(true);
  });

  it('should handle invalid packets', () => {
    const invalidPacket = new Uint8Array(10); // Too short
    const result = protocol.handlePacket(invalidPacket);
    expect(result.success).toBe(false);
    expect(result.result).toBe('Packet too short');
  });
});
