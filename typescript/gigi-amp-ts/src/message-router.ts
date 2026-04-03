import {
  AmpMessage,
  AgentInfo,
  AgentRegistry,
  MessageRouter,
  TextMessage,
  FileMessage,
  AgentSettingsQuery,
  AgentSettingsResponse,
} from './types';

export class InMemoryAgentRegistry implements AgentRegistry {
  private agents: Map<string, AgentInfo> = new Map();

  getAgentById(id: string): AgentInfo | undefined {
    return this.agents.get(id);
  }

  getAllAgents(): AgentInfo[] {
    return Array.from(this.agents.values());
  }

  updateAgentStatus(id: string, status: 'online' | 'offline' | 'busy'): void {
    const agent = this.agents.get(id);
    if (agent) {
      this.agents.set(id, { ...agent, status });
    }
  }

  registerAgent(agent: AgentInfo): void {
    this.agents.set(agent.id, agent);
  }

  unregisterAgent(agentId: string): void {
    this.agents.delete(agentId);
  }
}

export class AmpMessageRouter implements MessageRouter {
  private agentRegistry: AgentRegistry;
  private messageHandlers: Map<
    string,
    (message: AmpMessage, agentId?: string) => void
  > = new Map();

  constructor(agentRegistry: AgentRegistry) {
    this.agentRegistry = agentRegistry;
  }

  routeMessage(message: AmpMessage): void {
    switch (message.type) {
      case 'text':
        this.routeTextMessage(message as TextMessage);
        break;
      case 'file':
        this.routeFileMessage(message as FileMessage);
        break;
      case 'agent-settings-query':
        this.handleAgentSettingsQuery(message as AgentSettingsQuery);
        break;
      case 'agent-settings-response':
        this.handleAgentSettingsResponse(message as AgentSettingsResponse);
        break;
      default:
        console.warn(`Unknown message type: ${(message as any).type}`);
    }
  }

  private routeTextMessage(message: TextMessage): void {
    if (message.target.type === 'all') {
      // Route to all agents
      const agents = this.agentRegistry.getAllAgents();
      agents.forEach((agent) => {
        this.invokeMessageHandler('text', message, agent.id);
      });
    } else if (message.target.type === 'specific' && message.target.agentIds) {
      // Route to specific agents
      message.target.agentIds.forEach((agentId) => {
        const agent = this.agentRegistry.getAgentById(agentId);
        if (agent) {
          this.invokeMessageHandler('text', message, agentId);
        }
      });
    }
  }

  private routeFileMessage(message: FileMessage): void {
    if (message.target.type === 'all') {
      // Route to all agents
      const agents = this.agentRegistry.getAllAgents();
      agents.forEach((agent) => {
        this.invokeMessageHandler('file', message, agent.id);
      });
    } else if (message.target.type === 'specific' && message.target.agentIds) {
      // Route to specific agents
      message.target.agentIds.forEach((agentId) => {
        const agent = this.agentRegistry.getAgentById(agentId);
        if (agent) {
          this.invokeMessageHandler('file', message, agentId);
        }
      });
    }
  }

  private handleAgentSettingsQuery(message: AgentSettingsQuery): void {
    // Get requested agents or all agents
    const agents = message.agentIds
      ? message.agentIds
          .map((id) => this.agentRegistry.getAgentById(id))
          .filter((agent): agent is AgentInfo => agent !== undefined)
      : this.agentRegistry.getAllAgents();

    // Create response message
    const response: AgentSettingsResponse = {
      type: 'agent-settings-response',
      agents,
      sender: {
        id: 'system',
        name: 'System',
        type: 'agent',
      },
      timestamp: Date.now(),
      id: `response-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
    };

    // Send response to the sender
    this.invokeMessageHandler(
      'agent-settings-response',
      response,
      message.sender.id
    );
  }

  private handleAgentSettingsResponse(message: AgentSettingsResponse): void {
    // Route response to the owner or original requester
    this.invokeMessageHandler('agent-settings-response', message);
  }

  private invokeMessageHandler(
    type: string,
    message: AmpMessage,
    agentId?: string
  ): void {
    const handler = this.messageHandlers.get(type);
    if (handler) {
      try {
        handler(message, agentId);
      } catch (error) {
        console.error(`Error handling ${type} message:`, error);
      }
    }
  }

  registerMessageHandler(
    type: string,
    handler: (message: AmpMessage, agentId?: string) => void
  ): void {
    this.messageHandlers.set(type, handler);
  }

  unregisterMessageHandler(type: string): void {
    this.messageHandlers.delete(type);
  }

  registerAgent(agent: AgentInfo): void {
    if (this.agentRegistry instanceof InMemoryAgentRegistry) {
      (this.agentRegistry as InMemoryAgentRegistry).registerAgent(agent);
    }
  }

  unregisterAgent(agentId: string): void {
    if (this.agentRegistry instanceof InMemoryAgentRegistry) {
      (this.agentRegistry as InMemoryAgentRegistry).unregisterAgent(agentId);
    }
  }
}

export class AmpMessageFactory {
  static createTextMessage(
    content: string,
    target: { type: 'all' | 'specific'; agentIds?: string[] },
    sender: { id: string; name: string; type: 'owner' | 'agent' }
  ): TextMessage {
    return {
      type: 'text',
      content,
      target,
      sender,
      timestamp: Date.now(),
      id: `text-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
    };
  }

  static createFileMessage(
    filename: string,
    fileSize: number,
    fileHash: string,
    target: { type: 'all' | 'specific'; agentIds?: string[] },
    sender: { id: string; name: string; type: 'owner' | 'agent' }
  ): FileMessage {
    return {
      type: 'file',
      filename,
      fileSize,
      fileHash,
      target,
      sender,
      timestamp: Date.now(),
      id: `file-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
    };
  }

  static createAgentSettingsQuery(
    sender: { id: string; name: string; type: 'owner' | 'agent' },
    agentIds?: string[]
  ): AgentSettingsQuery {
    return {
      type: 'agent-settings-query',
      agentIds,
      sender,
      timestamp: Date.now(),
      id: `query-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
    };
  }

  static createAgentSettingsResponse(
    agents: any[],
    sender: { id: string; name: string; type: 'owner' | 'agent' }
  ): AgentSettingsResponse {
    return {
      type: 'agent-settings-response',
      agents,
      sender,
      timestamp: Date.now(),
      id: `response-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
    };
  }
}
