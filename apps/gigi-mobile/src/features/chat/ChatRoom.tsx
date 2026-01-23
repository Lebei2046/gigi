import { useEffect, useRef } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAppDispatch, useAppSelector } from '@/store'
import {
  resetChatRoomState,
  setNewMessage,
  clearThumbnailCache,
} from '@/store/chatRoomSlice'
import {
  ChatRoomHeader,
  MessageList,
  ChatRoomInput,
  LoadingState,
  EmptyState,
} from './chat-room'
import {
  useChatRoomInitialization,
  useMessagingEvents,
  useMessagePersistence,
  useMessageActions,
} from './chat-room/hooks'

export default function ChatRoom() {
  const navigate = useNavigate()
  const dispatch = useAppDispatch()
  const prevChatIdRef = useRef<string | undefined>()

  // Custom hooks (these handle initialization and provide all needed state)
  const {
    peer,
    group,
    messages,
    newMessage,
    sending,
    isLoading,
    isGroupChat,
    chatId,
    chatName,
  } = useChatRoomInitialization()

  // Debug: log when messages change
  const lastMessage = messages[messages.length - 1]
  if (lastMessage?.messageType === 'image' && lastMessage?.imageData) {
    console.log('✨ Latest image message in state:', {
      id: lastMessage.id,
      hasData: !!lastMessage.imageData,
      dataLength: lastMessage.imageData?.length,
      content: lastMessage.content,
    })
  }

  const { sentMessagesRef, downloadIdToMessageIdRef } = useMessagingEvents({
    chatId,
    isGroupChat,
    peer,
    group,
    messages,
  })
  const { saveFinalMessage, clearSaveTimeout } = useMessagePersistence({
    chatId,
    isGroupChat,
    isLoading,
    messages,
  })
  const {
    handleSendMessage,
    handleImageSelect,
    handleFileSelect,
    handleFileDownloadRequest,
  } = useMessageActions({
    newMessage,
    sending,
    isGroupChat,
    peer,
    group,
    messages,
    downloadIdToMessageIdRef,
  })

  // Debug log (only on mount and when state changes)
  useEffect(() => {
    console.log('📊 ChatRoom state update:', {
      isLoading,
      peer,
      group,
      chatId,
      chatName,
      messagesCount: messages.length,
    })

    // Reset state if chat ID changes (user switches to different chat)
    if (prevChatIdRef.current && prevChatIdRef.current !== chatId) {
      console.log(
        '🔄 Chat ID changed, resetting state:',
        prevChatIdRef.current,
        '->',
        chatId
      )
      dispatch(resetChatRoomState())
      // Also clear module-level storage when switching chats
      sentMessagesRef.current.clear()
      downloadIdToMessageIdRef.current.clear()
    }
    prevChatIdRef.current = chatId
  }, [
    isLoading,
    peer,
    group,
    chatId,
    chatName,
    messages.length,
    dispatch,
    sentMessagesRef,
    downloadIdToMessageIdRef,
  ])

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSendMessage()
    }
  }

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    dispatch(setNewMessage(e.target.value))
  }

  const goBack = () => {
    saveFinalMessage()
    // Clear thumbnail cache to free memory when leaving chat
    clearThumbnailCache()
    dispatch(resetChatRoomState())
    navigate('/chat')
  }

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      clearSaveTimeout()
      saveFinalMessage()
      // Clear thumbnail cache to free memory when unmounting
      clearThumbnailCache()
      // Don't reset state on unmount - this causes message loss during remounts
      // dispatch(resetChatRoomState())
      // Don't clear module-level storage - persist across remounts
      // sentMessagesRef.current.clear()
      // downloadIdToMessageIdRef.current.clear()
    }
  }, [dispatch])

  if (isLoading) {
    return <LoadingState />
  }

  if (!peer && !group) {
    return <EmptyState />
  }

  const chatTitle = chatName || (isGroupChat ? group?.name : peer?.nickname)

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <ChatRoomHeader
        chatTitle={chatTitle}
        chatId={chatId}
        isGroupChat={isGroupChat}
        onGoBack={goBack}
      />

      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        <MessageList
          messages={messages}
          isGroupChat={isGroupChat}
          onDownloadRequest={handleFileDownloadRequest}
        />
      </div>

      {/* Message Input */}
      <ChatRoomInput
        newMessage={newMessage}
        sending={sending}
        isGroupChat={isGroupChat}
        chatName={chatTitle}
        onSendMessage={handleSendMessage}
        onFileSelect={handleFileSelect}
        onImageSelect={handleImageSelect}
        onMessageChange={handleInputChange}
        onKeyDown={handleKeyDown}
      />
    </div>
  )
}
