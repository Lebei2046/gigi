// Tests for Gigi DNS behaviour

import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import type { PeerId } from '@libp2p/interface';
import { GigiDnsBehaviour, GigiDnsCommand } from '../behaviour';
import { defaultGigiDnsConfig } from '../types';
import { multiaddr } from '@multiformats/multiaddr';

describe('GigiDnsBehaviour', () => {
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

  it('should create a valid instance', () => {
    expect(behaviour).toBeInstanceOf(GigiDnsBehaviour);
  });

  it('should update listen addresses', () => {
    const addr = multiaddr('/ip4/192.168.1.100/tcp/8000');
    behaviour.updateListenAddresses([addr]);

    // We can't directly access the private listenAddresses, but we can verify the method runs without errors
    expect(() => behaviour.updateListenAddresses([addr])).not.toThrow();
  });

  it('should handle commands', () => {
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

    // All commands should run without errors
    expect(() =>
      behaviour.handleCommand({
        type: GigiDnsCommand.UpdateNickname,
        nickname: 'TestNickname',
      })
    ).not.toThrow();
  });

  it('should get discovered peers', () => {
    const peers = behaviour.getDiscoveredPeers();
    expect(peers).toBeInstanceOf(Map);
    expect(peers.size).toBe(0);
  });

  it('should find peer by ID', () => {
    // Create a mock PeerId with all required properties
    const mockPeerId = {
      toString: () => '12D3KooWBob',
      toBytes: () => new Uint8Array([1, 2, 3]),
      type: 'Ed25519',
      multihash: new Uint8Array([1, 2, 3]),
      toCID: () => ({ toString: () => 'cid' }),
      equals: () => false,
    } as unknown as PeerId;

    const peer = behaviour.findPeerById(mockPeerId);
    expect(peer).toBeUndefined();
  });

  it('should find peer by nickname', () => {
    const peer = behaviour.findPeerByNickname('TestNickname');
    expect(peer).toBeUndefined();
  });

  it('should stop gracefully', () => {
    expect(() => behaviour.stop()).not.toThrow();
  });

  it('should handle events', () => {
    // We can't directly trigger a discovery event, but we can verify the event listener is added
    expect(() => behaviour.on('Discovered', () => {})).not.toThrow();
  });
});
