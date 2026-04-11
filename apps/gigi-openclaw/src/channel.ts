import {
  type ChannelPlugin,
  type ChannelStatusIssue,
  type OpenClawConfig,
} from 'openclaw/plugin-sdk';
import { DEFAULT_ACCOUNT_ID } from 'openclaw/plugin-sdk/account-id';

import type { PluginRuntime } from 'openclaw/plugin-sdk';
import { createLogger } from '@gigi/logging';
import { generateMnemonic } from '@gigi/p2p';

const logger = createLogger({ name: 'gigi-plugin' });

// Runtime store for the Gigi plugin
let runtime: PluginRuntime | null = null;

/**
 * Set the plugin runtime
 */
export function setGigiRuntime(nextRuntime: PluginRuntime): void {
  runtime = nextRuntime;
}

/**
 * Get the plugin runtime
 */
export function getGigiRuntime(): PluginRuntime {
  if (!runtime) {
    throw new Error(
      'Gigi plugin runtime has not been initialised. Ensure setGigiRuntime() is called during plugin activation.'
    );
  }
  return runtime;
}

/**
 * Format pairing approval hint message
 */
function formatPairingApproveHint(channelId: string): string {
  const listCmd = `openclaw pairing list ${channelId}`;
  const approveCmd = `openclaw pairing approve ${channelId} <code>`;
  return `To approve: 1) Run \`${listCmd}\` to get code, 2) Run \`${approveCmd}\``;
}

import type { GigiAccount } from './types.js';
import { GigiClient } from './GigiClient.js';
import { OutboundManager } from './outbound.js';
import { listGigiAccountIds, resolveGigiAccount } from './accounts.js';
import { TextMessage, FileMessage, AmpMessageFactory } from '@gigi/amp';

const CHANNEL_ID = 'gigi-openclaw';
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
  cfg,
  agentId,
}: {
  to: string;
  content: string;
  accountId?: string;
  cfg: Record<string, any>;
  agentId?: string;
}): Promise<{ channel: string; messageId: string; chatId: string }> {
  const resolvedAccountId = accountId ?? DEFAULT_ACCOUNT_ID;
  let gateway = activeGateways.get(resolvedAccountId);

  if (!gateway || !gateway.client.isConnected()) {
    // Gateway not started or not connected, try to start it
    logger.info('Gateway not connected, starting it');

    // Get the account configuration
    const account = resolveGigiAccount({ cfg, accountId: resolvedAccountId });

    if (!account) {
      throw new Error(
        `Account configuration not found for ${resolvedAccountId}`
      );
    }

    // Create Gigi P2P client
    const client = new GigiClient({
      multiaddrs: account.multiaddrs,
      displayName: account.displayName,
      nickname: account.nickname,
      mnemonic: account.mnemonic,
      bootstrapPeers: account.bootstrapPeers,
      enableMdns: account.enableMdns,
      enableDht: account.enableDht,
      enableRelay: account.enableRelay,
    });

    // Create outbound manager
    const outbound = new OutboundManager(client);

    // Set up message handler
    client.onMessage(async (gigiMessage) => {
      logger.debug('Received message');

      // Handle different message types
      if ('type' in gigiMessage) {
        const msg = gigiMessage as { type: string };
        switch (msg.type) {
          case 'text': {
            logger.debug('Received text message');
            break;
          }
          case 'file': {
            logger.debug('Received file message');
            // Note: File sharing functionality would use the file hash or another identifier to retrieve the file
            break;
          }
          case 'agent-settings-query': {
            logger.debug('Received agent settings query');

            // Ensure gateway is defined
            if (!gateway) {
              logger.error('Gateway is not available to send response');
              break;
            }

            // Create and send agent settings response
            // Get OpenClaw agent information from config if available
            const openclawAgents = [];
            if (gateway.account.config && gateway.account.config.agents) {
              for (const [agentId, agentConfig] of Object.entries(
                gateway.account.config.agents
              )) {
                const agent = agentConfig as any;
                openclawAgents.push({
                  id: agentId,
                  name: agent.name || agentId,
                  model: agent.model || 'unknown',
                  status: 'active',
                });
              }
            }

            // Get peer ID from the client (it's derived from mnemonic)
            const peerId = gateway.client.getPeerId();
            const displayName =
              gateway.account.displayName || peerId.substring(0, 8);

            const responseMessage =
              AmpMessageFactory.createAgentSettingsResponse(
                [
                  {
                    id: peerId,
                    name: displayName,
                    type: 'openclaw-agent',
                    version: '1.0.0',
                    settings: [
                      {
                        id: 'enabled',
                        name: 'Enabled',
                        type: 'boolean',
                        value: true,
                      },
                      {
                        id: 'displayName',
                        name: 'Display Name',
                        type: 'string',
                        value: displayName,
                      },
                      {
                        id: 'peerId',
                        name: 'Peer ID',
                        type: 'string',
                        value: peerId,
                      },
                      {
                        id: 'multiaddrs',
                        name: 'Multiaddrs',
                        type: 'array',
                        value: gateway.account.multiaddrs || [],
                      },
                    ],
                    status: 'online',
                    openclawAgents,
                  },
                ],
                {
                  id: peerId,
                  name: displayName,
                  type: 'agent',
                }
              );
            // Send response back to the sender
            try {
              await gateway.client.sendGroupMessage(
                'gigi-agents',
                JSON.stringify(responseMessage)
              );
              logger.info('Sent agent settings response');
            } catch {
              logger.error('Error sending agent settings response');
            }
            break;
          }
          case 'agent-settings-response': {
            logger.debug('Received agent settings response');
            break;
          }
          default:
            logger.debug('Received unknown message type');
        }
      }
    });

    // Start P2P client
    logger.info('Starting P2P client');
    await client.start();
    logger.info('P2P client started');

    // Join the agent group
    try {
      await client.joinGroup('gigi-agents');
      logger.info('Joined agent group');
    } catch {
      logger.warn('Failed to join agent group');
    }

    // Create gateway context
    gateway = {
      accountId: resolvedAccountId,
      account,
      client,
      outbound,
      running: true,
      lastStartAt: Date.now(),
      lastStopAt: null,
      lastError: null,
    };

    // Store active gateway
    activeGateways.set(resolvedAccountId, gateway);
  }

  // Construct AMP message with proper sender information
  let senderName =
    gateway.account.displayName || gateway.client.getPeerId().substring(0, 8);
  let senderId = gateway.client.getPeerId();

  // Use accounts mapping if agentId is provided
  if (agentId && gateway.account.accounts) {
    const botName = gateway.account.accounts[agentId];
    if (botName) {
      senderName = botName;
      senderId = agentId;
    }
  }

  // Create AMP text message
  const targetType = to.startsWith('group:') ? 'all' : 'specific';
  const target =
    targetType === 'all'
      ? { type: 'all' as const }
      : { type: 'specific' as const, agentIds: [to] };

  const ampMessage = AmpMessageFactory.createTextMessage(content, target, {
    id: senderId,
    name: senderName,
    type: 'agent',
  });

  // Use outbound manager to send message (handles queuing and retries)
  await gateway.outbound.sendMessage(to, JSON.stringify(ampMessage));

  return {
    channel: CHANNEL_ID,
    messageId: `gigi-${Date.now()}`,
    chatId: to,
  };
}

/**
 * Join a Gigi P2P group
 */
async function joinGigiGroup({
  groupName,
  accountId,
}: {
  groupName: string;
  accountId?: string;
}): Promise<void> {
  const resolvedAccountId = accountId ?? DEFAULT_ACCOUNT_ID;
  const gateway = activeGateways.get(resolvedAccountId);

  if (!gateway || !gateway.client.isConnected()) {
    throw new Error(`Gateway not connected for account ${resolvedAccountId}`);
  }

  await gateway.client.joinGroup(groupName);
  logger.info('Joined group');
}

/**
 * Share a file via Gigi P2P
 */
async function shareGigiFile({
  filePath,
  accountId,
}: {
  filePath: string;
  accountId?: string;
}): Promise<string> {
  const resolvedAccountId = accountId ?? DEFAULT_ACCOUNT_ID;
  const gateway = activeGateways.get(resolvedAccountId);

  if (!gateway || !gateway.client.isConnected()) {
    throw new Error(`Gateway not connected for account ${resolvedAccountId}`);
  }

  const shareCode = await gateway.client.shareFile(filePath);
  logger.info('Shared file');
  return shareCode;
}

/**
 * Download a file via Gigi P2P
 */
async function downloadGigiFile({
  peerId,
  shareCode,
  accountId,
}: {
  peerId: string;
  shareCode: string;
  accountId?: string;
}): Promise<string> {
  const resolvedAccountId = accountId ?? DEFAULT_ACCOUNT_ID;
  const gateway = activeGateways.get(resolvedAccountId);

  if (!gateway || !gateway.client.isConnected()) {
    throw new Error(`Gateway not connected for account ${resolvedAccountId}`);
  }

  const downloadId = await gateway.client.downloadFile(peerId, shareCode);
  logger.info('Started download');
  return downloadId;
}

/**
 * Leave a Gigi P2P group
 */
async function leaveGigiGroup({
  groupName,
  accountId,
}: {
  groupName: string;
  accountId?: string;
}): Promise<void> {
  const resolvedAccountId = accountId ?? DEFAULT_ACCOUNT_ID;
  const gateway = activeGateways.get(resolvedAccountId);

  if (!gateway || !gateway.client.isConnected()) {
    throw new Error(`Gateway not connected for account ${resolvedAccountId}`);
  }

  await gateway.client.leaveGroup(groupName);
  logger.info('Left group');
}

/**
 * List joined Gigi P2P groups
 */
async function listGigiGroups({
  accountId,
}: {
  accountId?: string;
}): Promise<Array<{ id: string; name: string; memberCount: number }>> {
  const resolvedAccountId = accountId ?? DEFAULT_ACCOUNT_ID;
  const gateway = activeGateways.get(resolvedAccountId);

  if (!gateway || !gateway.client.isConnected()) {
    return [];
  }

  return gateway.client.listGroups().map((group: any) => ({
    id: `group:${group.name}`,
    name: group.name,
    memberCount: group.members?.length || 0,
  }));
}

/**
 * Gigi Channel Plugin implementation
 */
export const gigiPlugin: ChannelPlugin<GigiAccount> = {
  id: CHANNEL_ID,
  meta: {
    id: CHANNEL_ID,
    label: 'Gigi P2P',
    selectionLabel: 'Gigi P2P Network',
    detailLabel: 'Gigi P2P Network',
    docsPath: `/channels/${CHANNEL_ID}`,
    docsLabel: CHANNEL_ID,
    blurb: 'Connect to Gigi peers via P2P network',
    systemImage: 'network.fill',
    quickstartAllowFrom: true,
  },
  pairing: {
    idLabel: 'peerId',
    normalizeAllowEntry: (entry) =>
      entry.replace(new RegExp(`^(${CHANNEL_ID}|peer):`, 'i'), '').trim(),
    notifyApproval: async () => {
      // Send pairing approval message
    },
  },
  setup: {
    resolveAccountId: ({ accountId: _accountId }) =>
      _accountId || DEFAULT_ACCOUNT_ID,
    applyAccountName: ({ cfg, accountId: _accountId, name }) => {
      const gigiConfig = (cfg.channels?.[CHANNEL_ID] ?? {}) as any;
      return {
        ...cfg,
        channels: {
          ...cfg.channels,
          [CHANNEL_ID]: {
            ...gigiConfig,
            displayName: name,
          },
        },
      };
    },
    validateInput: () => {
      // No validation needed for Gigi P2P
      return null;
    },
    applyAccountConfig: ({ cfg, accountId: _accountId, input }) => {
      // Generate BIP-39 mnemonic
      const mnemonic = generateMnemonic();

      // Get display name from input or use default
      const displayName = input.name || 'My Gigi Node';
      const nickname = (input as any).nickname || displayName;

      // Apply configuration
      const gigiConfig = (cfg.channels?.[CHANNEL_ID] ?? {}) as any;
      return {
        ...cfg,
        channels: {
          ...cfg.channels,
          [CHANNEL_ID]: {
            ...gigiConfig,
            mnemonic: mnemonic,
            multiaddrs: ['/ip4/0.0.0.0/tcp/0', '/ip4/0.0.0.0/tcp/0/ws'],
            displayName: displayName,
            nickname: nickname,
            enabled: true,
          },
        },
      };
    },
  },
  setupWizard: {
    channel: CHANNEL_ID,
    status: {
      configuredLabel: 'Gigi P2P',
      unconfiguredLabel: 'Gigi P2P',
      resolveStatusLines: async ({ configured }) => {
        if (!configured) {
          return ['Not configured'];
        }
        return ['Configured'];
      },
      resolveConfigured: async ({ cfg }) => {
        const account = resolveGigiAccount({
          cfg,
          accountId: DEFAULT_ACCOUNT_ID,
        });
        return Boolean(
          (account?.peerId?.trim() || account?.mnemonic?.trim()) &&
          account?.multiaddrs?.length > 0
        );
      },
    },
    introNote: {
      title: 'Gigi P2P Setup',
      lines: [
        'Gigi P2P is a decentralized network for direct communication between peers.',
        'During setup, a temporary peer ID will be generated.',
        'After setup, you should generate a proper key pair using the provided script.',
      ],
    },
    credentials: [],
    textInputs: [
      {
        inputKey: 'name',
        message: 'Enter a display name for your Gigi node',
        placeholder: 'My Gigi Node',
        required: false,
      },
      {
        inputKey: 'nickname' as any,
        message: 'Enter a nickname for your Gigi node',
        placeholder: 'My Gigi Node',
        required: false,
      },
    ],
    finalize: async ({ cfg, accountId: _accountId, credentialValues }) => {
      // Apply the configuration
      const name = credentialValues.name?.trim() || 'My Gigi Node';
      const nickname = credentialValues.nickname?.trim() || name;

      // Generate BIP-39 mnemonic
      const mnemonic = generateMnemonic();

      // Apply configuration
      const gigiConfig = (cfg.channels?.[CHANNEL_ID] ?? {}) as any;
      const next = {
        ...cfg,
        channels: {
          ...cfg.channels,
          [CHANNEL_ID]: {
            ...gigiConfig,
            mnemonic: mnemonic,
            multiaddrs: ['/ip4/0.0.0.0/tcp/0', '/ip4/0.0.0.0/tcp/0/ws'],
            displayName: name,
            nickname: nickname,
            enabled: true,
          },
        },
      };

      return { cfg: next };
    },
  },
  capabilities: {
    chatTypes: ['direct', 'group'],
    reactions: false,
    threads: false,
    media: true,
    nativeCommands: false,
    blockStreaming: false,
  },
  reload: { configPrefixes: [`channels.${CHANNEL_ID}`] },
  config: {
    // List all configured account IDs
    listAccountIds: (cfg: Record<string, any>): string[] => {
      return listGigiAccountIds(cfg);
    },

    // Resolve account configuration by ID
    resolveAccount: (cfg: any, accountId?: string | null): GigiAccount => {
      const resolved = resolveGigiAccount({
        cfg,
        accountId: accountId || DEFAULT_ACCOUNT_ID,
      });
      return (
        resolved || {
          accountId: accountId || DEFAULT_ACCOUNT_ID,
          peerId: '',
          multiaddrs: [],
        }
      );
    },

    // Get default account ID
    defaultAccountId: () => DEFAULT_ACCOUNT_ID,

    // Set account enabled status
    setAccountEnabled: ({ cfg, enabled }) => {
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
    deleteAccount: ({ cfg }) => {
      return {
        ...cfg,
        channels: {
          ...cfg.channels,
          [CHANNEL_ID]: {},
        },
      };
    },

    // Check if account is configured
    isConfigured: (account: GigiAccount) => {
      return Boolean(
        (account.peerId?.trim() || account.mnemonic?.trim()) &&
        account.multiaddrs?.length > 0
      );
    },

    // Describe account info
    describeAccount: (account: GigiAccount) => ({
      accountId: account.accountId,
      name:
        account.displayName ||
        account.peerId?.substring(0, 8) ||
        account.accountId,
      nickname:
        account.nickname ||
        account.displayName ||
        account.peerId?.substring(0, 8) ||
        account.accountId,
      enabled: account.enabled !== false,
      configured: Boolean(
        account.peerId?.trim() && account.multiaddrs?.length > 0
      ),
      peerId: account.peerId,
      multiaddrs: account.multiaddrs,
    }),

    // Resolve allow from list
    resolveAllowFrom: ({ cfg }) => {
      const account = resolveGigiAccount({
        cfg,
        accountId: DEFAULT_ACCOUNT_ID,
      });
      return (account?.config?.allowFrom ?? []).map((entry: any) =>
        String(entry)
      );
    },

    // Format allow from list
    formatAllowFrom: ({ allowFrom }) =>
      allowFrom.map((entry: any) => String(entry).trim()).filter(Boolean),
  },
  security: {
    resolveDmPolicy: ({ account }) => {
      const basePath = `channels.${CHANNEL_ID}.`;
      return {
        policy: account.config?.dmPolicy ?? 'open',
        allowFrom: account.config?.allowFrom ?? [],
        policyPath: `${basePath}dmPolicy`,
        allowFromPath: basePath,
        approveHint: formatPairingApproveHint(CHANNEL_ID),
        normalizeEntry: (raw) =>
          raw.replace(new RegExp(`^${CHANNEL_ID}:`, 'i'), '').trim(),
      };
    },
    collectWarnings: ({ account, cfg }) => {
      const warnings: string[] = [];

      // DM policy warnings
      const dmPolicy = account.config?.dmPolicy ?? 'open';
      if (dmPolicy === 'open') {
        const hasWildcard = (account.config?.allowFrom ?? []).some(
          (entry: any) => String(entry).trim() === '*'
        );
        if (!hasWildcard) {
          warnings.push(
            `- Gigi P2P私信：dmPolicy="open" 但 allowFrom 未包含 "*"。任何人都可以发消息，但允许列表为空可能导致意外行为。建议设置 channels.${CHANNEL_ID}.allowFrom=["*"] 或使用 dmPolicy="pairing"。`
          );
        }
      }

      // Group policy warnings
      const defaultGroupPolicy = cfg.channels?.defaults?.groupPolicy;
      const groupPolicy =
        account.config?.groupPolicy ?? defaultGroupPolicy ?? 'open';
      if (groupPolicy === 'open') {
        warnings.push(
          `- Gigi P2P群组：groupPolicy="open" 允许所有群组中的成员触发。设置 channels.${CHANNEL_ID}.groupPolicy="allowlist" + channels.${CHANNEL_ID}.groupAllowFrom 来限制群组。`
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
      hint: '<peerId|group:groupName>',
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
        kind: 'user' as const,
        name:
          gateway.account.displayName ||
          gateway.client.getPeerId().substring(0, 8),
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
        kind: 'user' as const,
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
        kind: 'group' as const,
        name: group.name,
        avatar: null,
        memberCount: group.members?.length || 0,
      }));
    },
  },
  outbound: {
    deliveryMode: 'gateway',
    chunker: (text: string, limit: number) => {
      // Simple chunking for now
      const chunks: string[] = [];
      for (let i = 0; i < text.length; i += limit) {
        chunks.push(text.substring(i, i + limit));
      }
      return chunks;
    },
    textChunkLimit: TEXT_CHUNK_LIMIT,
    sendText: async (ctx: any) => {
      // Add "group:" prefix to group names
      let target = ctx.to;
      if (!target.startsWith('group:') && !target.includes('12D3Koo')) {
        target = `group:${target}`;
      }
      // Pass agentId from context to enable bot name mapping
      return sendGigiMessage({
        to: target,
        content: ctx.text,
        accountId: ctx.accountId ?? undefined,
        cfg: ctx.cfg,
        agentId: ctx.agentId,
      });
    },
    sendMedia: async (ctx: any) => {
      const resolvedAccountId = ctx.accountId ?? DEFAULT_ACCOUNT_ID;
      const gateway = activeGateways.get(resolvedAccountId);

      if (!gateway || !gateway.client.isConnected()) {
        throw new Error(
          `Gateway not connected for account ${resolvedAccountId}`
        );
      }

      // If no mediaUrl, fallback to text
      if (!ctx.mediaUrl) {
        // Add "group:" prefix to group names
        let target = ctx.to;
        if (!target.startsWith('group:') && !target.includes('12D3Koo')) {
          target = `group:${target}`;
        }
        return sendGigiMessage({
          to: target,
          content: ctx.text || '',
          accountId: resolvedAccountId,
          cfg: ctx.cfg,
          agentId: ctx.agentId,
        });
      }

      // Check if mediaUrl is a local file path
      if (ctx.mediaUrl.startsWith('/') || ctx.mediaUrl.includes(':')) {
        // Share the local file via Gigi P2P
        try {
          const shareCode = await gateway.client.shareFile(ctx.mediaUrl);

          // Get file information
          const file = gateway.client.getFileByShareCode(shareCode);
          const filename =
            file?.info.name || ctx.mediaUrl.split('/').pop() || 'unknown-file';
          const fileSize = file?.info.size || 0;
          const fileType = file?.info.mimeType || 'application/octet-stream';

          // Send file share message
          const fileShareContent = {
            type: 'fileShare' as const,
            shareCode,
            filename,
            fileSize,
            fileType,
          };

          // Add "group:" prefix to group names
          let target = ctx.to;
          if (!target.startsWith('group:') && !target.includes('12D3Koo')) {
            target = `group:${target}`;
          }

          return sendGigiMessage({
            to: target,
            content: JSON.stringify(fileShareContent),
            accountId: resolvedAccountId,
            cfg: ctx.cfg,
            agentId: ctx.agentId,
          });
        } catch (error) {
          console.error(`[GigiPlugin] Error sharing file:`, error);
          // Fallback to sending the file path as text
          const content = ctx.text
            ? `${ctx.text}\n📎 ${ctx.mediaUrl}`
            : `📎 ${ctx.mediaUrl}`;

          // Add "group:" prefix to group names
          let target = ctx.to;
          if (!target.startsWith('group:') && !target.includes('12D3Koo')) {
            target = `group:${target}`;
          }

          return sendGigiMessage({
            to: target,
            content,
            accountId: resolvedAccountId,
            cfg: ctx.cfg,
            agentId: ctx.agentId,
          });
        }
      } else {
        // For remote URLs, just send as text
        const content = ctx.text
          ? `${ctx.text}\n📎 ${ctx.mediaUrl}`
          : `📎 ${ctx.mediaUrl}`;

        // Add "group:" prefix to group names
        let target = ctx.to;
        if (!target.startsWith('group:') && !target.includes('12D3Koo')) {
          target = `group:${target}`;
        }

        return sendGigiMessage({
          to: target,
          content,
          accountId: resolvedAccountId,
          cfg: ctx.cfg,
          agentId: ctx.agentId,
        });
      }
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
            kind: 'config',
            message: 'Gigi P2P 未配置 mnemonic/peerId 或 multiaddrs',
            fix: 'Run: openclaw channels add gigi --mnemonic <mnemonic> --multiaddrs <multiaddrs>',
          });
        }
        return issues;
      }),
    buildChannelSummary: ({ snapshot }) => ({
      configured: snapshot.configured ?? false,
      running: snapshot.running ?? false,
      lastStartAt: snapshot.lastStartAt ?? null,
      lastStopAt: snapshot.lastStopAt ?? null,
      lastError: snapshot.lastError ?? null,
    }),
    probeAccount: async (ctx) => {
      const gateway = activeGateways.get(
        ctx.account.accountId || DEFAULT_ACCOUNT_ID
      );
      if (!gateway) {
        return { ok: false, status: 503, message: 'Gateway not started' };
      }
      return { ok: gateway.running, status: gateway.running ? 200 : 503 };
    },
    buildAccountSnapshot: ({ account, runtime }) => {
      const configured = Boolean(
        (account.peerId?.trim() && account.multiaddrs?.length > 0) ||
        account.mnemonic?.trim()
      );
      return {
        accountId: account.accountId,
        name: account.displayName || account.peerId?.substring(0, 8) || 'gigi',
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
      const { accountId, setStatus, cfg, runtime } = ctx;

      // Check if already started
      if (activeGateways.has(accountId)) {
        // Gateway already started, return early
        if (setStatus) {
          const gateway = activeGateways.get(accountId);
          setStatus({
            accountId,
            running: gateway?.running ?? false,
            lastStartAt: gateway?.lastStartAt ?? null,
            lastError: null,
          });
        }
        return;
      }

      try {
        // Get the account configuration
        const account = resolveGigiAccount({ cfg, accountId });

        if (!account) {
          throw new Error(`Account configuration not found for ${accountId}`);
        }

        logger.info('Starting gateway');

        // Log before creating GigiClient
        logger.debug('Creating GigiClient');

        // Create Gigi P2P client
        const client = new GigiClient({
          multiaddrs: account.multiaddrs,
          displayName: account.displayName,
          nickname: account.nickname,
          mnemonic: account.mnemonic,
          bootstrapPeers: account.bootstrapPeers,
          enableMdns: account.enableMdns,
          enableDht: account.enableDht,
          enableRelay: account.enableRelay,
        });

        logger.debug('GigiClient created');

        // Create outbound manager
        const outbound = new OutboundManager(client);

        logger.debug('OutboundManager created');

        // Create gateway context first
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

        // Set up message handler
        client.onMessage(async (gigiMessage) => {
          console.log('[GigiPlugin] Received message from P2P network');
          console.log(
            '[GigiPlugin] Message content:',
            JSON.stringify(gigiMessage, null, 2)
          );
          logger.info('Received message from P2P network');
          logger.debug(
            'Message content: ' + JSON.stringify(gigiMessage, null, 2)
          );

          // Handle different message types
          if ('type' in gigiMessage) {
            const msg = gigiMessage as { type: string };
            console.log('[GigiPlugin] Received message of type:', msg.type);
            logger.info(`Received message of type: ${msg.type}`);
            switch (msg.type) {
              case 'text': {
                const textMessage = gigiMessage as TextMessage;
                console.log('[GigiPlugin] Received text message');
                console.log(
                  '[GigiPlugin] Text message content:',
                  JSON.stringify(textMessage, null, 2)
                );
                logger.info('Received text message');
                logger.debug(
                  'Text message content: ' +
                    JSON.stringify(textMessage, null, 2)
                );

                // Use the OpenClaw plugin SDK to dispatch the message to all agents
                try {
                  // Get the list of available agents - default to main agent
                  let agents = [{ id: 'main', name: 'main' }];
                  console.log('[GigiPlugin] Found agents:', agents);
                  console.log('[GigiPlugin] Using main agent');
                  logger.info('Found agents');
                  logger.info('Using main agent');

                  // Try to get agents from config if available
                  try {
                    // Use gatewayContext.account instead of account
                    const agentsConfig = (gatewayContext.account as any)
                      .agents as any;
                    if (agentsConfig && agentsConfig.list) {
                      agents = agentsConfig.list;
                      console.log(
                        '[GigiPlugin] Using agents from config:',
                        agents
                      );
                      logger.info('Using agents from config');
                    }
                  } catch {
                    console.log(
                      '[GigiPlugin] No agents config found, using main agent'
                    );
                    logger.debug('No agents config found, using main agent');
                  }

                  // If target is specific, filter agents to only include the specified ones
                  if (
                    textMessage.target.type === 'specific' &&
                    textMessage.target.agentIds
                  ) {
                    agents = agents.filter((agent) =>
                      textMessage.target.agentIds?.includes(agent.id)
                    );
                    console.log('[GigiPlugin] Filtered agents:', agents);
                    logger.info('Filtered agents');
                  } else {
                    console.log('[GigiPlugin] Using all agents');
                    logger.info('Using all agents');
                  }

                  // Dispatch message to each agent
                  for (const agent of agents) {
                    console.log(
                      '[GigiPlugin] Dispatching message to agent:',
                      agent.id
                    );
                    logger.debug('Dispatching message to agent');

                    try {
                      // Get the global runtime object
                      const runtime = getGigiRuntime();
                      console.log('[GigiPlugin] Got runtime object');

                      // Build the inbound context payload for this agent
                      const ctxPayload = (
                        runtime as any
                      ).channel.reply.finalizeInboundContext({
                        channel: CHANNEL_ID,
                        accountId: gatewayContext.accountId,
                        from: textMessage.sender.id,
                        to: agent.id,
                        text: textMessage.content,
                        body: textMessage.content,
                        rawBody: textMessage.content,
                        payload: {
                          text: textMessage.content,
                          type: 'text',
                        },
                        senderName: textMessage.sender.name,
                        messageId: textMessage.id,
                        timestamp: textMessage.timestamp,
                        extraFields: {
                          target: textMessage.target,
                          senderType: textMessage.sender.type,
                          agentId: agent.id,
                        },
                      });
                      console.log('[GigiPlugin] Built context payload');

                      // Create a proper reply dispatcher for this agent
                      const { dispatcher, replyOptions } = (
                        runtime as any
                      ).channel.reply.createReplyDispatcherWithTyping({
                        responsePrefix: '',
                        responsePrefixContextProvider: () => ({}),
                        humanDelay: (
                          runtime as any
                        ).channel.reply.resolveHumanDelayConfig(cfg, agent.id),

                        onReplyStart: async () => {
                          console.log(
                            '[GigiPlugin] Reply started for agent:',
                            agent.id
                          );
                          logger.debug('Reply started for agent');
                        },

                        deliver: async (payload: any) => {
                          console.log(
                            '[GigiPlugin] Agent response received:',
                            payload
                          );
                          logger.info('Agent response received');
                          // Send the agent's response back to the P2P network
                          try {
                            if (payload.text) {
                              // Create a text message response
                              const responseMessage = {
                                type: 'text' as const,
                                content: payload.text,
                                target: {
                                  type: 'specific' as const,
                                  agentIds: [textMessage.sender.id],
                                },
                                sender: {
                                  id: agent.id,
                                  name: agent.name || agent.id,
                                  type: 'agent' as const,
                                },
                                timestamp: Date.now(),
                                id: `text-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
                              };
                              console.log(
                                '[GigiPlugin] Sending response back to P2P network:',
                                JSON.stringify(responseMessage, null, 2)
                              );
                              logger.info(
                                'Sending response back to P2P network'
                              );

                              // Send the response back to the sender
                              await gatewayContext.client.sendDirectMessage(
                                textMessage.sender.id,
                                JSON.stringify(responseMessage)
                              );
                              console.log(
                                '[GigiPlugin] Sent response from agent',
                                agent.id,
                                'to P2P network'
                              );
                              logger.info(
                                `Sent response from agent ${agent.id} to P2P network`
                              );
                            }
                          } catch (error) {
                            console.error(
                              '[GigiPlugin] Error sending response from agent',
                              agent.id,
                              'to P2P network:',
                              error
                            );
                            logger.error(
                              `Error sending response from agent ${agent.id} to P2P network`
                            );
                          }
                        },

                        onError: async (_err: any, _info: any) => {
                          console.error(
                            '[GigiPlugin] Reply error for agent:',
                            agent.id
                          );
                          logger.error('Reply error for agent');
                        },

                        onIdle: async () => {
                          console.log(
                            '[GigiPlugin] Reply idle for agent:',
                            agent.id
                          );
                          logger.debug('Reply idle for agent');
                        },

                        onCleanup: async () => {
                          console.log(
                            '[GigiPlugin] Reply cleanup for agent:',
                            agent.id
                          );
                          logger.debug('Reply cleanup for agent');
                        },
                      });
                      console.log('[GigiPlugin] Created reply dispatcher');

                      // Dispatch the message to the agent
                      console.log(
                        '[GigiPlugin] Dispatching reply from config for agent:',
                        agent.id
                      );
                      await (
                        runtime as any
                      ).channel.reply.dispatchReplyFromConfig({
                        ctx: ctxPayload,
                        cfg: cfg,
                        dispatcher,
                        replyOptions,
                      });

                      console.log(
                        '[GigiPlugin] Message dispatched to OpenClaw agent:',
                        agent.id
                      );
                      logger.debug('Message dispatched to OpenClaw agent');
                    } catch (agentError) {
                      console.error(
                        '[GigiPlugin] Error processing agent',
                        agent.id,
                        ':',
                        agentError
                      );
                      logger.error(`Error processing agent ${agent.id}`);
                    }
                  }
                } catch {
                  console.error(
                    '[GigiPlugin] Error dispatching message to agents'
                  );
                  logger.error('Error dispatching message to agents');
                }
                break;
              }
              case 'file': {
                const fileMessage = gigiMessage as FileMessage;
                logger.debug('Received file message');

                // Create file share message content
                const fileShareContent = {
                  type: 'fileShare' as const,
                  shareCode: fileMessage.fileHash,
                  filename: fileMessage.filename,
                  fileSize: fileMessage.fileSize,
                  fileType: 'application/octet-stream', // Default file type
                };

                // Use the OpenClaw plugin SDK to dispatch the message to all agents
                try {
                  // Get the list of available agents from config
                  const agentsConfig = (cfg as Record<string, unknown>)
                    .agents as
                    | { list?: Array<{ id: string; name?: string }> }
                    | undefined;
                  let agents = agentsConfig?.list ?? [];

                  // If target is specific, filter agents to only include the specified ones
                  if (
                    fileMessage.target.type === 'specific' &&
                    fileMessage.target.agentIds
                  ) {
                    agents = agents.filter((agent) =>
                      fileMessage.target.agentIds?.includes(agent.id)
                    );
                    logger.debug('Filtered agents');
                  } else {
                    logger.debug('Found agents');
                  }

                  // Dispatch message to each agent
                  for (const agent of agents) {
                    logger.debug('Dispatching file message to agent');

                    // Build the inbound context payload for this agent
                    const ctxPayload = (
                      runtime as any
                    ).channel.reply.finalizeInboundContext({
                      channel: CHANNEL_ID,
                      accountId: gatewayContext.accountId,
                      from: fileMessage.sender.id,
                      to: agent.id,
                      text: JSON.stringify(fileShareContent),
                      body: JSON.stringify(fileShareContent),
                      rawBody: JSON.stringify(fileShareContent),
                      payload: {
                        text: JSON.stringify(fileShareContent),
                        type: 'fileShare',
                      },
                      senderName: fileMessage.sender.name,
                      messageId: fileMessage.id,
                      timestamp: fileMessage.timestamp,
                      extraFields: {
                        target: fileMessage.target,
                        senderType: fileMessage.sender.type,
                        agentId: agent.id,
                        fileInfo: {
                          filename: fileMessage.filename,
                          fileSize: fileMessage.fileSize,
                          fileHash: fileMessage.fileHash,
                        },
                      },
                    });

                    // Create a proper reply dispatcher for this agent
                    const { dispatcher, replyOptions } = (
                      runtime as any
                    ).channel.reply.createReplyDispatcherWithTyping({
                      responsePrefix: '',
                      responsePrefixContextProvider: () => ({}),
                      humanDelay: (
                        runtime as any
                      ).channel.reply.resolveHumanDelayConfig(cfg, agent.id),

                      onReplyStart: async () => {
                        logger.debug('Reply started for agent');
                      },

                      deliver: async (payload: any) => {
                        logger.debug('Agent response');
                        // Send the agent's response back to the P2P network
                        try {
                          if (payload.text) {
                            // Create a text message response
                            const responseMessage = {
                              type: 'text' as const,
                              content: payload.text,
                              target: {
                                type: 'specific' as const,
                                agentIds: [fileMessage.sender.id],
                              },
                              sender: {
                                id: agent.id,
                                name: agent.name || agent.id,
                                type: 'agent' as const,
                              },
                              timestamp: Date.now(),
                              id: `text-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
                            };

                            // Send the response back to the sender
                            await gatewayContext.client.sendDirectMessage(
                              fileMessage.sender.id,
                              JSON.stringify(responseMessage)
                            );
                            console.log(
                              `[GigiPlugin] Sent response from agent ${agent.id} to P2P network`
                            );
                          }
                        } catch (error) {
                          console.error(
                            `[GigiPlugin] Error sending response from agent ${agent.id} to P2P network:`,
                            error
                          );
                        }
                      },

                      onError: async (_err: any, _info: any) => {
                        logger.error('Reply error for agent');
                      },

                      onIdle: async () => {
                        logger.debug('Reply idle for agent');
                      },

                      onCleanup: async () => {
                        logger.debug('Reply cleanup for agent');
                      },
                    });

                    // Dispatch the message to the agent
                    await (
                      runtime as any
                    ).channel.reply.dispatchReplyFromConfig({
                      ctx: ctxPayload,
                      cfg: cfg,
                      dispatcher,
                      replyOptions,
                    });

                    logger.debug('File message dispatched to OpenClaw agent');
                  }
                } catch {
                  logger.error('Error dispatching file message to agents');
                }
                break;
              }
              case 'agent-settings-query': {
                logger.debug('Received agent settings query');
                // Create and send agent settings response
                // Get OpenClaw agent information from config if available
                const openclawAgents = [];
                if (
                  gatewayContext.account.config &&
                  gatewayContext.account.config.agents
                ) {
                  for (const [agentId, agentConfig] of Object.entries(
                    gatewayContext.account.config.agents
                  )) {
                    const agent = agentConfig as any;
                    openclawAgents.push({
                      id: agentId,
                      name: agent.name || agentId,
                      model: agent.model || 'unknown',
                      status: 'active',
                    });
                  }
                }

                // Get peer ID from the client (it's derived from mnemonic)
                const peerId = gatewayContext.client.getPeerId();
                const displayName =
                  gatewayContext.account.displayName || peerId.substring(0, 8);

                const responseMessage =
                  AmpMessageFactory.createAgentSettingsResponse(
                    [
                      {
                        id: peerId,
                        name: displayName,
                        type: 'openclaw-agent',
                        version: '1.0.0',
                        settings: [
                          {
                            id: 'enabled',
                            name: 'Enabled',
                            type: 'boolean',
                            value: true,
                          },
                          {
                            id: 'displayName',
                            name: 'Display Name',
                            type: 'string',
                            value: displayName,
                          },
                          {
                            id: 'peerId',
                            name: 'Peer ID',
                            type: 'string',
                            value: peerId,
                          },
                          {
                            id: 'multiaddrs',
                            name: 'Multiaddrs',
                            type: 'array',
                            value: gatewayContext.account.multiaddrs || [],
                          },
                        ],
                        status: 'online',
                        openclawAgents,
                      },
                    ],
                    {
                      id: peerId,
                      name: displayName,
                      type: 'agent',
                    }
                  );
                // Send response back to the sender
                try {
                  await gatewayContext.client.sendGroupMessage(
                    'gigi-agents',
                    JSON.stringify(responseMessage)
                  );
                  logger.info('Sent agent settings response');
                } catch {
                  logger.error('Error sending agent settings response');
                }
                break;
              }
              case 'agent-settings-response': {
                logger.debug('Received agent settings response');
                break;
              }
              default:
                logger.debug('Received unknown message type');
            }
          }
        });

        console.log(`[GigiPlugin] Message handler set up for ${accountId}`);

        // Start P2P client
        console.log(`[GigiPlugin] Starting P2P client for ${accountId}`);
        await client.start();
        console.log(`[GigiPlugin] P2P client started for ${accountId}`);

        // Join the configured group for agent communication
        const agentGroup = account.group || 'gigi-agents';
        try {
          await client.joinGroup(agentGroup);
          console.log(`[GigiPlugin] Joined agent group: ${agentGroup}`);
        } catch (error) {
          console.warn(
            `[GigiPlugin] Failed to join agent group ${agentGroup}: ${error instanceof Error ? error.message : 'Unknown error'}`
          );
        }

        console.log(`[GigiPlugin] Gateway context created for ${accountId}`);

        // Store active gateway
        activeGateways.set(accountId, gatewayContext);

        console.log(`[GigiPlugin] Gateway stored for ${accountId}`);

        // Update status
        if (setStatus) {
          setStatus({
            accountId,
            running: true,
            lastStartAt: Date.now(),
            lastError: null,
          });
        }
        console.log(
          `[GigiPlugin] Gateway started successfully for ${accountId}`
        );
      } catch (error) {
        console.error(
          `[GigiPlugin] Error starting gateway for ${accountId}:`,
          error
        );
        console.error(
          `[GigiPlugin] Error stack:`,
          error instanceof Error ? error.stack : 'No stack'
        );
        if (setStatus) {
          setStatus({
            accountId,
            running: false,
            lastError: error instanceof Error ? error.message : 'Unknown error',
          });
        }
        throw error;
      }
    },
    logoutAccount: async ({ cfg }) => {
      const nextCfg = { ...cfg } as OpenClawConfig;
      const gigiConfig = (cfg.channels?.[CHANNEL_ID] ?? {}) as any;
      const nextGigi = { ...gigiConfig };
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
          nextCfg.channels = { ...nextCfg.channels, [CHANNEL_ID]: nextGigi };
        } else {
          const nextChannels = { ...nextCfg.channels };
          delete (nextChannels as Record<string, unknown>)[CHANNEL_ID];
          if (Object.keys(nextChannels).length > 0) {
            nextCfg.channels = nextChannels;
          } else {
            delete nextCfg.channels;
          }
        }
      }

      const resolved = resolveGigiAccount({
        cfg: changed ? nextCfg : cfg,
        accountId: DEFAULT_ACCOUNT_ID,
      });
      const loggedOut = resolved
        ? !resolved.peerId && !resolved.multiaddrs
        : true;

      return { cleared, envToken: false, loggedOut };
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
          setStatus({
            accountId,
            running: false,
            lastStopAt: Date.now(),
            lastError: null,
          });
        }

        console.log(
          `[GigiPlugin] Gateway stopped successfully for ${accountId}`
        );
        logger.info(`Gateway stopped successfully for ${accountId}`);
      } catch (error) {
        console.error(
          `[GigiPlugin] Error stopping gateway for ${accountId}:`,
          error
        );
        logger.error(`Error stopping gateway for ${accountId}`);
        if (setStatus) {
          setStatus({
            accountId,
            running: false,
            lastError: error instanceof Error ? error.message : 'Unknown error',
          });
        }
        throw error;
      }
    },
  },
};

// Export group management and file sharing functions
export {
  joinGigiGroup,
  leaveGigiGroup,
  listGigiGroups,
  shareGigiFile,
  downloadGigiFile,
};

// Export plugin type
export type GigiPlugin = typeof gigiPlugin;
