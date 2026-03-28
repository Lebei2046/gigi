import { createLibp2pInstance } from './libp2p-setup.js';
import { eventEmitter, P2pEvent } from './events.js';
import { P2pError, ErrorCode } from './errors.js';
import { FileSharingManager } from './file-sharing.js';
import { GroupManager } from './group.js';
import { PeerManager } from './peer-manager.js';
import { derivePeerId, derivePeerPrivateKey } from './key-derivation.js';
import { RequestResponse, JsonCodec } from '@gigi/request-response-ts';
import { multiaddr as multiaddrFromString } from '@multiformats/multiaddr';
import { peerIdFromString } from '@libp2p/peer-id';
import type { P2pConfig, PeerInfo, GroupInfo, ActiveDownload, MessageContent, MessageContentInput } from './types.js';

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
  shareCode: string;
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
  peerIdJson?: {
    id: string;
    privKey?: string;
    pubKey?: string;
    mnemonic?: string;
  };
}

export class P2pClient {
  private libp2p: any = null;
  private nickname: string;
  private outputDirectory: string;
  private config: P2pConfig;
  private peerIdJson: {
    id: string;
    privKey?: string;
    pubKey?: string;
    mnemonic?: string;
  } | undefined;
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
    this.peerIdJson = options.peerIdJson;

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
      // Handle mnemonic-based peer ID derivation
      let peerIdJson = this.peerIdJson;
      
      if (peerIdJson?.mnemonic) {
        // Derive peer ID from mnemonic
        const peerId = await derivePeerId(peerIdJson.mnemonic);
        // Derive private key from mnemonic
        const privateKey = derivePeerPrivateKey(peerIdJson.mnemonic);
        // Update peerIdJson with derived values
        peerIdJson = {
          ...peerIdJson,
          id: peerId,
          // Note: We don't store the private key in the config for security reasons
        };
      }

      this.libp2p = await createLibp2pInstance({
        nickname: this.nickname,
        listenAddrs: this.config.listenAddrs,
        bootstrapNodes: this.config.bootstrapNodes,
        enableMdns: this.config.enableMdns,
        enableKademlia: this.config.enableKademlia,
        enableRelay: this.config.enableRelay,
        peerIdJson: peerIdJson,
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
            await this.handleFileChunkRequest(event.peer.toString(), request, channel);
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
        // The event is a CustomEvent, data is in event.detail
        if (!event.detail) {
          console.warn('[P2pClient] Pubsub message event without detail:', event);
          return;
        }
        
        const detail = event.detail;
        const topic = detail.topic;
        
        if (!topic) {
          console.warn('[P2pClient] Pubsub message event without topic in detail:', event);
          return;
        }
        
        if (topic.startsWith('gigi-group:')) {
          // Check if detail.data exists (message content is in detail.data as a Buffer)
          if (!detail.data) {
            console.warn('[P2pClient] Pubsub message event without data:', detail);
            return;
          }
          
          const groupName = topic.replace('gigi-group:', '');
          // Decode the message from detail.data (Buffer)
          const messageData = new TextDecoder().decode(detail.data);
          
          try {
            // Parse the structured message
            const structuredMessage = JSON.parse(messageData);
            
            // Get the peer ID from the message
            // In libp2p pubsub, the message sender is in detail.from
            const from = detail.from || 'unknown';
            const senderNickname = structuredMessage.senderNickname || 'unknown';

            console.log(`[P2pClient] Received group message in ${groupName} from ${senderNickname}`);
            console.log(`[P2pClient] Message from peer ID: ${from}`);
            console.log(`[P2pClient] Message detail keys: ${Object.keys(detail)}`);
            
            // Check if we have any address information for the peer
            let addresses: string[] = [];
            if (detail.message && detail.message.multiaddrs) {
              addresses = detail.message.multiaddrs.map((ma: any) => ma.toString());
            } else if (detail.multiaddrs) {
              addresses = detail.multiaddrs.map((ma: any) => ma.toString());
            }
            console.log(`[P2pClient] Peer addresses: ${addresses.join(', ')}`);
            
            // Add the sender to peer manager if not already present
            let peerIdToAdd = from.toString();
            if (peerIdToAdd === 'unknown' && structuredMessage.content.type === 'fileShare' && structuredMessage.content.fromPeerId) {
              // For file share messages, use the peer ID from the content
              peerIdToAdd = structuredMessage.content.fromPeerId;
              console.log(`[P2pClient] Using peer ID from file share content: ${peerIdToAdd}`);
            }
            
            if (peerIdToAdd !== 'unknown' && senderNickname !== 'unknown') {
              console.log(`[P2pClient] Adding peer ${senderNickname} (${peerIdToAdd}) to peer manager with addresses: ${addresses.join(', ')}`);
              this.peerManager.discover(peerIdToAdd, senderNickname, addresses);
              console.log(`[P2pClient] Peers after adding: ${Array.from(this.peerManager.list()).map(p => `${p.nickname} (${p.peerId})`).join(', ')}`);
            }
            
            // Emit the group message event with structured content
            await eventEmitter.emit({
              type: 'group-message',
              from: peerIdToAdd,
              fromNickname: senderNickname,
              group: groupName,
              content: structuredMessage.content,
              timestamp: structuredMessage.timestamp,
            } as P2pEvent);
          } catch (error) {
            console.warn('[P2pClient] Error parsing structured message:', error);
            // Fallback to plain text message if parsing fails
            const from = detail.from || 'unknown';
            console.log(`[P2pClient] Received plain text message in ${groupName} from ${from}: ${messageData}`);
            
            const nickname = this.peerManager.getNickname(from.toString()) || from.toString();
            
            // Add the sender to peer manager if not already present
            if (from !== 'unknown') {
              this.peerManager.discover(from.toString(), nickname, []);
            }
            
            await eventEmitter.emit({
              type: 'group-message',
              from: from.toString(),
              fromNickname: nickname,
              group: groupName,
              content: { type: 'text', text: messageData },
              timestamp: Date.now(),
            } as P2pEvent);
          }
        }
      });
    }

    this.libp2p.addEventListener('peer:connect', async (event: any) => {
      if (!event.detail) {
        console.warn('[P2pClient] peer:connect event without detail:', event);
        return;
      }
      
      // Handle both formats: event.detail as PeerId or event.detail.remotePeer
      const peerId = event.detail.remotePeer ? event.detail.remotePeer.toString() : event.detail.toString();
      // Get the peer's nickname from the peer store or use peer ID as fallback
      const peerNickname = this.peerManager.getNickname(peerId) || peerId;
      this.peerManager.addConnected(peerId, peerNickname);

      await eventEmitter.emit({
        type: 'connected',
        peerId,
        nickname: peerNickname,
      } as P2pEvent);
    });

    this.libp2p.addEventListener('peer:disconnect', async (event: any) => {
      if (!event.detail) {
        console.warn('[P2pClient] peer:disconnect event without detail:', event);
        return;
      }
      
      // Handle both formats: event.detail as PeerId or event.detail.remotePeer
      const peerId = event.detail.remotePeer ? event.detail.remotePeer.toString() : event.detail.toString();
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

    // Listen for general peer discovery events (includes both DHT and mDNS)
    this.libp2p.addEventListener('peer:discovery', async (event: any) => {
      console.log('[P2pClient] Peer discovered event:', event);
      // Handle both formats: event.detail.id or event.detail as PeerId
      const peerId = event.detail.id ? event.detail.id.toString() : event.detail.toString();
      const multiaddrs = event.detail.multiaddrs ? event.detail.multiaddrs.map((m: any) => m.toString()) : [];

      console.log(`[P2pClient] Discovered peer: ${peerId} at ${multiaddrs.join(', ')}`);

      // Use peer ID as nickname for newly discovered peers
      this.peerManager.discover(peerId, peerId, multiaddrs);

      // Automatically connect to discovered peers
      try {
        if (multiaddrs.length > 0) {
          console.log(`[P2pClient] Attempting to connect to discovered peer: ${peerId}`);
          const addr = multiaddrFromString(multiaddrs[0]);
          await this.libp2p.dial(addr);
          console.log(`[P2pClient] Successfully connected to peer: ${peerId}`);
        }
      } catch (error) {
        console.warn(`[P2pClient] Failed to connect to discovered peer ${peerId}:`, error);
      }

      await eventEmitter.emit({
        type: 'peer-discovered',
        peerId,
        nickname: peerId,
        address: multiaddrs[0] || '',
      } as P2pEvent);
    });

    if (this.libp2p.services.dht) {
      this.libp2p.services.dht.addEventListener('peer', async (event: any) => {
        // Handle both formats: event.detail.id or event.detail as PeerId
        const peerId = event.detail.id ? event.detail.id.toString() : event.detail.toString();
        const multiaddrs = event.detail.multiaddrs ? event.detail.multiaddrs.map((m: any) => m.toString()) : [];

        // Use peer ID as nickname for newly discovered peers
        this.peerManager.discover(peerId, peerId, multiaddrs);

        await eventEmitter.emit({
          type: 'peer-discovered',
          peerId,
          nickname: peerId,
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
    console.log(`[P2pClient] ************* Handling file request from ${peerId}`);
    console.log(`[P2pClient] Request: ${JSON.stringify(request)}`);
    
    const file = this.fileManager.getByShareCode(request.shareCode);
    if (!file) {
      console.log(`[P2pClient] File not found for share code: ${request.shareCode}`);
      channel.send({ type: 'error', message: 'File not found' } as FileErrorResponse);
      return;
    }

    console.log(`[P2pClient] Found file: ${file.info.name}`);
    
    // Send file info response
    const response = {
      type: 'file-info',
      fileId: file.fileId,
      name: file.info.name,
      size: file.info.size,
      mimeType: file.info.mimeType,
      chunkCount: file.info.chunkCount,
      hash: file.info.hash,
    } as FileInfoResponse;
    
    console.log(`[P2pClient] Sending response: ${JSON.stringify(response)}`);
    channel.send(response);
    console.log(`[P2pClient] ************* Response sent`);

    await eventEmitter.emit({
      type: 'file-share-request',
      from: peerId,
      fromNickname: this.peerManager.getNickname(peerId) || peerId,
      shareCode: request.shareCode,
      filename: file.info.name,
      size: file.info.size,
    } as P2pEvent);
  }

  private async handleFileChunkRequest(peerId: string, request: FileChunkRequest, channel: any): Promise<void> {
    console.log(`[P2pClient] Handling file chunk request from ${peerId}`);
    console.log(`[P2pClient] Request: ${JSON.stringify(request)}`);
    
    const file = this.fileManager.getByShareCode(request.shareCode);
    if (!file) {
      console.log(`[P2pClient] File not found for share code: ${request.shareCode}`);
      channel.send({ type: 'error', message: 'File not found' } as FileErrorResponse);
      return;
    }

    try {
      // Use the stored file path
      const filePath = (file as any).filePath || file.info.name;
      const chunk = await this.fileManager.getChunk(file.fileId, request.chunkIndex, filePath);
      
      // Send chunk response
      const response = {
        type: 'chunk',
        downloadId: request.downloadId,
        chunkIndex: request.chunkIndex,
        totalChunks: request.totalChunks,
        chunk: chunk,
      } as FileChunkResponse;
      
      console.log(`[P2pClient] Sending chunk ${request.chunkIndex}/${request.totalChunks}`);
      channel.send(response);
    } catch (error) {
      console.error(`[P2pClient] Error sending chunk:`, error);
      channel.send({ type: 'error', message: 'Failed to send chunk' } as FileErrorResponse);
    }
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

    // Send the request and return the response directly
    return await this.fileRequestResponse!.sendRequest(targetPeerId, message);
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

  addPeer(nickname: string, peerId: string, addresses: string[]): void {
    this.peerManager.discover(peerId, nickname, addresses);
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

  async sendGroupMessage(groupName: string, content: MessageContentInput): Promise<void> {
    if (!this.libp2p || !this.started) {
      throw P2pError.notStarted();
    }

    const topic = `gigi-group:${groupName}`;
    
    // If content is a file share, add sender's peer ID and nickname
    let fullContent: MessageContent;
    if (content.type === 'fileShare') {
      fullContent = {
        ...content,
        fromPeerId: this.getPeerId(),
        fromNickname: this.nickname
      };
    } else {
      fullContent = content as MessageContent;
    }
    
    const message = JSON.stringify({
      type: 'group-message',
      senderNickname: this.nickname,
      content: fullContent,
      timestamp: Date.now()
    });
    const data = new TextEncoder().encode(message);

    try {
      await this.libp2p.services.pubsub?.publish(topic, data);
    } catch (error) {
      // Handle the case when no peers are subscribed to the topic
      if (error instanceof Error && error.message.includes('NoPeersSubscribedToTopic')) {
        // Silently ignore this error since it's expected when no one else is in the group
        console.log(`[P2pClient] No peers subscribed to topic ${topic}`);
      } else {
        // Re-throw other errors
        throw error;
      }
    }
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
    console.log(`[P2pClient] Attempting to download file from ${nickname}`);
    console.log(`[P2pClient] Peers in manager: ${Array.from(this.peerManager.list()).map(p => `${p.nickname} (${p.peerId})`).join(', ')}`);
    
    const peerId = this.peerManager.getPeerId(nickname);
    if (!peerId) {
      console.log(`[P2pClient] Peer ${nickname} not found in peer manager`);
      throw P2pError.peerNotFound(nickname);
    }
    console.log(`[P2pClient] Found peer ${nickname} with ID: ${peerId}`);
    
    return this.downloadFileByPeerId(peerId, nickname, shareCode);
  }
  
  async downloadFileByPeerId(peerId: string, nickname: string, shareCode: string): Promise<string> {
    console.log(`[P2pClient] ************* Entering downloadFileByPeerId`);
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
      console.log(`[P2pClient] Received response type: ${response.type}`);

      if (response.type === 'error') {
        throw P2pError.fileNotFound(shareCode);
      } else if (response.type === 'file-info') {
        console.log(`[P2pClient] Starting chunk requests`);
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

        // Request all file chunks
        console.log(`[P2pClient] Requesting ${response.chunkCount} chunks`);
        for (let i = 0; i < response.chunkCount; i++) {
          console.log(`[P2pClient] Requesting chunk ${i}`);
          const chunkRequest: FileChunkRequest = {
            type: 'chunk',
            downloadId,
            shareCode,
            chunkIndex: i,
            totalChunks: response.chunkCount,
            chunk: new Uint8Array(0), // Empty chunk, just requesting data
          };

          const chunkResponse = await this.sendFileMessage(peerId, chunkRequest);
          console.log(`[P2pClient] Received chunk ${i} response: ${chunkResponse.type}`);
          if (chunkResponse.type === 'chunk' && chunkResponse.chunk) {
            download.data.push(chunkResponse.chunk);
            download.downloadedChunks++;
            console.log(`[P2pClient] Chunk ${i} added, downloaded: ${download.downloadedChunks}/${download.totalChunks}`);

            await eventEmitter.emit({
              type: 'file-download-progress',
              downloadId,
              filename: download.filename,
              shareCode,
              fromPeerId: peerId,
              fromNickname: nickname,
              downloadedChunks: download.downloadedChunks,
              totalChunks: download.totalChunks,
            } as P2pEvent);
          }
        }
        console.log(`[P2pClient] Chunk requests completed`);

        // Complete the download
        if (download.downloadedChunks >= download.totalChunks) {
          await this.fileManager.saveFile(download.filename, download.data);
          download.completed = true;
          download.finalPath = `${this.outputDirectory}/${download.filename}`;

          await eventEmitter.emit({
            type: 'file-download-completed',
            downloadId,
            filename: download.filename,
            shareCode,
            fromPeerId: peerId,
            fromNickname: nickname,
            path: download.finalPath,
          } as P2pEvent);
        }

        return downloadId;
      } else {
        throw P2pError.fileNotFound(shareCode);
      }
    } catch (error) {
      console.error(`[P2pClient] Error in downloadFileByPeerId:`, error);
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

  getFileByShareCode(shareCode: string): any | undefined {
    return this.fileManager.getByShareCode(shareCode);
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

  async connectToPeer(multiaddr: string): Promise<void> {
    if (!this.libp2p || !this.started) {
      throw P2pError.notStarted();
    }

    try {
      const addr = multiaddrFromString(multiaddr);
      await this.libp2p.dial(addr);
      console.log(`[P2pClient] Connected to peer at ${multiaddr}`);
    } catch (error) {
      throw P2pError.networkError(`Failed to connect to peer at ${multiaddr}`, error as Error);
    }
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