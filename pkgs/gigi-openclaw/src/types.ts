import { Type, Static } from "@sinclair/typebox";

/**
 * Gigi account configuration schema
 */
export const GigiAccountConfigSchema = Type.Object({
  accountId: Type.String(),
  displayName: Type.Optional(Type.String()),
  peerId: Type.String(),
  multiaddrs: Type.Array(Type.String()),
});

export type GigiAccountConfig = Static<typeof GigiAccountConfigSchema>;

/**
 * Gigi message format
 */
export interface GigiMessage {
  from: string; // peerId
  to: string; // peerId
  content: string;
  timestamp: number;
  type: "direct" | "broadcast";
}

/**
 * Gigi client configuration
 */
export interface GigiClientConfig {
  peerId: string;
  multiaddrs: string[];
  bootstrapPeers?: string[];
  enableMdns?: boolean;
  enableDht?: boolean;
}

/**
 * Gigi client interface
 */
export interface IGigiClient {
  start(): Promise<void>;
  stop(): Promise<void>;
  sendMessage(targetPeerId: string, message: string): Promise<void>;
  onMessage(handler: (msg: GigiMessage) => void): void;
  getPeerId(): string;
  getMultiaddrs(): string[];
  isConnected(): boolean;
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
}
