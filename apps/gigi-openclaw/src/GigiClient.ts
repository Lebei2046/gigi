import { P2pClient, P2pClientOptions } from '@gigi/p2p-ts';
import type { P2pEvent } from '@gigi/p2p-ts';
import {
  AmpMessageRouter,
  AmpMessageFactory,
  InMemoryAgentRegistry,
} from '@gigi/amp-ts';
import type { IGigiClient, GigiClientConfig, GigiMessage } from './types.js';

export class GigiClient implements IGigiClient {
  private p2pClient: P2pClient;
  private messageHandlers: ((msg: GigiMessage) => void)[] = [];
  private config: GigiClientConfig;
  private started = false;
  private agentRegistry: InMemoryAgentRegistry;
  private messageRouter: AmpMessageRouter;

  constructor(config: GigiClientConfig) {
    this.config = config;

    // Create P2pClient with the provided config
    // Mnemonic derivation will be handled in the start method
    const p2pOptions: P2pClientOptions = {
      nickname:
        config.displayName ||
        `gigi-${Math.random().toString(36).substring(2, 10)}`,
      config: {
        bootstrapNodes: config.bootstrapPeers || [],
        enableKademlia: config.enableDht !== false,
        enableRelay: true,
        enableMdns: config.enableMdns !== false,
        ...(config.multiaddrs &&
          config.multiaddrs.length > 0 && { listenAddrs: config.multiaddrs }),
      },
      mnemonic: config.mnemonic,
    };

    this.p2pClient = new P2pClient(p2pOptions);

    // Initialize agent registry and message router
    this.agentRegistry = new InMemoryAgentRegistry();
    this.messageRouter = new AmpMessageRouter(this.agentRegistry);

    // Set up event listeners
    this.p2pClient.onEvent(async (event: P2pEvent) => {
      if (event.type === 'direct-message') {
        try {
          const messageData =
            typeof event.message === 'string'
              ? JSON.parse(event.message)
              : event.message;
          this.emitMessage(messageData as GigiMessage);
        } catch (error) {
          console.error('[GigiClient] Error parsing direct message:', error);
        }
      } else if (event.type === 'group-message') {
        try {
          // For group messages, the content is an object with type and text properties
          // The actual AMP message is in the text field
          if (
            event.content &&
            event.content.type === 'text' &&
            typeof event.content.text === 'string'
          ) {
            const messageData = JSON.parse(event.content.text);
            this.emitMessage(messageData as GigiMessage);
          } else {
            console.error(
              '[GigiClient] Unexpected group message format:',
              event.content
            );
          }
        } catch (error) {
          console.error('[GigiClient] Error parsing group message:', error);
        }
      }
    });
  }

  async start(): Promise<void> {
    if (this.started) {
      throw new Error('GigiClient already started');
    }

    await this.p2pClient.start();
    this.started = true;

    // Get peer ID and multiaddrs after setting started to true
    const peerId = this.p2pClient.getPeerId();
    const multiaddrs = this.p2pClient.getMultiaddrs();

    console.log(`[GigiClient] Started with peer ID: ${peerId}`);
    console.log(`[GigiClient] Listening on: ${multiaddrs.join(', ')}`);
  }

  async stop(): Promise<void> {
    if (!this.started) {
      return;
    }

    await this.p2pClient.stop();
    this.started = false;
    console.log('[GigiClient] Stopped');
  }

  async sendMessage(target: string, message: string): Promise<void> {
    if (!this.started) {
      throw new Error('GigiClient not started');
    }

    const peerId = this.p2pClient.getPeerId();
    const displayName = this.config.displayName || peerId.substring(0, 8);

    const textMessage = AmpMessageFactory.createTextMessage(
      message,
      { type: 'specific', agentIds: [target] },
      { id: peerId, name: displayName, type: 'owner' }
    );

    await this.p2pClient.sendDirectMessage(target, JSON.stringify(textMessage));
    console.log(`[GigiClient] Sent text message to ${target}`);
  }

  async sendFileMessage(
    target: string,
    filename: string,
    fileSize: number
  ): Promise<void> {
    if (!this.started) {
      throw new Error('GigiClient not started');
    }

    const peerId = this.p2pClient.getPeerId();
    const displayName = this.config.displayName || peerId.substring(0, 8);

    // Create a file hash placeholder (in a real implementation, this would be a actual hash of the file)
    const fileHash = `hash-${Date.now()}`;

    const fileMessage = AmpMessageFactory.createFileMessage(
      filename,
      fileSize,
      fileHash,
      { type: 'specific', agentIds: [target] },
      { id: peerId, name: displayName, type: 'owner' }
    );

    await this.p2pClient.sendDirectMessage(target, JSON.stringify(fileMessage));
    console.log(`[GigiClient] Sent file message to ${target}`);
  }

  async sendGroupMessage(groupName: string, content: string): Promise<void> {
    if (!this.started) {
      throw new Error('GigiClient not started');
    }

    // Create a MessageContentInput for the P2pClient
    const messageContent: { type: 'text'; text: string } = {
      type: 'text',
      text: content,
    };

    await this.p2pClient.sendGroupMessage(groupName, messageContent);
    console.log(`[GigiClient] Sent group text message to ${groupName}`);
  }

  async sendGroupFileMessage(
    groupName: string,
    filename: string,
    fileSize: number,
    fileType: string,
    shareCode: string
  ): Promise<void> {
    if (!this.started) {
      throw new Error('GigiClient not started');
    }

    // Create a MessageContentInput for the P2pClient
    const messageContent: {
      type: 'fileShare';
      shareCode: string;
      filename: string;
      fileSize: number;
      fileType: string;
    } = {
      type: 'fileShare',
      shareCode,
      filename,
      fileSize,
      fileType,
    };

    await this.p2pClient.sendGroupMessage(groupName, messageContent);
    console.log(`[GigiClient] Sent group file message to ${groupName}`);
  }

  async joinGroup(groupName: string): Promise<void> {
    if (!this.started) {
      throw new Error('GigiClient not started');
    }

    await this.p2pClient.joinGroup(groupName);
    console.log(`[GigiClient] Joined group: ${groupName}`);
  }

  async leaveGroup(groupName: string): Promise<void> {
    if (!this.started) {
      throw new Error('GigiClient not started');
    }

    await this.p2pClient.leaveGroup(groupName);
    console.log(`[GigiClient] Left group: ${groupName}`);
  }

  async shareFile(filePath: string): Promise<string> {
    if (!this.started) {
      throw new Error('GigiClient not started');
    }

    return await this.p2pClient.shareFile(filePath);
  }

  async downloadFile(peerId: string, shareCode: string): Promise<string> {
    if (!this.started) {
      throw new Error('GigiClient not started');
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
        console.error('[GigiClient] Error in message handler:', error);
      }
    }
  }

  getPeerId(): string {
    if (!this.started) {
      throw new Error('GigiClient not started');
    }
    return this.p2pClient.getPeerId();
  }

  getMultiaddrs(): string[] {
    if (!this.started) {
      throw new Error('GigiClient not started');
    }
    return this.p2pClient.getMultiaddrs();
  }

  isConnected(): boolean {
    return this.started && this.p2pClient.isStarted();
  }

  listPeers(): any[] {
    if (!this.started) {
      throw new Error('GigiClient not started');
    }
    return this.p2pClient.listPeers();
  }

  listGroups(): any[] {
    if (!this.started) {
      throw new Error('GigiClient not started');
    }
    return this.p2pClient.getJoinedGroups();
  }

  getFileByShareCode(shareCode: string): any {
    if (!this.started) {
      throw new Error('GigiClient not started');
    }
    return this.p2pClient.getFileByShareCode(shareCode);
  }
}
