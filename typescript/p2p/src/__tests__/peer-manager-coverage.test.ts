import { describe, it, expect, beforeEach, vi } from 'vitest';
import { PeerManager } from '../peer-manager';

describe('PeerManager Coverage Tests', () => {
  let peerManager: PeerManager;

  beforeEach(() => {
    peerManager = new PeerManager();
  });

  it('should handle discover with existing peer and changed nickname', () => {
    // Add a peer with initial nickname
    peerManager.discover('peer1', 'nickname1', ['/ip4/127.0.0.1/tcp/1234']);
    expect(peerManager.getNickname('peer1')).toBe('nickname1');
    expect(peerManager.getPeerId('nickname1')).toBe('peer1');

    // Discover the same peer with a different nickname
    peerManager.discover('peer1', 'nickname2', ['/ip4/127.0.0.1/tcp/5678']);
    expect(peerManager.getNickname('peer1')).toBe('nickname2');
    expect(peerManager.getPeerId('nickname2')).toBe('peer1');
    expect(peerManager.getPeerId('nickname1')).toBeUndefined();
  });

  it('should handle discover with existing peer and keep existing addresses', () => {
    // Add a peer with addresses
    peerManager.discover('peer1', 'nickname1', ['/ip4/127.0.0.1/tcp/1234']);
    const peer1 = peerManager.getByPeerId('peer1');
    expect(peer1?.addresses).toEqual(['/ip4/127.0.0.1/tcp/1234']);

    // Discover the same peer with new addresses
    peerManager.discover('peer1', 'nickname1', ['/ip4/127.0.0.1/tcp/5678']);
    const peer1Updated = peerManager.getByPeerId('peer1');
    expect(peer1Updated?.addresses).toEqual(['/ip4/127.0.0.1/tcp/1234']); // Should keep existing addresses
  });

  it('should handle addConnected with non-existent peer', () => {
    // Add a connected peer that doesn't exist
    peerManager.addConnected('peer1', 'nickname1');
    const peer1 = peerManager.getByPeerId('peer1');
    expect(peer1).toBeDefined();
    expect(peer1?.nickname).toBe('nickname1');
    expect(peer1?.connected).toBe(true);
  });

  it('should handle addConnected with existing peer and update nickname', () => {
    // Add a peer with initial nickname
    peerManager.discover('peer1', 'nickname1', ['/ip4/127.0.0.1/tcp/1234']);
    expect(peerManager.getNickname('peer1')).toBe('nickname1');

    // Add as connected with new nickname
    peerManager.addConnected('peer1', 'nickname2');
    const peer1 = peerManager.getByPeerId('peer1');
    expect(peer1?.nickname).toBe('nickname2');
    expect(peer1?.connected).toBe(true);
    expect(peerManager.getPeerId('nickname2')).toBe('peer1');
  });

  it('should handle removeConnected', () => {
    // Add a connected peer
    peerManager.addConnected('peer1', 'nickname1');
    let peer1 = peerManager.getByPeerId('peer1');
    expect(peer1?.connected).toBe(true);

    // Remove from connected
    peerManager.removeConnected('peer1');
    peer1 = peerManager.getByPeerId('peer1');
    expect(peer1?.connected).toBe(false);
  });

  it('should handle updateNickname with existing peer', () => {
    // Add a peer with initial nickname
    peerManager.discover('peer1', 'nickname1', ['/ip4/127.0.0.1/tcp/1234']);
    expect(peerManager.getNickname('peer1')).toBe('nickname1');
    expect(peerManager.getPeerId('nickname1')).toBe('peer1');

    // Update nickname
    peerManager.updateNickname('peer1', 'nickname2');
    expect(peerManager.getNickname('peer1')).toBe('nickname2');
    expect(peerManager.getPeerId('nickname2')).toBe('peer1');
    expect(peerManager.getPeerId('nickname1')).toBeUndefined();
  });

  it('should handle updateNickname with non-existent peer', () => {
    // Update nickname for non-existent peer (should not throw)
    expect(() =>
      peerManager.updateNickname('non-existent-peer', 'nickname1')
    ).not.toThrow();
  });

  it('should handle expire peer', () => {
    // Add a peer
    peerManager.discover('peer1', 'nickname1', ['/ip4/127.0.0.1/tcp/1234']);
    expect(peerManager.getByPeerId('peer1')).toBeDefined();
    expect(peerManager.getPeerId('nickname1')).toBe('peer1');

    // Expire the peer
    peerManager.expire('peer1');
    expect(peerManager.getByPeerId('peer1')).toBeUndefined();
    expect(peerManager.getPeerId('nickname1')).toBeUndefined();
  });

  it('should handle getByNickname with non-existent nickname', () => {
    const peer = peerManager.getByNickname('non-existent-nickname');
    expect(peer).toBeUndefined();
  });

  it('should handle listConnected method', () => {
    // Add some peers
    peerManager.discover('peer1', 'nickname1', ['/ip4/127.0.0.1/tcp/1234']);
    peerManager.discover('peer2', 'nickname2', ['/ip4/127.0.0.1/tcp/5678']);

    // Connect only one
    peerManager.addConnected('peer1', 'nickname1');

    const connectedPeers = peerManager.listConnected();
    expect(connectedPeers.length).toBe(1);
    expect(connectedPeers[0].peerId).toBe('peer1');
  });

  it('should handle cleanup method with expired peers', () => {
    // Mock Date.now to control time
    const mockNow = vi.spyOn(Date, 'now').mockReturnValue(1000000);

    // Add a peer with old lastSeen
    peerManager.discover('peer1', 'nickname1', ['/ip4/127.0.0.1/tcp/1234']);
    const peer1 = peerManager.getByPeerId('peer1');
    if (peer1) {
      // Manually set old lastSeen
      (peer1 as any).lastSeen = 1000; // 999 seconds old
    }

    // Add a recent peer
    mockNow.mockReturnValue(1000000);
    peerManager.discover('peer2', 'nickname2', ['/ip4/127.0.0.1/tcp/5678']);

    // Add a connected peer (should not be cleaned up)
    peerManager.addConnected('peer3', 'nickname3');
    const peer3 = peerManager.getByPeerId('peer3');
    if (peer3) {
      (peer3 as any).lastSeen = 1000; // Old but connected
    }

    // Cleanup with 500 second max age
    peerManager.cleanup(500000); // 500 seconds

    expect(peerManager.getByPeerId('peer1')).toBeUndefined(); // Should be cleaned up
    expect(peerManager.getByPeerId('peer2')).toBeDefined(); // Should not be cleaned up
    expect(peerManager.getByPeerId('peer3')).toBeDefined(); // Should not be cleaned up (connected)

    // Restore Date.now
    mockNow.mockRestore();
  });

  it('should handle cleanup with default max age', () => {
    // Add a peer
    peerManager.discover('peer1', 'nickname1', ['/ip4/127.0.0.1/tcp/1234']);
    expect(peerManager.getByPeerId('peer1')).toBeDefined();

    // Cleanup with default max age (1 hour)
    peerManager.cleanup();

    // Peer should still exist (not expired)
    expect(peerManager.getByPeerId('peer1')).toBeDefined();
  });

  it('should handle multiple peers with same nickname', () => {
    // Add first peer
    peerManager.discover('peer1', 'nickname1', ['/ip4/127.0.0.1/tcp/1234']);
    expect(peerManager.getPeerId('nickname1')).toBe('peer1');

    // Add second peer with same nickname (should override)
    peerManager.discover('peer2', 'nickname1', ['/ip4/127.0.0.1/tcp/5678']);
    expect(peerManager.getPeerId('nickname1')).toBe('peer2'); // Should now point to peer2
  });
});
