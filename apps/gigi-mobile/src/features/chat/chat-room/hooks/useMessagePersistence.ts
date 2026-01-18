import { useEffect, useRef } from 'react'
import { useAppSelector } from '@/store'
import { updateLatestMessage } from '@/utils/conversationUtils'
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

  // Note: Message history is now stored in backend SQLite
  // We only save metadata (latest message) to localStorage
  // The useEffect that saved full messages to localStorage has been removed

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
