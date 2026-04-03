import { MessagingClient, MessagingEvents } from './messaging';
import type {
  AmpMessage,
  AgentInfo
} from '@gigi/amp-ts';
import {
  InMemoryAgentRegistry,
  AmpMessageRouter,
  AmpMessageFactory
} from '@gigi/amp-ts';


// Extend AgentInfo to include OpenClaw agents
export interface Agent extends AgentInfo {
  openclawAgents?: {
    name: string;
    id: string;
    model: string;
    status: 'online' | 'offline' | 'busy';
  }[];
}

// Agent Messaging Client
export class AgentMessagingClient {
  private static instance: AgentMessagingClient;
  private agentRegistry: InMemoryAgentRegistry;
  private messageRouter: AmpMessageRouter;
  private peerId: string = '';
  private nickname: string = '';
  public static readonly AGENT_GROUP_NAME = 'gigi-agents';

  private constructor() {
    this.agentRegistry = new InMemoryAgentRegistry();
    this.messageRouter = new AmpMessageRouter(this.agentRegistry);
    this.initialize();
  }

  static getInstance(): AgentMessagingClient {
    if (!AgentMessagingClient.instance) {
      AgentMessagingClient.instance = new AgentMessagingClient();
    }
    return AgentMessagingClient.instance;
  }

  private async initialize(): Promise<void> {
    try {
      this.peerId = await MessagingClient.getPeerId();
      const config = await MessagingClient.getConfig();
      this.nickname = config.nickname;
      await this.joinAgentGroup();
      this.setupEventListeners();
    } catch (error) {
      console.error('Failed to initialize AgentMessagingClient:', error);
    }
  }

  private async joinAgentGroup(): Promise<void> {
    try {
      await MessagingClient.joinGroup(AgentMessagingClient.AGENT_GROUP_NAME);
      console.log(`Joined agent group: ${AgentMessagingClient.AGENT_GROUP_NAME}`);
    } catch (error) {
      console.warn(`Failed to join agent group: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  private setupEventListeners(): void {
    // Listen for group messages in the agent group
    MessagingEvents.on('group-message', (event: any) => {
      if (event.group_id === AgentMessagingClient.AGENT_GROUP_NAME) {
        this.handleAgentGroupMessage(event);
      }
    });

    // Listen for direct messages that might be AMP messages
    MessagingEvents.on('direct-message', (event: any) => {
      this.handleDirectMessage(event);
    });
  }

  private handleAgentGroupMessage(event: any): void {
    if (event.content.type === 'text' && event.content.text.startsWith('{"type":"')) {
      try {
        const ampMessage = JSON.parse(event.content.text) as AmpMessage;
        if (ampMessage.type) {
          console.log(`[AMP GROUP] ${event.from_nickname} sent an AMP message: ${ampMessage.type}`);
          this.messageRouter.routeMessage(ampMessage);
        }
      } catch {
        // Not an AMP message, treat as regular text
        console.log(`[AGENT GROUP] ${event.from_nickname}: ${event.content.text}`);
      }
    } else if (event.content.type === 'text') {
      console.log(`[AGENT GROUP] ${event.from_nickname}: ${event.content.text}`);
    } else if (event.content.type === 'fileShare') {
      console.log(`[AGENT GROUP] ${event.from_nickname} shared a file: ${event.content.filename} (${event.content.fileSize} bytes)`);
    }
  }

  private handleDirectMessage(event: any): void {
    try {
      const ampMessage = JSON.parse(event.message) as AmpMessage;
      if (ampMessage.type) {
        console.log(`[AMP DIRECT] ${event.from_nickname} sent an AMP message: ${ampMessage.type}`);
        this.messageRouter.routeMessage(ampMessage);
      }
    } catch {
      // Not an AMP message, treat as regular text
      console.log(`[DIRECT] ${event.from_nickname}: ${event.message}`);
    }
  }


  // Send text message to agents
  async sendTextMessage(
    message: string,
    targetType: 'all' | 'specific',
    agentIds?: string[]
  ): Promise<void> {
    const target = targetType === 'all' ? { type: 'all' as const } : { type: 'specific' as const, agentIds };
    const ampMessage = AmpMessageFactory.createTextMessage(
      message,
      target,
      { id: this.peerId, name: this.nickname, type: 'owner' as const }
    );

    await this.sendAmpMessage(ampMessage);
  }

  // Send file message to agents
  async sendFileMessage(
    shareCode: string,
    filename: string,
    fileSize: number,
    targetType: 'all' | 'specific',
    agentIds?: string[]
  ): Promise<void> {
    const target = targetType === 'all' ? { type: 'all' as const } : { type: 'specific' as const, agentIds };
    const ampMessage = AmpMessageFactory.createFileMessage(
      filename,
      fileSize,
      shareCode,
      target,
      { id: this.peerId, name: this.nickname, type: 'owner' as const }
    );

    await this.sendAmpMessage(ampMessage);
  }

  // Query agent settings
  async queryAgentSettings(agentIds?: string[]): Promise<void> {
    const ampMessage = AmpMessageFactory.createAgentSettingsQuery(
      { id: this.peerId, name: this.nickname, type: 'owner' as const },
      agentIds
    );

    await this.sendAmpMessage(ampMessage);
  }

  // Send AMP message to agent group
  private async sendAmpMessage(ampMessage: AmpMessage): Promise<void> {
    try {
      await MessagingClient.sendGroupMessage(
        AgentMessagingClient.AGENT_GROUP_NAME,
        JSON.stringify(ampMessage)
      );
      console.log(`Sent AMP message of type ${ampMessage.type} to agent group`);
    } catch (error) {
      console.error(`Error sending AMP message: ${error instanceof Error ? error.message : 'Unknown error'}`);
      throw error;
    }
  }

  // Register message handler
  registerMessageHandler(messageType: string, handler: (message: AmpMessage, agentId?: string) => void): void {
    this.messageRouter.registerMessageHandler(messageType, handler);
  }

  // Get all agents
  getAllAgents(): Agent[] {
    return this.agentRegistry.getAllAgents() as Agent[];
  }

  // Register an agent
  registerAgent(agent: Agent): void {
    this.agentRegistry.registerAgent(agent);
  }

  // Update agent status
  updateAgentStatus(agentId: string, status: 'online' | 'offline' | 'busy'): void {
    this.agentRegistry.updateAgentStatus(agentId, status);
  }
}

// Export singleton instance
export const agentMessagingClient = AgentMessagingClient.getInstance();