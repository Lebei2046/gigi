import type { Message } from '@/store/chatRoomSlice'
import { generateMessageId } from '@/store/chatRoomSlice'

/**
 * Create a text message object
 */
export function createTextMessage(
  content: string,
  timestamp: number,
  isOutgoing: boolean,
  isGroup: boolean,
  peerOrGroup: { id: string; nickname: string }
): Message {
  return {
    id: generateMessageId(),
    from_peer_id: peerOrGroup.id,
    from_nickname: peerOrGroup.nickname,
    content,
    timestamp,
    isOutgoing,
    isGroup,
  }
}

/**
 * Create an image message object for received messages
 */
export function createIncomingImageMessage(
  messageData: {
    from_peer_id: string
    from_nickname: string
    share_code: string
    filename: string
    file_size: number
    file_type: string
    timestamp: number
    download_error?: string
    download_id?: string
  },
  isGroup: boolean
): Message {
  return {
    id: generateMessageId(),
    shareCode: messageData.share_code,
    downloadId: messageData.download_id,
    from_peer_id: messageData.from_peer_id,
    from_nickname: messageData.from_nickname,
    content: messageData.download_error
      ? `❌ Image: ${messageData.filename} (Download failed)`
      : `⬇️ Image: ${messageData.filename} (Downloading...)`,
    timestamp: messageData.timestamp,
    isOutgoing: false,
    isGroup,
    messageType: 'image',
    imageId: messageData.share_code,
    imageData: undefined,
    filename: messageData.filename,
  }
}

/**
 * Create a file message object for received messages
 */
export function createIncomingFileMessage(
  messageData: {
    from_peer_id: string
    from_nickname: string
    share_code: string
    filename: string
    file_size: number
    file_type: string
    timestamp: number
    download_error?: string
  },
  isGroup: boolean,
  autoDownload: boolean = false
): Message {
  return {
    id: generateMessageId(),
    from_peer_id: messageData.from_peer_id,
    from_nickname: messageData.from_nickname,
    content: messageData.download_error
      ? `❌ File: ${messageData.filename} (Download failed)`
      : autoDownload
        ? `⬇️ File: ${messageData.filename} (Downloading...)`
        : `📄 File: ${messageData.filename}`,
    timestamp: messageData.timestamp,
    isOutgoing: false,
    isGroup,
    messageType: 'file',
    shareCode: messageData.share_code,
    filename: messageData.filename,
    fileSize: messageData.file_size,
    fileType: messageData.file_type,
    isDownloading: autoDownload,
    downloadProgress: 0,
  }
}

/**
 * Create an outgoing image message
 */
export function createOutgoingImageMessage(
  filename: string,
  timestamp: number,
  isGroup: boolean
): Message {
  return {
    id: generateMessageId(),
    from_peer_id: 'me',
    from_nickname: 'Me',
    content: `📷 Image: ${filename} (Processing...)`,
    timestamp,
    isOutgoing: true,
    isGroup,
    messageType: 'image',
    imageId: generateMessageId(),
    imageData: undefined,
    filename,
  }
}

/**
 * Create an outgoing file message
 */
export function createOutgoingFileMessage(
  fileInfo: { name: string; size: number; type: string },
  timestamp: number,
  isGroup: boolean
): Message {
  return {
    id: generateMessageId(),
    from_peer_id: 'me',
    from_nickname: 'Me',
    content: `📎 File: ${fileInfo.name} (Processing...)`,
    timestamp,
    isOutgoing: true,
    isGroup,
    messageType: 'file',
    filename: fileInfo.name,
    fileSize: fileInfo.size,
    fileType: fileInfo.type,
    isDownloading: false,
    downloadProgress: 0,
  }
}

/**
 * Update outgoing image message with response data
 */
export function updateOutgoingImageMessage(
  originalMessage: Message,
  messageId: string,
  imageData?: string,
  filename: string
): Message {
  const updatedMessage = {
    ...originalMessage,
    id: messageId,
  }

  if (imageData) {
    updatedMessage.imageData = `data:image/${filename.split('.').pop()};base64,${imageData}`
    updatedMessage.content = `📷 Image: ${filename}`
  } else {
    updatedMessage.content = `📎 File: ${filename}`
  }

  return updatedMessage
}

/**
 * Update outgoing file message with response data
 */
export function updateOutgoingFileMessage(
  originalMessage: Message,
  messageId: string,
  filename: string
): Message {
  return {
    ...originalMessage,
    id: messageId,
    content: `📎 File: ${filename}`,
  }
}
