import { invoke } from '@tauri-apps/api/core'
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

export async function get_peer_id(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|get_peer_id', args.length === 1 ? args[0] : { ...args });
}

export async function try_get_peer_id(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|try_get_peer_id', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_get_peers(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_get_peers', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_set_nickname(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_set_nickname', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_get_public_key(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_get_public_key', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_get_active_downloads(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_get_active_downloads', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_update_config(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_update_config', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_get_config(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_get_config', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_send_message(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_send_message', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_send_message_to_nickname(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_send_message_to_nickname', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_send_direct_share_group_message(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_send_direct_share_group_message', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_join_group(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_join_group', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_send_group_message(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_send_group_message', args.length === 1 ? args[0] : { ...args });
}

export async function emit_current_state(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|emit_current_state', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_get_message_history(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_get_message_history', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_save_shared_files(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_save_shared_files', args.length === 1 ? args[0] : { ...args });
}

export async function get_messages(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|get_messages', args.length === 1 ? args[0] : { ...args });
}

export async function search_messages(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|search_messages', args.length === 1 ? args[0] : { ...args });
}

export async function clear_messages_with_thumbnails(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|clear_messages_with_thumbnails', args.length === 1 ? args[0] : { ...args });
}

export async function get_file_thumbnail(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|get_file_thumbnail', args.length === 1 ? args[0] : { ...args });
}

export async function get_full_image_by_path(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|get_full_image_by_path', args.length === 1 ? args[0] : { ...args });
}

export async function get_full_image(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|get_full_image', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_send_file_message_with_path(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_send_file_message_with_path', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_send_group_file_message_with_path(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_send_group_file_message_with_path', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_share_file(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_share_file', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_request_file(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_request_file', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_request_file_from_nickname(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_request_file_from_nickname', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_cancel_download(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_cancel_download', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_get_shared_files(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_get_shared_files', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_remove_shared_file(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_remove_shared_file', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_get_image_data(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_get_image_data', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_get_file_info(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_get_file_info', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_select_any_file(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_select_any_file', args.length === 1 ? args[0] : { ...args });
}

export async function messaging_share_content_uri(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|messaging_share_content_uri', args.length === 1 ? args[0] : { ...args });
}

export async function clear_app_data(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|clear_app_data', args.length === 1 ? args[0] : { ...args });
}

export async function get_conversations(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|get_conversations', args.length === 1 ? args[0] : { ...args });
}

export async function get_conversation(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|get_conversation', args.length === 1 ? args[0] : { ...args });
}

export async function upsert_conversation(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|upsert_conversation', args.length === 1 ? args[0] : { ...args });
}

export async function update_conversation_last_message(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|update_conversation_last_message', args.length === 1 ? args[0] : { ...args });
}

export async function increment_conversation_unread(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|increment_conversation_unread', args.length === 1 ? args[0] : { ...args });
}

export async function mark_conversation_as_read(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|mark_conversation_as_read', args.length === 1 ? args[0] : { ...args });
}

export async function delete_conversation(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|delete_conversation', args.length === 1 ? args[0] : { ...args });
}

export async function auth_check_account(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|auth_check_account', args.length === 1 ? args[0] : { ...args });
}

export async function auth_signup(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|auth_signup', args.length === 1 ? args[0] : { ...args });
}

export async function auth_get_account_info(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|auth_get_account_info', args.length === 1 ? args[0] : { ...args });
}

export async function auth_delete_account(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|auth_delete_account', args.length === 1 ? args[0] : { ...args });
}

export async function auth_verify_password(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|auth_verify_password', args.length === 1 ? args[0] : { ...args });
}

export async function group_create(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|group_create', args.length === 1 ? args[0] : { ...args });
}

export async function group_join(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|group_join', args.length === 1 ? args[0] : { ...args });
}

export async function group_get_all(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|group_get_all', args.length === 1 ? args[0] : { ...args });
}

export async function group_get(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|group_get', args.length === 1 ? args[0] : { ...args });
}

export async function group_delete(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|group_delete', args.length === 1 ? args[0] : { ...args });
}

export async function group_update(...args: any[]): Promise<any> {
  return await invoke('plugin:gigi|group_update', args.length === 1 ? args[0] : { ...args });
}

// Event listeners
// Note: Tauri plugin system automatically handles the plugin prefix, so we don't need to add it manually

export async function onPeerDiscovered(callback: EventCallback<PeerDiscovered>): Promise<UnlistenFn> {
  return await listen('peer-discovered', callback);
}

export async function onPeerExpired(callback: EventCallback<PeerExpired>): Promise<UnlistenFn> {
  return await listen('peer-expired', callback);
}

export async function onNicknameUpdated(callback: EventCallback<NicknameUpdated>): Promise<UnlistenFn> {
  return await listen('nickname-updated', callback);
}

export async function onMessageReceived(callback: EventCallback<MessageReceived>): Promise<UnlistenFn> {
  return await listen('message-received', callback);
}

export async function onGroupMessage(callback: EventCallback<GroupMessage>): Promise<UnlistenFn> {
  return await listen('group-message', callback);
}

export async function onImageMessageReceived(callback: EventCallback<ImageMessageReceived>): Promise<UnlistenFn> {
  return await listen('image-message-received', callback);
}

export async function onGroupImageMessageReceived(callback: EventCallback<GroupImageMessageReceived>): Promise<UnlistenFn> {
  return await listen('group-image-message-received', callback);
}

export async function onFileMessageReceived(callback: EventCallback<FileMessageReceived>): Promise<UnlistenFn> {
  return await listen('file-message-received', callback);
}

export async function onGroupFileMessageReceived(callback: EventCallback<GroupFileMessageReceived>): Promise<UnlistenFn> {
  return await listen('group-file-message-received', callback);
}

export async function onGroupShareReceived(callback: EventCallback<GroupShareReceived>): Promise<UnlistenFn> {
  return await listen('group-share-received', callback);
}

export async function onFileShareRequest(callback: EventCallback<FileShareRequest>): Promise<UnlistenFn> {
  return await listen('file-share-request', callback);
}

export async function onFileDownloadProgress(callback: EventCallback<FileDownloadProgress>): Promise<UnlistenFn> {
  return await listen('file-download-progress', callback);
}

export async function onFileDownloadCompleted(callback: EventCallback<FileDownloadCompleted>): Promise<UnlistenFn> {
  return await listen('file-download-completed', callback);
}

export async function onFileDownloadStarted(callback: EventCallback<FileDownloadStarted>): Promise<UnlistenFn> {
  return await listen('file-download-started', callback);
}

export async function onFileDownloadFailed(callback: EventCallback<FileDownloadFailed>): Promise<UnlistenFn> {
  return await listen('file-download-failed', callback);
}

export async function onPeerConnected(callback: EventCallback<PeerConnected>): Promise<UnlistenFn> {
  return await listen('peer-connected', callback);
}

export async function onPeerDisconnected(callback: EventCallback<PeerDisconnected>): Promise<UnlistenFn> {
  return await listen('peer-disconnected', callback);
}

export async function onP2pError(callback: EventCallback<P2pError>): Promise<UnlistenFn> {
  return await listen('p2p-error', callback);
}

