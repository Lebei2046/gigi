// Message types for Agent Messaging Protocol
import { SenderInfo, TextMessage, FileMessage } from '@gigi/message-types';

export type MessageType =
  | 'text'
  | 'file'
  | 'agent-settings-query'
  | 'agent-settings-response'
  | 'session-create'
  | 'session-delete'
  | 'session-switch'
  | 'session-list'
  | 'session-response';

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

// Session management messages
export interface SessionCreate {
  type: 'session-create';
  label: string;
  target: { type: 'specific'; agentIds: string[] };
  sender: SenderInfo;
  timestamp: number;
  id: string;
}

export interface SessionDelete {
  type: 'session-delete';
  sessionKey: string;
  target: { type: 'specific'; agentIds: string[] };
  sender: SenderInfo;
  timestamp: number;
  id: string;
}

export interface SessionSwitch {
  type: 'session-switch';
  sessionKey: string;
  target: { type: 'specific'; agentIds: string[] };
  sender: SenderInfo;
  timestamp: number;
  id: string;
}

export interface SessionList {
  type: 'session-list';
  includeGlobal?: boolean;
  target: { type: 'specific'; agentIds: string[] };
  sender: SenderInfo;
  timestamp: number;
  id: string;
}

export interface SessionResponse {
  type: 'session-response';
  sessionKey?: string;
  label?: string;
  sessions?: Array<{
    key: string;
    displayName?: string;
    updatedAt?: number;
  }>;
  error?: string;
  sender: SenderInfo;
  timestamp: number;
  id: string;
}

export type AmpMessage =
  | TextMessage
  | FileMessage
  | AgentSettingsQuery
  | AgentSettingsResponse
  | SessionCreate
  | SessionDelete
  | SessionSwitch
  | SessionList
  | SessionResponse;

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
