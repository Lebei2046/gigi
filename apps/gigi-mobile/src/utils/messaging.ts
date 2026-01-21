import { open } from '@tauri-apps/plugin-dialog'
import * as GigiP2p from 'tauri-plugin-gigi-p2p-api'

// Re-export event types from plugin API for convenience
export type {
  PeerDiscovered,
  PeerExpired,
  NicknameUpdated,
  MessageReceived,
  ImageMessageReceived,
  GroupImageMessageReceived,
  FileMessageReceived,
  GroupFileMessageReceived,
  GroupShareReceived,
  FileShareRequest,
  FileDownloadStarted,
  FileDownloadProgress,
  FileDownloadCompleted,
  FileDownloadFailed,
  PeerConnected,
  PeerDisconnected,
  P2pError,
} from 'tauri-plugin-gigi-p2p-api'

// Check if running on Android
const isAndroid = () => {
  return (
    typeof navigator !== 'undefined' && navigator.userAgent.includes('Android')
  )
}

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
  messageType?: 'text' | 'image' | 'file'
  imageId?: string
  thumbnailData?: string // Base64 thumbnail (from backend)
  imageData?: string // Base64 full image (loaded on demand)
  filename?: string
  fileSize?: number
  fileType?: string
  shareCode?: string
  isDownloading?: boolean
  downloadProgress?: number
}

export interface LocalGroupMessage {
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

// Note: ImageMessageReceived, FileMessageReceived, FileDownloadStarted, etc. are now imported from plugin API

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
  // Send direct message
  static async sendMessage(toPeerId: string, message: string): Promise<string> {
    return GigiP2p.messaging_send_message({ toPeerId, message })
  }

  // Send direct message by nickname (preferred)
  static async sendMessageToNickname(
    nickname: string,
    message: string
  ): Promise<string> {
    return GigiP2p.messaging_send_message_to_nickname({
      nickname,
      message,
    })
  }

  // Send group share message to peer
  static async sendShareGroupMessage(
    targetNickname: string,
    groupId: string,
    groupName: string
  ): Promise<string> {
    return GigiP2p.messaging_send_direct_share_group_message({
      nickname: targetNickname,
      groupId,
      groupName,
    })
  }

  // Get connected peers
  static async getPeers(): Promise<Peer[]> {
    return GigiP2p.messaging_get_peers()
  }

  // Set nickname
  static async setNickname(nickname: string): Promise<void> {
    return GigiP2p.messaging_set_nickname({ nickname })
  }

  // Join a group
  static async joinGroup(groupId: string): Promise<void> {
    return GigiP2p.messaging_join_group({ groupId })
  }

  // Send group message
  static async sendGroupMessage(
    groupId: string,
    message: string
  ): Promise<string> {
    return GigiP2p.messaging_send_group_message({ groupId, message })
  }

  // Share a file
  static async shareFile(filePath: string): Promise<string> {
    return GigiP2p.messaging_share_file({ filePath })
  }

  // Request/download a file
  static async requestFile(
    fileId: string,
    fromPeerId: string
  ): Promise<string> {
    return GigiP2p.messaging_request_file({ fileId, fromPeerId })
  }

  // Request/download file by nickname (preferred)
  static async requestFileFromNickname(
    nickname: string,
    shareCode: string
  ): Promise<string> {
    return GigiP2p.messaging_request_file_from_nickname({
      nickname,
      shareCode,
    })
  }

  // Cancel download
  static async cancelDownload(downloadId: string): Promise<void> {
    return GigiP2p.messaging_cancel_download({ downloadId })
  }

  // Get shared files
  static async getSharedFiles(): Promise<FileInfo[]> {
    return GigiP2p.messaging_get_shared_files()
  }

  // Remove shared file
  static async removeSharedFile(shareCode: string): Promise<void> {
    return GigiP2p.messaging_remove_shared_file({ shareCode })
  }

  // Save shared files to disk
  static async saveSharedFiles(): Promise<void> {
    return GigiP2p.messaging_save_shared_files()
  }

  // Get current peer ID
  static async getPeerId(): Promise<string> {
    return GigiP2p.get_peer_id()
  }

  // Get public key
  static async getPublicKey(): Promise<string> {
    return GigiP2p.messaging_get_public_key()
  }

  // Get active downloads
  static async getActiveDownloads(): Promise<DownloadProgress[]> {
    return GigiP2p.messaging_get_active_downloads()
  }

  // Update configuration
  static async updateConfig(config: Config): Promise<void> {
    return GigiP2p.messaging_update_config({ config })
  }

  // Get current configuration
  static async getConfig(): Promise<Config> {
    return GigiP2p.messaging_get_config()
  }

  // Select image file using dialog
  static async selectImageFile(): Promise<string | null> {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: isAndroid()
          ? undefined
          : [
              {
                name: 'Image Files',
                extensions: ['png', 'jpg', 'jpeg', 'gif', 'bmp', 'webp'],
              },
            ],
      } as any)

      return selected || null
    } catch (error) {
      console.error('Failed to open file dialog:', error)
      return null
    }
  }

  // Select any file using dialog
  static async selectAnyFile(): Promise<string | null> {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: isAndroid()
          ? undefined
          : [
              {
                name: 'All Files',
                extensions: ['*'],
              },
              {
                name: 'Documents',
                extensions: ['pdf', 'doc', 'docx', 'txt', 'rtf'],
              },
              {
                name: 'Images',
                extensions: ['png', 'jpg', 'jpeg', 'gif', 'bmp', 'webp'],
              },
              {
                name: 'Videos',
                extensions: ['mp4', 'avi', 'mov', 'mkv', 'webm'],
              },
              {
                name: 'Audio',
                extensions: ['mp3', 'wav', 'flac', 'aac', 'ogg'],
              },
              {
                name: 'Archives',
                extensions: ['zip', 'rar', '7z', 'tar', 'gz'],
              },
            ],
      } as any)

      return selected || null
    } catch (error) {
      console.error('Failed to open file dialog:', error)
      return null
    }
  }

  // Get file info from path - using backend Rust function
  static async getFileInfo(
    filePath: string
  ): Promise<{ name: string; size: number; type: string }> {
    try {
      const response = await GigiP2p.messaging_get_file_info({ filePath })
      return {
        name: response.name,
        size: response.size,
        type: response.mime_type,
      }
    } catch (error) {
      console.error('Failed to get file info:', error)
      // Fallback - simple extraction from path
      const fileName = filePath.split(/[\\/]/).pop() || 'unknown'
      const ext = fileName.split('.').pop()?.toLowerCase() || ''
      const mimeType = this.getMimeTypeFromExtension(ext)

      return {
        name: fileName,
        size: 0,
        type: mimeType,
      }
    }
  }

  // Simple MIME type detection
  private static getMimeTypeFromExtension(ext: string): string {
    const mimeTypes: { [key: string]: string } = {
      // Images
      png: 'image/png',
      jpg: 'image/jpeg',
      jpeg: 'image/jpeg',
      gif: 'image/gif',
      bmp: 'image/bmp',
      webp: 'image/webp',

      // Videos
      mp4: 'video/mp4',
      avi: 'video/x-msvideo',
      mov: 'video/quicktime',
      mkv: 'video/x-matroska',
      webm: 'video/webm',

      // Audio
      mp3: 'audio/mpeg',
      wav: 'audio/wav',
      flac: 'audio/flac',
      aac: 'audio/aac',
      ogg: 'audio/ogg',

      // Documents
      pdf: 'application/pdf',
      doc: 'application/msword',
      docx: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
      txt: 'text/plain',
      rtf: 'application/rtf',

      // Archives
      zip: 'application/zip',
      rar: 'application/x-rar-compressed',
      '7z': 'application/x-7z-compressed',
      tar: 'application/x-tar',
      gz: 'application/gzip',
    }

    return mimeTypes[ext] || 'application/octet-stream'
  }

  // Send file message using file path
  static async sendFileMessageWithPath(
    nickname: string,
    filePath: string
  ): Promise<{ messageId: string; imageData?: string }> {
    // Directly pass the content URI to backend - it will handle it with android-fs plugin
    const response = await GigiP2p.messaging_send_file_message_with_path({
      nickname,
      filePath,
    })

    // Parse the response to extract message ID and optional base64 image data
    const parts = response.split('|')
    const messageId = parts[0]
    // Check if parts[1] is actual image data (not thumbnail metadata)
    const imageData =
      parts[1] && !parts[1].startsWith('thumbnail:') ? parts[1] : undefined
    return { messageId, imageData }
  }

  // Extract filename from content URI
  private static extractFileNameFromUri(uri: string): string {
    try {
      // Try to extract from content URI format:
      // content://com.android.providers.media.documents/document/image%3A19573
      if (uri.includes('image%3A')) {
        const id = uri.split('image%3A')[1]?.split('&')[0]
        return id ? `gigi_image_${id}.jpg` : 'image.jpg'
      }
      if (uri.includes('video%3A')) {
        const id = uri.split('video%3A')[1]?.split('&')[0]
        return id ? `gigi_video_${id}.mp4` : 'video.mp4'
      }
      if (uri.includes('audio%3A')) {
        const id = uri.split('audio%3A')[1]?.split('&')[0]
        return id ? `gigi_audio_${id}.mp3` : 'audio.mp3'
      }

      // Try to extract from the URI path
      const lastSlash = uri.lastIndexOf('/')
      if (lastSlash > -1) {
        const filename = uri.slice(lastSlash + 1)
        try {
          // URL decode
          return decodeURIComponent(filename)
        } catch {
          return filename
        }
      }
    } catch (error) {
      console.error('Failed to extract filename from URI:', error)
    }

    return 'file.dat'
  }

  // Send group file message using file path
  static async sendGroupFileMessageWithPath(
    groupId: string,
    filePath: string
  ): Promise<{ messageId: string; imageData?: string }> {
    // Directly pass the content URI to backend - it will handle it with android-fs plugin
    const response = await GigiP2p.messaging_send_group_file_message_with_path({
      groupId,
      filePath,
    })

    // Parse the response to extract message ID and optional base64 image data
    const parts = response.split('|')
    const messageId = parts[0]
    // Check if parts[1] is actual image data (not thumbnail metadata)
    const imageData =
      parts[1] && !parts[1].startsWith('thumbnail:') ? parts[1] : undefined
    return { messageId, imageData }
  }

  // Utility function to try get peer ID from private key
  static async tryGetPeerId(priv_key: Uint8Array): Promise<string> {
    return GigiP2p.try_get_peer_id({ privKey: Array.from(priv_key) })
  }

  // Clear app data from backend
  static async clearAppData(): Promise<void> {
    return GigiP2p.clear_app_data()
  }

  // Emit current P2P state (for catching up on missed events)
  static async emitCurrentState(): Promise<void> {
    return GigiP2p.emit_current_state()
  }

  // Get image data from local file path
  static async getImageData(filePath: string): Promise<string> {
    return GigiP2p.messaging_get_image_data({ filePath })
  }

  // Get message history with a peer
  static async getMessageHistory(peerId: string): Promise<Message[]> {
    return GigiP2p.messaging_get_message_history({ peerId })
  }

  // Get messages from backend with pagination
  static async getMessages(
    peerId: string,
    options?: { limit?: number; offset?: number }
  ): Promise<{
    messages: Message[]
    peerId: string
    limit: number
    offset: number
  }> {
    const result = await GigiP2p.get_messages({
      peerId,
      limit: options?.limit || 50,
      offset: options?.offset || 0,
    })
    return typeof result === 'string' ? JSON.parse(result) : result
  }

  // Search messages in backend
  static async searchMessages(
    query: string,
    peerId?: string
  ): Promise<{ messages: Message[]; query: string; peerId?: string }> {
    const result = await GigiP2p.search_messages({
      query,
      peerId: peerId || null,
    })
    return typeof result === 'string' ? JSON.parse(result) : result
  }

  // Clear messages and delete thumbnail files for incoming images
  static async clearMessagesWithThumbnails(peerId: string): Promise<number> {
    return await GigiP2p.clear_messages_with_thumbnails({ peerId })
  }

  // Get thumbnail for a shared file
  static async getFileThumbnail(filePath: string): Promise<string> {
    return await GigiP2p.get_file_thumbnail({ filePath })
  }

  // Get full-size image for a shared file by file path (for received files)
  static async getFullImageByPath(filePath: string): Promise<string> {
    return await GigiP2p.get_full_image_by_path({ filePath })
  }

  // Get full-size image for a shared file by share code (for sent files)
  static async getFullImage(shareCode: string): Promise<string> {
    return await GigiP2p.get_full_image({ shareCode })
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
