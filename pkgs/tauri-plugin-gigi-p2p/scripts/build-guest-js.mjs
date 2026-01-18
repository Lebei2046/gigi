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
  'messaging_send_message',
  'messaging_send_message_to_nickname',
  'messaging_send_direct_share_group_message',
  'messaging_join_group',
  'messaging_send_group_message',
  'emit_current_state',
  'messaging_get_message_history',
  'messaging_save_shared_files',
  'get_messages',
  'search_messages',
  'clear_messages_with_thumbnails',
  'get_file_thumbnail',
  'get_full_image_by_path',
  'get_full_image',
  // File commands
  'messaging_send_file_message_with_path',
  'messaging_send_group_file_message_with_path',
  'messaging_share_file',
  'messaging_request_file',
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
  // Conversation commands
  'get_conversations',
  'get_conversation',
  'upsert_conversation',
  'update_conversation_last_message',
  'increment_conversation_unread',
  'mark_conversation_as_read',
  'delete_conversation',
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

// Event type definitions
export interface PeerDiscovered {
  peer_id: string
  nickname: string
  address: string
}

export interface PeerExpired {
  peer_id: string
  nickname: string
}

export interface NicknameUpdated {
  peer_id: string
  nickname: string
}

export interface MessageReceived {
  from_peer_id: string
  from_nickname: string
  message: string
  timestamp: number
}

export interface GroupMessage {
  group_id: string
  from_peer_id: string
  from_nickname: string
  message: string
  timestamp: number
}

export interface ImageMessageReceived {
  from_peer_id: string
  from_nickname: string
  share_code: string
  filename: string
  file_size: number
  file_type: string
  timestamp: number
  download_error?: string
  download_id?: string
  thumbnailPath?: string
}

export interface GroupImageMessageReceived {
  group_id: string
  from_peer_id: string
  from_nickname: string
  share_code: string
  filename: string
  file_size: number
  file_type: string
  timestamp: number
  download_id?: string
  thumbnailPath?: string
}

export interface FileMessageReceived {
  from_peer_id: string
  from_nickname: string
  share_code: string
  filename: string
  file_size: number
  file_type: string
  timestamp: number
}

export interface GroupFileMessageReceived {
  group_id: string
  from_peer_id: string
  from_nickname: string
  share_code: string
  filename: string
  file_size: number
  file_type: string
  timestamp: number
}

export interface GroupShareReceived {
  from_peer_id: string
  from_nickname: string
  group_id: string
  group_name: string
  timestamp: number
}

export interface FileShareRequest {
  from_peer_id: string
  from_nickname: string
  share_code: string
  filename: string
  size: number
}

export interface FileDownloadStarted {
  from_peer_id: string
  from_nickname: string
  filename: string
  download_id: string
  share_code: string
  timestamp: number
}

export interface FileDownloadProgress {
  download_id: string
  filename: string
  share_code: string
  from_nickname: string
  downloaded_chunks: number
  total_chunks: number
  progress_percent: number
  timestamp: number
}

export interface FileDownloadCompleted {
  download_id: string
  filename: string
  share_code: string
  from_nickname: string
  path: string
  thumbnail_filename?: string
  timestamp: number
}

export interface FileDownloadFailed {
  download_id: string
  filename: string
  share_code: string
  from_nickname: string
  error: string
  timestamp: number
}

export interface PeerConnected {
  peer_id: string
  nickname: string
}

export interface PeerDisconnected {
  peer_id: string
  nickname: string
}

export interface P2pError {
  error: string
}

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
// Note: Tauri plugin system automatically handles the plugin prefix, so we don't need to add it manually

`;
const eventTypes = {
  'peer-discovered': 'PeerDiscovered',
  'peer-expired': 'PeerExpired',
  'nickname-updated': 'NicknameUpdated',
  'message-received': 'MessageReceived',
  'group-message': 'GroupMessage',
  'image-message-received': 'ImageMessageReceived',
  'group-image-message-received': 'GroupImageMessageReceived',
  'file-message-received': 'FileMessageReceived',
  'group-file-message-received': 'GroupFileMessageReceived',
  'group-share-received': 'GroupShareReceived',
  'file-share-request': 'FileShareRequest',
  'file-download-progress': 'FileDownloadProgress',
  'file-download-completed': 'FileDownloadCompleted',
  'file-download-started': 'FileDownloadStarted',
  'file-download-failed': 'FileDownloadFailed',
  'peer-connected': 'PeerConnected',
  'peer-disconnected': 'PeerDisconnected',
  'p2p-error': 'P2pError',
};

events.forEach(event => {
  const eventName = event.replace(/-([a-z])/g, (_, c) => c.toUpperCase()).replace(/^-/, '');
  const eventType = eventTypes[event] || 'any';
  tsCode += `export async function on${eventName.charAt(0).toUpperCase() + eventName.slice(1)}(callback: EventCallback<${eventType}>): Promise<UnlistenFn> {
  return await listen('${event}', callback);
}

`;
});

// Write to guest-js/index.ts
writeFileSync(resolve(__dirname, '../guest-js/index.ts'), tsCode);

console.log('Generated guest-js/index.ts with', commands.length, 'commands and', events.length, 'event listeners');
