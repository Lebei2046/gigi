// Message types for Agent Messaging Protocol
import { SenderInfo, TextMessage, FileMessage } from '@gigi/message-types';

export type MessageType =
  | 'text'
  | 'file'
  | 'agent-settings-query'
  | 'agent-settings-response';

export interface AgentSettingsQuery {
  type: 'agent-settings-query';
  agentIds?: string[];
  nodeId?: string; // Optional node ID for node-level queries
  sender: SenderInfo;
  timestamp: number;
  id: string;
}

export interface AgentSetting {
  id: string;
  name: string;
  type: string;
  value: any;
  description?: string;
}

export interface AgentInfo {
  id: string;
  name: string;
  type: string;
  version: string;
  settings: AgentSetting[];
  status: 'online' | 'offline' | 'busy';
}

export interface AgentSettingsResponse {
  type: 'agent-settings-response';
  agents: AgentInfo[];
  sender: SenderInfo;
  timestamp: number;
  id: string;
}

export type AmpMessage =
  | TextMessage
  | FileMessage
  | AgentSettingsQuery
  | AgentSettingsResponse;

export interface AgentRegistry {
  getAgentById(id: string): AgentInfo | undefined;
  getAllAgents(): AgentInfo[];
  updateAgentStatus(id: string, status: 'online' | 'offline' | 'busy'): void;
}

export interface MessageRouter {
  routeMessage(message: AmpMessage): void;
  registerAgent(agent: AgentInfo): void;
  unregisterAgent(agentId: string): void;
}

// Re-export shared types
export { SenderInfo, TextMessage, FileMessage } from '@gigi/message-types';
