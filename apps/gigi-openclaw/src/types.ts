import { Type, Static } from '@sinclair/typebox';
import type {
  TextMessage,
  FileMessage,
  AgentSettingsQuery,
  AgentSettingsResponse,
  SenderInfo,
  AgentInfo,
} from '@gigi/amp';

export type {
  TextMessage,
  FileMessage,
  AgentSettingsQuery,
  AgentSettingsResponse,
  SenderInfo,
  AgentInfo,
};

/**
 * Gigi account configuration schema
 */
export const GigiAccountConfigSchema = Type.Object({
  accountId: Type.String(),
  displayName: Type.Optional(Type.String()),
  peerId: Type.String(),
  multiaddrs: Type.Array(Type.String()),
  bootstrapPeers: Type.Optional(Type.Array(Type.String())),
  enableMdns: Type.Optional(Type.Boolean()),
  enableDht: Type.Optional(Type.Boolean()),
  enableRelay: Type.Optional(Type.Boolean()),
});

export type GigiAccountConfig = Static<typeof GigiAccountConfigSchema>;

/**
 * Gigi message format - using AMP types
 */
export type GigiMessage =
  | TextMessage
  | FileMessage
  | AgentSettingsQuery
  | AgentSettingsResponse;

/**
 * Gigi client configuration
 */
export interface GigiClientConfig {
  peerId: string;
  multiaddrs: string[];
  displayName?: string;
  bootstrapPeers?: string[];
  enableMdns?: boolean;
  enableDht?: boolean;
  enableRelay?: boolean;
  mnemonic?: string;
}

/**
 * Gigi client interface
 */
export interface IGigiClient {
  start(): Promise<void>;
  stop(): Promise<void>;
  sendMessage(target: string, message: string): Promise<void>;
  sendFileMessage(
    target: string,
    filename: string,
    fileSize: number,
    fileType: string,
    shareCode: string
  ): Promise<void>;
  sendGroupMessage(groupName: string, content: string): Promise<void>;
  sendGroupFileMessage(
    groupName: string,
    filename: string,
    fileSize: number,
    fileType: string,
    shareCode: string
  ): Promise<void>;
  joinGroup(groupName: string): Promise<void>;
  leaveGroup(groupName: string): Promise<void>;
  shareFile(filePath: string): Promise<string>;
  downloadFile(peerId: string, shareCode: string): Promise<string>;
  onMessage(handler: (msg: GigiMessage) => void): void;
  getPeerId(): string;
  getMultiaddrs(): string[];
  isConnected(): boolean;
  listPeers(): any[];
  listGroups(): any[];
  getFileByShareCode(shareCode: string): any;
}

/**
 * Gateway connection configuration
 */
export interface GatewayConfig {
  url: string; // ws://127.0.0.1:18789
  token?: string;
  channelId: string;
  accountId: string;
}

/**
 * Account info for OpenClaw
 */
export interface GigiAccount {
  accountId: string;
  displayName?: string;
  peerId: string;
  multiaddrs: string[];
  bootstrapPeers?: string[];
  enableMdns?: boolean;
  enableDht?: boolean;
  enableRelay?: boolean;
  config?: Record<string, any>;
  enabled?: boolean;
  mnemonic?: string;
  group?: string; // Group name for agent communication
  accounts?: Record<string, string>; // Map of agent IDs to bot names
}
