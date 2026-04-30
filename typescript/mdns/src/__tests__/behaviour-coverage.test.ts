// Additional tests for GigiDnsBehaviour

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import type { PeerId } from '@libp2p/interface';
import { GigiDnsBehaviour, GigiDnsCommand } from '../behaviour';
import { defaultGigiDnsConfig, GigiDnsEvent, OfflineReason } from '../types';
import { multiaddr } from '@multiformats/multiaddr';

// Mock dgram module
vi.mock('dgram', () => ({
  createSocket: vi.fn(() => ({
    on: vi.fn(),
    bind: vi.fn((port, address, callback) => {
      // Simulate successful binding
      callback();
    }),
    addMembership: vi.fn(),
    send: vi.fn((msg, offset, length, port, address, callback) => {
      // Simulate successful send
      callback(null);
    }),
    close: vi.fn((callback) => {
      callback();
    }),
  })),
}));

// Mock logging
vi.mock('@gigi/logging', () => ({
  createLogger: vi.fn(() => ({
    info: vi.fn(),
    error: vi.fn(),
  })),
}));

describe('GigiDnsBehaviour Coverage Tests', () => {
  let behaviour: GigiDnsBehaviour;
  let localPeerId: { toString: () => string; toBytes: () => Uint8Array };

  beforeEach(() => {
    // Mock peer ID
    localPeerId = {
      toString: () => '12D3KooWBob',
      toBytes: () => new Uint8Array([1, 2, 3]),
    };
    behaviour = new GigiDnsBehaviour(
      localPeerId as unknown as PeerId,
      defaultGigiDnsConfig
    );
  });

  afterEach(() => {
    behaviour.stop();
  });

  it('should start service and send initial query', () => {
    // Start the service
    behaviour.startService();

    // The service should start without errors
    expect(() => behaviour.startService()).not.toThrow();
  });

  it('should emit discovered peers', () => {
    // Mock event listener
    const mockListener = vi.fn();
    behaviour.on('Discovered', mockListener);

    // Emit discovered peers
    behaviour.emitDiscoveredPeers();

    // The listener should be called for each discovered peer
    // Since we haven't discovered any peers yet, it should not be called
    expect(mockListener).not.toHaveBeenCalled();
  });

  it('should handle sendQuery when socket is not ready', () => {
    // Mock the socket bound status
    (behaviour as any).socketBound = false;

    // Send query (should be queued)
    behaviour.sendQuery();

    // The query should be queued without errors
    expect(() => behaviour.sendQuery()).not.toThrow();
  });

  it('should handle event listeners with off method', () => {
    // Mock event listener
    const mockListener = vi.fn();

    // Add listener
    behaviour.on('Discovered', mockListener);

    // Remove listener
    behaviour.off('Discovered', mockListener);

    // The off method should not throw
    expect(() => behaviour.off('Discovered', mockListener)).not.toThrow();
  });

  it('should handle cleanup of expired peers', () => {
    // Add a peer with old expiration time
    const mockPeerId = {
      toString: () => '12D3KooWPeer',
      toBytes: () => new Uint8Array([4, 5, 6]),
    } as unknown as PeerId;

    // Add peer to discoveredPeers
    const peerInfo = {
      peerId: mockPeerId,
      nickname: 'TestPeer',
      multiaddr: multiaddr('/ip4/127.0.0.1/tcp/1234'),
      capabilities: [],
      metadata: {},
      expiresAt: new Date(Date.now() - 10000), // 10 seconds ago
    };
    (behaviour as any).discoveredPeers.set(mockPeerId.toString(), peerInfo);

    // Cleanup
    (behaviour as any).cleanup();

    // The peer should be removed
    expect((behaviour as any).discoveredPeers.size).toBe(0);
  });

  it('should handle processEvent for different event types', () => {
    // Mock event listener
    const mockListener = vi.fn();
    behaviour.on('*', mockListener);

    // Test Discovered event
    const mockPeerId = {
      toString: () => '12D3KooWPeer',
      toBytes: () => new Uint8Array([4, 5, 6]),
    } as unknown as PeerId;

    const now = new Date();
    const peerInfo = {
      peerId: mockPeerId,
      nickname: 'TestPeer',
      multiaddr: multiaddr('/ip4/127.0.0.1/tcp/1234'),
      capabilities: [],
      metadata: {},
      discoveredAt: now,
      expiresAt: new Date(Date.now() + 60000),
    };

    const discoveredEvent: GigiDnsEvent = {
      type: 'Discovered',
      peerInfo,
    };

    (behaviour as any).processEvent(discoveredEvent);

    // Test Updated event
    const updatedEvent: GigiDnsEvent = {
      type: 'Updated',
      peerId: mockPeerId,
      oldInfo: peerInfo,
      newInfo: {
        ...peerInfo,
        nickname: 'UpdatedPeer',
      },
    };

    (behaviour as any).processEvent(updatedEvent);

    // Test Expired event
    const expiredEvent: GigiDnsEvent = {
      type: 'Expired',
      peerId: mockPeerId,
      info: peerInfo,
    };

    (behaviour as any).processEvent(expiredEvent);

    // Test Offline event
    const offlineEvent: GigiDnsEvent = {
      type: 'Offline',
      peerId: mockPeerId,
      info: peerInfo,
      reason: OfflineReason.TtlExpired,
    };

    (behaviour as any).processEvent(offlineEvent);

    // The listener should be called for each event
    expect(mockListener).toHaveBeenCalled();
  });

  it('should handle updateListenAddresses method', () => {
    const addr = multiaddr('/ip4/192.168.1.100/tcp/8000');

    // Update listen addresses
    behaviour.updateListenAddresses([addr]);

    // The method should not throw
    expect(() => behaviour.updateListenAddresses([addr])).not.toThrow();
  });

  it('should handle findPeerByNickname with existing peer', () => {
    // Add a peer
    const mockPeerId = {
      toString: () => '12D3KooWPeer',
      toBytes: () => new Uint8Array([4, 5, 6]),
    } as unknown as PeerId;

    const peerInfo = {
      peerId: mockPeerId,
      nickname: 'TestPeer',
      multiaddr: multiaddr('/ip4/127.0.0.1/tcp/1234'),
      capabilities: [],
      metadata: {},
      expiresAt: new Date(Date.now() + 60000),
    };
    (behaviour as any).discoveredPeers.set(mockPeerId.toString(), peerInfo);

    // Find peer by nickname
    const foundPeer = behaviour.findPeerByNickname('TestPeer');

    // The peer should be found
    expect(foundPeer).toBeDefined();
    expect(foundPeer?.nickname).toBe('TestPeer');
  });

  it('should handle handleCommand with different command types', () => {
    // Test UpdateNickname command
    behaviour.handleCommand({
      type: GigiDnsCommand.UpdateNickname,
      nickname: 'NewNickname',
    });

    // Test UpdateCapabilities command
    behaviour.handleCommand({
      type: GigiDnsCommand.UpdateCapabilities,
      capabilities: ['chat', 'file_sharing'],
    });

    // Test UpdateMetadata command
    behaviour.handleCommand({
      type: GigiDnsCommand.UpdateMetadata,
      key: 'version',
      value: '1.0',
    });

    // All commands should execute without errors
    expect(() => {
      behaviour.handleCommand({
        type: GigiDnsCommand.UpdateNickname,
        nickname: 'TestNickname',
      });
    }).not.toThrow();
  });

  it('should handle start method', async () => {
    // The start method should not throw
    await expect(behaviour.start()).resolves.not.toThrow();
  });

  it('should handle stop method multiple times', () => {
    // Stop the behaviour
    behaviour.stop();

    // Stop again (should not throw)
    expect(() => behaviour.stop()).not.toThrow();
  });

  it('should handle event emission with multiple listeners', () => {
    // Mock event listeners
    const mockListener1 = vi.fn();
    const mockListener2 = vi.fn();

    // Add listeners
    behaviour.on('Discovered', mockListener1);
    behaviour.on('Discovered', mockListener2);

    // Emit an event
    const mockPeerId = {
      toString: () => '12D3KooWPeer',
      toBytes: () => new Uint8Array([4, 5, 6]),
    } as unknown as PeerId;

    const discoveredEvent: GigiDnsEvent = {
      type: 'Discovered',
      peerInfo: {
        peerId: mockPeerId,
        nickname: 'TestPeer',
        multiaddr: multiaddr('/ip4/127.0.0.1/tcp/1234'),
        capabilities: [],
        metadata: {},
        discoveredAt: new Date(),
        expiresAt: new Date(Date.now() + 60000),
      },
    };

    (behaviour as any).emit(discoveredEvent);

    // Both listeners should be called
    expect(mockListener1).toHaveBeenCalled();
    expect(mockListener2).toHaveBeenCalled();
  });

  it('should handle error events', () => {
    // Mock error listener
    const mockListener = vi.fn();
    behaviour.on('Error', mockListener);

    // Emit an error event
    const testError = new Error('Test error');
    (behaviour as any).emitError(testError, 'Test context');

    // The listener should be called
    expect(mockListener).toHaveBeenCalled();
  });
});
