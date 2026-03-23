import {
  DEFAULT_ACCOUNT_ID,
  formatPairingApproveHint,
  type ChannelPlugin,
  type ChannelStatusIssue,
  type OpenClawConfig,
} from "openclaw/plugin-sdk";

import type { GigiAccount } from "./types.js";
import { GigiClient } from "./GigiClient.js";
import { OutboundManager } from "./outbound.js";
import { getStatusSummary } from "./probe.js";
import {
  listGigiAccountIds,
  resolveGigiAccount,
} from "./accounts.js";

const CHANNEL_ID = "gigi";
const TEXT_CHUNK_LIMIT = 4000;

/**
 * Gateway context for active account
 */
interface GatewayContext {
  accountId: string;
  account: GigiAccount;
  client: GigiClient;
  outbound: OutboundManager;
  wsConnection?: WebSocket;
  running: boolean;
  lastStartAt: number | null;
  lastStopAt: number | null;
  lastError: string | null;
}

/**
 * Active gateway instances
 */
const activeGateways = new Map<string, GatewayContext>();

/**
 * Send a message through the P2P network
 */
async function sendGigiMessage({
  to,
  content,
  accountId,
}: {
  to: string;
  content: string;
  accountId?: string;
}): Promise<{ channel: string; messageId: string; chatId: string }> {
  const resolvedAccountId = accountId ?? DEFAULT_ACCOUNT_ID;
  const gateway = activeGateways.get(resolvedAccountId);
  
  if (!gateway || !gateway.client.isConnected()) {
    throw new Error(`Gateway not connected for account ${resolvedAccountId}`);
  }

  // Use outbound manager to send message (handles queuing and retries)
  await gateway.outbound.sendMessage(to, content);
  
  return {
    channel: CHANNEL_ID,
    messageId: `gigi-${Date.now()}`,
    chatId: to,
  };
}

/**
 * Gigi Channel Plugin implementation
 */
export const gigiPlugin: ChannelPlugin<GigiAccount> = {
  id: CHANNEL_ID,
  meta: {
    id: CHANNEL_ID,
    label: "Gigi P2P",
    selectionLabel: "Gigi P2P Network",
    detailLabel: "Gigi P2P Network",
    docsPath: `/channels/${CHANNEL_ID}`,
    docsLabel: CHANNEL_ID,
    blurb: "Connect to Gigi peers via P2P network",
    systemImage: "network.fill",
    quickstartAllowFrom: true,
  },
  pairing: {
    idLabel: "peerId",
    normalizeAllowEntry: (entry) => entry.replace(new RegExp(`^(${CHANNEL_ID}|peer):`, "i"), "").trim(),
    notifyApproval: async ({ cfg, id }) => {
      // Send pairing approval message
    },
  },
  capabilities: {
    chatTypes: ["direct", "group"],
    reactions: false,
    threads: false,
    media: true,
    nativeCommands: false,
    blockStreaming: false,
  },
  reload: {configPrefixes: [`channels.${CHANNEL_ID}`]},
  config: {
    // List all configured account IDs
    listAccountIds: (cfg: Record<string, any>): string[] => {
      return listGigiAccountIds(cfg);
    },

    // Resolve account configuration by ID
    resolveAccount: ({
      cfg,
      accountId,
    }: {
      cfg: Record<string, any>;
      accountId: string;
    }): GigiAccount | null => {
      return resolveGigiAccount({ cfg, accountId });
    },

    // Get default account ID
    defaultAccountId: () => DEFAULT_ACCOUNT_ID,

    // Set account enabled status
    setAccountEnabled: ({cfg, enabled}) => {
      const gigiConfig = (cfg.channels?.[CHANNEL_ID] ?? {}) as any;
      return {
        ...cfg,
        channels: {
          ...cfg.channels,
          [CHANNEL_ID]: {
            ...gigiConfig,
            enabled,
          },
        },
      };
    },

    // Delete account
    deleteAccount: ({cfg}) => {
      const gigiConfig = (cfg.channels?.[CHANNEL_ID] ?? {}) as any;
      const { peerId, multiaddrs, ...rest } = gigiConfig;
      return {
        ...cfg,
        channels: {
          ...cfg.channels,
          [CHANNEL_ID]: rest,
        },
      };
    },

    // Check if account is configured
    isConfigured: (account: GigiAccount) => {
      return Boolean(account.peerId?.trim() && account.multiaddrs?.length > 0);
    },

    // Describe account info
    describeAccount: (account: GigiAccount) => ({
      accountId: account.accountId,
      name: account.displayName || account.peerId.substring(0, 8),
      enabled: account.enabled !== false,
      configured: Boolean(account.peerId?.trim() && account.multiaddrs?.length > 0),
      peerId: account.peerId,
      multiaddrs: account.multiaddrs,
    }),

    // Resolve allow from list
    resolveAllowFrom: ({cfg}) => {
      const account = resolveGigiAccount({ cfg, accountId: DEFAULT_ACCOUNT_ID });
      return (account?.config?.allowFrom ?? []).map((entry) => String(entry));
    },

    // Format allow from list
    formatAllowFrom: ({allowFrom}) =>
      allowFrom
        .map((entry) => String(entry).trim())
        .filter(Boolean),
  },
  security: {
    resolveDmPolicy: ({account}) => {
      const basePath = `channels.${CHANNEL_ID}.`;
      return {
        policy: account.config?.dmPolicy ?? "open",
        allowFrom: account.config?.allowFrom ?? [],
        policyPath: `${basePath}dmPolicy`,
        allowFromPath: basePath,
        approveHint: formatPairingApproveHint(CHANNEL_ID),
        normalizeEntry: (raw) => raw.replace(new RegExp(`^${CHANNEL_ID}:`, "i"), "").trim(),
      };
    },
    collectWarnings: ({account, cfg}) => {
      const warnings: string[] = [];

      // DM policy warnings
      const dmPolicy = account.config?.dmPolicy ?? "open";
      if (dmPolicy === "open") {
        const hasWildcard = (account.config?.allowFrom ?? []).some(
          (entry) => String(entry).trim() === "*"
        );
        if (!hasWildcard) {
          warnings.push(
            `- Gigi P2P私信：dmPolicy="open" 但 allowFrom 未包含 "*"。任何人都可以发消息，但允许列表为空可能导致意外行为。建议设置 channels.${CHANNEL_ID}.allowFrom=["*"] 或使用 dmPolicy="pairing"。`,
          );
        }
      }

      // Group policy warnings
      const defaultGroupPolicy = cfg.channels?.defaults?.groupPolicy;
      const groupPolicy = account.config?.groupPolicy ?? defaultGroupPolicy ?? "open"
      if (groupPolicy === "open") {
        warnings.push(
          `- Gigi P2P群组：groupPolicy="open" 允许所有群组中的成员触发。设置 channels.${CHANNEL_ID}.groupPolicy="allowlist" + channels.${CHANNEL_ID}.groupAllowFrom 来限制群组。`,
        );
      }

      return warnings;
    },
  },
  messaging: {
    normalizeTarget: (target) => {
      const trimmed = target.trim();
      if (!trimmed) return undefined;
      return trimmed;
    },
    targetResolver: {
      looksLikeId: (id) => {
        const trimmed = id?.trim();
        return Boolean(trimmed);
      },
      hint: "<peerId|group:groupName>",
    },
  },
  directory: {
    self: async (ctx) => {
      const gateway = activeGateways.get(ctx.accountId || DEFAULT_ACCOUNT_ID);
      if (!gateway || !gateway.client.isConnected()) {
        return null;
      }
      return {
        id: gateway.client.getPeerId(),
        name: gateway.account.displayName || gateway.client.getPeerId().substring(0, 8),
        avatar: null,
      };
    },
    listPeers: async (ctx) => {
      const gateway = activeGateways.get(ctx.accountId || DEFAULT_ACCOUNT_ID);
      if (!gateway || !gateway.client.isConnected()) {
        return [];
      }
      return gateway.client.listPeers().map((peer) => ({
        id: peer.peerId,
        name: peer.nickname || peer.peerId.substring(0, 8),
        avatar: null,
      }));
    },
    listGroups: async (ctx) => {
      const gateway = activeGateways.get(ctx.accountId || DEFAULT_ACCOUNT_ID);
      if (!gateway || !gateway.client.isConnected()) {
        return [];
      }
      return gateway.client.listGroups().map((group) => ({
        id: `group:${group.name}`,
        name: group.name,
        avatar: null,
        memberCount: group.members?.length || 0,
      }));
    },
  },
  outbound: {
    deliveryMode: "gateway",
    chunker: (text, limit) => {
      // Simple chunking for now
      const chunks = [];
      for (let i = 0; i < text.length; i += limit) {
        chunks.push(text.substring(i, i + limit));
      }
      return chunks;
    },
    textChunkLimit: TEXT_CHUNK_LIMIT,
    sendText: async ({to, text, accountId}) => {
      return sendGigiMessage({to, content: text, accountId: accountId ?? undefined});
    },
    sendMedia: async ({to, text, mediaUrl, mediaLocalRoots, accountId}) => {
      const resolvedAccountId = accountId ?? DEFAULT_ACCOUNT_ID;
      const gateway = activeGateways.get(resolvedAccountId);
      
      if (!gateway || !gateway.client.isConnected()) {
        throw new Error(`Gateway not connected for account ${resolvedAccountId}`);
      }

      // If no mediaUrl, fallback to text
      if (!mediaUrl) {
        return sendGigiMessage({to, content: text || "", accountId: resolvedAccountId});
      }

      // For now, just send the media URL as text
      const content = text
        ? `${text}\n📎 ${mediaUrl}`
        : `📎 ${mediaUrl}`;
      
      return sendGigiMessage({to, content, accountId: resolvedAccountId});
    },
  },
  status: {
    defaultRuntime: {
      accountId: DEFAULT_ACCOUNT_ID,
      running: false,
      lastStartAt: null,
      lastStopAt: null,
      lastError: null,
    },
    collectStatusIssues: (accounts): ChannelStatusIssue[] =>
      accounts.flatMap((entry) => {
        const accountId = String(entry.accountId ?? DEFAULT_ACCOUNT_ID);
        const enabled = entry.enabled !== false;
        const configured = entry.configured === true;
        if (!enabled) {
          return [];
        }
        const issues: ChannelStatusIssue[] = [];
        if (!configured) {
          issues.push({
            channel: CHANNEL_ID,
            accountId,
            kind: "config",
            message: "Gigi P2P 未配置 peerId 或 multiaddrs",
            fix: "Run: openclaw channels add gigi --peer-id <peerId> --multiaddrs <multiaddrs>",
          });
        }
        return issues;
      }),
    buildChannelSummary: ({snapshot}) => ({
      configured: snapshot.configured ?? false,
      running: snapshot.running ?? false,
      lastStartAt: snapshot.lastStartAt ?? null,
      lastStopAt: snapshot.lastStopAt ?? null,
      lastError: snapshot.lastError ?? null,
    }),
    probeAccount: async (ctx) => {
      const gateway = activeGateways.get(ctx.accountId || DEFAULT_ACCOUNT_ID);
      if (!gateway) {
        return {ok: false, status: 503, message: "Gateway not started"};
      }
      return {ok: gateway.running, status: gateway.running ? 200 : 503};
    },
    buildAccountSnapshot: ({account, runtime}) => {
      const configured = Boolean(
        account.peerId?.trim() &&
        account.multiaddrs?.length > 0
      );
      return {
        accountId: account.accountId,
        name: account.displayName || account.peerId?.substring(0, 8),
        enabled: account.enabled !== false,
        configured,
        running: runtime?.running ?? false,
        lastStartAt: runtime?.lastStartAt ?? null,
        lastStopAt: runtime?.lastStopAt ?? null,
        lastError: runtime?.lastError ?? null,
      };
    },
  },
  gateway: {
    startAccount: async (ctx) => {
      const { accountId, account, config, onMessage, setStatus } = ctx;

      // Check if already started
      if (activeGateways.has(accountId)) {
        throw new Error(`Gateway for ${accountId} already started`);
      }

      try {
        // Create Gigi P2P client
        const client = new GigiClient({
          peerId: account.peerId,
          multiaddrs: account.multiaddrs,
          displayName: account.displayName,
          bootstrapPeers: account.bootstrapPeers,
          enableMdns: account.enableMdns,
          enableDht: account.enableDht,
          enableRelay: account.enableRelay,
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
              channel: CHANNEL_ID,
            };
            onMessage(openclawMessage);
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
          running: true,
          lastStartAt: Date.now(),
          lastStopAt: null,
          lastError: null,
        };

        // Connect to OpenClaw Gateway if configured
        const gatewayUrl = config.gatewayUrl || "ws://127.0.0.1:18789";
        if (config.autoConnect !== false) {
          const ws = new WebSocket(`${gatewayUrl}/channel/${CHANNEL_ID}/account/${accountId}`);
          
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
                await sendGigiMessage({
                  to: data.to,
                  content: data.content,
                  accountId,
                });
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

        // Update status
        if (setStatus) {
          setStatus({ running: true, lastStartAt: Date.now(), lastError: null });
        }
      } catch (error) {
        console.error(`[GigiPlugin] Error starting gateway for ${accountId}:`, error);
        if (setStatus) {
          setStatus({ running: false, lastError: error instanceof Error ? error.message : "Unknown error" });
        }
        throw error;
      }
    },
    logoutAccount: async ({cfg}) => {
      const nextCfg = {...cfg} as OpenClawConfig;
      const gigiConfig = (cfg.channels?.[CHANNEL_ID] ?? {}) as any;
      const nextGigi = {...gigiConfig};
      let cleared = false;
      let changed = false;

      if (nextGigi.peerId || nextGigi.multiaddrs) {
        delete nextGigi.peerId;
        delete nextGigi.multiaddrs;
        cleared = true;
        changed = true;
      }

      if (changed) {
        if (Object.keys(nextGigi).length > 0) {
          nextCfg.channels = {...nextCfg.channels, [CHANNEL_ID]: nextGigi};
        } else {
          const nextChannels = {...nextCfg.channels};
          delete (nextChannels as Record<string, unknown>)[CHANNEL_ID];
          if (Object.keys(nextChannels).length > 0) {
            nextCfg.channels = nextChannels;
          } else {
            delete nextCfg.channels;
          }
        }
      }

      const resolved = resolveGigiAccount({ cfg: changed ? nextCfg : cfg, accountId: DEFAULT_ACCOUNT_ID });
      const loggedOut = !resolved.peerId && !resolved.multiaddrs;

      return {cleared, envToken: false, loggedOut};
    },

    /**
     * Stop a gateway for an account
     */
    stopAccount: async (ctx) => {
      const { accountId, setStatus } = ctx;
      const gateway = activeGateways.get(accountId);
      
      if (!gateway) {
        throw new Error(`No active gateway for ${accountId}`);
      }

      try {
        // Close WebSocket connection
        if (gateway.wsConnection) {
          gateway.wsConnection.close();
        }

        // Clear outbound messages
        gateway.outbound.clear();

        // Stop P2P client
        await gateway.client.stop();

        // Remove from active gateways
        activeGateways.delete(accountId);

        // Update status
        if (setStatus) {
          setStatus({ running: false, lastStopAt: Date.now(), lastError: null });
        }

        console.log(`[GigiPlugin] Stopped gateway for ${accountId}`);
      } catch (error) {
        console.error(`[GigiPlugin] Error stopping gateway for ${accountId}:`, error);
        if (setStatus) {
          setStatus({ running: false, lastError: error instanceof Error ? error.message : "Unknown error" });
        }
        throw error;
      }
    },
  },
};

// Export plugin type
export type GigiPlugin = typeof gigiPlugin;
