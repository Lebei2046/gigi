#!/usr/bin/env node

import { readFileSync, writeFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

// Read the rollup config to get plugin manifest
const rollupConfig = readFileSync(resolve(__dirname, '../rollup.config.js'), 'utf-8');

// Extract commands manually - we'll build them manually
const commands = [
  // Peer commands
  'get_peer_id',
  'try_get_peer_id',
  // Config commands
  'messaging_get_peers',
  'messaging_set_nickname',
  'messaging_get_public_key',
  'messaging_get_active_downloads',
  'messaging_update_config',
  'messaging_get_config',
  // Messaging commands
  'messaging_initialize_with_key',
  'messaging_send_message_to_nickname',
  'messaging_send_direct_share_group_message',
  'messaging_join_group',
  'messaging_send_group_message',
  'emit_current_state',
  // File commands
  'messaging_send_file_message_with_path',
  'messaging_send_group_file_message_with_path',
  'messaging_share_file',
  'messaging_request_file_from_nickname',
  'messaging_cancel_download',
  'messaging_get_shared_files',
  'messaging_remove_shared_file',
  'messaging_get_image_data',
  'messaging_get_file_info',
  'messaging_select_any_file',
  'messaging_share_content_uri',
  // Utils commands
  'clear_app_data',
];

// Events that the plugin emits
const events = [
  'peer-discovered',
  'peer-expired',
  'nickname-updated',
  'message-received',
  'group-message',
  'image-message-received',
  'group-image-message-received',
  'file-message-received',
  'group-file-message-received',
  'group-share-received',
  'file-share-request',
  'file-download-progress',
  'file-download-completed',
  'file-download-started',
  'file-download-failed',
  'peer-connected',
  'peer-disconnected',
  'p2p-error',
];

// Generate TypeScript code
let tsCode = `import { invoke } from '@tauri-apps/api/core'
import { listen, type EventCallback, type UnlistenFn } from '@tauri-apps/api/event'

`;

// Generate functions for each command
commands.forEach(cmd => {
  tsCode += `export async function ${cmd}(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|${cmd}', args.length === 1 ? args[0] : { ...args });
}

`;
});

// Generate event listener functions for each event
tsCode += `// Event listeners

`;
events.forEach(event => {
  const eventName = event.replace(/-([a-z])/g, (_, c) => c.toUpperCase()).replace(/^-/, ''); // Convert to camelCase for function name
  tsCode += `export async function on${eventName.charAt(0).toUpperCase() + eventName.slice(1)}(callback: EventCallback<any>): Promise<UnlistenFn> {
  return await listen('plugin:gigi-p2p|${event}', callback);
}

`;
});

// Write to guest-js/index.ts
writeFileSync(resolve(__dirname, '../guest-js/index.ts'), tsCode);

console.log('Generated guest-js/index.ts with', commands.length, 'commands and', events.length, 'event listeners');
