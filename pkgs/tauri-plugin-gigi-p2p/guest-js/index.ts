import { invoke } from '@tauri-apps/api/core'
import { listen, type EventCallback, type UnlistenFn } from '@tauri-apps/api/event'

export async function get_peer_id(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|get_peer_id', args.length === 1 ? args[0] : { ...args });
}

export async function try_get_peer_id(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|try_get_peer_id', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_get_peers(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_get_peers', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_set_nickname(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_set_nickname', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_get_public_key(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_get_public_key', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_get_active_downloads(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_get_active_downloads', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_update_config(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_update_config', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_get_config(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_get_config', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_initialize_with_key(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_initialize_with_key', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_send_message_to_nickname(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_send_message_to_nickname', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_send_direct_share_group_message(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_send_direct_share_group_message', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_join_group(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_join_group', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_send_group_message(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_send_group_message', args.length === 1 ? args[0] : { ...args });
}

export async function emit_current_state(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|emit_current_state', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_send_file_message_with_path(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_send_file_message_with_path', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_send_group_file_message_with_path(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_send_group_file_message_with_path', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_share_file(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_share_file', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_request_file_from_nickname(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_request_file_from_nickname', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_cancel_download(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_cancel_download', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_get_shared_files(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_get_shared_files', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_remove_shared_file(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_remove_shared_file', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_get_image_data(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_get_image_data', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_get_file_info(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_get_file_info', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_select_any_file(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_select_any_file', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_share_content_uri(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|messaging_share_content_uri', args.length === 1 ? args[0] : { ...args });
}

export async function clear_app_data(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi-p2p|clear_app_data', args.length === 1 ? args[0] : { ...args });
}

// Event listeners

export async function onPeerDiscovered(callback: EventCallback<any>): Promise<UnlistenFn> {
  return await listen('plugin:gigi-p2p|peer-discovered', callback);
}

export async function onPeerExpired(callback: EventCallback<any>): Promise<UnlistenFn> {
  return await listen('plugin:gigi-p2p|peer-expired', callback);
}

export async function onNicknameUpdated(callback: EventCallback<any>): Promise<UnlistenFn> {
  return await listen('plugin:gigi-p2p|nickname-updated', callback);
}

export async function onMessageReceived(callback: EventCallback<any>): Promise<UnlistenFn> {
  return await listen('plugin:gigi-p2p|message-received', callback);
}

export async function onGroupMessage(callback: EventCallback<any>): Promise<UnlistenFn> {
  return await listen('plugin:gigi-p2p|group-message', callback);
}

export async function onImageMessageReceived(callback: EventCallback<any>): Promise<UnlistenFn> {
  return await listen('plugin:gigi-p2p|image-message-received', callback);
}

export async function onGroupImageMessageReceived(callback: EventCallback<any>): Promise<UnlistenFn> {
  return await listen('plugin:gigi-p2p|group-image-message-received', callback);
}

export async function onFileMessageReceived(callback: EventCallback<any>): Promise<UnlistenFn> {
  return await listen('plugin:gigi-p2p|file-message-received', callback);
}

export async function onGroupFileMessageReceived(callback: EventCallback<any>): Promise<UnlistenFn> {
  return await listen('plugin:gigi-p2p|group-file-message-received', callback);
}

export async function onGroupShareReceived(callback: EventCallback<any>): Promise<UnlistenFn> {
  return await listen('plugin:gigi-p2p|group-share-received', callback);
}

export async function onFileShareRequest(callback: EventCallback<any>): Promise<UnlistenFn> {
  return await listen('plugin:gigi-p2p|file-share-request', callback);
}

export async function onFileDownloadProgress(callback: EventCallback<any>): Promise<UnlistenFn> {
  return await listen('plugin:gigi-p2p|file-download-progress', callback);
}

export async function onFileDownloadCompleted(callback: EventCallback<any>): Promise<UnlistenFn> {
  return await listen('plugin:gigi-p2p|file-download-completed', callback);
}

export async function onFileDownloadStarted(callback: EventCallback<any>): Promise<UnlistenFn> {
  return await listen('plugin:gigi-p2p|file-download-started', callback);
}

export async function onFileDownloadFailed(callback: EventCallback<any>): Promise<UnlistenFn> {
  return await listen('plugin:gigi-p2p|file-download-failed', callback);
}

export async function onPeerConnected(callback: EventCallback<any>): Promise<UnlistenFn> {
  return await listen('plugin:gigi-p2p|peer-connected', callback);
}

export async function onPeerDisconnected(callback: EventCallback<any>): Promise<UnlistenFn> {
  return await listen('plugin:gigi-p2p|peer-disconnected', callback);
}

export async function onP2pError(callback: EventCallback<any>): Promise<UnlistenFn> {
  return await listen('plugin:gigi-p2p|p2p-error', callback);
}

