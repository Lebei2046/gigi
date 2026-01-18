import { useRef, useEffect, memo } from 'react'
import type { Message } from '@/store/chatRoomSlice'
import { MessageBubble } from './bubbles'

interface MessageListProps {
  messages: Message[]
  isGroupChat: boolean
  onDownloadRequest?: (
    messageId: string,
    shareCode: string,
    filename: string
  ) => void
}

function MessageList({
  messages,
  isGroupChat,
  onDownloadRequest,
}: MessageListProps) {
  const messagesEndRef = useRef<HTMLDivElement>(null)

  // Scroll to bottom when new messages arrive
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  if (messages.length === 0) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-gray-500">
          {isGroupChat
            ? 'No messages in this group yet. Start the conversation!'
            : 'No messages yet. Start a conversation!'}
        </p>
      </div>
    )
  }

  return (
    <>
      {messages.slice(-100).map(message => (
        <div
          key={message.id}
          className={`flex ${message.isOutgoing ? 'justify-end' : 'justify-start'}`}
        >
          <div
            className={`max-w-xs px-4 py-2 rounded-lg ${
              message.isOutgoing
                ? 'bg-blue-500 text-white'
                : message.isGroup
                  ? 'bg-green-100 text-gray-900 border border-green-200'
                  : 'bg-gray-200 text-gray-900'
            }`}
          >
            <MessageBubble
              message={message}
              onDownloadRequest={onDownloadRequest}
            />
            <p className="text-xs mt-1 opacity-60">
              {new Date(message.timestamp).toLocaleTimeString([], {
                hour: '2-digit',
                minute: '2-digit',
              })}
            </p>
          </div>
        </div>
      ))}
      <div ref={messagesEndRef} />
    </>
  )
}

// Memoize to prevent unnecessary re-renders
export default memo(MessageList, (prevProps, nextProps) => {
  return (
    prevProps.messages.length === nextProps.messages.length &&
    prevProps.messages.every((msg, i) => msg.id === nextProps.messages[i]?.id)
  )
})
