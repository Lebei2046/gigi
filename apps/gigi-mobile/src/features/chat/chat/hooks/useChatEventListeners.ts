import { useEffect } from 'react'
import type { Peer } from '@/utils/messaging'
import { MessagingClient, MessagingEvents } from '@/utils/messaging'
import { useAppDispatch } from '@/store'
import { addLog } from '@/store/logsSlice'
import {
  addPeer,
  removePeer,
  updateDirectMessage,
  updateGroupMessage,
} from '@/store/chatSlice'
import { updateLatestMessage, ensureMilliseconds } from '@/utils/chatUtils'

/**
 * Hook for setting up all messaging event listeners
 * Handles peer connections, messages, images, files, and group shares
 */
export function useChatEventListeners() {
  const dispatch = useAppDispatch()

  useEffect(() => {
    try {
      const handlePeerConnected = (peer: any) => {
        try {
          if (peer && peer.id && peer.nickname) {
            dispatch(addPeer(peer))
            dispatch(
              addLog({
                event: 'peer_connected',
                data: `Peer connected: ${peer.nickname} (${peer.id})`,
                type: 'info',
              })
            )
          }
        } catch (error) {
          console.error('Error in handlePeerConnected:', error)
        }
      }

      const handlePeerDisconnected = (peer: any) => {
        try {
          if (peer && peer.id && peer.nickname) {
            dispatch(removePeer(peer.id))
            dispatch(
              addLog({
                event: 'peer_disconnected',
                data: `Peer disconnected: ${peer.nickname} (${peer.id})`,
                type: 'info',
              })
            )
          }
        } catch (error) {
          console.error('Error in handlePeerDisconnected:', error)
        }
      }

      const handleMessageReceived = (message: any) => {
        // Don't process if we're currently in a chat room
        const currentPath = window.location.pathname
        if (currentPath.startsWith('/chat/') && currentPath !== '/chat') {
          console.log(
            '🔄 Skipping message processing in Chat component - ChatRoom is active'
          )
          return
        }

        // Save message to localStorage
        try {
          const historyKey = `chat_history_${message.from_peer_id}`
          const savedHistory = localStorage.getItem(historyKey)
          let history = savedHistory ? JSON.parse(savedHistory) : []
          const newMessage = {
            ...message,
            isOutgoing: false,
          }
          history = [...history, newMessage]
          localStorage.setItem(historyKey, JSON.stringify(history))
        } catch (error) {
          console.error('Failed to save received message to history:', error)
        }

        const timestampMs = message.timestamp
          ? ensureMilliseconds(message.timestamp)
          : Date.now()

        updateLatestMessage(
          message.from_peer_id,
          message.content,
          timestampMs,
          false,
          false,
          true
        )

        dispatch(
          updateDirectMessage({
            from_peer_id: message.from_peer_id,
            content: message.content,
            timestamp: timestampMs,
          })
        )

        dispatch(
          addLog({
            event: 'message_received',
            data: `Direct message from ${message.from_nickname}: ${message.content}`,
            type: 'info',
          })
        )
      }

      const handleGroupMessageReceived = (message: any) => {
        const currentPath = window.location.pathname
        if (currentPath.startsWith('/chat/') && currentPath !== '/chat') {
          console.log(
            '🔄 Skipping group message processing in Chat component - ChatRoom is active'
          )
          return
        }

        try {
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
        } catch (error) {
          console.error(
            'Failed to save received group message to history:',
            error
          )
        }

        const timestampMs = message.timestamp
          ? ensureMilliseconds(message.timestamp)
          : Date.now()

        updateLatestMessage(
          message.group_id,
          message.content,
          timestampMs,
          false,
          true,
          true
        )

        dispatch(
          updateGroupMessage({
            group_id: message.group_id,
            content: message.content,
            timestamp: timestampMs,
          })
        )

        dispatch(
          addLog({
            event: 'group_message_received',
            data: `Group message from ${message.from_nickname} in group ${message.group_id}: ${message.content}`,
            type: 'info',
          })
        )
      }

      const handleImageMessageReceived = (messageData: any) => {
        dispatch(
          updateDirectMessage({
            from_peer_id: messageData.from_peer_id,
            content: `📷 Image: ${messageData.filename}`,
            timestamp: messageData.timestamp,
          })
        )
      }

      const handleGroupImageMessageReceived = (messageData: any) => {
        const messageText = messageData.download_error
          ? `❌ Image: ${messageData.filename}`
          : `⬇️ Image: ${messageData.filename}`

        dispatch(
          updateGroupMessage({
            group_id: messageData.group_id,
            content: messageText,
            timestamp: messageData.timestamp,
          })
        )
      }

      const handleFileMessageReceived = (messageData: any) => {
        dispatch(
          updateDirectMessage({
            from_peer_id: messageData.from_peer_id,
            content: `📎 File: ${messageData.filename}`,
            timestamp: messageData.timestamp,
          })
        )
      }

      const handleFileDownloadCompleted = (messageData: any) => {
        dispatch(
          updateDirectMessage({
            from_peer_id: messageData.from_nickname,
            content: `📷 Image: ${messageData.filename}`,
            timestamp: messageData.timestamp,
          })
        )
      }

      const handleGroupShareReceived = (shareMessage: any) => {
        dispatch(
          addLog({
            event: 'group_share_received',
            data: `Group share received from ${shareMessage.from_nickname}`,
            type: 'info',
          })
        )
      }

      // Register event listeners
      MessagingEvents.on('peer-connected', handlePeerConnected)
      MessagingEvents.on('peer-disconnected', handlePeerDisconnected)
      MessagingEvents.on('message-received', handleMessageReceived)
      MessagingEvents.on('group-message', handleGroupMessageReceived)
      MessagingEvents.on('image-message-received', handleImageMessageReceived)
      MessagingEvents.on(
        'group-image-message-received',
        handleGroupImageMessageReceived
      )
      MessagingEvents.on('file-message-received', handleFileMessageReceived)
      MessagingEvents.on('file-download-completed', handleFileDownloadCompleted)
      MessagingEvents.on('group-share-received', handleGroupShareReceived)

      return () => {
        MessagingEvents.off('peer-connected', handlePeerConnected)
        MessagingEvents.off('peer-disconnected', handlePeerDisconnected)
        MessagingEvents.off('message-received', handleMessageReceived)
        MessagingEvents.off('group-message', handleGroupMessageReceived)
        MessagingEvents.off(
          'image-message-received',
          handleImageMessageReceived
        )
        MessagingEvents.off(
          'group-image-message-received',
          handleGroupImageMessageReceived
        )
        MessagingEvents.off('file-message-received', handleFileMessageReceived)
        MessagingEvents.off(
          'file-download-completed',
          handleFileDownloadCompleted
        )
        MessagingEvents.off('group-share-received', handleGroupShareReceived)
        console.log('🧹 Cleaned up Chat event listeners')
      }
    } catch (error) {
      console.error('Error setting up event listeners:', error)
      return () => {}
    }
  }, [dispatch])
}
