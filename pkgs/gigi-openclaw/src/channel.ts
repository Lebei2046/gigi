import type { GigiAccount } from "./types.js";
import { GigiClient } from "./GigiClient.js";
import { OutboundManager } from "./outbound.js";
import { getStatusSummary } from "./probe.js";
import {
  listGigiAccountIds,
  resolveGigiAccount,
} from "./accounts.js";

/**
 * Gateway context for active account
 */
interface GatewayContext {
  accountId: string;
  account: GigiAccount;
  client: GigiClient;
  outbound: OutboundManager;
  wsConnection?: WebSocket;
}

/**
 * Active gateway instances
 */
const activeGateways = new Map<string, GatewayContext>();

/**
 * Gigi Channel Plugin implementation
 */
export const gigiPlugin = {
  // Plugin metadata
  meta: {
    id: "gigi",
    label: "Gigi P2P",
    selectionLabel: "Gigi P2P Network",
    blurb: "Connect to Gigi peers via P2P network",
    aliases: ["p2p"],
    order: 100,
  },

  // Configuration management
  config: {
    /**
     * List all configured account IDs
     */
    listAccountIds: (cfg: Record<string, any>): string[] => {
      return listGigiAccountIds(cfg);
    },

    /**
     * Resolve account configuration by ID
     */
    resolveAccount: ({
      cfg,
      accountId,
    }: {
      cfg: Record<string, any>;
      accountId: string;
    }): GigiAccount | null => {
      return resolveGigiAccount({ cfg, accountId });
    },

    /**
     * Validate account configuration
     */
    validateAccount: (accountId: string, accountConfig: any): { valid: boolean; error?: string } => {
      if (!accountConfig.peerId || typeof accountConfig.peerId !== "string") {
        return { valid: false, error: "Invalid peer ID" };
      }
      
      if (!Array.isArray(accountConfig.multiaddrs)) {
        return { valid: false, error: "multiaddrs must be an array" };
      }

      return { valid: true };
    },

    /**
     * Schema for account configuration
     */
    accountSchema: {
      type: "object",
      properties: {
        peerId: { type: "string" },
        multiaddrs: { type: "array", items: { type: "string" } },
        displayName: { type: "string" },
      },
      required: ["peerId", "multiaddrs"],
    },
  },

  // Status and health monitoring
  status: {
    /**
     * Check status of an account
     */
    async checkStatus(accountId: string): Promise<{
      status: "connected" | "disconnected" | "error";
      message: string;
      details?: Record<string, any>;
    }> {
      const gateway = activeGateways.get(accountId);
      
      if (!gateway) {
        return {
          status: "disconnected",
          message: "Gateway not started",
        };
      }

      return getStatusSummary(gateway.client);
    },

    /**
     * List all active gateways
     */
    listActiveGateways(): string[] {
      return Array.from(activeGateways.keys());
    },
  },

  // Gateway management
  gateway: {
    /**
     * Start a gateway for an account
     */
    async startAccount(ctx: {
      accountId: string;
      account: GigiAccount;
      config: {
        gatewayUrl?: string;
        token?: string;
        autoConnect?: boolean;
      };
      onMessage?: (msg: any) => void;
    }): Promise<void> {
      const { accountId, account, config, onMessage } = ctx;

      // Check if already started
      if (activeGateways.has(accountId)) {
        throw new Error(`Gateway for ${accountId} already started`);
      }

      // Create Gigi P2P client
      const client = new GigiClient({
        peerId: account.peerId,
        multiaddrs: account.multiaddrs,
      });

      // Create outbound manager
      const outbound = new OutboundManager(client);

      // Set up message handler
      client.onMessage(async (gigiMessage) => {
        console.log(`[GigiPlugin] Received message from ${gigiMessage.from}:`, gigiMessage.content);
        
        if (onMessage) {
          // Convert Gigi message to OpenClaw format
          const openclawMessage = {
            from: gigiMessage.from,
            to: gigiMessage.to,
            content: gigiMessage.content,
            timestamp: gigiMessage.timestamp,
            channel: "gigi",
          };
          onMessage(openclawMessage);
        }

        // If WebSocket connection is active, forward message
        const gateway = activeGateways.get(accountId);
        if (gateway?.wsConnection?.readyState === WebSocket.OPEN) {
          gateway.wsConnection.send(JSON.stringify(openclawMessage));
        }
      });

      // Start P2P client
      await client.start();

      // Create gateway context
      const gatewayContext: GatewayContext = {
        accountId,
        account,
        client,
        outbound,
      };

      // Connect to OpenClaw Gateway if configured
      const gatewayUrl = config.gatewayUrl || "ws://127.0.0.1:18789";
      if (config.autoConnect !== false) {
        const ws = new WebSocket(`${gatewayUrl}/channel/gigi/account/${accountId}`);
        
        ws.onopen = () => {
          console.log(`[GigiPlugin] Connected to OpenClaw Gateway: ${gatewayUrl}`);
          // Send auth token if provided
          if (config.token) {
            ws.send(JSON.stringify({ type: "auth", token: config.token }));
          }
          // Send peer info
          ws.send(JSON.stringify({
            type: "peer-info",
            peerId: account.peerId,
            multiaddrs: account.multiaddrs,
          }));
        };

        ws.onmessage = async (event) => {
          try {
            const data = JSON.parse(event.data);
            
            if (data.type === "send-message") {
              // Send message via P2P
              await outbound.sendMessage(data.to, data.content);
            }
          } catch (error) {
            console.error("[GigiPlugin] Error processing gateway message:", error);
          }
        };

        ws.onerror = (error) => {
          console.error("[GigiPlugin] Gateway WebSocket error:", error);
        };

        ws.onclose = () => {
          console.log("[GigiPlugin] Gateway connection closed");
        };

        gatewayContext.wsConnection = ws;
      }

      // Store active gateway
      activeGateways.set(accountId, gatewayContext);
    },

    /**
     * Stop a gateway for an account
     */
    async stopAccount(accountId: string): Promise<void> {
      const gateway = activeGateways.get(accountId);
      
      if (!gateway) {
        throw new Error(`No active gateway for ${accountId}`);
      }

      // Close WebSocket connection
      if (gateway.wsConnection) {
        gateway.wsConnection.close();
      }

      // Stop outbound manager
      gateway.outbound.clear();

      // Stop P2P client
      await gateway.client.stop();

      // Remove from active gateways
      activeGateways.delete(accountId);

      console.log(`[GigiPlugin] Stopped gateway for ${accountId}`);
    },

    /**
     * Send a message through the gateway
     */
    async sendMessage(ctx: {
      accountId: string;
      to: string;
      content: string;
    }): Promise<void> {
      const gateway = activeGateways.get(ctx.accountId);
      
      if (!gateway) {
        throw new Error(`No active gateway for ${ctx.accountId}`);
      }

      await gateway.outbound.sendMessage(ctx.to, ctx.content);
    },
  },
} as const;

// Export plugin type
export type GigiPlugin = typeof gigiPlugin;
