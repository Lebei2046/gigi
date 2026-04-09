// Gigi DNS Behaviour - libp2p integration with network interface support
//
// This module implements the libp2p NetworkBehaviour interface for Gigi DNS.
// It orchestrates per-interface mDNS handlers and manages peer discovery state.

import type { PeerId } from '@libp2p/interface';
import { Multiaddr } from '@multiformats/multiaddr';
import { GigiDnsConfig, GigiDnsEvent, GigiPeerInfo } from './types';
import { GigiDnsProtocol } from './protocol';
import * as dgram from 'dgram';
import { createSocket } from 'dgram';
// networkInterfaces is available for future use when per-interface socket binding is needed
// import { networkInterfaces } from 'os';

/// Commands that can be sent to the GigiDnsBehaviour
///
/// These commands allow runtime updates to the advertised information.
export enum GigiDnsCommand {
  /// Update the advertised nickname
  UpdateNickname = 'UPDATE_NICKNAME',
  /// Update the list of capabilities
  UpdateCapabilities = 'UPDATE_CAPABILITIES',
  /// Update or add a metadata key-value pair
  UpdateMetadata = 'UPDATE_METADATA',
}

/// Interface for Gigi DNS command parameters
export type GigiDnsCommandParams =
  | { type: GigiDnsCommand.UpdateNickname; nickname: string }
  | { type: GigiDnsCommand.UpdateCapabilities; capabilities: string[] }
  | { type: GigiDnsCommand.UpdateMetadata; key: string; value: string };

/// Gigi DNS service for libp2p
///
/// This service manages DNS-based peer discovery.
/// It creates per-interface UDP sockets for mDNS communication and aggregates discovered peers.
///
/// # Architecture
/// - Monitors network interfaces using Node.js os.networkInterfaces()
/// - Creates one UDP socket per network interface
/// - Each socket runs independently and processes DNS packets
/// - Central peer state management in discoveredPeers (single source of truth)
/// - Broadcasts address changes to all interface sockets
export class GigiDnsBehaviour {
  private config: GigiDnsConfig;
  private localPeerId: PeerId;
  private listenAddresses: Multiaddr[];
  private discoveredPeers: Map<string, GigiPeerInfo>; // peerId string -> peer info
  private protocol: GigiDnsProtocol;
  private cleanupInterval: NodeJS.Timeout | null;
  private udpSocket: dgram.Socket | null = null; // Single UDP socket bound to 0.0.0.0

  /// Creates a new GigiDnsBehaviour instance
  ///
  /// @param localPeerId - Our libp2p peer ID
  /// @param config - Configuration for DNS behavior
  constructor(localPeerId: PeerId, config: GigiDnsConfig) {
    this.config = config;
    this.localPeerId = localPeerId;
    this.listenAddresses = [];
    this.discoveredPeers = new Map();
    this.protocol = new GigiDnsProtocol(localPeerId, config);
    this.cleanupInterval = null;

    // Create single UDP socket bound to 0.0.0.0
    this.createUdpSocket();

    // Start cleanup interval
    this.startCleanupInterval();
  }

  /// Creates a single UDP socket bound to 0.0.0.0
  private createUdpSocket(): void {
    try {
      // Create socket with reuseAddr to allow multiple processes to bind to the same port
      this.udpSocket = createSocket({ type: 'udp4', reuseAddr: true });

      this.udpSocket.on('error', (err) => {
        this.emitError(err as Error, 'Socket error');
      });

      this.udpSocket.on('message', (msg, rinfo) => {
        this.handleMessage(msg, rinfo, '0.0.0.0');
      });

      // Bind to the Gigi DNS port on all interfaces
      this.udpSocket.bind(GIGI_DNS_PORT, '0.0.0.0', () => {
        try {
          // Enable multicast on the default interface
          this.udpSocket!.addMembership(IPV4_MDNS_MULTICAST_ADDRESS);
          if (this.config.enableIpv6) {
            this.udpSocket!.addMembership(IPV6_MDNS_MULTICAST_ADDRESS);
          }
          console.log(`Gigi DNS listening on 0.0.0.0:${GIGI_DNS_PORT}`);
        } catch (err) {
          this.emitError(err as Error, 'Failed to configure multicast');
        }
      });

      // Send initial query
      this.sendQuery();
    } catch (err) {
      this.emitError(err as Error, 'Failed to create UDP socket');
    }
  }

  /// Handles incoming UDP messages
  ///
  /// @param msg - Incoming message
  /// @param rinfo - Remote address information
  /// @param interfaceAddress - Local interface address
  private handleMessage(
    msg: Buffer,
    rinfo: { address: string; port: number; family: string; size: number },
    _interfaceAddress: string
  ): void {
    // Process the packet with the protocol, passing the source address for validation
    const result = this.protocol.handlePacket(msg, rinfo.address);

    if (result.success && result.result) {
      const event = result.result as GigiDnsEvent;
      this.processEvent(event);
    }

    // If it's a query, send a response
    if (this.protocol.isQuery(msg)) {
      this.sendResponse();
    }
  }

  /// Sends a DNS query using the single socket
  private sendQuery(): void {
    if (!this.udpSocket) return;

    const query = this.protocol.buildQuery();

    // Send to IPv4 multicast address
    this.udpSocket.send(
      query,
      0,
      query.length,
      GIGI_DNS_PORT,
      IPV4_MDNS_MULTICAST_ADDRESS,
      (err: Error | null) => {
        if (err) {
          this.emitError(err, 'Error sending DNS query');
        }
      }
    );

    // If IPv6 is enabled, send to IPv6 multicast address
    if (this.config.enableIpv6) {
      this.udpSocket.send(
        query,
        0,
        query.length,
        GIGI_DNS_PORT,
        IPV6_MDNS_MULTICAST_ADDRESS,
        (err: Error | null) => {
          if (err) {
            this.emitError(err, 'Error sending IPv6 DNS query');
          }
        }
      );
    }
  }

  /// Sends DNS responses for all listen addresses
  ///
  /// @param interfaceAddress - Local interface address
  private sendResponse(): void {
    if (!this.udpSocket) return;

    const responseResult = this.protocol.buildResponse();
    if (!responseResult.success) {
      this.emitError(
        new Error(responseResult.result as string),
        'Error building response'
      );
      return;
    }

    const packets = responseResult.result as Uint8Array[];

    for (const packet of packets) {
      // Send to IPv4 multicast address
      this.udpSocket.send(
        packet,
        0,
        packet.length,
        GIGI_DNS_PORT,
        IPV4_MDNS_MULTICAST_ADDRESS,
        (err: Error | null) => {
          if (err) {
            this.emitError(err, 'Error sending DNS response');
          }
        }
      );

      // If IPv6 is enabled, send to IPv6 multicast address
      if (this.config.enableIpv6) {
        this.udpSocket.send(
          packet,
          0,
          packet.length,
          GIGI_DNS_PORT,
          IPV6_MDNS_MULTICAST_ADDRESS,
          (err: Error | null) => {
            if (err) {
              this.emitError(err, 'Error sending IPv6 DNS response');
            }
          }
        );
      }
    }
  }

  /// Starts the cleanup interval for expired peers and pending queries
  private startCleanupInterval(): void {
    this.cleanupInterval = setInterval(() => {
      this.cleanup();
    }, this.config.cleanupInterval);
  }

  /// Cleans up expired peers and pending queries
  private cleanup(): void {
    // Cleanup expired peers
    const now = new Date();
    for (const [peerId, peerInfo] of this.discoveredPeers.entries()) {
      if (now > peerInfo.expiresAt) {
        this.emit({
          type: 'Expired',
          peerId: peerInfo.peerId,
          info: peerInfo,
        });
        this.discoveredPeers.delete(peerId);
      }
    }

    // Cleanup protocol's pending queries
    this.protocol.cleanupExpired();
  }

  /// Handles commands sent to the behaviour
  ///
  /// @param command - Command to execute
  handleCommand(command: GigiDnsCommandParams): void {
    switch (command.type) {
      case GigiDnsCommand.UpdateNickname:
        this.protocol.updateNickname(command.nickname);
        break;
      case GigiDnsCommand.UpdateCapabilities:
        this.config.capabilities = command.capabilities;
        break;
      case GigiDnsCommand.UpdateMetadata:
        this.config.metadata[command.key] = command.value;
        break;
    }
  }

  // Event emitter functionality
  private eventListeners: Map<string, ((event: GigiDnsEvent) => void)[]> =
    new Map();

  /// Adds an event listener
  ///
  /// @param event - Event type to listen for
  /// @param listener - Listener function
  on(event: string, listener: (event: GigiDnsEvent) => void): void {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, []);
    }
    this.eventListeners.get(event)!.push(listener);
  }

  /// Removes an event listener
  ///
  /// @param event - Event type to remove listener from
  /// @param listener - Listener function to remove
  off(event: string, listener: (event: GigiDnsEvent) => void): void {
    if (this.eventListeners.has(event)) {
      const listeners = this.eventListeners.get(event)!;
      const index = listeners.indexOf(listener);
      if (index > -1) {
        listeners.splice(index, 1);
      }
    }
  }

  /// Emits an event to all listeners
  ///
  /// @param event - Event to emit
  private emit(event: GigiDnsEvent): void {
    const listeners = this.eventListeners.get('*') || [];
    listeners.forEach((listener) => listener(event));
    const typeListeners = this.eventListeners.get(event.type) || [];
    typeListeners.forEach((listener) => listener(event));
  }

  /// Emits an error event and logs the error
  ///
  /// @param error - The error that occurred
  /// @param context - Contextual information about where the error occurred
  private emitError(error: Error, context: string): void {
    console.error(`Gigi DNS error (${context}):`, error);
    this.emit({ type: 'Error', error, context });
  }

  /// Processes a Gigi DNS event and emits it
  ///
  /// @param event - Gigi DNS event
  private processEvent(event: GigiDnsEvent): void {
    switch (event.type) {
      case 'Discovered': {
        const peerIdStr = event.peerInfo.peerId.toString();
        const existingPeer = this.discoveredPeers.get(peerIdStr);

        if (existingPeer) {
          // Check if this is an update
          if (
            existingPeer.nickname !== event.peerInfo.nickname ||
            existingPeer.multiaddr.toString() !==
              event.peerInfo.multiaddr.toString()
          ) {
            // Generate Updated event
            const updatedEvent: GigiDnsEvent = {
              type: 'Updated',
              peerId: event.peerInfo.peerId,
              oldInfo: existingPeer,
              newInfo: event.peerInfo,
            };
            this.discoveredPeers.set(peerIdStr, event.peerInfo);
            this.emit(updatedEvent);
          } else {
            // Update TTL
            this.discoveredPeers.set(peerIdStr, event.peerInfo);
          }
        } else {
          // New peer
          this.discoveredPeers.set(peerIdStr, event.peerInfo);
          this.emit(event);
        }
        break;
      }
      case 'Expired': {
        const peerIdStr = event.peerId.toString();
        this.discoveredPeers.delete(peerIdStr);
        this.emit(event);
        break;
      }
      case 'Offline': {
        const peerIdStr = event.peerId.toString();
        this.discoveredPeers.delete(peerIdStr);
        this.emit(event);
        break;
      }
      case 'Updated': {
        const peerIdStr = event.peerId.toString();
        this.discoveredPeers.set(peerIdStr, event.newInfo);
        this.emit(event);
        break;
      }
    }
  }

  /// Updates the listen addresses
  ///
  /// @param addresses - New listen addresses
  updateListenAddresses(addresses: Multiaddr[]): void {
    this.listenAddresses = addresses;
    this.protocol.updateListenAddresses(addresses);
  }

  /// Gets the list of discovered peers
  ///
  /// @returns Map of peer ID strings to peer information
  getDiscoveredPeers(): Map<string, GigiPeerInfo> {
    return this.discoveredPeers;
  }

  /// Finds a peer by peer ID
  ///
  /// @param peerId - Peer ID to search for
  /// @returns Peer information if found, undefined otherwise
  findPeerById(peerId: PeerId): GigiPeerInfo | undefined {
    return this.discoveredPeers.get(peerId.toString());
  }

  /// Finds a peer by nickname
  ///
  /// @param nickname - Nickname to search for
  /// @returns Peer information if found, undefined otherwise
  findPeerByNickname(nickname: string): GigiPeerInfo | undefined {
    for (const peerInfo of this.discoveredPeers.values()) {
      if (peerInfo.nickname === nickname) {
        return peerInfo;
      }
    }
    return undefined;
  }

  /// Stops the behaviour and cleans up resources
  stop(): void {
    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval);
      this.cleanupInterval = null;
    }

    // Close the UDP socket
    if (this.udpSocket) {
      this.udpSocket.close(() => {
        console.log('Gigi DNS UDP socket closed');
      });
      this.udpSocket = null;
    }
  }
}

// Helper types and constants

import {
  GIGI_DNS_PORT,
  IPV4_MDNS_MULTICAST_ADDRESS,
  IPV6_MDNS_MULTICAST_ADDRESS,
} from './types';
