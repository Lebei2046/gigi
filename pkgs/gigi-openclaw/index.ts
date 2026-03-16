/**
 * Gigi OpenClaw Plugin
 * 
 * This plugin integrates Gigi P2P network with OpenClaw,
 * enabling P2P messaging between Gigi peers.
 */

export { gigiPlugin } from "./src/channel.js";
export type { GigiPlugin } from "./src/channel.js";

export { GigiClient } from "./src/GigiClient.js";
export type { IGigiClient, GigiClientConfig, GigiMessage, GatewayConfig, GigiAccount } from "./src/types.js";

export { OutboundManager } from "./src/outbound.js";
export type { OutboundMessage } from "./src/outbound.js";

export * from "./src/config-schema.js";

export * from "./src/accounts.js";

export { probeGigiClient, getStatusSummary } from "./src/probe.js";
export type { HealthCheckResult } from "./src/probe.js";

// Default export for OpenClaw
export default gigiPlugin;
