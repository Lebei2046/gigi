import { useEffect, useState, useRef } from 'react'
import { useNavigate, useLocation } from 'react-router-dom'
import type { Peer } from '@/utils/messaging'
import { MessagingClient, MessagingEvents } from '@/utils/messaging'
import { useAppDispatch } from '@/store'
import { addLog } from '@/store/logsSlice'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { ArrowLeft as BackIcon } from 'lucide-react'

interface Message {
  id: string
  from_peer_id: string
  from_nickname: string
  content: string
  timestamp: number
  isOutgoing: boolean
}

export default function ChatRoom() {
  const navigate = useNavigate()
  const location = useLocation()
  const dispatch = useAppDispatch()
  const messagesEndRef = useRef<HTMLDivElement>(null)

  const [peer, setPeer] = useState<Peer | null>(location.state?.peer || null)
  const [messages, setMessages] = useState<Message[]>([])
  const [newMessage, setNewMessage] = useState('')
  const [isLoading, setIsLoading] = useState(true)
  const [sending, setSending] = useState(false)

  // Scroll to bottom when new messages arrive
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  useEffect(() => {
    if (!peer) {
      navigate('/chat')
      return
    }

    console.log('ChatRoom mounted for peer:', peer)
    dispatch(
      addLog({
        event: 'chat_room_opened',
        data: `Opened chat with ${peer.nickname}`,
        type: 'info',
      })
    )

    // Load message history from localStorage
    const loadMessageHistory = () => {
      try {
        const historyKey = `chat_history_${peer.id}`
        const savedHistory = localStorage.getItem(historyKey)
        if (savedHistory) {
          const history = JSON.parse(savedHistory)
          console.log(
            '📚 Loaded message history for',
            peer.nickname,
            ':',
            history
          )
          setMessages(history)
        }
      } catch (error) {
        console.error('Failed to load message history:', error)
      }
      setIsLoading(false)
    }

    loadMessageHistory()

    // Save messages to localStorage
    const saveMessageHistory = (messages: Message[]) => {
      try {
        const historyKey = `chat_history_${peer.id}`
        localStorage.setItem(historyKey, JSON.stringify(messages))
      } catch (error) {
        console.error('Failed to save message history:', error)
      }
    }

    // Listen for new messages
    const handleMessageReceived = (message: any) => {
      console.log('New message in ChatRoom:', message)

      // Only process messages from this peer
      if (message.from_peer_id === peer.id) {
        const newMessage = {
          ...message,
          isOutgoing: false,
        }

        setMessages(prev => {
          const updatedMessages = [...prev, newMessage]
          saveMessageHistory(updatedMessages)
          return updatedMessages
        })

        dispatch(
          addLog({
            event: 'chat_message_received',
            data: `Message from ${message.from_nickname}: ${message.content}`,
            type: 'info',
          })
        )
      }
    }

    MessagingEvents.on('message-received', handleMessageReceived)

    return () => {
      MessagingEvents.off('message-received', handleMessageReceived)
    }
  }, [peer, navigate, dispatch])

  const handleSendMessage = async () => {
    if (!newMessage.trim() || !peer || sending) return

    setSending(true)
    try {
      console.log('Sending message:', newMessage)

      // Add message to local state immediately
      const messageToAdd = {
        id: Date.now().toString(),
        from_peer_id: peer.id,
        from_nickname: peer.nickname,
        content: newMessage.trim(),
        timestamp: Date.now(),
        isOutgoing: true,
      }

      setMessages(prev => {
        const updatedMessages = [...prev, messageToAdd]
        // Save to localStorage
        try {
          const historyKey = `chat_history_${peer.id}`
          localStorage.setItem(historyKey, JSON.stringify(updatedMessages))
        } catch (error) {
          console.error('Failed to save message history:', error)
        }
        return updatedMessages
      })

      // Send via backend using nickname (preferred method)
      const result = await MessagingClient.sendMessageToNickname(
        peer.nickname,
        newMessage.trim()
      )
      console.log(
        '✅ Message sent successfully to',
        peer.nickname,
        'result:',
        result
      )

      dispatch(
        addLog({
          event: 'message_sent',
          data: `Message sent to ${peer.nickname}: ${newMessage.trim()}`,
          type: 'info',
        })
      )

      setNewMessage('')
    } catch (error) {
      console.error('❌ Failed to send message to', peer.nickname, ':', error)

      // Remove the message from local state if sending failed
      setMessages(prev => prev.filter(msg => msg.id !== messageToAdd.id))

      dispatch(
        addLog({
          event: 'message_send_error',
          data: `Failed to send message to ${peer.nickname}: ${error}`,
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
      handleSendMessage()
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

  if (!peer) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-gray-500">Peer not found</p>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center p-4 border-b bg-white">
        <Button variant="ghost" size="sm" onClick={goBack} className="mr-3">
          <BackIcon size={20} />
        </Button>
        <div>
          <h2 className="text-lg font-semibold">{peer.nickname}</h2>
          <p className="text-sm text-gray-500">{peer.id}</p>
        </div>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        {messages.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <p className="text-gray-500">
              No messages yet. Start a conversation!
            </p>
          </div>
        ) : (
          messages.map(message => (
            <div
              key={message.id}
              className={`flex ${message.isOutgoing ? 'justify-end' : 'justify-start'}`}
            >
              <div
                className={`max-w-xs px-4 py-2 rounded-lg ${
                  message.isOutgoing
                    ? 'bg-blue-500 text-white'
                    : 'bg-gray-200 text-gray-900'
                }`}
              >
                {!message.isOutgoing && (
                  <p className="text-xs font-medium mb-1 opacity-70">
                    {message.from_nickname}
                  </p>
                )}
                <p className="text-sm break-words">{message.content}</p>
              </div>
            </div>
          ))
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
            placeholder={`Message ${peer.nickname}...`}
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
