import type { MessageContent } from '@gigi/message-types';

export interface PeerInfo {
  peerId: string;
  nickname: string;
  addresses: string[];
  lastSeen: number;
  connected: boolean;
}

export interface GroupInfo {
  name: string;
  topic: string;
  joinedAt: number;
}

export interface FileInfo {
  fileId: string;
  shareCode: string;
  name: string;
  size: number;
  mimeType: string;
  chunkCount: number;
  hash: string;
  createdAt: number;
  revoked: boolean;
}

export interface ChunkInfo {
  fileId: string;
  chunkIndex: number;
  data: Uint8Array;
  hash: string;
}

export interface ActiveDownload {
  downloadId: string;
  filename: string;
  shareCode: string;
  fromPeerId: string;
  fromNickname: string;
  totalChunks: number;
  downloadedChunks: number;
  startedAt: number;
  completed: boolean;
  failed: boolean;
  errorMessage?: string;
  finalPath?: string;
  data: Uint8Array[];
}

export interface GroupMessage {
  senderNickname: string;
  content: string;
  timestamp: number;
  hasFileShare: boolean;
  shareCode?: string;
  filename?: string;
  fileSize?: number;
  fileType?: string;
}

export interface DirectMessage {
  from: string;
  fromNickname: string;
  content: string;
  timestamp: number;
}

export interface SharedFile {
  fileId: string;
  shareCode: string;
  info: FileInfo;
}

export type MessageType = 'direct' | 'group';
export type MessageDirection = 'sent' | 'received';
export type SyncStatus = 'pending' | 'synced' | 'delivered' | 'acknowledged';
export type QueueStatus = 'pending' | 'in_progress' | 'delivered' | 'expired';

export interface StoredMessage {
  id: string;
  msgType: MessageType;
  direction: MessageDirection;
  content: MessageContent;
  senderNickname: string;
  recipientNickname?: string;
  groupName?: string;
  peerId: string;
  timestamp: number;
  createdAt: number;
  delivered: boolean;
  deliveredAt?: number;
  read: boolean;
  readAt?: number;
  syncStatus: SyncStatus;
  syncAttempts: number;
  lastSyncAttempt?: number;
  expiresAt: number;
}

// Re-export shared message types
export type { MessageContent } from '@gigi/message-types';
export type { MessageContentInput } from '@gigi/message-types';

export interface P2pConfig {
  bootstrapNodes: string[];
  enableKademlia: boolean;
  enableRelay: boolean;
  enableMdns: boolean;
  listenAddrs: string[];
  nicknames?: Record<string, string>;
}

export const DEFAULT_CONFIG: P2pConfig = {
  bootstrapNodes: [],
  enableKademlia: true,
  enableRelay: true,
  enableMdns: true,
  listenAddrs: ['/ip4/0.0.0.0/tcp/0'],
};
