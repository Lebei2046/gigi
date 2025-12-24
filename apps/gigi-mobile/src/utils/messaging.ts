import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

// Types matching the Rust backend
export interface Peer {
  id: string
  nickname: string
  capabilities: string[]
}

export interface Message {
  id: string
  from_peer_id: string
  from_nickname: string
  content: string
  timestamp: number
  messageType?: 'text' | 'image'
  imageId?: string
  imageData?: string
  filename?: string
}

export interface GroupMessage {
  id: string
  group_id: string
  from_peer_id: string
  from_nickname: string
  content: string
  timestamp: number
}

export interface GroupShareMessage {
  from_peer_id: string
  from_nickname: string
  group_id: string
  group_name: string
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

export interface FileDownloadStarted {
  from_peer_id: string
  from_nickname: string
  filename: string
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

export interface FileInfo {
  id: string
  name: string
  size: number
  mime_type: string
  peer_id: string
}

export interface DownloadProgress {
  download_id: string
  progress: number
  speed: number
}

export interface Config {
  nickname: string
  auto_accept_files: boolean
  download_folder: string
  max_concurrent_downloads: number
  port: number
}

// Command functions for the messaging backend
export class MessagingClient {
  // Initialize messaging with existing private key
  static async initializeWithKey(
    privateKey: Uint8Array,
    nickname: string
  ): Promise<string> {
    return invoke('messaging_initialize_with_key', {
      privateKey: Array.from(privateKey),
      nickname,
    })
  }

  // Send direct message
  static async sendMessage(toPeerId: string, message: string): Promise<string> {
    return invoke('messaging_send_message', { toPeerId, message })
  }

  // Send direct message by nickname (preferred)
  static async sendMessageToNickname(
    nickname: string,
    message: string
  ): Promise<string> {
    return invoke('messaging_send_message_to_nickname', { nickname, message })
  }

  // Send group share message to peer
  static async sendShareGroupMessage(
    targetNickname: string,
    groupId: string,
    groupName: string
  ): Promise<string> {
    return invoke('messaging_send_direct_share_group_message', {
      nickname: targetNickname,
      groupId,
      groupName,
    })
  }

  // Get connected peers
  static async getPeers(): Promise<Peer[]> {
    return invoke<Peer[]>('messaging_get_peers')
  }

  // Set nickname
  static async setNickname(nickname: string): Promise<void> {
    return invoke('messaging_set_nickname', { nickname })
  }

  // Join a group
  static async joinGroup(groupId: string): Promise<void> {
    return invoke('messaging_join_group', { groupId })
  }

  // Send group message
  static async sendGroupMessage(
    groupId: string,
    message: string
  ): Promise<string> {
    return invoke('messaging_send_group_message', { groupId, message })
  }

  // Share a file
  static async shareFile(filePath: string): Promise<string> {
    return invoke('messaging_share_file', { filePath })
  }

  // Request/download a file
  static async requestFile(
    fileId: string,
    fromPeerId: string
  ): Promise<string> {
    return invoke('messaging_request_file', { fileId, fromPeerId })
  }

  // Request/download file by nickname (preferred)
  static async requestFileFromNickname(
    nickname: string,
    shareCode: string
  ): Promise<string> {
    return invoke('messaging_request_file_from_nickname', {
      nickname,
      shareCode,
    })
  }

  // Cancel download
  static async cancelDownload(downloadId: string): Promise<void> {
    return invoke('messaging_cancel_download', { downloadId })
  }

  // Get shared files
  static async getSharedFiles(): Promise<FileInfo[]> {
    return invoke('messaging_get_shared_files')
  }

  // Remove shared file
  static async removeSharedFile(shareCode: string): Promise<void> {
    return invoke('messaging_remove_shared_file', { shareCode })
  }

  // Save shared files to disk
  static async saveSharedFiles(): Promise<void> {
    return invoke('messaging_save_shared_files')
  }

  // Get current peer ID
  static async getPeerId(): Promise<string> {
    return invoke<string>('get_peer_id')
  }

  // Get public key
  static async getPublicKey(): Promise<string> {
    return invoke<string>('messaging_get_public_key')
  }

  // Get active downloads
  static async getActiveDownloads(): Promise<DownloadProgress[]> {
    return invoke<DownloadProgress[]>('messaging_get_active_downloads')
  }

  // Update configuration
  static async updateConfig(config: Config): Promise<void> {
    return invoke<void>('messaging_update_config', { config })
  }

  // Get current configuration
  static async getConfig(): Promise<Config> {
    return invoke<Config>('messaging_get_config')
  }

  // Select image file using dialog
  static async selectImageFile(): Promise<string | null> {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [
          {
            name: 'Image Files',
            extensions: ['png', 'jpg', 'jpeg', 'gif', 'bmp', 'webp'],
          },
        ],
      })

      return selected || null
    } catch (error) {
      console.error('Failed to open file dialog:', error)
      return null
    }
  }

  // Send file message using file path
  static async sendFileMessageWithPath(
    nickname: string,
    filePath: string
  ): Promise<{ messageId: string; imageData: string }> {
    const response = await invoke<string>(
      'messaging_send_file_message_with_path',
      {
        nickname,
        filePath,
      }
    )

    // Parse the response to extract message ID and base64 image data
    const parts = response.split('|')
    const [messageId, imageData] = parts
    return { messageId, imageData }
  }

  // Send group file message using file path
  static async sendGroupFileMessageWithPath(
    groupId: string,
    filePath: string
  ): Promise<{ messageId: string; imageData: string }> {
    const response = await invoke<string>(
      'messaging_send_group_file_message_with_path',
      {
        groupId,
        filePath,
      }
    )

    // Parse the response to extract message ID and base64 image data
    const parts = response.split('|')
    const [messageId, imageData] = parts
    return { messageId, imageData }
  }

  // Utility function to try get peer ID from private key
  static async tryGetPeerId(priv_key: Uint8Array): Promise<string> {
    return invoke('try_get_peer_id', { privKey: Array.from(priv_key) })
  }

  // Clear app data from backend
  static async clearAppData(): Promise<void> {
    return invoke('clear_app_data')
  }

  // Emit current P2P state (for catching up on missed events)
  static async emitCurrentState(): Promise<void> {
    return invoke('emit_current_state')
  }

  // Get image data from local file path
  static async getImageData(filePath: string): Promise<string> {
    return invoke<string>('messaging_get_image_data', { filePath })
  }

  // Get message history with a peer
  static async getMessageHistory(peerId: string): Promise<Message[]> {
    return invoke<Message[]>('messaging_get_message_history', { peerId })
  }
}

// Event listening utilities
export class MessagingEvents {
  private static listeners: Map<string, ((data: any) => void)[]> = new Map()

  // Register event listener
  static on(eventType: string, callback: (data: any) => void): void {
    console.log(`🔔 Registering listener for event: ${eventType}`)

    // Check if this exact callback is already registered to prevent duplicates
    const existingCallbacks = this.listeners.get(eventType)
    if (existingCallbacks && existingCallbacks.includes(callback)) {
      console.warn(
        `⚠️ Callback already registered for ${eventType}, skipping duplicate`
      )
      return
    }

    if (!this.listeners.has(eventType)) {
      this.listeners.set(eventType, [])
      // Start listening to Tauri events
      import('@tauri-apps/api/event')
        .then(({ listen }) => {
          listen(eventType, event => {
            const callbacks = this.listeners.get(eventType)
            if (callbacks) {
              callbacks.forEach(cb => {
                try {
                  cb(event.payload)
                } catch (error) {
                  console.error(`Error in callback for ${eventType}:`, error)
                }
              })
            }
          }).catch(error => {
            console.error(`Failed to listen to event ${eventType}:`, error)
          })
        })
        .catch(error => {
          console.error(
            `Failed to import Tauri event module for ${eventType}:`,
            error
          )
        })
    }
    const callbacks = this.listeners.get(eventType)!

    // Prevent memory leak by limiting callbacks to max 10 per event type
    if (callbacks.length >= 10) {
      console.warn(
        `⚠️ Too many callbacks for ${eventType} (${callbacks.length}), removing oldest`
      )
      callbacks.shift() // Remove the oldest callback
    }

    callbacks.push(callback)
    console.log(
      `📝 Added callback for: ${eventType}. Total callbacks: ${callbacks.length}`
    )
  }

  // Remove event listener
  static off(eventType: string, callback: (data: any) => void): void {
    const callbacks = this.listeners.get(eventType)
    if (callbacks) {
      const index = callbacks.indexOf(callback)
      if (index > -1) {
        callbacks.splice(index, 1)
        console.log(
          `🗑️ Removed callback for: ${eventType}. Remaining callbacks: ${callbacks.length}`
        )
      } else {
        console.warn(`⚠️ Callback not found for ${eventType}, cannot remove`)
      }
    } else {
      console.warn(`⚠️ No callbacks found for ${eventType}`)
    }
  }
}

// Utility functions
export class MessagingUtils {
  // Convert ArrayBuffer to Base64 string
  static arrayBufferToBase64(buffer: ArrayBuffer): string {
    const bytes = new Uint8Array(buffer)
    let binary = ''
    for (let i = 0; i < bytes.byteLength; i++) {
      binary += String.fromCharCode(bytes[i])
    }
    return btoa(binary)
  }

  // Convert Base64 string to ArrayBuffer
  static base64ToArrayBuffer(base64: string): ArrayBuffer {
    const binary = atob(base64)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i)
    }
    return bytes.buffer
  }

  // Format file size
  static formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 Bytes'
    const k = 1024
    const sizes = ['Bytes', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  // Format download speed
  static formatDownloadSpeed(bytesPerSecond: number): string {
    return this.formatFileSize(bytesPerSecond) + '/s'
  }

  // Format duration
  static formatDuration(seconds: number): string {
    const hours = Math.floor(seconds / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    const secs = Math.floor(seconds % 60)

    if (hours > 0) {
      return `${hours}h ${minutes}m ${secs}s`
    } else if (minutes > 0) {
      return `${minutes}m ${secs}s`
    } else {
      return `${secs}s`
    }
  }
}
