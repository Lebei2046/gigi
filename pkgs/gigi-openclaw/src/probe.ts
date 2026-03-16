import type { IGigiClient } from "./types.js";

/**
 * Health check result
 */
export interface HealthCheckResult {
  healthy: boolean;
  peerId?: string;
  peerCount?: number;
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
    const peerCount = 0; // TODO: Implement peer count tracking in GigiClient

    return {
      healthy: true,
      peerId,
      peerCount,
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
    message: `Connected as ${health.peerId}`,
    details: {
      peerId: health.peerId,
      peerCount: health.peerCount,
      listeningAddresses: health.multiaddrs,
    },
  };
}
