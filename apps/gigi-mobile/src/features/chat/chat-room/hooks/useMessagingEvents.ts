import { useEffect, useRef } from 'react'
import { MessagingClient, MessagingEvents } from '@/utils/messaging'
import { useAppDispatch, useAppSelector } from '@/store'
import { addLog } from '@/store/logsSlice'
import type { Message } from '@/store/chatRoomSlice'
import {
  handleDirectMessage,
  handleGroupMessage,
  addMessage,
  addImageMessage,
  addGroupImageMessage,
  updateMessage,
  updateGroupMessage,
  generateMessageId,
} from '@/store/chatRoomSlice'
import { updateLatestMessage, ensureMilliseconds } from '@/utils/chatUtils'
import {
  createIncomingImageMessage,
  createIncomingFileMessage,
  getFileIcon,
} from '@/utils/messageHelpers'

// Module-level storage to persist across component remounts
const downloadIdMapping = new Map<string, string>()
const sentMessages = new Set<string>()

interface UseMessagingEventsParams {
  chatId?: string
  isGroupChat: boolean
  peer?: { id: string; nickname: string } | null
  group?: { id: string; name: string } | null
  messages: Message[]
}

export function useMessagingEvents({
  chatId,
  isGroupChat,
  peer,
  group,
  messages,
}: UseMessagingEventsParams) {
  const dispatch = useAppDispatch()
  const sentMessagesRef = useRef<Set<string>>(sentMessages)
  const downloadIdToMessageIdRef =
    useRef<Map<string, string>>(downloadIdMapping)
  const messagesRef = useRef<Message[]>(messages)

  // Keep messages ref updated
  useEffect(() => {
    messagesRef.current = messages
  }, [messages])

  useEffect(() => {
    if (!chatId) return

    const handleMessageReceived = (message: any) => {
      if (message.id && sentMessagesRef.current.has(message.id)) {
        sentMessagesRef.current.delete(message.id)
        return
      }

      if (!isGroupChat && message.from_peer_id === peer?.id) {
        dispatch(
          handleDirectMessage({
            from_peer_id: message.from_peer_id,
            from_nickname: message.from_nickname,
            content: message.content,
            timestamp: message.timestamp,
          })
        )

        setTimeout(() => {
          updateLatestMessage(
            peer!.id,
            message.content,
            ensureMilliseconds(message.timestamp),
            false,
            false,
            true
          )
        }, 0)

        dispatch(
          addLog({
            event: 'chat_message_received',
            data: `Message from ${message.from_nickname}: ${message.content}`,
            type: 'info',
          })
        )
      } else if (isGroupChat && message.group_id === group?.id) {
        dispatch(
          handleGroupMessage({
            id: message.id,
            from_peer_id: message.from_peer_id,
            from_nickname: message.from_nickname,
            content: message.content,
            timestamp: message.timestamp,
            group_id: message.group_id,
          })
        )

        setTimeout(() => {
          updateLatestMessage(
            group!.id,
            message.content,
            ensureMilliseconds(message.timestamp),
            false,
            true,
            true
          )
        }, 0)

        dispatch(
          addLog({
            event: 'group_message_received',
            data: `Group message from ${message.from_nickname}: ${message.content}`,
            type: 'info',
          })
        )
      } else {
        // Save message for later (background chat)
        saveMessageForLater(message)
      }
    }

    const handleImageMessageReceived = (messageData: any) => {
      console.log('🖼️ Image message received (direct):', messageData)
      if (chatId && messageData.from_peer_id === chatId && !isGroupChat) {
        const messageId = generateMessageId()
        const imageMessage = createIncomingImageMessage(messageData, false)

        if (messageData.download_id) {
          downloadIdToMessageIdRef.current.set(
            messageData.download_id,
            messageId
          )
          console.log('📝 Set downloadId mapping for auto-download:', {
            downloadId: messageData.download_id,
            messageId,
          })
        }

        dispatch(addImageMessage(imageMessage))
      }
    }

    const handleGroupImageMessageReceived = (messageData: any) => {
      console.log('🖼️ Image message received (group):', messageData)
      if (chatId && messageData.group_id === chatId && isGroupChat) {
        const messageId = generateMessageId()
        const imageMessage = createIncomingImageMessage(messageData, true)

        if (messageData.download_id) {
          downloadIdToMessageIdRef.current.set(
            messageData.download_id,
            messageId
          )
          console.log('📝 Set downloadId mapping for group auto-download:', {
            downloadId: messageData.download_id,
            messageId,
          })
        }

        dispatch(addGroupImageMessage(imageMessage))
      }
    }

    const handleFileMessageReceived = (messageData: any) => {
      if (chatId && messageData.from_peer_id === chatId && !isGroupChat) {
        const fileMessage = createIncomingFileMessage(messageData, false, false)
        console.log('📨 Adding file message to ChatRoom:', fileMessage)
        dispatch(addMessage(fileMessage))
      }
    }

    const handleGroupFileMessageReceived = (messageData: any) => {
      if (chatId && messageData.group_id === chatId && isGroupChat) {
        const fileMessage = createIncomingFileMessage(messageData, true, true)
        dispatch(addGroupImageMessage(fileMessage))
      }
    }

    const handleFileDownloadStarted = (data: any) => {
      console.log('🚀 File download started:', data)
      let messageId = downloadIdToMessageIdRef.current.get(data.download_id)
      console.log(
        '🔍 Current downloadIdToMessageIdRef mapping:',
        Array.from(downloadIdToMessageIdRef.current.entries())
      )
      console.log(
        '🔍 Lookup downloadId -> messageId:',
        data.download_id,
        '=>',
        messageId
      )

      if (!messageId) {
        const message = messagesRef.current.find(
          msg => msg.downloadId === data.download_id
        )
        if (message) {
          messageId = message.id
          downloadIdToMessageIdRef.current.set(data.download_id, message.id)
          console.log(
            '🔄 Found message by downloadId and updated mapping:',
            messageId
          )
        }
      }

      if (messageId) {
        const updateFields = {
          id: messageId,
          downloadId: data.download_id,
          isDownloading: true,
          downloadProgress: 0,
        }

        if (isGroupChat) {
          dispatch(updateGroupMessage(updateFields))
        } else {
          dispatch(updateMessage(updateFields))
        }
      }
    }

    const handleFileDownloadProgress = (data: any) => {
      console.log('📊 File download progress:', data)
      let messageId = downloadIdToMessageIdRef.current.get(data.download_id)

      if (!messageId) {
        console.warn('⚠️ No messageId found for downloadId:', data.download_id)
        return
      }

      const progressText = `⬇️ Downloading ${data.filename}: ${data.progress_percent.toFixed(1)}%`
      const updateFields = {
        id: messageId,
        downloadId: data.download_id,
        content: progressText,
        downloadProgress: data.progress_percent,
        isDownloading: data.progress_percent < 100,
      }

      if (isGroupChat) {
        dispatch(updateGroupMessage(updateFields))
      } else {
        dispatch(updateMessage(updateFields))
      }
    }

    const handleFileDownloadCompleted = async (data: any) => {
      console.log('✅ File download completed:', data)
      let messageId = downloadIdToMessageIdRef.current.get(data.download_id)

      if (!messageId) {
        console.warn('⚠️ No messageId found for downloadId:', data.download_id)
        return
      }

      const message = messages.find(msg => msg.id === messageId)
      const originalFilename = message?.filename
      const actualFilename =
        originalFilename ||
        (data.filename === 'Loading...' || data.filename.includes('...')
          ? data.path.split(/[\\/]/).pop() || data.filename
          : data.filename)

      const isImage =
        data.file_type?.startsWith('image/') ||
        /\.(jpg|jpeg|png|gif|bmp|webp)$/i.test(actualFilename)

      let updatedMessage: Partial<Message> = {
        isDownloading: false,
        downloadProgress: 100,
      }

      if (isImage) {
        try {
          // If thumbnail_filename is available, load thumbnail using it
          if (data.thumbnail_filename) {
            console.log(
              '📸 Loading thumbnail from file:',
              data.thumbnail_filename
            )
            const thumbnailData = await MessagingClient.getImageData(
              data.thumbnail_filename
            )
            console.log(
              '✅ Thumbnail data received, length:',
              thumbnailData?.length
            )
            updatedMessage = {
              ...updatedMessage,
              content: `📷 Image: ${actualFilename}`,
              thumbnailData,
              filePath: data.path, // Set filePath for full image viewing
            }
          } else {
            // Fallback: load full image
            console.log('📸 Getting image data from path:', data.path)
            const imageData = await MessagingClient.getImageData(data.path)
            console.log('✅ Image data received, length:', imageData?.length)
            updatedMessage = {
              ...updatedMessage,
              content: `📷 Image: ${actualFilename}`,
              thumbnailData: imageData, // Use as thumbnail for display
              filePath: data.path, // Set filePath for full image viewing
            }
          }
          console.log('📝 Updated message with image data:', updatedMessage)
        } catch (error) {
          console.error(
            '❌ Failed to convert downloaded image to base64:',
            error
          )
          // Don't set imageData on error - just show filename with file:// fallback
          updatedMessage = {
            ...updatedMessage,
            content: `📷 Image: ${actualFilename}`,
          }
          console.log(
            '📝 Updated message with fallback (no imageData):',
            updatedMessage
          )
        }
      } else {
        const fileIcon = getFileIcon(data.file_type, actualFilename)
        updatedMessage = {
          ...updatedMessage,
          content: `${fileIcon} File: ${actualFilename} (Downloaded)`,
        }
      }

      const updateFields = {
        id: messageId,
        downloadId: data.download_id,
        ...updatedMessage,
      }

      console.log('🔧 Dispatching update with fields:', updateFields)

      if (isGroupChat) {
        dispatch(updateGroupMessage(updateFields))
      } else {
        dispatch(updateMessage(updateFields))
      }

      if (messageId) {
        downloadIdToMessageIdRef.current.delete(data.download_id)
      }
    }

    const handleFileDownloadFailed = (data: any) => {
      console.log('❌ File download failed:', data)
      let messageId = downloadIdToMessageIdRef.current.get(data.download_id)

      if (!messageId) {
        console.warn('⚠️ No messageId found for downloadId:', data.download_id)
        return
      }

      const fileIcon = getFileIcon(data.file_type, data.filename)
      const errorMessage = `❌ ${fileIcon} File: ${data.filename} (Download failed: ${data.error})`
      const updateFields = {
        id: messageId,
        content: errorMessage,
        isDownloading: false,
        downloadProgress: undefined,
      }

      if (isGroupChat) {
        dispatch(updateGroupMessage(updateFields))
      } else {
        dispatch(updateMessage(updateFields))
      }

      downloadIdToMessageIdRef.current.delete(data.download_id)
    }

    MessagingEvents.on('message-received', handleMessageReceived)
    MessagingEvents.on('group-message', handleMessageReceived)
    MessagingEvents.on('image-message-received', handleImageMessageReceived)
    MessagingEvents.on(
      'group-image-message-received',
      handleGroupImageMessageReceived
    )
    MessagingEvents.on('file-message-received', handleFileMessageReceived)
    MessagingEvents.on(
      'group-file-message-received',
      handleGroupFileMessageReceived
    )
    MessagingEvents.on('file-download-started', handleFileDownloadStarted)
    MessagingEvents.on('file-download-progress', handleFileDownloadProgress)
    MessagingEvents.on('file-download-completed', handleFileDownloadCompleted)
    MessagingEvents.on('file-download-failed', handleFileDownloadFailed)

    return () => {
      MessagingEvents.off('message-received', handleMessageReceived)
      MessagingEvents.off('group-message', handleMessageReceived)
      MessagingEvents.off('image-message-received', handleImageMessageReceived)
      MessagingEvents.off(
        'group-image-message-received',
        handleGroupImageMessageReceived
      )
      MessagingEvents.off('file-message-received', handleFileMessageReceived)
      MessagingEvents.off(
        'group-file-message-received',
        handleGroupFileMessageReceived
      )
      MessagingEvents.off('file-download-started', handleFileDownloadStarted)
      MessagingEvents.off('file-download-progress', handleFileDownloadProgress)
      MessagingEvents.off(
        'file-download-completed',
        handleFileDownloadCompleted
      )
      MessagingEvents.off('file-download-failed', handleFileDownloadFailed)

      // Don't clear module-level storage - persist across remounts
      // sentMessagesRef.current.clear()
      // downloadIdToMessageIdRef.current.clear()
    }
  }, [chatId, isGroupChat, peer?.id, group?.id, dispatch])

  return { sentMessagesRef, downloadIdToMessageIdRef }
}

function saveMessageForLater(message: any) {
  try {
    if (message.group_id) {
      const historyKey = `chat_history_group_${message.group_id}`
      const savedHistory = localStorage.getItem(historyKey)
      let history = savedHistory ? JSON.parse(savedHistory) : []

      const newMessage = {
        ...message,
        isOutgoing: false,
        isGroup: true,
      }
      history = [...history, newMessage]
      localStorage.setItem(historyKey, JSON.stringify(history))

      setTimeout(() => {
        updateLatestMessage(
          message.group_id,
          message.content,
          ensureMilliseconds(message.timestamp),
          false,
          true,
          true
        )
      }, 0)
    } else {
      const historyKey = `chat_history_${message.from_peer_id}`
      const savedHistory = localStorage.getItem(historyKey)
      let history = savedHistory ? JSON.parse(savedHistory) : []

      const newMessage = {
        ...message,
        isOutgoing: false,
        isGroup: false,
      }
      history = [...history, newMessage]
      localStorage.setItem(historyKey, JSON.stringify(history))

      setTimeout(() => {
        updateLatestMessage(
          message.from_peer_id,
          message.content,
          ensureMilliseconds(message.timestamp),
          false,
          false,
          true
        )
      }, 0)
    }
  } catch (error) {
    console.error('Failed to save message for later processing:', error)
  }
}
