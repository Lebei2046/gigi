import { describe, it, expect, beforeEach } from 'vitest';
import { PeerManager } from '../peer-manager.js';

describe('PeerManager', () => {
  let peerManager: PeerManager;

  beforeEach(() => {
    peerManager = new PeerManager();
  });

  it('should initialize with empty peers', () => {
    expect(peerManager).toBeInstanceOf(PeerManager);
    expect(peerManager.list()).toEqual([]);
  });

  it('should add a peer', () => {
    const peerId = 'peer1';
    const nickname = 'Peer 1';
    peerManager.discover(peerId, nickname, []);

    const peers = peerManager.list();
    expect(peers.length).toBe(1);
    expect(peers[0].peerId).toBe(peerId);
    expect(peers[0].nickname).toBe(nickname);
  });

  it('should remove a peer', () => {
    const peerId = 'peer1';
    const nickname = 'Peer 1';
    peerManager.discover(peerId, nickname, []);
    expect(peerManager.list().length).toBe(1);

    peerManager.expire(peerId);
    expect(peerManager.list()).toEqual([]);
  });

  it('should get a peer by ID', () => {
    const peerId = 'peer1';
    const nickname = 'Peer 1';
    peerManager.discover(peerId, nickname, []);

    const peer = peerManager.getByPeerId(peerId);
    expect(peer?.peerId).toBe(peerId);
    expect(peer?.nickname).toBe(nickname);
  });

  it('should return undefined for non-existent peer ID', () => {
    const peer = peerManager.getByPeerId('non-existent-peer');
    expect(peer).toBeUndefined();
  });

  it('should get a peer by nickname', () => {
    const peerId = 'peer1';
    const nickname = 'Peer 1';
    peerManager.discover(peerId, nickname, []);

    const foundPeerId = peerManager.getPeerId(nickname);
    expect(foundPeerId).toBe(peerId);
  });

  it('should return undefined for non-existent nickname', () => {
    const peerId = peerManager.getPeerId('non-existent-nickname');
    expect(peerId).toBeUndefined();
  });

  it('should list all peers', () => {
    peerManager.discover('peer1', 'Peer 1', []);
    peerManager.discover('peer2', 'Peer 2', []);
    peerManager.discover('peer3', 'Peer 3', []);

    const peers = peerManager.list();
    expect(peers.length).toBe(3);
    expect(peers.map((p) => p.peerId)).toEqual(['peer1', 'peer2', 'peer3']);
    expect(peers.map((p) => p.nickname)).toEqual([
      'Peer 1',
      'Peer 2',
      'Peer 3',
    ]);
  });

  it('should update an existing peer', () => {
    const peerId = 'peer1';
    const oldNickname = 'Old Nickname';
    const newNickname = 'New Nickname';

    peerManager.discover(peerId, oldNickname, []);
    expect(peerManager.getByPeerId(peerId)?.nickname).toBe(oldNickname);

    peerManager.updateNickname(peerId, newNickname);
    expect(peerManager.getByPeerId(peerId)?.nickname).toBe(newNickname);
  });
});
