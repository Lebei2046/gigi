import { createLibp2pInstance } from './libp2p-setup.js';
import { eventEmitter, P2pEvent } from './events.js';
import { P2pError, ErrorCode } from './errors.js';
import { FileSharingManager } from './file-sharing.js';
import { GroupManager } from './group.js';
import { PeerManager } from './peer-manager.js';
import { RequestResponse, JsonCodec } from '@gigi/request-response-ts';
import type { P2pConfig, PeerInfo, GroupInfo, ActiveDownload } from './types.js';

// Define file protocol request and response types
export interface FileRequest {
  type: 'request';
  action: 'request';
  shareCode: string;
  downloadId: string;
}

export interface FileChunkRequest {
  type: 'chunk';
  downloadId: string;
  chunkIndex: number;
  totalChunks: number;
  chunk: Uint8Array;
}

export interface FileErrorResponse {
  type: 'error';
  message: string;
}

export interface FileInfoResponse {
  type: 'file-info';
  fileId: string;
  name: string;
  size: number;
  mimeType: string;
  chunkCount: number;
  hash: string;
}

export interface FileChunkResponse {
  type: 'chunk';
  downloadId: string;
  chunkIndex: number;
  totalChunks: number;
  chunk: Uint8Array;
}

export type FileRequestMessage = FileRequest | FileChunkRequest;
export type FileResponseMessage = FileErrorResponse | FileInfoResponse | FileChunkResponse;

const DEFAULT_OUTPUT_DIR = './downloads';

export interface P2pClientOptions {
  nickname: string;
  outputDirectory?: string;
  config?: Partial<P2pConfig>;
}

export class P2pClient {
  private libp2p: any = null;
  private nickname: string;
  private outputDirectory: string;
  private config: P2pConfig;
  private started = false;

  private peerManager: PeerManager;
  private groupManager: GroupManager;
  private fileManager: FileSharingManager;
  private downloadManager: DownloadManager;
  private fileRequestResponse: RequestResponse<FileRequestMessage, FileResponseMessage, string> | null = null;

  private readonly DIRECT_PROTOCOL = '/gigi/direct/1.0.0';
  private readonly FILE_PROTOCOL = '/gigi/file/1.0.0';

  constructor(options: P2pClientOptions) {
    this.nickname = options.nickname;
    this.outputDirectory = options.outputDirectory || DEFAULT_OUTPUT_DIR;
    this.config = {
      bootstrapNodes: [],
      enableKademlia: true,
      enableRelay: true,
      enableMdns: true,
      listenAddrs: ['/ip4/0.0.0.0/tcp/0'],
      ...options.config,
    };

    this.peerManager = new PeerManager();
    this.groupManager = new GroupManager();
    this.fileManager = new FileSharingManager(this.outputDirectory);
    this.downloadManager = new DownloadManager(this.outputDirectory);
  }

  async start(): Promise<void> {
    if (this.started) {
      throw P2pError.alreadyStarted();
    }

    try {
      this.libp2p = await createLibp2pInstance({
        nickname: this.nickname,
        listenAddrs: this.config.listenAddrs,
        bootstrapNodes: this.config.bootstrapNodes,
        enableMdns: this.config.enableMdns,
        enableKademlia: this.config.enableKademlia,
        enableRelay: this.config.enableRelay,
      });

      // Initialize request-response protocol for file sharing
      this.fileRequestResponse = new RequestResponse<FileRequestMessage, FileResponseMessage, string>(
        this.libp2p,
        new JsonCodec<FileRequestMessage, FileResponseMessage, string>(this.FILE_PROTOCOL)
      );

      // Set up request-response event listener
      this.fileRequestResponse.onEvent(async (event: any) => {
        if (event.type === 'Message' && event.message.type === 'Request') {
          const { request, channel } = event.message;
          if (request.type === 'request') {
            await this.handleFileRequest(event.peer.toString(), request, channel);
          } else if (request.type === 'chunk') {
            await this.handleFileChunk(event.peer.toString(), request, channel);
          }
        }
      });

      await this.setupProtocolHandlers();
      await this.libp2p.start();

      this.started = true;
      console.log(`[P2pClient] Started with peer ID: ${this.getPeerId()}`);
      console.log(`[P2pClient] Listening on: ${this.getMultiaddrs().join(', ')}`);

      for (const addr of this.getMultiaddrs()) {
        await eventEmitter.emit({ type: 'listening-on', address: addr } as P2pEvent);
      }

      this.processSwarmEvents();
    } catch (error) {
      throw P2pError.networkError('Failed to start P2P client', error as Error);
    }
  }

  async stop(): Promise<void> {
    if (!this.started || !this.libp2p) {
      return;
    }

    // Close request-response protocol
    if (this.fileRequestResponse) {
      this.fileRequestResponse.close();
      this.fileRequestResponse = null;
    }

    await this.libp2p.stop();
    this.started = false;
    this.libp2p = null as any;
    console.log('[P2pClient] Stopped');
  }

  private async setupProtocolHandlers(): Promise<void> {
    if (!this.libp2p) return;

    await this.libp2p.handle(this.DIRECT_PROTOCOL, async ({ stream, connection }: any) => {
      try {
        const fromPeerId = connection?.remotePeer?.toString() || 'unknown';
        const message = await this.readStreamMessage(stream);

        await eventEmitter.emit({
          type: 'direct-message',
          from: fromPeerId,
          fromNickname: this.peerManager.getNickname(fromPeerId) || fromPeerId,
          message,
        } as P2pEvent);
      } catch (error) {
        console.error('[P2pClient] Error handling direct message:', error);
      }
    });

    // File protocol is now handled by request-response protocol
    // The old stream-based handler is no longer needed

    if (this.libp2p.services.pubsub) {
      this.libp2p.services.pubsub.addEventListener('message', async (event: any) => {
        const topic = event.topic;
        if (topic.startsWith('gigi-group:')) {
          const groupName = topic.replace('gigi-group:', '');
          const message = new TextDecoder().decode(event.detail.message.data);

          await eventEmitter.emit({
            type: 'group-message',
            from: event.detail.from.toString(),
            fromNickname: this.peerManager.getNickname(event.detail.from.toString()) || event.detail.from.toString(),
            group: groupName,
            message,
          } as P2pEvent);
        }
      });
    }

    this.libp2p.addEventListener('peer:connect', async (event: any) => {
      const peerId = event.detail.remotePeer.toString();
      this.peerManager.addConnected(peerId, this.nickname);

      await eventEmitter.emit({
        type: 'connected',
        peerId,
        nickname: this.peerManager.getNickname(peerId) || peerId,
      } as P2pEvent);
    });

    this.libp2p.addEventListener('peer:disconnect', async (event: any) => {
      const peerId = event.detail.remotePeer.toString();
      this.peerManager.removeConnected(peerId);

      await eventEmitter.emit({
        type: 'disconnected',
        peerId,
        nickname: this.peerManager.getNickname(peerId) || peerId,
      } as P2pEvent);
    });
  }

  private processSwarmEvents(): void {
    if (!this.libp2p) return;

    if (this.libp2p.services.dht) {
      this.libp2p.services.dht.addEventListener('peer', async (event: any) => {
        const peerId = event.detail.id.toString();
        const multiaddrs = event.detail.multiaddrs.map((m: any) => m.toString());

        this.peerManager.discover(peerId, this.nickname, multiaddrs);

        await eventEmitter.emit({
          type: 'peer-discovered',
          peerId,
          nickname: this.nickname,
          address: multiaddrs[0] || '',
        } as P2pEvent);
      });
    }
  }

  private async readStreamMessage(stream: any): Promise<string> {
    const chunks: Uint8Array[] = [];
    for await (const chunk of stream.source) {
      chunks.push(chunk);
    }
    const allBytes = new Uint8Array(chunks.reduce((sum, c) => sum + c.length, 0));
    let offset = 0;
    for (const chunk of chunks) {
      allBytes.set(chunk, offset);
      offset += chunk.length;
    }
    return new TextDecoder().decode(allBytes);
  }

  private async handleFileRequest(peerId: string, request: FileRequest, channel: any): Promise<void> {
    const file = this.fileManager.getByShareCode(request.shareCode);
    if (!file) {
      channel.send({ type: 'error', message: 'File not found' } as FileErrorResponse);
      return;
    }

    // Send file info response
    channel.send({
      type: 'file-info',
      fileId: file.fileId,
      name: file.info.name,
      size: file.info.size,
      mimeType: file.info.mimeType,
      chunkCount: file.info.chunkCount,
      hash: file.info.hash,
    } as FileInfoResponse);

    await eventEmitter.emit({
      type: 'file-share-request',
      from: peerId,
      fromNickname: this.peerManager.getNickname(peerId) || peerId,
      shareCode: request.shareCode,
      filename: file.info.name,
      size: file.info.size,
    } as P2pEvent);
  }

  private async handleFileChunk(peerId: string, request: FileChunkRequest, channel: any): Promise<void> {
    const download = this.downloadManager.get(request.downloadId);
    if (!download) {
      channel.send({ type: 'error', message: 'Download not found' } as FileErrorResponse);
      return;
    }

    download.downloadedChunks++;
    download.data.push(request.chunk);

    await eventEmitter.emit({
      type: 'file-download-progress',
      downloadId: request.downloadId,
      filename: download.filename,
      shareCode: download.shareCode,
      fromPeerId: peerId,
      fromNickname: this.peerManager.getNickname(peerId) || peerId,
      downloadedChunks: download.downloadedChunks,
      totalChunks: download.totalChunks,
    } as P2pEvent);

    if (download.downloadedChunks >= download.totalChunks) {
      await this.fileManager.saveFile(download.filename, download.data);
      download.completed = true;
      download.finalPath = `${this.outputDirectory}/${download.filename}`;

      // Send completion response
      channel.send({ type: 'chunk', downloadId: request.downloadId, chunkIndex: request.chunkIndex, totalChunks: request.totalChunks, chunk: new Uint8Array(0) } as FileChunkResponse);

      await eventEmitter.emit({
        type: 'file-download-completed',
        downloadId: request.downloadId,
        filename: download.filename,
        shareCode: download.shareCode,
        fromPeerId: peerId,
        fromNickname: this.peerManager.getNickname(peerId) || peerId,
        path: download.finalPath,
      } as P2pEvent);
    } else {
      // Send acknowledgment response
      channel.send({ type: 'chunk', downloadId: request.downloadId, chunkIndex: request.chunkIndex, totalChunks: request.totalChunks, chunk: new Uint8Array(0) } as FileChunkResponse);
    }
  }

  private async sendFileMessage(targetPeerId: string, message: FileRequest | FileChunkRequest): Promise<FileResponseMessage> {
    if (!this.libp2p || !this.started || !this.fileRequestResponse) {
      throw P2pError.notStarted();
    }

    try {
      // Create a promise to wait for the response
      return new Promise<FileResponseMessage>((resolve, reject) => {
        // Send the request
        this.fileRequestResponse!.sendRequest(targetPeerId, message).catch(reject);

        // Listen for the response
        const unsubscribe = this.fileRequestResponse!.onEvent((event: any) => {
          if (event.type === 'Message' && event.message.type === 'Response') {
            resolve(event.message.response);
            unsubscribe();
          } else if (event.type === 'OutboundFailure') {
            reject(new Error(`Outbound failure: ${event.error}`));
            unsubscribe();
          }
        });
      });
    } catch (error) {
      throw P2pError.networkError(`Failed to send file message to ${targetPeerId}`, error as Error);
    }
  }

  getPeerId(): string {
    if (!this.libp2p || !this.started) {
      throw P2pError.notStarted();
    }
    return this.libp2p.peerId.toString();
  }

  getMultiaddrs(): string[] {
    if (!this.libp2p || !this.started) {
      throw P2pError.notStarted();
    }
    return this.libp2p.getMultiaddrs().map((m: any) => m.toString());
  }

  isStarted(): boolean {
    return this.started;
  }

  async sendDirectMessage(targetPeerId: string, message: string): Promise<void> {
    if (!this.libp2p || !this.started) {
      throw P2pError.notStarted();
    }

    try {
      const stream = await this.libp2p.dialProtocol(targetPeerId, this.DIRECT_PROTOCOL);
      const data = new TextEncoder().encode(message);
      await stream.sink([data]);
    } catch (error) {
      throw P2pError.networkError(`Failed to send message to ${targetPeerId}`, error as Error);
    }
  }

  async sendDirectMessageToNickname(nickname: string, message: string): Promise<void> {
    const peerId = this.peerManager.getPeerId(nickname);
    if (!peerId) {
      throw P2pError.peerNotFound(nickname);
    }
    await this.sendDirectMessage(peerId, message);
  }

  async joinGroup(groupName: string): Promise<void> {
    if (!this.libp2p || !this.started) {
      throw P2pError.notStarted();
    }

    const topic = `gigi-group:${groupName}`;
    this.libp2p.services.pubsub?.subscribe(topic);
    this.groupManager.join(groupName, topic);

    await eventEmitter.emit({ type: 'group-joined', group: groupName } as P2pEvent);
  }

  async leaveGroup(groupName: string): Promise<void> {
    if (!this.libp2p || !this.started) {
      throw P2pError.notStarted();
    }

    const topic = `gigi-group:${groupName}`;
    this.libp2p.services.pubsub?.unsubscribe(topic);
    this.groupManager.leave(groupName);

    await eventEmitter.emit({ type: 'group-left', group: groupName } as P2pEvent);
  }

  async sendGroupMessage(groupName: string, message: string): Promise<void> {
    if (!this.libp2p || !this.started) {
      throw P2pError.notStarted();
    }

    const topic = `gigi-group:${groupName}`;
    const data = new TextEncoder().encode(message);

    await this.libp2p.services.pubsub?.publish(topic, data);
  }

  getJoinedGroups(): GroupInfo[] {
    return this.groupManager.list();
  }

  async shareFile(filePath: string): Promise<string> {
    const sharedFile = await this.fileManager.share(filePath);

    await eventEmitter.emit({
      type: 'file-shared',
      fileId: sharedFile.fileId,
      info: sharedFile.info,
    } as P2pEvent);

    return sharedFile.shareCode;
  }

  async downloadFile(nickname: string, shareCode: string): Promise<string> {
    const peerId = this.peerManager.getPeerId(nickname);
    if (!peerId) {
      throw P2pError.peerNotFound(nickname);
    }

    const downloadId = crypto.randomUUID();

    // Send file request to get file info
    const fileRequest: FileRequest = {
      type: 'request',
      action: 'request',
      shareCode,
      downloadId,
    };

    try {
      const response = await this.sendFileMessage(peerId, fileRequest);

      if (response.type === 'error') {
        throw P2pError.fileNotFound(shareCode);
      } else if (response.type === 'file-info') {
        // Create download entry
        const download: ActiveDownload = {
          downloadId,
          filename: response.name,
          shareCode,
          fromPeerId: peerId,
          fromNickname: nickname,
          totalChunks: response.chunkCount,
          downloadedChunks: 0,
          startedAt: Date.now(),
          completed: false,
          failed: false,
          data: [],
        };

        this.downloadManager.add(download);

        await eventEmitter.emit({
          type: 'file-download-started',
          from: peerId,
          fromNickname: nickname,
          filename: download.filename,
          downloadId,
          shareCode,
        } as P2pEvent);

        // For demonstration purposes, we'll just return the download ID
        // In a real implementation, you would request each chunk here
        return downloadId;
      } else {
        throw P2pError.fileNotFound(shareCode);
      }
    } catch (error) {
      throw P2pError.networkError(`Failed to download file: ${error instanceof Error ? error.message : 'Unknown error'}`, error as Error);
    }
  }

  async revokeFile(shareCode: string): Promise<void> {
    const file = this.fileManager.getByShareCode(shareCode);
    if (!file) {
      throw P2pError.fileNotFound(shareCode);
    }

    this.fileManager.revoke(shareCode);

    await eventEmitter.emit({
      type: 'file-revoked',
      fileId: file.fileId,
    } as P2pEvent);
  }

  listSharedFiles(): any[] {
    return this.fileManager.list();
  }

  getActiveDownloads(): ActiveDownload[] {
    return this.downloadManager.list();
  }

  async cancelDownload(downloadId: string): Promise<void> {
    this.downloadManager.remove(downloadId);
  }

  getPeerByNickname(nickname: string): PeerInfo | undefined {
    return this.peerManager.getByNickname(nickname);
  }

  getPeerById(peerId: string): PeerInfo | undefined {
    return this.peerManager.getByPeerId(peerId);
  }

  listPeers(): PeerInfo[] {
    return this.peerManager.list();
  }

  onEvent(listener: (event: P2pEvent) => void | Promise<void>): () => void {
    return eventEmitter.on('any', listener as any);
  }

  offEvent(listener: (event: P2pEvent) => void | Promise<void>): void {
    eventEmitter.off('any', listener as any);
  }

  async waitForEvent<T extends P2pEvent>(eventType: string, timeout: number = 30000): Promise<T> {
    return new Promise((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        eventEmitter.off(eventType, listener as any);
        reject(P2pError.timeout(`Wait for ${eventType}`));
      }, timeout);

      const listener = async (event: P2pEvent) => {
        clearTimeout(timeoutId);
        resolve(event as T);
      };

      eventEmitter.on(eventType, listener as any);
    });
  }
}

class DownloadManager {
  private downloads: Map<string, ActiveDownload> = new Map();
  private outputDirectory: string;

  constructor(outputDirectory: string) {
    this.outputDirectory = outputDirectory;
  }

  add(download: ActiveDownload): void {
    this.downloads.set(download.downloadId, download);
  }

  get(downloadId: string): ActiveDownload | undefined {
    return this.downloads.get(downloadId);
  }

  remove(downloadId: string): void {
    this.downloads.delete(downloadId);
  }

  list(): ActiveDownload[] {
    return Array.from(this.downloads.values());
  }
}