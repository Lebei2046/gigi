import { useEffect, useRef } from 'react'
import { useAppSelector } from '@/store'
import { updateLatestMessage } from '@/utils/chatUtils'
import type { Message } from '@/store/chatRoomSlice'

interface UseMessagePersistenceParams {
  chatId?: string
  isGroupChat: boolean
  isLoading: boolean
  messages: Message[]
}

export function useMessagePersistence({
  chatId,
  isGroupChat,
  isLoading,
  messages,
}: UseMessagePersistenceParams) {
  const saveTimeoutRef = useRef<number | null>(null)

  // Save messages to localStorage when they change
  useEffect(() => {
    if (!chatId || isLoading) return

    const saveTimeout = setTimeout(() => {
      try {
        const historyKey = isGroupChat
          ? `chat_history_group_${chatId}`
          : `chat_history_${chatId}`
        localStorage.setItem(historyKey, JSON.stringify(messages))
      } catch (error) {
        console.error('Failed to save message history:', error)
      }
    }, 300)

    return () => {
      if (saveTimeout) {
        clearTimeout(saveTimeout)
      }
    }
  }, [messages, chatId, isGroupChat, isLoading])

  const saveFinalMessage = () => {
    if (messages.length > 0 && chatId) {
      const lastMessage = messages[messages.length - 1]
      updateLatestMessage(
        chatId,
        lastMessage.content,
        lastMessage.timestamp,
        lastMessage.isOutgoing,
        isGroupChat,
        false
      )
    }
  }

  const clearSaveTimeout = () => {
    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current)
    }
  }

  return { saveFinalMessage, clearSaveTimeout }
}
