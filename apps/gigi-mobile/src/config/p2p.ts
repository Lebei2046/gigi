/**
 * P2P Network Configuration for Gigi Mobile
 * 
 * This file contains the default configuration for connecting to the
 * Gigi P2P network via cloud bootstrap and relay nodes.
 */

/**
 * Default bootstrap nodes for Kademlia DHT discovery
 * These are the cloud-hosted entry points for the P2P network
 */
export const DEFAULT_BOOTSTRAP_NODES = [
  // Primary bootstrap nodes (replace with your actual cloud node addresses)
  // Format: "/ip4/<ip>/tcp/<port>/p2p/<peer_id>"
  
  // Example bootstrap node 1
  '/ip4/203.0.113.10/tcp/4001/p2p/12D3KooWxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
  
  // Example bootstrap node 2
  '/ip4/203.0.113.11/tcp/4002/p2p/12D3KooWyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy',
  
  // Example relay node
  '/ip4/203.0.113.12/tcp/4003/p2p/12D3KooWzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz',
];

/**
 * Default P2P configuration
 */
export const DEFAULT_P2P_CONFIG = {
  bootstrapNodes: DEFAULT_BOOTSTRAP_NODES,
  enableKademlia: true,
  enableRelay: true,
  port: 0, // 0 = auto-select
};

/**
 * Production bootstrap nodes
 * Update these with your actual deployed cloud node addresses
 */
export const PRODUCTION_BOOTSTRAP_NODES = [
  // Add your production bootstrap node addresses here after deployment
  // Example:
  // '/dns4/bootstrap1.yourdomain.com/tcp/4001/p2p/12D3KooW...',
  // '/dns4/bootstrap2.yourdomain.com/tcp/4002/p2p/12D3KooW...',
];

/**
 * Development bootstrap nodes (for local testing)
 */
export const DEVELOPMENT_BOOTSTRAP_NODES = [
  // Local bootstrap node for development
  '/ip4/127.0.0.1/tcp/4001/p2p/12D3KooWxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
];

/**
 * Get bootstrap nodes based on environment
 */
export function getBootstrapNodes(environment: 'production' | 'development' = 'production'): string[] {
  switch (environment) {
    case 'production':
      return PRODUCTION_BOOTSTRAP_NODES.length > 0 
        ? PRODUCTION_BOOTSTRAP_NODES 
        : DEFAULT_BOOTSTRAP_NODES;
    case 'development':
      return DEVELOPMENT_BOOTSTRAP_NODES;
    default:
      return DEFAULT_BOOTSTRAP_NODES;
  }
}
