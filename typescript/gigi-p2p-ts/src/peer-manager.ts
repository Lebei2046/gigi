import type { PeerInfo } from './types.js';

export class PeerManager {
  private peers: Map<string, PeerInfo> = new Map();
  private nicknameToPeerId: Map<string, string> = new Map();
  private connectedPeers: Set<string> = new Set();

  discover(peerId: string, nickname: string, addresses: string[]): void {
    const existing = this.peers.get(peerId);
    
    console.log(`[PeerManager] discover: peerId=${peerId}, nickname=${nickname}, addresses=${addresses.join(', ')}`);
    console.log(`[PeerManager] Existing peer: ${existing ? JSON.stringify(existing) : 'none'}`);
    
    const peer: PeerInfo = {
      peerId,
      nickname,
      addresses: existing && existing.addresses.length > 0 ? existing.addresses : addresses,
      lastSeen: Date.now(),
      connected: this.connectedPeers.has(peerId),
    };

    console.log(`[PeerManager] New peer: ${JSON.stringify(peer)}`);
    
    this.peers.set(peerId, peer);
    this.nicknameToPeerId.set(nickname, peerId);
  }

  addConnected(peerId: string, nickname: string): void {
    this.connectedPeers.add(peerId);

    const existing = this.peers.get(peerId);
    if (existing) {
      existing.connected = true;
      existing.lastSeen = Date.now();
      if (nickname) {
        existing.nickname = nickname;
        this.nicknameToPeerId.set(nickname, peerId);
      }
    } else {
      this.discover(peerId, nickname, []);
    }
  }

  removeConnected(peerId: string): void {
    this.connectedPeers.delete(peerId);

    const peer = this.peers.get(peerId);
    if (peer) {
      peer.connected = false;
    }
  }

  updateNickname(peerId: string, nickname: string): void {
    const peer = this.peers.get(peerId);
    if (peer) {
      for (const [nick, id] of this.nicknameToPeerId.entries()) {
        if (id === peerId && nick !== nickname) {
          this.nicknameToPeerId.delete(nick);
          break;
        }
      }

      peer.nickname = nickname;
      this.nicknameToPeerId.set(nickname, peerId);
    }
  }

  expire(peerId: string): void {
    this.peers.delete(peerId);

    for (const [nickname, id] of this.nicknameToPeerId.entries()) {
      if (id === peerId) {
        this.nicknameToPeerId.delete(nickname);
        break;
      }
    }
  }

  getPeerId(nickname: string): string | undefined {
    return this.nicknameToPeerId.get(nickname);
  }

  getNickname(peerId: string): string | undefined {
    return this.peers.get(peerId)?.nickname;
  }

  getByNickname(nickname: string): PeerInfo | undefined {
    const peerId = this.nicknameToPeerId.get(nickname);
    return peerId ? this.peers.get(peerId) : undefined;
  }

  getByPeerId(peerId: string): PeerInfo | undefined {
    return this.peers.get(peerId);
  }

  list(): PeerInfo[] {
    return Array.from(this.peers.values());
  }

  listConnected(): PeerInfo[] {
    return Array.from(this.peers.values()).filter(p => p.connected);
  }

  cleanup(maxAge: number = 3600000): void {
    const now = Date.now();

    for (const [peerId, peer] of this.peers.entries()) {
      if (now - peer.lastSeen > maxAge && !this.connectedPeers.has(peerId)) {
        this.expire(peerId);
      }
    }
  }
}