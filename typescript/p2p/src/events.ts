import type { MessageContent } from './types';

export enum P2pEventType {
  PEER_DISCOVERED = 'peer-discovered',
  PEER_EXPIRED = 'peer-expired',
  NICKNAME_UPDATED = 'nickname-updated',
  DIRECT_MESSAGE = 'direct-message',
  DIRECT_FILE_SHARE_MESSAGE = 'direct-file-share-message',
  DIRECT_GROUP_SHARE_MESSAGE = 'direct-group-share-message',
  GROUP_MESSAGE = 'group-message',
  GROUP_FILE_SHARE_MESSAGE = 'group-file-share-message',
  GROUP_JOINED = 'group-joined',
  GROUP_LEFT = 'group-left',
  FILE_SHARE_REQUEST = 'file-share-request',
  FILE_SHARED = 'file-shared',
  FILE_REVOKED = 'file-revoked',
  FILE_INFO_RECEIVED = 'file-info-received',
  CHUNK_RECEIVED = 'chunk-received',
  FILE_LIST_RECEIVED = 'file-list-received',
  FILE_DOWNLOAD_STARTED = 'file-download-started',
  FILE_DOWNLOAD_PROGRESS = 'file-download-progress',
  FILE_DOWNLOAD_COMPLETED = 'file-download-completed',
  FILE_DOWNLOAD_FAILED = 'file-download-failed',
  LISTENING_ON = 'listening-on',
  CONNECTED = 'connected',
  DISCONNECTED = 'disconnected',
  ERROR = 'error',
  PENDING_MESSAGES_AVAILABLE = 'pending-messages-available',
}

export type P2pEvent =
  | {
      type: 'peer-discovered';
      peerId: string;
      nickname: string;
      address: string;
    }
  | { type: 'peer-expired'; peerId: string; nickname: string }
  | { type: 'nickname-updated'; peerId: string; nickname: string }
  | {
      type: 'direct-message';
      from: string;
      fromNickname: string;
      message: string;
    }
  | {
      type: 'direct-file-share-message';
      from: string;
      fromNickname: string;
      shareCode: string;
      filename: string;
      fileSize: number;
      fileType: string;
    }
  | {
      type: 'direct-group-share-message';
      from: string;
      fromNickname: string;
      groupId: string;
      groupName: string;
    }
  | {
      type: 'group-message';
      from: string;
      fromNickname: string;
      group: string;
      content: MessageContent;
      timestamp: number;
    }
  | {
      type: 'group-file-share-message';
      from: string;
      fromNickname: string;
      group: string;
      shareCode: string;
      filename: string;
      fileSize: number;
      fileType: string;
      message: string;
    }
  | { type: 'group-joined'; group: string }
  | { type: 'group-left'; group: string }
  | {
      type: 'file-share-request';
      from: string;
      fromNickname: string;
      shareCode: string;
      filename: string;
      size: number;
    }
  | { type: 'file-shared'; fileId: string; info: any }
  | { type: 'file-revoked'; fileId: string }
  | { type: 'file-info-received'; from: string; info: any }
  | {
      type: 'chunk-received';
      from: string;
      fileId: string;
      chunkIndex: number;
      data: Uint8Array;
      hash: string;
    }
  | { type: 'file-list-received'; from: string; files: any[] }
  | {
      type: 'file-download-started';
      from: string;
      fromNickname: string;
      filename: string;
      downloadId: string;
      shareCode: string;
    }
  | {
      type: 'file-download-progress';
      downloadId: string;
      filename: string;
      shareCode: string;
      fromPeerId: string;
      fromNickname: string;
      downloadedChunks: number;
      totalChunks: number;
    }
  | {
      type: 'file-download-completed';
      downloadId: string;
      filename: string;
      shareCode: string;
      fromPeerId: string;
      fromNickname: string;
      path: string;
    }
  | {
      type: 'file-download-failed';
      downloadId: string;
      filename: string;
      shareCode: string;
      fromPeerId: string;
      fromNickname: string;
      error: string;
    }
  | { type: 'listening-on'; address: string }
  | { type: 'connected'; peerId: string; nickname: string }
  | { type: 'disconnected'; peerId: string; nickname: string }
  | { type: 'error'; error: string }
  | { type: 'pending-messages-available'; peer: string; nickname: string };

type Listener = (event: P2pEvent) => void | Promise<void>;

// Create a singleton event emitter
class EventEmitter {
  private listeners: Map<string, Set<Listener>> = new Map();
  private id: string = Math.random().toString(36).substr(2, 9);

  on(eventType: string, listener: Listener): () => void {
    if (!this.listeners.has(eventType)) {
      this.listeners.set(eventType, new Set());
    }
    this.listeners.get(eventType)!.add(listener);
    return () => this.listeners.get(eventType)?.delete(listener);
  }

  off(eventType: string, listener: Listener): void {
    this.listeners.get(eventType)?.delete(listener);
  }

  async emit(event: P2pEvent): Promise<void> {
    const eventType = event.type;
    console.log(
      `EventEmitter ${this.id} - Event emitted:`,
      eventType,
      'Listeners count:',
      this.listenerCount(eventType)
    );
    const listenersForEvent = this.listeners.get(eventType);
    if (listenersForEvent) {
      console.log(
        `EventEmitter ${this.id} - Listeners for event:`,
        listenersForEvent.size
      );
      await Promise.all(
        Array.from(listenersForEvent).map((listener) =>
          Promise.resolve(listener(event)).catch((err) =>
            console.error(
              `EventEmitter ${this.id} - Error in event listener for ${eventType}:`,
              err
            )
          )
        )
      );
    }
    // Also notify 'any' listeners
    const anyListeners = this.listeners.get('any');
    if (anyListeners) {
      await Promise.all(
        Array.from(anyListeners).map((listener) =>
          Promise.resolve(listener(event)).catch((err) =>
            console.error(
              `EventEmitter ${this.id} - Error in 'any' event listener:`,
              err
            )
          )
        )
      );
    }
  }

  getId(): string {
    return this.id;
  }

  listenerCount(eventType: string): number {
    return this.listeners.get(eventType)?.size ?? 0;
  }

  removeAllListeners(eventType?: string): void {
    if (eventType) {
      this.listeners.delete(eventType);
    } else {
      this.listeners.clear();
    }
  }
}

// Export the singleton instance
export const eventEmitter = new EventEmitter();
