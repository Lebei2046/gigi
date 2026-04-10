import { createLibp2pInstance, Libp2pInstance } from './libp2p-setup';
import { eventEmitter, P2pEvent } from './events';
import { P2pError } from './errors';
import { FileSharingManager } from './file-sharing';
import { GroupManager } from './group';
import { PeerManager } from './peer-manager';
import { multiaddr as multiaddrFromString } from '@multiformats/multiaddr';
import { peerIdFromString } from '@libp2p/peer-id';
import { randomUUID } from 'crypto';
import { RequestResponse } from '@gigi/request-response';
import { JsonCodec } from '@gigi/request-response';
import { createLogger } from '@gigi/logging';

const logger = createLogger({ name: 'gigi-p2p' });
import type {
  P2pConfig,
  PeerInfo,
  GroupInfo,
  ActiveDownload,
  MessageContent,
  MessageContentInput,
} from './types';

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
export type FileResponseMessage =
  | FileErrorResponse
  | FileInfoResponse
  | FileChunkResponse;

const DEFAULT_OUTPUT_DIR = './downloads';

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

export interface P2pClientOptions {
  nickname: string;
  outputDirectory?: string;
  config?: Partial<P2pConfig>;
  mnemonic?: string;
}

export class P2pClient {
  private libp2p: any = null;
  private gigiDns: any = null;
  private nickname: string;
  private outputDirectory: string;
  private config: P2pConfig;
  private mnemonic: string | undefined;
  private started = false;

  private peerManager: PeerManager;
  private groupManager: GroupManager;
  private fileManager: FileSharingManager;
  private downloadManager: DownloadManager;
  private fileRequestResponse: RequestResponse<
    FileRequestMessage,
    FileResponseMessage,
    string
  > | null = null;

  private readonly DIRECT_PROTOCOL = '/gigi/direct/1.0.0';
  private readonly FILE_PROTOCOL = '/gigi/file/1.0.0';

  private cleanupInterval: NodeJS.Timeout | null = null;
  private eventListeners: Array<{
    event: string;
    listener: (event: any) => void;
  }> = [];

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
    this.mnemonic = options.mnemonic;

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
      let libp2pInstance: Libp2pInstance;
      if (this.mnemonic) {
        // For mnemonic-based configuration, use the existing peer ID from the config
        // This will ensure consistency with the configuration
        logger.info('[P2pClient] Using mnemonic for key derivation');
        libp2pInstance = await createLibp2pInstance({
          nickname: this.nickname,
          listenAddrs: this.config.listenAddrs,
          bootstrapNodes: this.config.bootstrapNodes,
          enableMdns: this.config.enableMdns,
          enableKademlia: this.config.enableKademlia,
          enableRelay: this.config.enableRelay,
          mnemonic: this.mnemonic,
        });
      } else {
        // For non-mnemonic configuration, create libp2p instance without mnemonic
        logger.info('[P2pClient] Creating libp2p instance without mnemonic');
        libp2pInstance = await createLibp2pInstance({
          nickname: this.nickname,
          listenAddrs: this.config.listenAddrs,
          bootstrapNodes: this.config.bootstrapNodes,
          enableMdns: this.config.enableMdns,
          enableKademlia: this.config.enableKademlia,
          enableRelay: this.config.enableRelay,
        });
      }
      this.libp2p = libp2pInstance.libp2p;
      this.gigiDns = libp2pInstance.gigiDns;

      // Add error event listener to prevent uncaught exceptions
      this.addEventListener('error', (error: any) => {
        console.warn('[P2pClient] Libp2p error:', error);
      });

      // Set up Gigi DNS event listeners for peer discovery with nicknames
      if (this.gigiDns) {
        logger.info('[P2pClient] Setting up Gigi DNS event listeners');

        this.gigiDns.on('Discovered', (event: any) => {
          const peerInfo = event.peerInfo;
          logger.info({
            message: '[P2pClient] Gigi DNS discovered peer',
            nickname: peerInfo.nickname,
            peerId: peerInfo.peerId.toString(),
          });

          // Add peer to peer manager with nickname
          this.peerManager.discover(
            peerInfo.peerId.toString(),
            peerInfo.nickname,
            [peerInfo.multiaddr.toString()]
          );

          // Emit peer-discovered event
          eventEmitter.emit({
            type: 'peer-discovered',
            peerId: peerInfo.peerId.toString(),
            nickname: peerInfo.nickname,
            address: peerInfo.multiaddr.toString(),
          } as P2pEvent);

          // Automatically connect to discovered peer with retry
          const connectWithRetry = async (attempts = 3, delay = 1000) => {
            for (let i = 0; i < attempts; i++) {
              try {
                // Try to connect using the multiaddr
                await this.libp2p.dial(peerInfo.multiaddr);
                logger.info({
                  message: '[P2pClient] Successfully connected to peer',
                  nickname: peerInfo.nickname,
                });
                return;
              } catch (error) {
                logger.warn({
                  message: `[P2pClient] Attempt ${i + 1} failed to dial discovered peer`,
                  error: error,
                });
                if (i < attempts - 1) {
                  await new Promise((resolve) => setTimeout(resolve, delay));
                }
              }
            }
          };

          // Only try to connect if the multiaddr is valid
          if (peerInfo.multiaddr) {
            connectWithRetry().catch(console.error);
          }
        });

        this.gigiDns.on('Offline', (event: any) => {
          const peerInfo = event.peerInfo;
          logger.info({
            message: '[P2pClient] Gigi DNS peer went offline',
            nickname: peerInfo.nickname,
            peerId: peerInfo.peerId.toString(),
          });

          eventEmitter.emit({
            type: 'peer-expired',
            peerId: peerInfo.peerId.toString(),
            nickname: peerInfo.nickname,
          } as P2pEvent);
        });
      }

      // Initialize request-response protocol for file sharing
      this.fileRequestResponse = new RequestResponse<
        FileRequestMessage,
        FileResponseMessage,
        string
      >(
        this.libp2p,
        new JsonCodec<FileRequestMessage, FileResponseMessage, string>(
          this.FILE_PROTOCOL
        )
      );

      // Set up request-response event listener
      this.fileRequestResponse.onEvent(async (event: any) => {
        if (event.type === 'Message' && event.message.type === 'Request') {
          const { request, channel } = event.message;
          if (request.type === 'request') {
            await this.handleFileRequest(
              event.peer.toString(),
              request,
              channel
            );
          } else if (request.type === 'chunk') {
            await this.handleFileChunkRequest(
              event.peer.toString(),
              request,
              channel
            );
          }
        }
      });

      await this.libp2p.start();
      await this.setupProtocolHandlers();

      this.started = true;
      logger.info({
        message: '[P2pClient] Started',
        peerId: this.getPeerId(),
        listenAddrs: this.getMultiaddrs(),
      });

      for (const addr of this.getMultiaddrs()) {
        await eventEmitter.emit({
          type: 'listening-on',
          address: addr,
        } as P2pEvent);
      }

      this.processSwarmEvents();

      // Start peer cleanup interval
      this.cleanupInterval = setInterval(() => {
        this.peerManager.cleanup();
      }, 60000); // Clean up every minute
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

    // Clear cleanup interval
    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval);
      this.cleanupInterval = null;
    }

    // Remove all event listeners
    for (const { event, listener } of this.eventListeners) {
      if (event === 'dht:peer' && this.libp2p.services.dht) {
        this.libp2p.services.dht.removeEventListener('peer', listener);
      } else {
        this.libp2p.removeEventListener(event, listener);
      }
    }
    this.eventListeners = [];

    // Stop Gigi DNS if initialized
    if (this.gigiDns) {
      this.gigiDns.stop();
      this.gigiDns = null;
    }

    await this.libp2p.stop();
    this.started = false;
    this.libp2p = null as any;
    logger.info('[P2pClient] Stopped');
  }

  private async setupProtocolHandlers(): Promise<void> {
    if (!this.libp2p) return;

    await this.libp2p.handle(this.DIRECT_PROTOCOL, async (data: any) => {
      try {
        // Handle different parameter structures
        const stream = data.stream || data;
        const connection = data.connection;

        if (!stream) {
          logger.warn({
            message: '[P2pClient] No stream provided in protocol handler',
          });
          return;
        }

        const fromPeerId = connection?.remotePeer?.toString() || 'unknown';
        const message = await this.readStreamMessage(stream);

        console.log('Emitting direct-message event:', {
          type: 'direct-message',
          from: fromPeerId,
          fromNickname: this.peerManager.getNickname(fromPeerId) || fromPeerId,
          message,
        });
        await eventEmitter.emit({
          type: 'direct-message',
          from: fromPeerId,
          fromNickname: this.peerManager.getNickname(fromPeerId) || fromPeerId,
          message,
        } as P2pEvent);
      } catch (error) {
        logger.error({
          message: '[P2pClient] Error handling direct message',
          error: error,
        });
      }
    });

    // File protocol is now handled by request-response protocol
    // The old stream-based handler is no longer needed

    if (this.libp2p.services.pubsub) {
      this.libp2p.services.pubsub.addEventListener(
        'message',
        async (event: any) => {
          // The event is a CustomEvent, data is in event.detail
          if (!event.detail) {
            logger.warn({
              message: '[P2pClient] Pubsub message event without detail',
              event: event,
            });
            return;
          }

          const detail = event.detail;
          const topic = detail.topic;

          if (!topic) {
            logger.warn({
              message:
                '[P2pClient] Pubsub message event without topic in detail',
              event: event,
            });
            return;
          }

          if (topic.startsWith('gigi-group:')) {
            // Check if detail.data exists (message content is in detail.data as a Buffer)
            if (!detail.data) {
              logger.warn({
                message: '[P2pClient] Pubsub message event without data',
                detail: detail,
              });
              return;
            }

            const groupName = topic.replace('gigi-group:', '');
            // Decode the message from detail.data (Buffer)
            const messageData = new TextDecoder().decode(detail.data);

            try {
              // Parse the structured message
              const structuredMessage = JSON.parse(messageData);

              // Get the peer ID from the message
              const from =
                structuredMessage.senderPeerId ||
                structuredMessage.content.fromPeerId ||
                'unknown';
              const senderNickname =
                structuredMessage.senderNickname ||
                structuredMessage.content.fromNickname ||
                'unknown';

              // Check if we have any address information for the peer
              let addresses: string[] = [];
              if (detail.message && detail.message.multiaddrs) {
                addresses = detail.message.multiaddrs.map((ma: any) =>
                  ma.toString()
                );
              } else if (detail.multiaddrs) {
                addresses = detail.multiaddrs.map((ma: any) => ma.toString());
              }

              // Add the sender to peer manager if not already present
              const peerIdToAdd = from.toString();
              if (peerIdToAdd !== 'unknown' && senderNickname !== 'unknown') {
                const existingPeer = this.peerManager.getByPeerId(peerIdToAdd);
                if (
                  !existingPeer ||
                  existingPeer.nickname !== senderNickname ||
                  (addresses.length > 0 && existingPeer.addresses.length === 0)
                ) {
                  this.peerManager.discover(
                    peerIdToAdd,
                    senderNickname,
                    addresses
                  );
                }
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
              logger.warn({
                message: '[P2pClient] Error parsing structured message',
                error: error,
              });
              // Fallback to plain text message if parsing fails
              const from = detail.from || 'unknown';
              logger.info({
                message: '[P2pClient] Received plain text message',
                group: groupName,
                from: from,
                content: messageData,
              });

              const nickname =
                this.peerManager.getNickname(from.toString()) ||
                from.toString();

              // Add the sender to peer manager if not already present
              if (from !== 'unknown') {
                const existingPeer = this.peerManager.getByPeerId(
                  from.toString()
                );
                if (!existingPeer) {
                  this.peerManager.discover(from.toString(), nickname, []);
                }
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
        }
      );
    }

    this.libp2p.addEventListener('peer:connect', async (event: any) => {
      if (!event.detail) {
        console.warn('[P2pClient] peer:connect event without detail:', event);
        return;
      }

      // Handle both formats: event.detail as PeerId or event.detail.remotePeer
      const peerId = event.detail.remotePeer
        ? event.detail.remotePeer.toString()
        : event.detail.toString();
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
        console.warn(
          '[P2pClient] peer:disconnect event without detail:',
          event
        );
        return;
      }

      // Handle both formats: event.detail as PeerId or event.detail.remotePeer
      const peerId = event.detail.remotePeer
        ? event.detail.remotePeer.toString()
        : event.detail.toString();
      this.peerManager.removeConnected(peerId);

      await eventEmitter.emit({
        type: 'disconnected',
        peerId,
        nickname: this.peerManager.getNickname(peerId) || peerId,
      } as P2pEvent);
    });
  }

  private addEventListener(
    event: string,
    listener: (event: any) => void
  ): void {
    if (this.libp2p) {
      this.libp2p.addEventListener(event, listener);
      this.eventListeners.push({ event, listener });
    }
  }

  private processSwarmEvents(): void {
    if (!this.libp2p) return;

    // Only listen for DHT peer events if DHT is enabled
    if (this.libp2p.services.dht) {
      const dhtListener = async (event: any) => {
        // Handle both formats: event.detail.id or event.detail as PeerId
        const peerId = event.detail.id
          ? event.detail.id.toString()
          : event.detail.toString();

        // Skip peers with no nickname or with a nickname that looks like a peer ID
        // DHT doesn't provide nicknames, so we'll rely on Gigi DNS for nickname information
        logger.info({
          message: '[P2pClient] DHT discovered peer',
          peerId: peerId,
        });
      };
      this.libp2p.services.dht.addEventListener('peer', dhtListener);
      this.eventListeners.push({ event: 'dht:peer', listener: dhtListener });
    }
  }

  private async readStreamMessage(stream: any): Promise<string> {
    const chunks: Uint8Array[] = [];

    try {
      // Handle nested stream objects (common in some libp2p implementations)
      if (stream.stream) {
        stream = stream.stream;
      }

      // Try for streams with async iterator first (YamuxStream supports this)
      if (typeof stream[Symbol.asyncIterator] === 'function') {
        for await (const chunk of stream) {
          // Handle different chunk types
          if (Buffer.isBuffer(chunk)) {
            chunks.push(new Uint8Array(chunk));
          } else if (chunk instanceof Uint8Array) {
            chunks.push(chunk);
          } else if (Array.isArray(chunk)) {
            chunks.push(new Uint8Array(chunk));
          } else if (chunk && typeof chunk.toUint8Array === 'function') {
            // Handle Uint8ArrayList
            chunks.push(chunk.toUint8Array());
          } else if (chunk && typeof chunk.slice === 'function') {
            // Try to slice as a last resort
            chunks.push(new Uint8Array(chunk.slice(0)));
          } else {
            // Try to convert to Uint8Array anyway
            try {
              chunks.push(new Uint8Array(chunk));
            } catch (error) {
              console.error('[P2pClient] Error converting chunk:', error);
            }
          }
        }
      } else if (typeof (stream as any).receive === 'function') {
        // Try YamuxStream receive method
        let chunk;
        while ((chunk = await (stream as any).receive()) !== null) {
          chunks.push(new Uint8Array(chunk));
        }
      } else if (typeof (stream as any).read === 'function') {
        // Try stream.read() method
        let chunk;
        while ((chunk = (stream as any).read()) !== null) {
          chunks.push(new Uint8Array(chunk));
        }
      }
      // Try the most common duplex stream API
      else if (stream.source) {
        for await (const chunk of stream.source) {
          chunks.push(chunk);
        }
      }
      // Try Web Streams API
      else if (stream.readable) {
        const reader = stream.readable.getReader();
        while (true) {
          const { done, value } = await reader.read();
          if (done) {
            break;
          }
          if (value) {
            chunks.push(value);
          }
        }
      }
      // Try event emitter pattern
      else if (typeof stream.on === 'function') {
        await new Promise<void>((resolve, reject) => {
          stream.on('data', (chunk: Buffer) => {
            chunks.push(new Uint8Array(chunk));
          });
          stream.on('end', () => {
            resolve();
          });
          stream.on('error', (error: Error) => {
            reject(error);
          });
        });
      }
      // Try to get data from stream directly
      else {
        // For some stream types, the data might be available immediately
        throw new Error('Unsupported stream type');
      }
    } catch (error) {
      console.error('[P2pClient] Error reading stream:', error);
      throw error;
    }

    if (chunks.length === 0) {
      return '';
    }

    const allBytes = new Uint8Array(
      chunks.reduce((sum, c) => sum + c.length, 0)
    );
    let offset = 0;
    for (const chunk of chunks) {
      allBytes.set(chunk, offset);
      offset += chunk.length;
    }
    return new TextDecoder().decode(allBytes);
  }

  private async handleFileRequest(
    peerId: string,
    request: FileRequest,
    channel: any
  ): Promise<void> {
    logger.info(`[P2pClient] ====== Handling file request ======`);
    logger.info(`[P2pClient] Peer ID: ${peerId}`);
    logger.info(`[P2pClient] Request: ${JSON.stringify(request)}`);
    logger.info(`[P2pClient] Share code from request: ${request.shareCode}`);
    logger.info(
      `[P2pClient] Number of files in manager: ${this.fileManager.list().length}`
    );
    logger.info(
      `[P2pClient] Available files: ${this.fileManager
        .list()
        .map((f) => `${f.info.name} (${f.shareCode})`)
        .join(', ')}`
    );

    const file = this.fileManager.getByShareCode(request.shareCode);
    logger.info(
      `[P2pClient] Found file: ${file ? file.info.name : 'undefined'}`
    );
    if (!file) {
      logger.info(
        `[P2pClient] File not found for share code: ${request.shareCode}`
      );
      logger.info(
        `[P2pClient] Available share codes: ${this.fileManager
          .list()
          .map((f) => f.shareCode)
          .join(', ')}`
      );
      channel.send({
        type: 'error',
        message: 'File not found',
      } as FileErrorResponse);
      return;
    }

    logger.info({
      message: '[P2pClient] Found file',
      filename: file.info.name,
    });

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

    logger.debug({
      message: '[P2pClient] Sending file info response',
      response: response,
    });
    channel.send(response);
    logger.info('[P2pClient] File info response sent');

    await eventEmitter.emit({
      type: 'file-share-request',
      from: peerId,
      fromNickname: this.peerManager.getNickname(peerId) || peerId,
      shareCode: request.shareCode,
      filename: file.info.name,
      size: file.info.size,
    } as P2pEvent);
  }

  private async handleFileChunkRequest(
    peerId: string,
    request: FileChunkRequest,
    channel: any
  ): Promise<void> {
    logger.info({
      message: '[P2pClient] Handling file chunk request',
      peerId: peerId,
    });
    logger.debug({
      message: '[P2pClient] File chunk request details',
      request: request,
    });

    const file = this.fileManager.getByShareCode(request.shareCode);
    if (!file) {
      logger.warn({
        message: '[P2pClient] File not found',
        shareCode: request.shareCode,
      });
      channel.send({
        type: 'error',
        message: 'File not found',
      } as FileErrorResponse);
      return;
    }

    try {
      // Use the stored file path
      const filePath = (file as any).filePath || file.info.name;
      const chunk = await this.fileManager.getChunk(
        file.fileId,
        request.chunkIndex,
        filePath
      );

      // Send chunk response
      const response = {
        type: 'chunk',
        downloadId: request.downloadId,
        chunkIndex: request.chunkIndex,
        totalChunks: request.totalChunks,
        chunk: chunk,
      } as FileChunkResponse;

      logger.debug({
        message: '[P2pClient] Sending file chunk',
        chunkIndex: request.chunkIndex,
        totalChunks: request.totalChunks,
      });
      channel.send(response);
    } catch (error) {
      logger.error({
        message: '[P2pClient] Error sending chunk',
        error: error,
      });
      channel.send({
        type: 'error',
        message: 'Failed to send chunk',
      } as FileErrorResponse);
    }
  }

  private async handleFileChunk(
    peerId: string,
    request: FileChunkRequest,
    channel: any
  ): Promise<void> {
    const download = this.downloadManager.get(request.downloadId);
    if (!download) {
      channel.send({
        type: 'error',
        message: 'Download not found',
      } as FileErrorResponse);
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
      channel.send({
        type: 'chunk',
        downloadId: request.downloadId,
        chunkIndex: request.chunkIndex,
        totalChunks: request.totalChunks,
        chunk: new Uint8Array(0),
      } as FileChunkResponse);

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
      channel.send({
        type: 'chunk',
        downloadId: request.downloadId,
        chunkIndex: request.chunkIndex,
        totalChunks: request.totalChunks,
        chunk: new Uint8Array(0),
      } as FileChunkResponse);
    }
  }

  private async sendFileMessage(
    targetPeerId: string,
    message: FileRequest | FileChunkRequest
  ): Promise<FileResponseMessage> {
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

  async sendDirectMessage(targetPeerId: string, message: any): Promise<void> {
    if (!this.libp2p || !this.started) {
      throw P2pError.notStarted();
    }

    try {
      // Always use peerId directly to avoid multiaddr version issues
      const peerId = peerIdFromString(targetPeerId);
      const stream = await this.libp2p.dialProtocol(
        peerId,
        this.DIRECT_PROTOCOL
      );
      const messageString =
        typeof message === 'string' ? message : JSON.stringify(message);
      const data = new TextEncoder().encode(messageString);

      // Try multiple methods to send data, similar to RequestResponse
      if (typeof (stream as any).send === 'function') {
        // Use send method if available (YamuxStream)
        await (stream as any).send(data);
        // Close the stream
        if (typeof (stream as any).close === 'function') {
          await (stream as any).close();
        }
      } else if (typeof stream.write === 'function') {
        // Use write method if available
        await new Promise<void>((resolve, reject) => {
          stream.write(data, (error: Error | null) => {
            if (error) {
              reject(error);
            } else {
              if (typeof stream.end === 'function') {
                stream.end(resolve);
              } else {
                resolve();
              }
            }
          });
        });
      } else if (typeof stream.sink === 'function') {
        // Use sink method if available
        await stream.sink([data]);
      } else if (
        typeof stream.writable === 'object' &&
        stream.writable.writable
      ) {
        // Use Web Streams API
        const writer = stream.writable.getWriter();
        await writer.write(data);
        await writer.close();
      } else {
        // Last resort - throw error
        throw new Error('No write method available for stream');
      }
    } catch (error) {
      throw P2pError.networkError(
        `Failed to send message to ${targetPeerId}`,
        error as Error
      );
    }
  }

  async sendDirectMessageToNickname(
    nickname: string,
    message: string
  ): Promise<void> {
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

    await eventEmitter.emit({
      type: 'group-joined',
      group: groupName,
    } as P2pEvent);
  }

  async leaveGroup(groupName: string): Promise<void> {
    if (!this.libp2p || !this.started) {
      throw P2pError.notStarted();
    }

    const topic = `gigi-group:${groupName}`;
    this.libp2p.services.pubsub?.unsubscribe(topic);
    this.groupManager.leave(groupName);

    await eventEmitter.emit({
      type: 'group-left',
      group: groupName,
    } as P2pEvent);
  }

  async sendGroupMessage(
    groupName: string,
    content: MessageContentInput
  ): Promise<void> {
    if (!this.libp2p || !this.started) {
      throw P2pError.notStarted();
    }

    const topic = `gigi-group:${groupName}`;

    // Add sender's peer ID and nickname to all message types
    let fullContent: MessageContent;
    if (content.type === 'fileShare') {
      fullContent = {
        ...content,
        fromPeerId: this.getPeerId(),
        fromNickname: this.nickname,
      };
    } else {
      fullContent = {
        ...content,
        fromPeerId: this.getPeerId(),
        fromNickname: this.nickname,
      };
    }

    const message = JSON.stringify({
      type: 'group-message',
      senderNickname: this.nickname,
      senderPeerId: this.getPeerId(),
      content: fullContent,
      timestamp: Date.now(),
    });
    const data = new TextEncoder().encode(message);

    try {
      await this.libp2p.services.pubsub?.publish(topic, data);
    } catch (error) {
      // Handle the case when no peers are subscribed to the topic
      if (
        error instanceof Error &&
        error.message.includes('NoPeersSubscribedToTopic')
      ) {
        // Silently ignore this error since it's expected when no one else is in the group
        logger.info({
          message: '[P2pClient] No peers subscribed to topic',
          topic: topic,
        });
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
    if (!this.libp2p || !this.started) {
      throw P2pError.notStarted();
    }

    logger.info(`[P2pClient] Sharing file: ${filePath}`);
    const sharedFile = await this.fileManager.share(filePath);
    logger.info(
      `[P2pClient] Shared file: ${sharedFile.info.name} with share code: ${sharedFile.shareCode}`
    );
    logger.info(
      `[P2pClient] Available files after sharing: ${this.fileManager
        .list()
        .map((f) => `${f.info.name} (${f.shareCode})`)
        .join(', ')}`
    );
    logger.info(
      `[P2pClient] Share code index size: ${this.fileManager.getShareCodes().length}`
    );
    logger.info(
      `[P2pClient] Share codes in index: ${this.fileManager.getShareCodes().join(', ')}`
    );

    await eventEmitter.emit({
      type: 'file-shared',
      fileId: sharedFile.fileId,
      info: sharedFile.info,
    } as P2pEvent);

    return sharedFile.shareCode;
  }

  async downloadFile(nickname: string, shareCode: string): Promise<string> {
    if (!this.libp2p || !this.started) {
      throw P2pError.notStarted();
    }

    logger.info({
      message: '[P2pClient] Attempting to download file',
      nickname: nickname,
    });
    logger.debug({
      message: '[P2pClient] Peers in manager',
      peers: this.peerManager
        .list()
        .map((p) => ({ nickname: p.nickname, peerId: p.peerId })),
    });

    const peerId = this.peerManager.getPeerId(nickname);
    if (!peerId) {
      logger.warn({
        message: '[P2pClient] Peer not found in peer manager',
        nickname: nickname,
      });
      throw P2pError.peerNotFound(nickname);
    }
    logger.info({
      message: '[P2pClient] Found peer',
      nickname: nickname,
      peerId: peerId,
    });

    return this.downloadFileByPeerId(peerId, nickname, shareCode);
  }

  async downloadFileByPeerId(
    peerId: string,
    nickname: string,
    shareCode: string
  ): Promise<string> {
    logger.info('[P2pClient] Entering downloadFileByPeerId');
    const downloadId = randomUUID();
    // Send file request to get file info
    const fileRequest: FileRequest = {
      type: 'request',
      action: 'request',
      shareCode,
      downloadId,
    };

    try {
      // Always use peerId for sendFileMessage, not multiaddr
      const response = await this.sendFileMessage(peerId, fileRequest);
      logger.debug({
        message: '[P2pClient] Received response',
        type: response.type,
      });

      if (response.type === 'error') {
        throw P2pError.fileNotFound(shareCode);
      } else if (response.type === 'file-info') {
        logger.info('[P2pClient] Starting chunk requests');
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
        logger.info({
          message: '[P2pClient] Requesting chunks',
          chunkCount: response.chunkCount,
        });
        for (let i = 0; i < response.chunkCount; i++) {
          logger.debug({
            message: '[P2pClient] Requesting chunk',
            chunkIndex: i,
          });
          const chunkRequest: FileChunkRequest = {
            type: 'chunk',
            downloadId,
            shareCode,
            chunkIndex: i,
            totalChunks: response.chunkCount,
            chunk: new Uint8Array(0), // Empty chunk, just requesting data
          };

          // Always use peerId for sendFileMessage, not multiaddr
          const chunkResponse = await this.sendFileMessage(
            peerId,
            chunkRequest
          );
          logger.debug({
            message: '[P2pClient] Received chunk response',
            chunkIndex: i,
            type: chunkResponse.type,
          });
          if (chunkResponse.type === 'chunk' && chunkResponse.chunk) {
            download.data.push(chunkResponse.chunk);
            download.downloadedChunks++;
            logger.debug({
              message: '[P2pClient] Chunk added',
              chunkIndex: i,
              downloaded: download.downloadedChunks,
              total: download.totalChunks,
            });

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
        logger.info('[P2pClient] Chunk requests completed');

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
      logger.error({
        message: '[P2pClient] Error in downloadFileByPeerId',
        error: error,
      });
      throw P2pError.networkError(
        `Failed to download file: ${error instanceof Error ? error.message : 'Unknown error'}`,
        error as Error
      );
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

  listConnectedPeers(): PeerInfo[] {
    return this.peerManager.listConnected();
  }

  async connectToPeer(multiaddr: string): Promise<void> {
    if (!this.libp2p || !this.started) {
      throw P2pError.notStarted();
    }

    try {
      const addr = multiaddrFromString(multiaddr);
      await this.libp2p.dial(addr);
      logger.info({
        message: '[P2pClient] Connected to peer',
        multiaddr: multiaddr,
      });
    } catch (error) {
      throw P2pError.networkError(
        `Failed to connect to peer at ${multiaddr}`,
        error as Error
      );
    }
  }

  onEvent(listener: (event: P2pEvent) => void | Promise<void>): () => void {
    return eventEmitter.on('any', listener as any);
  }

  offEvent(listener: (event: P2pEvent) => void | Promise<void>): void {
    eventEmitter.off('any', listener as any);
  }

  async waitForEvent<T extends P2pEvent>(
    eventType: string,
    timeout: number = 30000
  ): Promise<T> {
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
