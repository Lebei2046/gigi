import type { GigiAccount, GigiAccountConfig } from "./types.js";

/**
 * List all Gigi account IDs from channel config
 */
export function listGigiAccountIds(cfg: Record<string, any>): string[] {
  // Check both legacy accounts format and new channels format
  if (cfg.accounts && typeof cfg.accounts === "object") {
    return Object.keys(cfg.accounts);
  }
  if (cfg.channels?.gigi && typeof cfg.channels.gigi === "object") {
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
  
  // Check legacy accounts format
  if (cfg.accounts && typeof cfg.accounts === "object") {
    accountConfig = cfg.accounts[accountId];
  }
  
  // Check new channels format
  if (!accountConfig && cfg.channels?.gigi && typeof cfg.channels.gigi === "object") {
    accountConfig = cfg.channels.gigi;
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
