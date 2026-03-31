// Message types for Agent Messaging Protocol

export type MessageType = 'text' | 'file' | 'agent-settings-query' | 'agent-settings-response';

export interface SenderInfo {
  id: string;
  name: string;
  type: 'owner' | 'agent';
}

export interface TargetInfo {
  type: 'all' | 'specific';
  agentIds?: string[];
}

export interface TextMessage {
  type: 'text';
  content: string;
  target: TargetInfo;
  sender: SenderInfo;
  timestamp: number;
  id: string;
}

export interface FileMessage {
  type: 'file';
  filename: string;
  fileSize: number;
  fileHash: string;
  target: TargetInfo;
  sender: SenderInfo;
  timestamp: number;
  id: string;
}

export interface AgentSettingsQuery {
  type: 'agent-settings-query';
  agentIds?: string[];
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

export type AmpMessage = TextMessage | FileMessage | AgentSettingsQuery | AgentSettingsResponse;

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
