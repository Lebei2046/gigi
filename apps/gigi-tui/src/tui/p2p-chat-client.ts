import { randomUUID } from "node:crypto";
import { P2pClient } from "@gigi/p2p";
import { RequestResponse } from "@gigi/request-response";
import { createLogger } from "@gigi/logging";
import { AmpMessageFactory } from "@gigi/amp";

const logger = createLogger({ name: "gigi-tui" });

// Reuse the same event and session types as openclaw to maintain compatibility
export type GatewayEvent = {
  event: string;
  payload?: unknown;
  seq?: number;
};

export type ChatSendOptions = {
  sessionKey: string;
  message: string;
  thinking?: string;
  deliver?: boolean;
  timeoutMs?: number;
  runId?: string;
};

export type GatewaySessionList = {
  ts: number;
  path: string;
  count: number;
  defaults?: {
    model?: string | null;
    modelProvider?: string | null;
    contextTokens?: number | null;
  };
  sessions: Array<
    {
      key: string;
      sessionId?: string;
      updatedAt?: number | null;
      fastMode?: boolean;
      sendPolicy?: string;
      responseUsage?: string;
      label?: string;
      provider?: string;
      groupChannel?: string;
      space?: string;
      subject?: string;
      chatType?: string;
      lastProvider?: string;
      lastTo?: string;
      lastAccountId?: string;
      derivedTitle?: string;
      lastMessagePreview?: string;
      thinkingLevel?: string;
      verboseLevel?: string;
      reasoningLevel?: string;
      model?: string;
      contextTokens?: number;
      inputTokens?: number;
      outputTokens?: number;
      totalTokens?: number;
      modelProvider?: string;
      displayName?: string;
    }
  >;
};

export type GatewayAgentsList = {
  defaultId: string;
  mainKey: string;
  scope: string;
  agents: Array<{
    id: string;
    name?: string;
  }>;
};

export type GatewayModelChoice = {
  id: string;
  name: string;
  provider: string;
  contextWindow?: number;
  reasoning?: boolean;
};

export type P2PConnectionOptions = {
  nickname: string;
  host?: string;
  port?: number;
};

export class P2PChatClient {
  private client: P2pClient;
  private nickname: string;
  private readyPromise: Promise<void>;
  private resolveReady?: () => void;
  private connected = false;

  onEvent?: (evt: GatewayEvent) => void;
  onConnected?: () => void;
  onDisconnected?: (reason: string) => void;
  onGap?: (info: { expected: number; received: number }) => void;

  constructor(options: P2PConnectionOptions) {
    this.nickname = options.nickname;

    this.readyPromise = new Promise((resolve) => {
      this.resolveReady = resolve;
    });

    // Initialize Gigi P2P client
    this.client = new P2pClient({
      nickname: options.nickname,
      config: {
        bootstrapNodes: options.host && options.port ? [`${options.host}:${options.port}`] : undefined,
      },
    });

    // Set up event listeners
    this.client.onEvent((event) => {
      if (event.type === 'connected') {
        logger.info(`Connected to peer ${event.peerId} (${event.nickname})`);
        this.connected = true;
        this.resolveReady?.();
        this.onConnected?.();
      } else if (event.type === 'disconnected') {
        logger.info(`Disconnected from peer ${event.peerId}`);
        this.connected = false;
        // Reset ready promise for reconnection
        this.readyPromise = new Promise((resolve) => {
          this.resolveReady = resolve;
        });
        this.onDisconnected?.('peer disconnected');
      } else if (event.type === 'direct-message') {
        try {
          const ampMessage = JSON.parse(event.message);
          // Check if it's an AMP text message (format from openclaw logs)
          if (ampMessage.type === "text" && ampMessage.content) {
            // The content is the actual response text from the agent
            this.onEvent?.({
              event: 'chat',
              payload: { content: ampMessage.content },
              seq: ampMessage.id,
            });
          } else if (ampMessage.event) {
            // Fallback for non-AMP messages
            this.onEvent?.({
              event: ampMessage.event,
              payload: ampMessage.payload,
              seq: ampMessage.seq,
            });
          } else if (ampMessage.content) {
            // Fallback for messages with just content
            this.onEvent?.({
              event: 'chat',
              payload: { content: ampMessage.content },
              seq: ampMessage.id || Date.now(),
            });
          }
        } catch (error) {
          logger.error('Failed to parse incoming message: ' + String(error));
        }
      } else if (event.type === 'group-message' && event.group === 'gigi-agents') {
        try {
          // Parse group message data
          const messageData = event.content;
          if (messageData.type === 'text' && messageData.text) {
            // Parse the message content
            const messageContent = JSON.parse(messageData.text);
            
            // Check if it's a response message
            if (messageContent.type === "response" && messageContent.event === 'sessions.list' && messageContent.payload) {
              // Handle session list events
              this.onEvent?.({
                event: 'sessions.list',
                payload: messageContent.payload,
                seq: messageContent.id,
              });
            } else if (messageContent.type === "text" && messageContent.content && messageContent.sender && messageContent.sender.id === 'main') {
              // The content is the actual response text from the agent
              this.onEvent?.({
                event: 'chat',
                payload: { content: messageContent.content },
                seq: messageContent.id,
              });
            } else if (messageContent.event) {
              // Handle other events
              this.onEvent?.({
                event: messageContent.event,
                payload: messageContent.payload,
                seq: messageContent.id,
              });
            }
          }
        } catch (error) {
          logger.error('Failed to parse group message: ' + String(error));
        }
      }
    });
  }

  static async connect(opts: P2PConnectionOptions): Promise<P2PChatClient> {
    const client = new P2PChatClient(opts);
    await client.start();
    return client;
  }

  async start() {
    await this.client.start();
    logger.info('Gigi P2P client started');
    
    // Join the gigi-agents group (as per test case)
    try {
      await this.client.joinGroup('gigi-agents');
      logger.info('Joined gigi-agents group successfully');
    } catch (error) {
      logger.error('Failed to join gigi-agents group: ' + String(error));
    }
  }

  stop() {
    this.client.stop();
    logger.info('Gigi P2P client stopped');
  }

  async waitForReady() {
    await this.readyPromise;
  }

  async sendChat(opts: ChatSendOptions): Promise<{ runId: string }> {
    const runId = opts.runId ?? randomUUID();
    await this.waitForReady();

    // Create AMP-compliant message using AmpMessageFactory (as per test case)
    const ampMessage = AmpMessageFactory.createTextMessage(
      opts.message,  // Message content
      { type: 'specific', agentIds: ['main'] },  // Target main agent
      {
        id: this.client.getPeerId(),  // Sender ID (gigi-tui's peer ID)
        name: "gigi-tui",  // Sender name
        type: "agent"  // Sender type
      }
    );

    // Send the AMP message via group chat (as per test case)
    try {
      // Format message correctly for P2pClient
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent to gigi-agents successfully');
    } catch (error) {
      logger.error('Failed to send group message: ' + String(error));
      // Retry sending message
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent after retry');
    }
    return { runId };
  }

  async abortChat(opts: { sessionKey: string; runId: string }) {
    await this.waitForReady();

    // Create AMP-compliant message using AmpMessageFactory
    const ampMessage = AmpMessageFactory.createTextMessage(
      `Abort chat: ${opts.runId}`,  // Message content
      { type: 'specific', agentIds: ['main'] },  // Target main agent
      {
        id: this.client.getPeerId(),  // Sender ID (gigi-tui's peer ID)
        name: this.nickname,  // Sender name
        type: "agent"  // Sender type
      }
    );

    // Send the AMP message via group chat
    try {
      // Format message correctly for P2pClient
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent to gigi-agents successfully');
    } catch (error) {
      logger.error('Failed to send group message: ' + String(error));
      // Retry sending message
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent after retry');
    }
  }

  async loadHistory(opts: { sessionKey: string; limit?: number }) {
    await this.waitForReady();

    // Create AMP-compliant message using AmpMessageFactory
    const ampMessage = AmpMessageFactory.createTextMessage(
      `Load history for session: ${opts.sessionKey}`,  // Message content
      { type: 'specific', agentIds: ['main'] },  // Target main agent
      {
        id: this.client.getPeerId(),  // Sender ID (gigi-tui's peer ID)
        name: this.nickname,  // Sender name
        type: "agent"  // Sender type
      }
    );

    // Send the AMP message via group chat
    try {
      // Format message correctly for P2pClient
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent to gigi-agents successfully');
    } catch (error) {
      logger.error('Failed to send group message: ' + String(error));
      // Retry sending message
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent after retry');
    }
    // Return a placeholder - actual data will come through onEvent
    return { messages: [] };
  }

  async listSessions(opts?: { limit?: number; activeMinutes?: number; includeGlobal?: boolean; includeUnknown?: boolean; includeDerivedTitles?: boolean; includeLastMessage?: boolean; agentId?: string }) {
    await this.waitForReady();

    // Create structured request message
    const requestMessage = {
      type: 'request' as const,
      method: 'sessions.list' as const,
      params: opts || {},
      id: `req-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      sender: {
        id: this.client.getPeerId(),
        name: this.nickname,
        type: "agent" as const
      }
    };

    // Send the structured request via group chat
    try {
      // Format message correctly for P2pClient
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(requestMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Session list request sent to gigi-agents successfully');
    } catch (error) {
      logger.error('Failed to send session list request: ' + String(error));
      // Retry sending message
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(requestMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Session list request sent after retry');
    }
    // Return a placeholder - actual data will come through onEvent
    return { sessions: [], ts: Date.now(), path: "", count: 0 } as GatewaySessionList;
  }

  async listAgents() {
    await this.waitForReady();

    // Create AMP-compliant message using AmpMessageFactory
    const ampMessage = AmpMessageFactory.createTextMessage(
      'List agents',  // Message content
      { type: 'specific', agentIds: ['main'] },  // Target main agent
      {
        id: this.client.getPeerId(),  // Sender ID (gigi-tui's peer ID)
        name: this.nickname,  // Sender name
        type: "agent"  // Sender type
      }
    );

    // Send the AMP message via group chat
    try {
      // Format message correctly for P2pClient
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent to gigi-agents successfully');
    } catch (error) {
      logger.error('Failed to send group message: ' + String(error));
      // Retry sending message
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent after retry');
    }
    // Return a placeholder - actual data will come through onEvent
    return { agents: [], defaultId: 'default', mainKey: 'main', scope: 'per-sender' } as GatewayAgentsList;
  }

  async patchSession(opts: any): Promise<any> {
    await this.waitForReady();

    // Create AMP-compliant message using AmpMessageFactory
    const ampMessage = AmpMessageFactory.createTextMessage(
      `Patch session: ${JSON.stringify(opts)}`,  // Message content
      { type: 'specific', agentIds: ['main'] },  // Target main agent
      {
        id: this.client.getPeerId(),  // Sender ID (gigi-tui's peer ID)
        name: this.nickname,  // Sender name
        type: "agent"  // Sender type
      }
    );

    // Send the AMP message via group chat
    try {
      // Format message correctly for P2pClient
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent to gigi-agents successfully');
    } catch (error) {
      logger.error('Failed to send group message: ' + String(error));
      // Retry sending message
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent after retry');
    }
    return {};
  }

  async resetSession(key: string, reason?: "new" | "reset") {
    await this.waitForReady();

    // Create AMP-compliant message using AmpMessageFactory
    const ampMessage = AmpMessageFactory.createTextMessage(
      `Reset session: ${key} ${reason ? `(${reason})` : ''}`,  // Message content
      { type: 'specific', agentIds: ['main'] },  // Target main agent
      {
        id: this.client.getPeerId(),  // Sender ID (gigi-tui's peer ID)
        name: this.nickname,  // Sender name
        type: "agent"  // Sender type
      }
    );

    // Send the AMP message via group chat
    try {
      // Format message correctly for P2pClient
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent to gigi-agents successfully');
    } catch (error) {
      logger.error('Failed to send group message: ' + String(error));
      // Retry sending message
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent after retry');
    }
  }

  async getGatewayStatus() {
    await this.waitForReady();

    // Create AMP-compliant message using AmpMessageFactory
    const ampMessage = AmpMessageFactory.createTextMessage(
      'Get gateway status',  // Message content
      { type: 'specific', agentIds: ['main'] },  // Target main agent
      {
        id: this.client.getPeerId(),  // Sender ID (gigi-tui's peer ID)
        name: this.nickname,  // Sender name
        type: "agent"  // Sender type
      }
    );

    // Send the AMP message via group chat
    try {
      // Format message correctly for P2pClient
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent to gigi-agents successfully');
    } catch (error) {
      logger.error('Failed to send group message: ' + String(error));
      // Retry sending message
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent after retry');
    }
    return {};
  }

  async listModels(): Promise<GatewayModelChoice[]> {
    await this.waitForReady();

    // Create AMP-compliant message using AmpMessageFactory
    const ampMessage = AmpMessageFactory.createTextMessage(
      'List models',  // Message content
      { type: 'specific', agentIds: ['main'] },  // Target main agent
      {
        id: this.client.getPeerId(),  // Sender ID (gigi-tui's peer ID)
        name: this.nickname,  // Sender name
        type: "agent"  // Sender type
      }
    );

    // Send the AMP message via group chat
    try {
      // Format message correctly for P2pClient
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent to gigi-agents successfully');
    } catch (error) {
      logger.error('Failed to send group message: ' + String(error));
      // Retry sending message
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent after retry');
    }
    return [];
  }

  async createSession(label: string): Promise<string> {
    await this.waitForReady();
    const sessionKey = `session-${Date.now()}`;

    // Create structured request message
    const requestMessage = {
      type: 'request' as const,
      method: 'sessions.create' as const,
      params: { label },
      id: `req-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      sender: {
        id: this.client.getPeerId(),
        name: this.nickname,
        type: "agent" as const
      }
    };

    // Send the structured request via group chat
    try {
      // Format message correctly for P2pClient
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(requestMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Session create request sent to gigi-agents successfully');
    } catch (error) {
      logger.error('Failed to send session create request: ' + String(error));
      // Retry sending message
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(requestMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Session create request sent after retry');
    }
    return sessionKey;
  }

  async deleteSession(sessionKey: string): Promise<void> {
    await this.waitForReady();

    // Create AMP-compliant message using AmpMessageFactory
    const ampMessage = AmpMessageFactory.createTextMessage(
      `Delete session: ${sessionKey}`,  // Message content
      { type: 'specific', agentIds: ['main'] },  // Target main agent
      {
        id: this.client.getPeerId(),  // Sender ID (gigi-tui's peer ID)
        name: this.nickname,  // Sender name
        type: "agent"  // Sender type
      }
    );

    // Send the AMP message via group chat
    try {
      // Format message correctly for P2pClient
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent to gigi-agents successfully');
    } catch (error) {
      logger.error('Failed to send group message: ' + String(error));
      // Retry sending message
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent after retry');
    }
  }

  async switchSession(sessionKey: string): Promise<void> {
    await this.waitForReady();

    // Create AMP-compliant message using AmpMessageFactory
    const ampMessage = AmpMessageFactory.createTextMessage(
      `Switch session: ${sessionKey}`,  // Message content
      { type: 'specific', agentIds: ['main'] },  // Target main agent
      {
        id: this.client.getPeerId(),  // Sender ID (gigi-tui's peer ID)
        name: this.nickname,  // Sender name
        type: "agent"  // Sender type
      }
    );

    // Send the AMP message via group chat
    try {
      // Format message correctly for P2pClient
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent to gigi-agents successfully');
    } catch (error) {
      logger.error('Failed to send group message: ' + String(error));
      // Retry sending message
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await this.client.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent after retry');
    }
  }
}
