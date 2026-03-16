import type { GigiAccount, GigiAccountConfig } from "./types.js";

/**
 * List all Gigi account IDs from channel config
 */
export function listGigiAccountIds(cfg: Record<string, any>): string[] {
  if (!cfg.accounts || typeof cfg.accounts !== "object") {
    return [];
  }
  return Object.keys(cfg.accounts);
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
  if (!cfg.accounts || typeof cfg.accounts !== "object") {
    return null;
  }
  
  const accountConfig = cfg.accounts[accountId];
  if (!accountConfig) {
    return null;
  }

  return {
    accountId,
    displayName: accountConfig.displayName || accountId,
    peerId: accountConfig.peerId,
    multiaddrs: accountConfig.multiaddrs || [],
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
