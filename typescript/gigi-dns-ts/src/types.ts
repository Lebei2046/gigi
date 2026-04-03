// Gigi DNS Types
//
// This module defines all types used by the Gigi DNS protocol, including:
// - Configuration constants and structures
// - Peer information
// - DNS events
// - DNS record encoding/decoding

import type { PeerId } from '@libp2p/interface';
import { Multiaddr } from '@multiformats/multiaddr';

/// IPv4 multicast address for mDNS (224.0.0.251)
/// This is the standard mDNS multicast address used by Apple Bonjour
export const IPV4_MDNS_MULTICAST_ADDRESS = '224.0.0.251';

/// IPv6 multicast address for mDNS (FF02::FB)
/// This is the IPv6 link-local multicast address for mDNS
export const IPV6_MDNS_MULTICAST_ADDRESS = 'FF02::FB';

/// UDP port for Gigi DNS protocol (7173)
/// This port is used for all Gigi DNS multicast communication
export const GIGI_DNS_PORT = 7173;

/// Configuration for Gigi DNS behavior
///
/// This interface contains all configurable parameters for the Gigi DNS protocol.
export interface GigiDnsConfig {
  /// Human-readable nickname for this peer (max 64 characters)
  nickname: string;
  /// Time-to-live for DNS records (min: 60s, max: 24h)
  ttl: number; // in milliseconds
  /// Interval between discovery queries (min: 5s, max: 1h)
  queryInterval: number; // in milliseconds
  /// Interval between announcements (min: 5s, max: 10min)
  announceInterval: number; // in milliseconds
  /// Interval for cleanup operations (min: 10s, max: 5min)
  cleanupInterval: number; // in milliseconds
  /// Enable IPv6 multicast (disabled by default)
  enableIpv6: boolean;
  /// List of capabilities this peer provides (e.g., "file-sharing", "chat")
  capabilities: string[];
  /// Optional metadata key-value pairs for additional peer information
  metadata: Record<string, string>;
  /// Use localhost unicast instead of multicast for testing
  useLocalhost: boolean;
}

/// Default configuration for Gigi DNS
export const defaultGigiDnsConfig: GigiDnsConfig = {
  nickname: 'Anonymous',
  ttl: 6 * 60 * 1000, // 6 minutes
  queryInterval: 5 * 60 * 1000, // 5 minutes
  announceInterval: 15 * 1000, // 15 seconds
  cleanupInterval: 30 * 1000, // 30 seconds
  enableIpv6: false,
  capabilities: [],
  metadata: {},
  useLocalhost: false,
};

/// Validation constants for GigiDnsConfig
export const GigiDnsConfigValidation = {
  MIN_TTL: 60 * 1000, // 60 seconds
  MAX_TTL: 24 * 60 * 60 * 1000, // 24 hours
  MIN_QUERY_INTERVAL: 5 * 1000, // 5 seconds
  MAX_QUERY_INTERVAL: 60 * 60 * 1000, // 1 hour
  MIN_ANNOUNCE_INTERVAL: 5 * 1000, // 5 seconds
  MAX_ANNOUNCE_INTERVAL: 10 * 60 * 1000, // 10 minutes
  MIN_CLEANUP_INTERVAL: 10 * 1000, // 10 seconds
  MAX_CLEANUP_INTERVAL: 5 * 60 * 1000, // 5 minutes
  MAX_NICKNAME_LENGTH: 64,
};

/// Validates the GigiDnsConfig
///
/// Ensures all parameters are within acceptable ranges to prevent
/// misconfiguration that could lead to network issues or resource exhaustion.
///
/// @param config - The configuration to validate
/// @returns - `null` if valid, or an error message string if invalid
export function validateGigiDnsConfig(config: GigiDnsConfig): string | null {
  // Validate nickname
  if (!config.nickname) {
    return 'Nickname cannot be empty';
  }
  if (config.nickname.length > GigiDnsConfigValidation.MAX_NICKNAME_LENGTH) {
    return `Nickname too long: ${config.nickname.length} chars (max: ${GigiDnsConfigValidation.MAX_NICKNAME_LENGTH})`;
  }

  // Validate TTL
  if (config.ttl < GigiDnsConfigValidation.MIN_TTL) {
    return `TTL too short: ${config.ttl}ms (min: ${GigiDnsConfigValidation.MIN_TTL}ms)`;
  }
  if (config.ttl > GigiDnsConfigValidation.MAX_TTL) {
    return `TTL too long: ${config.ttl}ms (max: ${GigiDnsConfigValidation.MAX_TTL}ms)`;
  }

  // Validate queryInterval
  if (config.queryInterval < GigiDnsConfigValidation.MIN_QUERY_INTERVAL) {
    return `Query interval too short: ${config.queryInterval}ms (min: ${GigiDnsConfigValidation.MIN_QUERY_INTERVAL}ms)`;
  }
  if (config.queryInterval > GigiDnsConfigValidation.MAX_QUERY_INTERVAL) {
    return `Query interval too long: ${config.queryInterval}ms (max: ${GigiDnsConfigValidation.MAX_QUERY_INTERVAL}ms)`;
  }

  // Validate announceInterval
  if (config.announceInterval < GigiDnsConfigValidation.MIN_ANNOUNCE_INTERVAL) {
    return `Announce interval too short: ${config.announceInterval}ms (min: ${GigiDnsConfigValidation.MIN_ANNOUNCE_INTERVAL}ms)`;
  }
  if (config.announceInterval > GigiDnsConfigValidation.MAX_ANNOUNCE_INTERVAL) {
    return `Announce interval too long: ${config.announceInterval}ms (max: ${GigiDnsConfigValidation.MAX_ANNOUNCE_INTERVAL}ms)`;
  }

  // Validate cleanupInterval
  if (config.cleanupInterval < GigiDnsConfigValidation.MIN_CLEANUP_INTERVAL) {
    return `Cleanup interval too short: ${config.cleanupInterval}ms (min: ${GigiDnsConfigValidation.MIN_CLEANUP_INTERVAL}ms)`;
  }
  if (config.cleanupInterval > GigiDnsConfigValidation.MAX_CLEANUP_INTERVAL) {
    return `Cleanup interval too long: ${config.cleanupInterval}ms (max: ${GigiDnsConfigValidation.MAX_CLEANUP_INTERVAL}ms)`;
  }

  return null;
}

/// Information about a discovered peer
///
/// Contains all information advertised by a peer via DNS TXT records,
/// along with timestamps for discovery and expiration.
export interface GigiPeerInfo {
  /// libp2p peer ID
  peerId: PeerId;
  /// Human-readable nickname
  nickname: string;
  /// libp2p multiaddress for connecting to this peer
  multiaddr: Multiaddr;
  /// List of capabilities provided by this peer
  capabilities: string[];
  /// Additional metadata key-value pairs
  metadata: Record<string, string>;
  /// When this peer was first discovered
  discoveredAt: Date;
  /// When this peer's information expires (based on DNS TTL)
  expiresAt: Date;
}

/// Reasons why a peer might go offline
export enum OfflineReason {
  /// Peer's DNS record TTL expired without renewal
  TtlExpired = 'TTL_EXPIRED',
  /// Health check to peer failed
  HealthCheckFailed = 'HEALTH_CHECK_FAILED',
}

/// Events emitted by the Gigi DNS behavior
///
/// These events inform the application about peer lifecycle changes.
export type GigiDnsEvent =
  /// A new peer was discovered
  | { type: 'Discovered'; peerInfo: GigiPeerInfo }
  /// An existing peer's information was updated
  | {
      type: 'Updated';
      peerId: PeerId;
      oldInfo: GigiPeerInfo;
      newInfo: GigiPeerInfo;
    }
  /// A peer's information expired (not seen recently)
  | { type: 'Expired'; peerId: PeerId; info: GigiPeerInfo }
  /// A peer went offline (determined via health check)
  | {
      type: 'Offline';
      peerId: PeerId;
      info: GigiPeerInfo;
      reason: OfflineReason;
    }
  /// An error occurred
  | { type: 'Error'; error: Error; context: string };

/// DNS record format for Gigi peer information
///
/// This interface represents the data encoded in DNS TXT records.
/// The encoding format is: "peer_id=<id> nickname=<name> addr=<addr> caps=<caps> meta=<metadata>"
export interface GigiDnsRecord {
  /// libp2p peer ID as a string
  peerId: string;
  /// Human-readable nickname
  nickname: string;
  /// libp2p multiaddress as a string
  addr: string;
  /// Comma-separated list of capabilities
  capabilities: string;
  /// Comma-separated key:value pairs for metadata
  metadata: string;
}

/// Maximum length of encoded TXT record in bytes
export const MAX_TXT_LENGTH = 4096;

/// Encodes a GigiDnsRecord into a string suitable for DNS TXT records
///
/// Format: "peer_id=<id> nickname=<name> addr=<addr> caps=<caps> meta=<metadata>"
///
/// Caps and metadata are optional and omitted if empty.
///
/// @param record - The record to encode
/// @returns - The encoded record string, or an error message if too long
export function encodeGigiDnsRecord(record: GigiDnsRecord): {
  success: boolean;
  result: string | string;
} {
  const parts: string[] = [];
  parts.push(`peer_id=${record.peerId}`);
  parts.push(`nickname=${record.nickname}`);
  parts.push(`addr=${record.addr}`);

  if (record.capabilities) {
    parts.push(`caps=${record.capabilities}`);
  }

  if (record.metadata) {
    parts.push(`meta=${record.metadata}`);
  }

  const encoded = parts.join(' ');

  if (encoded.length > MAX_TXT_LENGTH) {
    return {
      success: false,
      result: `Record too long: ${encoded.length} bytes (max: ${MAX_TXT_LENGTH})`,
    };
  }

  return { success: true, result: encoded };
}

/// Decodes a string into a GigiDnsRecord
///
/// Parses the format: "peer_id=<id> nickname=<name> addr=<addr> caps=<caps> meta=<metadata>"
///
/// @param input - The encoded record string
/// @returns - The decoded record, or an error message if missing required fields
export function decodeGigiDnsRecord(input: string): {
  success: boolean;
  result: GigiDnsRecord | string;
} {
  let peerId: string | undefined;
  let nickname: string | undefined;
  let addr: string | undefined;
  let capabilities = '';
  let metadata = '';

  // Parse key=value pairs
  for (const pair of input.split(' ')) {
    const [key, value] = pair.split('=');
    if (!key) continue;

    const val = value || '';
    switch (key) {
      case 'peer_id':
        peerId = val;
        break;
      case 'nickname':
        nickname = val;
        break;
      case 'addr':
        addr = val;
        break;
      case 'caps':
        capabilities = val;
        break;
      case 'meta':
        metadata = val;
        break;
    }
  }

  // Validate required fields
  if (!peerId) {
    return { success: false, result: 'Missing peer_id' };
  }
  if (!nickname) {
    return { success: false, result: 'Missing nickname' };
  }
  if (!addr) {
    return { success: false, result: 'Missing addr' };
  }

  return {
    success: true,
    result: {
      peerId,
      nickname,
      addr,
      capabilities,
      metadata,
    },
  };
}
