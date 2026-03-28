/**
 * Gigi OpenClaw Plugin
 * 
 * This plugin integrates Gigi P2P network with OpenClaw,
 * enabling P2P messaging between Gigi peers.
 */

import { definePluginEntry } from 'openclaw/plugin-sdk/plugin-entry';
import { gigiPlugin } from "./src/channel.js";
export type { GigiPlugin } from "./src/channel.js";

export { GigiClient } from "./src/GigiClient.js";
export type { IGigiClient, GigiClientConfig, GigiMessage, GatewayConfig, GigiAccount } from "./src/types.js";

export { OutboundManager } from "./src/outbound.js";
export type { OutboundMessage } from "./src/outbound.js";

export * from "./src/config-schema.js";

export * from "./src/accounts.js";

export { probeGigiClient, getStatusSummary } from "./src/probe.js";
export type { HealthCheckResult } from "./src/probe.js";

// Import group management functions
import { joinGigiGroup, leaveGigiGroup, listGigiGroups } from "./src/channel.js";
import { generateMnemonic } from "@gigi/p2p-ts";

// Plugin registration for OpenClaw
export default definePluginEntry({
  id: 'gigi-p2p-bundled',
  name: 'Gigi P2P',
  description: 'Connect to Gigi P2P network and join groups',
  register: (api) => {
    // Register the channel
    api.registerChannel({ plugin: gigiPlugin });
    
    // Register group management tools
    api.registerTool(
      {
        name: 'gigi_join_group',
        label: 'Join Gigi Group',
        description: 'Join a Gigi P2P group',
        parameters: {
          type: 'object',
          properties: {
            groupName: {
              type: 'string',
              description: 'Name of the group to join'
            },
            accountId: {
              type: 'string',
              description: 'Account ID to use (optional)'
            }
          },
          required: ['groupName']
        },
        execute: async (toolCallId, params) => {
          await joinGigiGroup({ groupName: params.groupName, accountId: params.accountId });
          return {
            content: [{ type: 'text', text: `Joined group: ${params.groupName}` }],
            details: { success: true, message: `Joined group: ${params.groupName}` }
          };
        }
      }
    );
    
    api.registerTool(
      {
        name: 'gigi_leave_group',
        label: 'Leave Gigi Group',
        description: 'Leave a Gigi P2P group',
        parameters: {
          type: 'object',
          properties: {
            groupName: {
              type: 'string',
              description: 'Name of the group to leave'
            },
            accountId: {
              type: 'string',
              description: 'Account ID to use (optional)'
            }
          },
          required: ['groupName']
        },
        execute: async (toolCallId, params) => {
          await leaveGigiGroup({ groupName: params.groupName, accountId: params.accountId });
          return {
            content: [{ type: 'text', text: `Left group: ${params.groupName}` }],
            details: { success: true, message: `Left group: ${params.groupName}` }
          };
        }
      }
    );
    
    api.registerTool(
      {
        name: 'gigi_list_groups',
        label: 'List Gigi Groups',
        description: 'List joined Gigi P2P groups',
        parameters: {
          type: 'object',
          properties: {
            accountId: {
              type: 'string',
              description: 'Account ID to use (optional)'
            }
          }
        },
        execute: async (toolCallId, params) => {
          const groups = await listGigiGroups({ accountId: params.accountId });
          return {
            content: [{ type: 'text', text: `Found ${groups.length} groups: ${groups.map(g => g.name).join(', ')}` }],
            details: { groups }
          };
        }
      }
    );
    
    // Register generate mnemonic tool
    api.registerTool(
      {
        name: 'gigi_generate_mnemonic',
        label: 'Generate Gigi Mnemonic',
        description: 'Generate a new BIP-39 mnemonic phrase for Gigi P2P',
        parameters: {
          type: 'object',
          properties: {}
        },
        execute: async (toolCallId, params) => {
          try {
            // Generate a new BIP-39 mnemonic phrase
            const mnemonic = generateMnemonic();
            return {
              content: [
                { 
                  type: 'text', 
                  text: `Generated BIP-39 mnemonic phrase:` 
                },
                { 
                  type: 'text', 
                  text: `${mnemonic}` 
                },
                { 
                  type: 'text', 
                  text: `This mnemonic can be used in your channel configuration to derive your peer ID and private key.` 
                }
              ],
              details: { 
                mnemonic: mnemonic,
                instructions: 'Add this mnemonic to the peerIdJson field in your channel configuration' 
              }
            };
          } catch (error) {
            return {
              content: [{ type: 'text', text: `Error generating mnemonic: ${error instanceof Error ? error.message : 'Unknown error'}` }],
              details: { error: error instanceof Error ? error.message : 'Unknown error' }
            };
          }
        }
      }
    );
  },
});
