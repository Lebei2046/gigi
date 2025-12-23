import { useEffect, useRef } from 'react'
import { useNavigate, useLocation, useParams } from 'react-router-dom'
import { MessagingClient, MessagingEvents } from '@/utils/messaging'
import { useAppDispatch, useAppSelector } from '@/store'
import { addLog } from '@/store/logsSlice'
import {
  initializeChatRoomAsync,
  loadMessageHistoryAsync,
  initializeChatInfoAsync,
  sendMessageAsync,
  setNewMessage,
  handleDirectMessage,
  handleGroupMessage,
  addMessage,
  addImageMessage,
  addGroupImageMessage,
  updateMessage,
  updateGroupMessage,
  removeMessage,
  resetChatRoomState,
  generateMessageId,
  type Message,
} from '@/store/chatRoomSlice'
import { updateLatestMessage } from '@/utils/chatUtils'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { ArrowLeft as BackIcon } from 'lucide-react'
import { formatShortPeerId } from '@/utils/peerUtils'

export default function ChatRoom() {
  const navigate = useNavigate()
  const location = useLocation()
  const { id } = useParams<{ id: string }>()
  const dispatch = useAppDispatch()
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const saveTimeoutRef = useRef<NodeJS.Timeout>()
  const sentMessagesRef = useRef<Set<string>>(new Set())

  // Redux state
  const {
    peer,
    group,
    messages,
    newMessage,
    isLoading,
    sending,
    isGroupChat,
    unreadResetDone,
    error,
    chatId,
    chatName,
  } = useAppSelector(state => state.chatRoom)

  // Scroll to bottom when new messages arrive
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  // Initialize chat room
  useEffect(() => {
    if (!id) {
      navigate('/chat')
      return
    }

    dispatch(initializeChatRoomAsync({ id, peer: location.state?.peer }))
      .unwrap()
      .then(result => {
        if (result.isGroupChat && result.group) {
          dispatch(
            addLog({
              event: 'group_chat_room_opened',
              data: `Opened group chat: ${result.group.name}`,
              type: 'info',
            })
          )
        } else if (!result.isGroupChat && result.peer) {
          dispatch(
            addLog({
              event: 'chat_room_opened',
              data: `Opened chat with ${result.peer.nickname}`,
              type: 'info',
            })
          )
        } else {
          navigate('/chat')
        }
      })
      .catch(error => {
        console.error('Failed to initialize chat room:', error)
        console.error('Error details:', {
          id,
          peer: location.state?.peer,
          errorMessage: error.message || error,
          errorStack: error.stack,
        })

        // Reset state before navigating back
        dispatch(resetChatRoomState())
        navigate('/chat')
      })
  }, [id, location.state?.peer, navigate, dispatch])

  // Load message history and initialize chat info when chat room is ready
  useEffect(() => {
    if (isLoading || !chatId || !chatName) return

    // Load message history
    dispatch(loadMessageHistoryAsync({ chatId, isGroupChat }))

    // Initialize chat info
    dispatch(
      initializeChatInfoAsync({
        chatId,
        chatName,
        isGroupChat,
        unreadResetDone,
      })
    )
  }, [isLoading, chatId, chatName, isGroupChat, unreadResetDone, dispatch])

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
    }, 300) // Debounce for 300ms

    return () => {
      if (saveTimeout) {
        clearTimeout(saveTimeout)
      }
    }
  }, [messages, chatId, isGroupChat, isLoading])

  // Message handling and event listeners
  useEffect(() => {
    if (isLoading || !chatId || !chatName) return

    // Listen for new messages (both direct and group)
    const handleMessageReceived = (message: any) => {
      // Skip duplicate messages that we already sent optimistically
      if (message.id && sentMessagesRef.current.has(message.id)) {
        sentMessagesRef.current.delete(message.id) // Clean up
        return
      }

      // Process direct messages for peer chats
      if (!isGroupChat && message.from_peer_id === peer?.id) {
        dispatch(
          handleDirectMessage({
            from_peer_id: message.from_peer_id,
            from_nickname: message.from_nickname,
            content: message.content,
            timestamp: message.timestamp,
          })
        )

        // Defer IndexedDB update to avoid blocking UI
        setTimeout(() => {
          updateLatestMessage(
            peer!.id,
            message.content,
            message.timestamp < 1000000000000
              ? message.timestamp * 1000
              : message.timestamp,
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
      // Process group messages for group chats
      else if (isGroupChat && message.group_id === group?.id) {
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

        // Defer IndexedDB update to avoid blocking UI
        setTimeout(() => {
          updateLatestMessage(
            group!.id,
            message.content,
            message.timestamp < 1000000000000
              ? message.timestamp * 1000
              : message.timestamp,
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
      // Handle messages that are not for the current chat - save them for later
      else {
        // Save message to localStorage for when user returns to main chat screen
        try {
          if (message.group_id) {
            // Group message not for current group
            const historyKey = `chat_history_group_${message.group_id}`
            const savedHistory = localStorage.getItem(historyKey)
            let history = savedHistory ? JSON.parse(savedHistory) : []

            const newMessage = {
              ...message,
              isOutgoing: false, // Received message
              isGroup: true, // Group message
            }
            history = [...history, newMessage]
            localStorage.setItem(historyKey, JSON.stringify(history))

            // Update IndexedDB to increment unread count
            setTimeout(() => {
              updateLatestMessage(
                message.group_id,
                message.content,
                message.timestamp < 1000000000000
                  ? message.timestamp * 1000
                  : message.timestamp,
                false, // Incoming message
                true, // Group message
                true // Increment unread
              )
            }, 0)
          } else {
            // Direct message not for current peer
            const historyKey = `chat_history_${message.from_peer_id}`
            const savedHistory = localStorage.getItem(historyKey)
            let history = savedHistory ? JSON.parse(savedHistory) : []

            const newMessage = {
              ...message,
              isOutgoing: false, // Received message
              isGroup: false, // Direct message
            }
            history = [...history, newMessage]
            localStorage.setItem(historyKey, JSON.stringify(history))

            // Update IndexedDB to increment unread count
            setTimeout(() => {
              updateLatestMessage(
                message.from_peer_id,
                message.content,
                message.timestamp < 1000000000000
                  ? message.timestamp * 1000
                  : message.timestamp,
                false, // Incoming message
                false, // Direct message
                true // Increment unread
              )
            }, 0)
          }

          dispatch(
            addLog({
              event: 'message_saved_for_later',
              data: `Saved message from ${message.from_nickname} for later: ${message.content}`,
              type: 'info',
            })
          )
        } catch (error) {
          console.error('Failed to save message for later processing:', error)
        }
      }
    }

    // Listen for direct messages
    MessagingEvents.on('message-received', handleMessageReceived)

    // Listen for group messages separately
    const handleGroupMessageReceived = (message: any) => {
      handleMessageReceived(message)
    }
    MessagingEvents.on('group-message', handleGroupMessageReceived)

    // Listen for image messages
    const handleImageMessageReceived = (messageData: any) => {
      if (chatId && messageData.from_peer_id === chatId && !isGroupChat) {
        const imageMessage: Message = {
          id: messageData.share_code, // Use share_code as ID
          from_peer_id: messageData.from_peer_id,
          from_nickname: messageData.from_nickname,
          content: messageData.download_error
            ? `❌ Image: ${messageData.filename} (Download failed)`
            : `⬇️ Image: ${messageData.filename} (Downloading...)`,
          timestamp: messageData.timestamp,
          isOutgoing: false,
          isGroup: false,
          messageType: 'image',
          imageId: messageData.share_code, // Use share_code as imageId
          imageData: undefined, // Will be set after download
          filename: messageData.filename,
        }
        dispatch(addImageMessage(imageMessage))
      }
    }

    // Listen for group image messages
    const handleGroupImageMessageReceived = (messageData: any) => {
      if (chatId && messageData.group_id === chatId && isGroupChat) {
        const imageMessage: Message = {
          id: messageData.share_code, // Use share_code as ID
          from_peer_id: messageData.from_peer_id,
          from_nickname: messageData.from_nickname,
          content: messageData.download_error
            ? `❌ Image: ${messageData.filename} (Download failed)`
            : `⬇️ Image: ${messageData.filename} (Downloading...)`,
          timestamp: messageData.timestamp,
          isOutgoing: false,
          isGroup: true,
          messageType: 'image',
          imageId: messageData.share_code, // Use share_code as imageId
          imageData: undefined, // Will be set after download
          filename: messageData.filename,
        }
        dispatch(addGroupImageMessage(imageMessage))
      }
    }

    // Listen for file download events
    const handleFileDownloadStarted = (data: any) => {
      // This event is now handled automatically by the auto-download in DirectFileShareMessage handler
      // No need to update message here since it's already created with downloading status
    }

    const handleFileDownloadProgress = (data: any) => {
      // Update message content with progress
      const progressText = `⬇️ Downloading ${data.filename}: ${data.progress_percent.toFixed(1)}%`

      if (isGroupChat) {
        dispatch(
          updateGroupMessage({
            id: data.share_code,
            content: progressText,
          })
        )
      } else if (!isGroupChat && data.from_peer_id === chatId) {
        dispatch(
          updateMessage({
            id: data.share_code,
            content: progressText,
          })
        )
      }
    }

    const handleFileDownloadCompleted = async (data: any) => {
      try {
        // Convert local file to base64 for display
        const imageData = await MessagingClient.getImageData(data.path)

        // Extract actual filename from path if data.filename is wrong
        const actualFilename =
          data.filename === 'Loading...' || data.filename.includes('...')
            ? data.path.split(/[\\/]/).pop() || data.filename
            : data.filename

        const updatedMessage: Partial<Message> = {
          content: `📷 Image: ${actualFilename}`,
          imageData, // Base64 data URL
        }

        if (isGroupChat) {
          dispatch(
            updateGroupMessage({
              id: data.share_code,
              ...updatedMessage,
            })
          )
        } else if (!isGroupChat && data.from_peer_id === chatId) {
          dispatch(
            updateMessage({
              id: data.share_code,
              ...updatedMessage,
            })
          )
        }
      } catch (error) {
        console.error('Failed to convert downloaded image to base64:', error)
        // Fallback to file path if conversion fails
        const fallbackMessage: Partial<Message> = {
          content: `📷 Image: ${data.filename}`,
          imageData: `file://${data.path}`,
        }

        if (isGroupChat) {
          dispatch(
            updateGroupMessage({
              id: data.share_code,
              ...fallbackMessage,
            })
          )
        } else if (!isGroupChat && data.from_peer_id === chatId) {
          dispatch(
            updateMessage({
              id: data.share_code,
              ...fallbackMessage,
            })
          )
        }
      }
    }

    const handleFileDownloadFailed = (data: any) => {
      const errorMessage = `❌ Image: ${data.filename} (Download failed: ${data.error})`

      if (isGroupChat) {
        dispatch(
          updateGroupMessage({
            id: data.share_code,
            content: errorMessage,
          })
        )
      } else if (!isGroupChat && data.from_peer_id === chatId) {
        dispatch(
          updateMessage({
            id: data.share_code,
            content: errorMessage,
          })
        )
      }
    }

    MessagingEvents.on('image-message-received', handleImageMessageReceived)
    MessagingEvents.on(
      'group-image-message-received',
      handleGroupImageMessageReceived
    )

    // Register download event listeners
    MessagingEvents.on('file-download-started', handleFileDownloadStarted)
    MessagingEvents.on('file-download-progress', handleFileDownloadProgress)
    MessagingEvents.on('file-download-completed', handleFileDownloadCompleted)
    MessagingEvents.on('file-download-failed', handleFileDownloadFailed)

    return () => {
      MessagingEvents.off('message-received', handleMessageReceived)
      MessagingEvents.off('group-message', handleGroupMessageReceived)
      MessagingEvents.off('image-message-received', handleImageMessageReceived)
      MessagingEvents.off(
        'group-image-message-received',
        handleGroupImageMessageReceived
      )
      MessagingEvents.off('file-download-started', handleFileDownloadStarted)
      MessagingEvents.off('file-download-progress', handleFileDownloadProgress)
      MessagingEvents.off(
        'file-download-completed',
        handleFileDownloadCompleted
      )
      MessagingEvents.off('file-download-failed', handleFileDownloadFailed)
      console.log('🧹 Cleaned up ChatRoom event listeners')

      // Clear any pending save timeout
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current)
      }

      // Final update to ensure everything is saved when leaving chat
      if (messages.length > 0 && chatId) {
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
  }, [
    chatId,
    isGroupChat,
    isLoading,
    dispatch,
    peer,
    group,
    messages.length,
    chatId,
    chatName,
  ])

  const handleImageSelectWithDialog = async () => {
    if (!isGroupChat && !peer) return
    if (isGroupChat && !group) return

    try {
      // Use dialog to select image file
      const filePath = await MessagingClient.selectImageFile()
      if (!filePath) {
        return // User cancelled the dialog
      }

      const timestamp = Date.now()
      const filename = filePath.split(/[\\/]/).pop() || 'image'

      // Add image message to local state immediately for optimistic UI

      if (isGroupChat) {
        const imageMessage: Message = {
          id: generateMessageId(),
          from_peer_id: 'me',
          from_nickname: 'Me',
          content: `📷 Image: ${filename} (Processing...)`,
          timestamp,
          isOutgoing: true,
          isGroup: true,
          messageType: 'image',
          imageId: generateMessageId(),
          imageData: undefined, // Will be set after backend returns base64 data
          filename,
        }

        // Track this message to avoid duplicates
        sentMessagesRef.current.add(imageMessage.id)
        // Dispatch via Redux
        dispatch(addImageMessage(imageMessage))

        // Send image using file path and get response with image data
        try {
          const response = await MessagingClient.sendGroupImageMessageWithPath(
            group!.id,
            filePath
          )

          // Update the message with the returned message ID and image data
          const updatedMessage: Message = {
            ...imageMessage,
            id: response.messageId,
            imageData: `data:image/${filename.split('.').pop()};base64,${response.imageData}`,
            content: `📷 Image: ${filename}`,
          }

          // Update the message in the store
          dispatch(
            updateGroupMessage({
              id: imageMessage.id,
              content: updatedMessage.content,
              imageData: updatedMessage.imageData,
              newId: updatedMessage.id,
            })
          )
        } catch (error) {
          console.error('❌ Failed to send group image:', error)
          // Update message to show error
          dispatch(
            updateGroupMessage({
              id: imageMessage.id,
              content: `❌ Image: ${filename} (Failed to send)`,
            })
          )
        }
      } else {
        const imageMessage: Message = {
          id: generateMessageId(),
          from_peer_id: peer!.id,
          from_nickname: peer!.nickname,
          content: `📷 Image: ${filename} (Processing...)`,
          timestamp,
          isOutgoing: true,
          isGroup: false,
          messageType: 'image',
          imageId: generateMessageId(),
          imageData: undefined, // Will be set after backend returns base64 data
          filename,
        }

        // Track this message to avoid duplicates
        sentMessagesRef.current.add(imageMessage.id)
        // Dispatch via Redux
        dispatch(addImageMessage(imageMessage))

        // Send image using file path and get response with image data
        try {
          const response = await MessagingClient.sendImageMessageWithPath(
            peer!.nickname,
            filePath
          )

          // Update the message with the returned message ID and image data
          const updatedMessage: Message = {
            ...imageMessage,
            id: response.messageId,
            imageData: `data:image/${filename.split('.').pop()};base64,${response.imageData}`,
            content: `📷 Image: ${filename}`,
          }

          // Update message in the store
          dispatch(
            updateMessage({
              id: imageMessage.id,
              content: updatedMessage.content,
              imageData: updatedMessage.imageData,
              newId: updatedMessage.id,
            })
          )
        } catch (error) {
          console.error('❌ Failed to send direct image:', error)
          // Update message to show error
          dispatch(
            updateMessage({
              id: imageMessage.id,
              content: `❌ Image: ${filename} (Failed to send)`,
            })
          )
        }
      }
    } catch (error) {
      console.error('Failed to send image:', error)
    }
  }

  const handleSendMessage = async () => {
    if (!newMessage.trim() || sending) return
    if (!isGroupChat && !peer) return
    if (isGroupChat && !group) return

    let messageToAdd: Message | null = null
    const timestamp = Date.now()

    try {
      // Add message to local state immediately for optimistic UI
      if (isGroupChat) {
        messageToAdd = {
          id: generateMessageId(),
          from_peer_id: 'me',
          from_nickname: 'Me',
          content: newMessage.trim(),
          timestamp,
          isOutgoing: true,
          isGroup: true,
        }

        // Track this message to avoid duplicates
        sentMessagesRef.current.add(messageToAdd.id)
        // Dispatch via Redux
        dispatch(addMessage(messageToAdd))
      } else {
        messageToAdd = {
          id: generateMessageId(),
          from_peer_id: peer!.id,
          from_nickname: peer!.nickname,
          content: newMessage.trim(),
          timestamp,
          isOutgoing: true,
          isGroup: false,
        }

        // Track this message to avoid duplicates
        sentMessagesRef.current.add(messageToAdd.id)
        // Dispatch via Redux
        dispatch(addMessage(messageToAdd))
      }

      // Send message via Redux async thunk
      await dispatch(
        sendMessageAsync({
          content: newMessage.trim(),
          isGroupChat,
          peer,
          group,
        })
      ).unwrap()

      // Log success
      const targetName = isGroupChat ? group!.name : peer!.nickname
      const eventType = isGroupChat ? 'group_message_sent' : 'message_sent'

      dispatch(
        addLog({
          event: eventType,
          data: `${isGroupChat ? 'Group message' : 'Message'} sent to ${targetName}: ${newMessage.trim()}`,
          type: 'info',
        })
      )
    } catch (error) {
      console.error('❌ Failed to send message:', error)

      // Remove the message from local state if sending failed
      if (messageToAdd) {
        dispatch(removeMessage(messageToAdd.id))
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
    }
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSendMessage()
    }
  }

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    dispatch(setNewMessage(e.target.value))
  }

  const goBack = () => {
    dispatch(resetChatRoomState())
    navigate('/chat')
  }

  // Reset chat room state when unmounting
  useEffect(() => {
    return () => {
      dispatch(resetChatRoomState())
      sentMessagesRef.current.clear()
    }
  }, [dispatch])

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

  const chatTitle = chatName || (isGroupChat ? group?.name : peer?.nickname)

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
                  {/* Render message content based on type */}
                  {message.messageType === 'image' ? (
                    <div className="flex flex-col gap-2">
                      {message.imageData && (
                        <img
                          src={message.imageData}
                          alt={message.filename}
                          className="max-w-xs max-h-48 rounded-lg object-cover"
                        />
                      )}
                      <p className="text-sm break-words">{message.content}</p>
                    </div>
                  ) : (
                    <p className="text-sm break-words">{message.content}</p>
                  )}
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
          {/* Image upload button */}
          <Button
            variant="outline"
            size="sm"
            onClick={handleImageSelectWithDialog}
            disabled={sending}
            title="Select image file"
          >
            📷
          </Button>

          <Input
            value={newMessage}
            onChange={handleInputChange}
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
