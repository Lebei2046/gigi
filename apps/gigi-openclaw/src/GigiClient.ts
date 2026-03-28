import { P2pClient, P2pClientOptions, derivePeerId, derivePeerPrivateKey } from "@gigi/p2p-ts";
import { RequestResponse, JsonCodec } from "@gigi/request-response-ts";
import type { IGigiClient, GigiClientConfig, GigiMessage } from "./types.js";

// Define file protocol request and response types
interface FileRequest {
  type: 'request';
  action: 'request';
  shareCode: string;
  downloadId: string;
}

interface FileChunkRequest {
  type: 'chunk';
  downloadId: string;
  chunkIndex: number;
  totalChunks: number;
  chunk: Uint8Array;
}

interface FileErrorResponse {
  type: 'error';
  message: string;
}

interface FileInfoResponse {
  type: 'file-info';
  fileId: string;
  name: string;
  size: number;
  mimeType: string;
  chunkCount: number;
  hash: string;
}

interface FileChunkResponse {
  type: 'chunk';
  downloadId: string;
  chunkIndex: number;
  totalChunks: number;
  chunk: Uint8Array;
}

type FileRequestMessage = FileRequest | FileChunkRequest;
type FileResponseMessage = FileErrorResponse | FileInfoResponse | FileChunkResponse;

export class GigiClient implements IGigiClient {
  private p2pClient: P2pClient;
  private messageHandlers: ((msg: GigiMessage) => void)[] = [];
  private config: GigiClientConfig;
  private started = false;

  constructor(config: GigiClientConfig) {
    this.config = config;
    
    // Create P2pClient with the provided config
    // Mnemonic derivation will be handled in the start method
    const p2pOptions: P2pClientOptions = {
      nickname: config.displayName || `gigi-${config.peerId.substring(0, 8)}`,
      config: {
        bootstrapNodes: config.bootstrapPeers || [],
        enableKademlia: config.enableDht !== false,
        enableRelay: true,
        enableMdns: config.enableMdns !== false,
        listenAddrs: config.multiaddrs,
      },
      peerIdJson: config.peerIdJson,
    };
    
    this.p2pClient = new P2pClient(p2pOptions);
    
    // Set up event listeners
    this.p2pClient.onEvent(async (event) => {
      if (event.type === 'direct-message') {
          const message: GigiMessage = {
            from: event.from,
            to: this.getPeerId(),
            content: typeof event.message === 'string' ? event.message : JSON.stringify(event.message),
            timestamp: Date.now(),
            type: 'direct',
          };
          this.emitMessage(message);
        } else if (event.type === 'group-message') {
          const message: GigiMessage = {
            from: event.from,
            to: event.group,
            content: typeof event.content === 'string' ? event.content : JSON.stringify(event.content),
            timestamp: Date.now(),
            type: 'broadcast',
          };
          this.emitMessage(message);
        }
    });
  }

  async start(): Promise<void> {
    if (this.started) {
      throw new Error("GigiClient already started");
    }

    await this.p2pClient.start();
    this.started = true;

    console.log(`[GigiClient] Started with peer ID: ${this.getPeerId()}`);
    console.log(`[GigiClient] Listening on: ${this.getMultiaddrs().join(", ")}`);
  }

  async stop(): Promise<void> {
    if (!this.started) {
      return;
    }

    await this.p2pClient.stop();
    this.started = false;
    console.log("[GigiClient] Stopped");
  }

  async sendMessage(targetPeerId: string, content: string): Promise<void> {
    if (!this.started) {
      throw new Error("GigiClient not started");
    }

    await this.p2pClient.sendDirectMessage(targetPeerId, content);
    console.log(`[GigiClient] Sent message to ${targetPeerId}`);
  }

  async sendGroupMessage(groupName: string, content: string): Promise<void> {
    if (!this.started) {
      throw new Error("GigiClient not started");
    }

    await this.p2pClient.sendGroupMessage(groupName, { type: 'text', text: content });
    console.log(`[GigiClient] Sent group message to ${groupName}`);
  }

  async joinGroup(groupName: string): Promise<void> {
    if (!this.started) {
      throw new Error("GigiClient not started");
    }

    await this.p2pClient.joinGroup(groupName);
    console.log(`[GigiClient] Joined group: ${groupName}`);
  }

  async leaveGroup(groupName: string): Promise<void> {
    if (!this.started) {
      throw new Error("GigiClient not started");
    }

    await this.p2pClient.leaveGroup(groupName);
    console.log(`[GigiClient] Left group: ${groupName}`);
  }

  async shareFile(filePath: string): Promise<string> {
    if (!this.started) {
      throw new Error("GigiClient not started");
    }

    return await this.p2pClient.shareFile(filePath);
  }

  async downloadFile(peerId: string, shareCode: string): Promise<string> {
    if (!this.started) {
      throw new Error("GigiClient not started");
    }

    return await this.p2pClient.downloadFile(peerId, shareCode);
  }

  onMessage(handler: (msg: GigiMessage) => void): void {
    this.messageHandlers.push(handler);
  }

  private emitMessage(message: GigiMessage): void {
    for (const handler of this.messageHandlers) {
      try {
        handler(message);
      } catch (error) {
        console.error("[GigiClient] Error in message handler:", error);
      }
    }
  }

  getPeerId(): string {
    if (!this.started) {
      throw new Error("GigiClient not started");
    }
    return this.p2pClient.getPeerId();
  }

  getMultiaddrs(): string[] {
    if (!this.started) {
      throw new Error("GigiClient not started");
    }
    return this.p2pClient.getMultiaddrs();
  }

  isConnected(): boolean {
    return this.started && this.p2pClient.isStarted();
  }

  listPeers(): any[] {
    if (!this.started) {
      throw new Error("GigiClient not started");
    }
    return this.p2pClient.listPeers();
  }

  listGroups(): any[] {
    if (!this.started) {
      throw new Error("GigiClient not started");
    }
    return this.p2pClient.getJoinedGroups();
  }
}
