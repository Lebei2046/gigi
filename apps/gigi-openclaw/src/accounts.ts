import type { GigiAccount, GigiAccountConfig } from "./types.js";

const CHANNEL_ID = "gigi-p2p-bundled";

/**
 * List all Gigi account IDs from channel config
 */
export function listGigiAccountIds(cfg: Record<string, any>): string[] {
  // Check both legacy accounts format and new channels format
  if (cfg.accounts && typeof cfg.accounts === "object") {
    return Object.keys(cfg.accounts);
  }
  if (cfg.channels?.[CHANNEL_ID] && typeof cfg.channels[CHANNEL_ID] === "object") {
    return ["default"];
  }
  return [];
}

/**
 * Resolve a Gigi account by ID
 */
export function resolveGigiAccount({
  cfg,
  accountId,
}: {
  cfg: Record<string, any>;
  accountId: string;
}): GigiAccount | null {
  let accountConfig: any = null;
  
  // Check if cfg is already the account config (when called from resolveAccount)
  if (cfg.mnemonic || cfg.peerId || cfg.multiaddrs) {
    accountConfig = cfg;
  }
  // Check legacy accounts format
  else if (cfg.accounts && typeof cfg.accounts === "object") {
    accountConfig = cfg.accounts[accountId];
  }
  // Check new channels format
  else if (cfg.channels?.[CHANNEL_ID] && typeof cfg.channels[CHANNEL_ID] === "object") {
    accountConfig = cfg.channels[CHANNEL_ID];
  }
  
  if (!accountConfig) {
    return null;
  }

  return {
    accountId,
    displayName: accountConfig.displayName || accountId,
    peerId: accountConfig.peerId,
    multiaddrs: accountConfig.multiaddrs || [],
    bootstrapPeers: accountConfig.bootstrapPeers || [],
    enableMdns: accountConfig.enableMdns !== false,
    enableDht: accountConfig.enableDht !== false,
    enableRelay: accountConfig.enableRelay !== false,
    config: accountConfig.config || {},
    mnemonic: accountConfig.mnemonic,
  };
}

/**
 * Validate Gigi account configuration
 */
export function validateAccountConfig(config: any): config is GigiAccountConfig {
  if (!config || typeof config !== "object") {
    return false;
  }
  
  if (typeof config.peerId !== "string" || !config.peerId) {
    return false;
  }
  
  if (!Array.isArray(config.multiaddrs)) {
    return false;
  }
  
  return true;
}

/**
 * Format account for display
 */
export function formatAccountDisplay(account: GigiAccount): string {
  return account.displayName || account.accountId;
}
