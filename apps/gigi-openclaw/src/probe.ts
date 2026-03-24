import type { IGigiClient } from "./types.js";

/**
 * Health check result
 */
export interface HealthCheckResult {
  healthy: boolean;
  peerId?: string;
  peerCount?: number;
  groupCount?: number;
  multiaddrs?: string[];
  error?: string;
}

/**
 * Perform health check on Gigi client
 */
export async function probeGigiClient(
  client: IGigiClient
): Promise<HealthCheckResult> {
  try {
    if (!client.isConnected()) {
      return {
        healthy: false,
        error: "Client not connected",
      };
    }

    const peerId = client.getPeerId();
    const multiaddrs = client.getMultiaddrs();

    // Get connected peers count
    const peers = client.listPeers();
    const peerCount = peers.length;

    // Get joined groups count
    const groups = client.listGroups();
    const groupCount = groups.length;

    return {
      healthy: true,
      peerId,
      peerCount,
      groupCount,
      multiaddrs,
    };
  } catch (error) {
    return {
      healthy: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

/**
 * Get status summary for OpenClaw UI
 */
export async function getStatusSummary(client: IGigiClient): Promise<{
  status: "connected" | "disconnected" | "error";
  message: string;
  details?: Record<string, any>;
}> {
  const health = await probeGigiClient(client);

  if (!health.healthy) {
    return {
      status: "error",
      message: health.error || "Connection failed",
      details: health,
    };
  }

  return {
    status: "connected",
    message: `Connected as ${health.peerId} (${health.peerCount} peers, ${health.groupCount} groups)`,
    details: {
      peerId: health.peerId,
      peerCount: health.peerCount,
      groupCount: health.groupCount,
      listeningAddresses: health.multiaddrs,
    },
  };
}
