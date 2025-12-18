import { useEffect, useState, useRef } from 'react'
import { useNavigate, useLocation, useParams } from 'react-router-dom'
import type { Peer } from '@/utils/messaging'
import { MessagingClient, MessagingEvents } from '@/utils/messaging'
import { useAppDispatch } from '@/store'
import { addLog } from '@/store/logsSlice'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { ArrowLeft as BackIcon } from 'lucide-react'
import { formatShortPeerId } from '@/utils/peerUtils'
import {
  updateChatInfo,
  updateLatestMessage,
  getChatInfo,
  resetUnreadCount,
  getGroup,
  getAllChats,
} from '@/utils/chatUtils'
import type { Group } from '@/models/db'

interface Message {
  id: string
  from_peer_id: string
  from_nickname: string
  content: string
  timestamp: number
  isOutgoing: boolean
  isGroup?: boolean // true for group messages, false for direct messages
}

export default function ChatRoom() {
  const navigate = useNavigate()
  const location = useLocation()
  const { id } = useParams<{ id: string }>()
  const dispatch = useAppDispatch()
  const messagesEndRef = useRef<HTMLDivElement>(null)

  const [peer, setPeer] = useState<Peer | null>(location.state?.peer || null)
  const [group, setGroup] = useState<Group | null>(null)
  const [messages, setMessages] = useState<Message[]>([])
  const [newMessage, setNewMessage] = useState('')
  const [isLoading, setIsLoading] = useState(true)
  const [sending, setSending] = useState(false)
  const [isGroupChat, setIsGroupChat] = useState(false)
  const unreadResetDone = useRef(false)
  const saveTimeoutRef = useRef<NodeJS.Timeout>()

  // Scroll to bottom when new messages arrive
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  useEffect(() => {
    // Determine if this is a group chat or peer chat
    const initializeChatRoom = async () => {
      if (!id) {
        navigate('/chat')
        return
      }

      // Check if this is a group by looking up the group in database
      const groupData = await getGroup(id)

      if (groupData) {
        // This is a group chat
        setIsGroupChat(true)
        setGroup(groupData)
        setPeer(null)

        dispatch(
          addLog({
            event: 'group_chat_room_opened',
            data: `Opened group chat: ${groupData.name}`,
            type: 'info',
          })
        )

        // Join the group topic if we've joined the group (not if we own it)
        // joined=false means we own the group, joined=true means we were invited
        if (groupData.joined) {
          try {
            await MessagingClient.joinGroup(id)
          } catch (error) {
            console.error('Failed to join group topic:', error)
          }
        } else {
          // Group owners also need to subscribe to their own group topics to send messages
          try {
            await MessagingClient.joinGroup(id)
          } catch (error) {
            console.error('Failed to subscribe to own group topic:', error)
          }
        }
      } else if (location.state?.peer) {
        // This is a peer chat
        setIsGroupChat(false)
        setPeer(location.state.peer)
        setGroup(null)

        dispatch(
          addLog({
            event: 'chat_room_opened',
            data: `Opened chat with ${location.state.peer.nickname}`,
            type: 'info',
          })
        )
      } else {
        // Invalid chat room
        navigate('/chat')
        return
      }

      setIsLoading(false)
    }

    initializeChatRoom()
  }, [id, location.state?.peer, navigate, dispatch])

  useEffect(() => {
    if (isLoading || (!peer && !group)) return

    const chatId = isGroupChat ? group?.id : peer?.id
    const chatName = isGroupChat ? group?.name : peer?.nickname
    if (!chatId || !chatName) return

    // Load message history from localStorage with performance optimizations
    const loadMessageHistory = () => {
      try {
        const historyKey = isGroupChat
          ? `chat_history_group_${chatId}`
          : `chat_history_${chatId}`
        const savedHistory = localStorage.getItem(historyKey)
        if (savedHistory) {
          // Use requestAnimationFrame to avoid blocking UI
          requestAnimationFrame(() => {
            try {
              const history = JSON.parse(savedHistory)
              // Batch process timestamps for better performance
              const normalizedHistory = history.map((msg: any) => ({
                ...msg,
                timestamp:
                  msg.timestamp < 1000000000000
                    ? msg.timestamp * 1000
                    : msg.timestamp,
                isGroup: isGroupChat,
              }))

              // Only load last 100 messages initially for better performance
              setMessages(normalizedHistory.slice(-100))
            } catch (parseError) {
              console.error('Failed to parse message history:', parseError)
            }
          })
        }
      } catch (error) {
        console.error('Failed to load message history:', error)
      }
    }

    loadMessageHistory()

    // Optimize chat info initialization - reduce database calls
    const initializeChatInfo = async () => {
      try {
        const existingChat = await getChatInfo(chatId)
        if (!existingChat) {
          // Only create new chat entry if it doesn't exist
          await updateChatInfo(
            chatId,
            chatName,
            '',
            Date.now(),
            false,
            isGroupChat
          )
        } else if (!unreadResetDone.current) {
          // Reset unread count when user opens the chat (only once)
          console.log(
            `🏠 Entering chat room for ${chatId}, resetting unread count`
          )
          await resetUnreadCount(chatId)
          unreadResetDone.current = true

          // Trigger refresh after a short delay (reduced timeout)
          setTimeout(() => {
            window.dispatchEvent(new Event('focus'))
          }, 50)
        }
      } catch (error) {
        console.error('Failed to initialize chat info:', error)
      }
    }

    initializeChatInfo()

    // Debounced save to localStorage to reduce write operations
    const saveMessageHistory = (messages: Message[]) => {
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current)
      }

      saveTimeoutRef.current = setTimeout(() => {
        try {
          const historyKey = isGroupChat
            ? `chat_history_group_${chatId}`
            : `chat_history_${chatId}`
          localStorage.setItem(historyKey, JSON.stringify(messages))
        } catch (error) {
          console.error('Failed to save message history:', error)
        }
      }, 300) // Debounce for 300ms
    }

    // Listen for new messages (both direct and group)
    const handleMessageReceived = (message: any) => {
      // Process direct messages for peer chats
      if (!isGroupChat && message.from_peer_id === peer?.id) {
        handleDirectMessage(message, saveMessageHistory)
      }
      // Process group messages for group chats
      else if (isGroupChat && message.group_id === group?.id) {
        handleGroupMessage(message, saveMessageHistory)
      }
    }

    const handleDirectMessage = (
      message: any,
      saveMessageHistory: (messages: Message[]) => void
    ) => {
      // Convert timestamp to milliseconds if needed
      const timestampMs =
        message.timestamp < 1000000000000
          ? message.timestamp * 1000
          : message.timestamp

      const newMessage = {
        ...message,
        timestamp: timestampMs,
        isOutgoing: false,
        isGroup: false,
      }

      setMessages(prev => {
        const updatedMessages = [...prev, newMessage]
        saveMessageHistory(updatedMessages)
        return updatedMessages
      })

      // Defer IndexedDB update to avoid blocking UI
      setTimeout(() => {
        updateLatestMessage(
          peer!.id,
          newMessage.content,
          newMessage.timestamp,
          false, // Incoming message
          false,
          true // Increment unread for incoming messages
        )
      }, 0)

      dispatch(
        addLog({
          event: 'chat_message_received',
          data: `Message from ${message.from_nickname}: ${message.content}`,
          type: 'info',
        })
      )
    }

    const handleGroupMessage = (
      message: any,
      saveMessageHistory: (messages: Message[]) => void
    ) => {
      // Convert timestamp to milliseconds if needed
      const timestampMs =
        message.timestamp < 1000000000000
          ? message.timestamp * 1000
          : message.timestamp

      const newMessage = {
        id: message.id || crypto.randomUUID(),
        from_peer_id: message.from_peer_id,
        from_nickname: message.from_nickname,
        content: message.content,
        timestamp: timestampMs,
        isOutgoing: false,
        isGroup: true,
      }

      setMessages(prev => {
        const updatedMessages = [...prev, newMessage]
        saveMessageHistory(updatedMessages)
        return updatedMessages
      })

      // Defer IndexedDB update to avoid blocking UI
      setTimeout(() => {
        updateLatestMessage(
          group!.id,
          newMessage.content,
          newMessage.timestamp,
          false, // Incoming message
          true,
          true // Increment unread for incoming messages
        )
      }, 0)

      dispatch(
        addLog({
          event: 'group_message_received',
          data: `Group message from ${message.from_nickname}: ${message.content}`,
          type: 'info',
        })
      )
    }

    // Listen for both direct and group messages
    MessagingEvents.on('message-received', handleMessageReceived)
    MessagingEvents.on('group-message', handleMessageReceived)

    return () => {
      MessagingEvents.off('message-received', handleMessageReceived)
      MessagingEvents.off('group-message', handleMessageReceived)
      console.log('🧹 Cleaned up ChatRoom event listeners')

      // Clear any pending save timeout
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current)
      }

      // Final update to ensure everything is saved when leaving chat
      if (messages.length > 0) {
        const lastMessage = messages[messages.length - 1]
        updateLatestMessage(
          chatId,
          lastMessage.content,
          lastMessage.timestamp,
          lastMessage.isOutgoing,
          isGroupChat,
          false // Don't increment unread when leaving chat room
        )
      }
    }
  }, [peer?.id, group?.id, isGroupChat, isLoading, dispatch])

  const handleSendMessage = async () => {
    if (!newMessage.trim() || sending) return
    if (!isGroupChat && !peer) return
    if (isGroupChat && !group) return

    setSending(true)
    let messageToAdd: Message | null = null
    const timestamp = Date.now()

    try {
      if (isGroupChat) {
        // Send group message

        messageToAdd = {
          id: Date.now().toString(),
          from_peer_id: 'me', // This would be the current user's peer ID
          from_nickname: 'Me', // This would be the current user's nickname
          content: newMessage.trim(),
          timestamp: timestamp,
          isOutgoing: true,
          isGroup: true,
        }

        // Add message to local state immediately
        setMessages(prev => {
          const updatedMessages = [...prev, messageToAdd!]
          // Defer localStorage save to avoid blocking
          setTimeout(() => {
            try {
              const historyKey = `chat_history_group_${group!.id}`
              localStorage.setItem(historyKey, JSON.stringify(updatedMessages))
            } catch (error) {
              console.error('Failed to save group message history:', error)
            }
          }, 0)

          // Defer IndexedDB update to avoid blocking
          setTimeout(() => {
            updateLatestMessage(
              group!.id,
              messageToAdd!.content,
              messageToAdd!.timestamp,
              true,
              true,
              false // Don't increment unread for outgoing messages
            )
          }, 0)

          return updatedMessages
        })

        // Send group message via backend
        const result = await MessagingClient.sendGroupMessage(
          group!.id,
          newMessage.trim()
        )

        dispatch(
          addLog({
            event: 'group_message_sent',
            data: `Group message sent to ${group!.name}: ${newMessage.trim()}`,
            type: 'info',
          })
        )
      } else {
        // Send direct message

        messageToAdd = {
          id: Date.now().toString(),
          from_peer_id: peer!.id,
          from_nickname: peer!.nickname,
          content: newMessage.trim(),
          timestamp: timestamp,
          isOutgoing: true,
          isGroup: false,
        }

        setMessages(prev => {
          const updatedMessages = [...prev, messageToAdd!]
          // Defer localStorage save to avoid blocking
          setTimeout(() => {
            try {
              const historyKey = `chat_history_${peer!.id}`
              localStorage.setItem(historyKey, JSON.stringify(updatedMessages))
            } catch (error) {
              console.error('Failed to save message history:', error)
            }
          }, 0)

          // Defer IndexedDB update to avoid blocking
          setTimeout(() => {
            updateLatestMessage(
              peer!.id,
              messageToAdd!.content,
              messageToAdd!.timestamp,
              true,
              false,
              false // Don't increment unread for outgoing messages
            )
          }, 0)

          return updatedMessages
        })

        // Send via backend using nickname (preferred method)
        const result = await MessagingClient.sendMessageToNickname(
          peer!.nickname,
          newMessage.trim()
        )

        dispatch(
          addLog({
            event: 'message_sent',
            data: `Message sent to ${peer!.nickname}: ${newMessage.trim()}`,
            type: 'info',
          })
        )
      }

      setNewMessage('')
    } catch (error) {
      console.error('❌ Failed to send message:', error)

      // Remove the message from local state if sending failed
      if (messageToAdd) {
        setMessages(prev => prev.filter(msg => msg.id !== messageToAdd!.id))
      }

      const targetName = isGroupChat ? group!.name : peer!.nickname
      const eventType = isGroupChat
        ? 'group_message_send_error'
        : 'message_send_error'

      dispatch(
        addLog({
          event: eventType,
          data: `Failed to send message to ${targetName}: ${error}`,
          type: 'error',
        })
      )
    } finally {
      setSending(false)
    }
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      // Defer send operation to avoid blocking input
      setTimeout(handleSendMessage, 0)
    }
  }

  const goBack = () => {
    navigate('/chat')
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-gray-500">Loading chat...</p>
      </div>
    )
  }

  if (!peer && !group) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-gray-500">Chat not found</p>
      </div>
    )
  }

  const chatTitle = isGroupChat ? group?.name : peer?.nickname
  const chatId = isGroupChat ? group?.id : peer?.id

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center p-4 border-b bg-white">
        <Button variant="ghost" size="sm" onClick={goBack} className="mr-3">
          <BackIcon size={20} />
        </Button>
        <div>
          <h2 className="text-lg font-semibold">
            {isGroupChat ? `👥 ${chatTitle}` : chatTitle}
          </h2>
          <p className="text-sm text-gray-500">
            {isGroupChat
              ? `Group • ${formatShortPeerId(chatId)}`
              : `Direct • ${formatShortPeerId(chatId)}`}
          </p>
        </div>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        {messages.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <p className="text-gray-500">
              {isGroupChat
                ? 'No messages in this group yet. Start the conversation!'
                : 'No messages yet. Start a conversation!'}
            </p>
          </div>
        ) : (
          messages.slice(-100).map(
            (
              message // Only render last 100 messages for performance
            ) => (
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
                  {/* Show sender name for group messages and incoming direct messages */}
                  {!message.isOutgoing &&
                    (message.isGroup || !message.isGroup) && (
                      <p className="text-xs font-medium mb-1 opacity-70">
                        {message.isGroup && '👥'} {message.from_nickname}
                      </p>
                    )}
                  <p className="text-sm break-words">{message.content}</p>
                  <p className="text-xs mt-1 opacity-60">
                    {new Date(message.timestamp).toLocaleTimeString([], {
                      hour: '2-digit',
                      minute: '2-digit',
                    })}
                  </p>
                </div>
              </div>
            )
          )
        )}
        <div ref={messagesEndRef} />
      </div>

      {/* Message Input */}
      <div className="border-t bg-white p-4">
        <div className="flex gap-2">
          <Input
            value={newMessage}
            onChange={e => setNewMessage(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder={
              isGroupChat
                ? `Message ${group?.name}...`
                : `Message ${peer?.nickname}...`
            }
            disabled={sending}
            className="flex-1"
          />
          <Button
            onClick={handleSendMessage}
            disabled={!newMessage.trim() || sending}
            size="sm"
          >
            {sending ? 'Sending...' : 'Send'}
          </Button>
        </div>
      </div>
    </div>
  )
}
