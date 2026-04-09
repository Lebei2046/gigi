// Shared message types for Gigi P2P ecosystem

// Sender information
export interface SenderInfo {
  id: string;
  name: string;
  type: 'owner' | 'agent' | 'node';
  nodeId?: string; // Optional node ID for agent senders
}

// Target information for message routing
export interface TargetInfo {
  type: 'all' | 'specific' | 'node' | 'node-agent';
  agentIds?: string[];
  nodeId?: string; // Required for node and node-agent types
}

// Text message
export interface TextMessage {
  type: 'text';
  content: string;
  target: TargetInfo;
  sender: SenderInfo;
  timestamp: number;
  id: string;
}

// File message
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

// Group share message
export interface GroupShareMessage {
  type: 'shareGroup';
  groupId: string;
  groupName: string;
  inviterNickname: string;
  target: TargetInfo;
  sender: SenderInfo;
  timestamp: number;
  id: string;
}

// File share message (for P2P file sharing)
export interface FileShareMessage {
  type: 'fileShare';
  shareCode: string;
  filename: string;
  fileSize: number;
  fileType: string;
  target: TargetInfo;
  sender: SenderInfo;
  timestamp: number;
  id: string;
}

// Union of all message types
export type GigiMessage = 
  | TextMessage
  | FileMessage
  | GroupShareMessage
  | FileShareMessage;

// Message content types (for P2P client)
export type MessageContent = 
  | { type: 'text'; text: string; fromPeerId: string; fromNickname: string }
  | { type: 'fileShare'; shareCode: string; filename: string; fileSize: number; fileType: string; fromPeerId: string; fromNickname: string }
  | { type: 'shareGroup'; groupId: string; groupName: string; inviterNickname: string; fromPeerId: string; fromNickname: string };

// Input type for sending messages (without auto-populated fields)
export type MessageContentInput = 
  | { type: 'text'; text: string }
  | { type: 'fileShare'; shareCode: string; filename: string; fileSize: number; fileType: string }
  | { type: 'shareGroup'; groupId: string; groupName: string; inviterNickname: string };

// Type guards
export function isTextMessage(message: any): message is TextMessage {
  return message && message.type === 'text' && typeof message.content === 'string';
}

export function isFileMessage(message: any): message is FileMessage {
  return message && message.type === 'file' && typeof message.filename === 'string';
}

export function isGroupShareMessage(message: any): message is GroupShareMessage {
  return message && message.type === 'shareGroup' && typeof message.groupId === 'string';
}

export function isFileShareMessage(message: any): message is FileShareMessage {
  return message && message.type === 'fileShare' && typeof message.shareCode === 'string';
}
