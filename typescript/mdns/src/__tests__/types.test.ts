// Tests for Gigi DNS types

import { describe, it, expect } from 'vitest';
import {
  defaultGigiDnsConfig,
  validateGigiDnsConfig,
  GigiDnsConfigValidation,
  encodeGigiDnsRecord,
  decodeGigiDnsRecord,
  GigiDnsRecord,
} from '../types';

describe('GigiDnsConfig', () => {
  it('should have valid default configuration', () => {
    const config = defaultGigiDnsConfig;
    expect(validateGigiDnsConfig(config)).toBeNull();
  });

  it('should validate nickname length', () => {
    const config = { ...defaultGigiDnsConfig, nickname: 'a'.repeat(65) };
    expect(validateGigiDnsConfig(config)).toBe(
      `Nickname too long: 65 chars (max: ${GigiDnsConfigValidation.MAX_NICKNAME_LENGTH})`
    );
  });

  it('should validate empty nickname', () => {
    const config = { ...defaultGigiDnsConfig, nickname: '' };
    expect(validateGigiDnsConfig(config)).toBe('Nickname cannot be empty');
  });

  it('should validate TTL range', () => {
    // TTL too short
    const config1 = {
      ...defaultGigiDnsConfig,
      ttl: GigiDnsConfigValidation.MIN_TTL - 1,
    };
    expect(validateGigiDnsConfig(config1)).toBe(
      `TTL too short: ${GigiDnsConfigValidation.MIN_TTL - 1}ms (min: ${GigiDnsConfigValidation.MIN_TTL}ms)`
    );

    // TTL too long
    const config2 = {
      ...defaultGigiDnsConfig,
      ttl: GigiDnsConfigValidation.MAX_TTL + 1,
    };
    expect(validateGigiDnsConfig(config2)).toBe(
      `TTL too long: ${GigiDnsConfigValidation.MAX_TTL + 1}ms (max: ${GigiDnsConfigValidation.MAX_TTL}ms)`
    );
  });

  it('should validate queryInterval range', () => {
    // Query interval too short
    const config1 = {
      ...defaultGigiDnsConfig,
      queryInterval: GigiDnsConfigValidation.MIN_QUERY_INTERVAL - 1,
    };
    expect(validateGigiDnsConfig(config1)).toBe(
      `Query interval too short: ${GigiDnsConfigValidation.MIN_QUERY_INTERVAL - 1}ms (min: ${GigiDnsConfigValidation.MIN_QUERY_INTERVAL}ms)`
    );

    // Query interval too long
    const config2 = {
      ...defaultGigiDnsConfig,
      queryInterval: GigiDnsConfigValidation.MAX_QUERY_INTERVAL + 1,
    };
    expect(validateGigiDnsConfig(config2)).toBe(
      `Query interval too long: ${GigiDnsConfigValidation.MAX_QUERY_INTERVAL + 1}ms (max: ${GigiDnsConfigValidation.MAX_QUERY_INTERVAL}ms)`
    );
  });

  it('should validate announceInterval range', () => {
    // Announce interval too short
    const config1 = {
      ...defaultGigiDnsConfig,
      announceInterval: GigiDnsConfigValidation.MIN_ANNOUNCE_INTERVAL - 1,
    };
    expect(validateGigiDnsConfig(config1)).toBe(
      `Announce interval too short: ${GigiDnsConfigValidation.MIN_ANNOUNCE_INTERVAL - 1}ms (min: ${GigiDnsConfigValidation.MIN_ANNOUNCE_INTERVAL}ms)`
    );

    // Announce interval too long
    const config2 = {
      ...defaultGigiDnsConfig,
      announceInterval: GigiDnsConfigValidation.MAX_ANNOUNCE_INTERVAL + 1,
    };
    expect(validateGigiDnsConfig(config2)).toBe(
      `Announce interval too long: ${GigiDnsConfigValidation.MAX_ANNOUNCE_INTERVAL + 1}ms (max: ${GigiDnsConfigValidation.MAX_ANNOUNCE_INTERVAL}ms)`
    );
  });

  it('should validate cleanupInterval range', () => {
    // Cleanup interval too short
    const config1 = {
      ...defaultGigiDnsConfig,
      cleanupInterval: GigiDnsConfigValidation.MIN_CLEANUP_INTERVAL - 1,
    };
    expect(validateGigiDnsConfig(config1)).toBe(
      `Cleanup interval too short: ${GigiDnsConfigValidation.MIN_CLEANUP_INTERVAL - 1}ms (min: ${GigiDnsConfigValidation.MIN_CLEANUP_INTERVAL}ms)`
    );

    // Cleanup interval too long
    const config2 = {
      ...defaultGigiDnsConfig,
      cleanupInterval: GigiDnsConfigValidation.MAX_CLEANUP_INTERVAL + 1,
    };
    expect(validateGigiDnsConfig(config2)).toBe(
      `Cleanup interval too long: ${GigiDnsConfigValidation.MAX_CLEANUP_INTERVAL + 1}ms (max: ${GigiDnsConfigValidation.MAX_CLEANUP_INTERVAL}ms)`
    );
  });
});

describe('GigiDnsRecord encoding/decoding', () => {
  it('should encode and decode a basic record', () => {
    const record: GigiDnsRecord = {
      peerId: '12D3KooWBob',
      nickname: 'Alice',
      addr: '/ip4/192.168.1.100/tcp/8000',
      capabilities: '',
      metadata: '',
    };

    const encodeResult = encodeGigiDnsRecord(record);
    expect(encodeResult.success).toBe(true);
    expect(encodeResult.result).toBe(
      'peer_id=12D3KooWBob nickname=Alice addr=/ip4/192.168.1.100/tcp/8000'
    );

    const decodeResult = decodeGigiDnsRecord(encodeResult.result as string);
    expect(decodeResult.success).toBe(true);
    expect(decodeResult.result).toEqual(record);
  });

  it('should encode and decode a record with capabilities and metadata', () => {
    const record: GigiDnsRecord = {
      peerId: '12D3KooWBob',
      nickname: 'Alice',
      addr: '/ip4/192.168.1.100/tcp/8000',
      capabilities: 'chat,file_sharing',
      metadata: 'version:1.0,os:Linux',
    };

    const encodeResult = encodeGigiDnsRecord(record);
    expect(encodeResult.success).toBe(true);
    expect(encodeResult.result).toBe(
      'peer_id=12D3KooWBob nickname=Alice addr=/ip4/192.168.1.100/tcp/8000 caps=chat,file_sharing meta=version:1.0,os:Linux'
    );

    const decodeResult = decodeGigiDnsRecord(encodeResult.result as string);
    expect(decodeResult.success).toBe(true);
    expect(decodeResult.result).toEqual(record);
  });

  it('should handle missing required fields', () => {
    // Missing peerId
    let decodeResult = decodeGigiDnsRecord(
      'nickname=Alice addr=/ip4/192.168.1.100/tcp/8000'
    );
    expect(decodeResult.success).toBe(false);
    expect(decodeResult.result).toBe('Missing peer_id');

    // Missing nickname
    decodeResult = decodeGigiDnsRecord(
      'peer_id=12D3KooWBob addr=/ip4/192.168.1.100/tcp/8000'
    );
    expect(decodeResult.success).toBe(false);
    expect(decodeResult.result).toBe('Missing nickname');

    // Missing addr
    decodeResult = decodeGigiDnsRecord('peer_id=12D3KooWBob nickname=Alice');
    expect(decodeResult.success).toBe(false);
    expect(decodeResult.result).toBe('Missing addr');
  });

  it('should handle record too long error', () => {
    const record: GigiDnsRecord = {
      peerId: '12D3KooW' + 'o'.repeat(1000), // Long peer ID
      nickname: 'Alice' + 'a'.repeat(1000), // Long nickname
      addr: '/ip4/192.168.1.100/tcp/8000' + '/tcp/8000'.repeat(100), // Long address
      capabilities: 'cap1,cap2,cap3'.repeat(100), // Long capabilities
      metadata: 'key1:value1,key2:value2'.repeat(100), // Long metadata
    };

    const encodeResult = encodeGigiDnsRecord(record);
    expect(encodeResult.success).toBe(false);
    expect(encodeResult.result).toMatch(/Record too long:/);
  });
});
