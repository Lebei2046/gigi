import { createSlice, createAsyncThunk } from '@reduxjs/toolkit'
import type { PayloadAction } from '@reduxjs/toolkit'
import type { Peer } from '@/utils/messaging'
import type { Group } from '@/models/db'
import {
  getChatInfo,
  updateChatInfo,
  resetUnreadCount,
  getGroup,
  ensureMilliseconds,
} from '@/utils/chatUtils'
import { MessagingClient } from '@/utils/messaging'

// Serializable version of Group for Redux state (Date converted to string)
interface SerializableGroup {
  id: string
  name: string
  joined: boolean
  createdAt: string
}

// Helper function to convert Group to SerializableGroup
function toSerializableGroup(group: Group | null): SerializableGroup | null {
  if (!group) return null

  return {
    id: group.id,
    name: group.name,
    joined: group.joined,
    createdAt: group.createdAt.toISOString(),
  }
}

// Generate unique IDs to prevent collisions
let messageIdCounter = 0
export const generateMessageId = () => {
  const now = Date.now()
  const counter = ++messageIdCounter
  const random = Math.random().toString(36).substring(2, 8)
  return `${now}-${counter}-${random}`
}

export interface Message {
  id: string
  from_peer_id: string
  from_nickname: string
  content: string
  timestamp: number
  isOutgoing: boolean
  isGroup?: boolean
  messageType?: 'text' | 'image' | 'file'
  imageId?: string
  imageData?: string
  filename?: string
  fileSize?: number
  fileType?: string
  shareCode?: string
  isDownloading?: boolean
  downloadProgress?: number
  downloadId?: string // Unique ID to track specific download
  isUploading?: boolean // For showing upload progress
}

export interface ChatRoomState {
  // Current chat context
  peer: Peer | null
  group: SerializableGroup | null
  isGroupChat: boolean
  chatId: string | null
  chatName: string | null

  // Messages
  messages: Message[]
  isLoading: boolean
  sending: boolean

  // UI state
  newMessage: string
  unreadResetDone: boolean

  // Error handling
  error: string | null
}

const initialState: ChatRoomState = {
  peer: null,
  group: null,
  isGroupChat: false,
  chatId: null,
  chatName: null,
  messages: [],
  isLoading: true,
  sending: false,
  newMessage: '',
  unreadResetDone: false,
  error: null,
}

// Async thunks for ChatRoom operations
export const initializeChatRoomAsync = createAsyncThunk(
  'chatRoom/initialize',
  async ({ id, peer }: { id: string; peer?: Peer }) => {
    let isGroupChat = false
    let group: Group | null = null
    let chatPeer: Peer | null = peer || null

    if (!peer) {
      // Check if this is a group by looking up the group in database
      group = (await getGroup(id)) || null
      if (group) {
        isGroupChat = true
        chatPeer = null

        // Join the group topic
        if (group.joined) {
          await MessagingClient.joinGroup(id)
        } else {
          // Group owners also need to subscribe to their own group topics
          await MessagingClient.joinGroup(id)
        }
      }
    }

    // Validate that we have either a valid peer or group
    if (!chatPeer && !group) {
      throw new Error(`Invalid chat room: No peer or group found for ID ${id}`)
    }

    return {
      peer: chatPeer,
      group: toSerializableGroup(group),
      isGroupChat,
      chatId: isGroupChat ? group?.id || null : chatPeer?.id || null,
      chatName: isGroupChat ? group?.name || null : chatPeer?.nickname || null,
    }
  }
)

export const loadMessageHistoryAsync = createAsyncThunk(
  'chatRoom/loadMessageHistory',
  async ({ chatId, isGroupChat }: { chatId: string; isGroupChat: boolean }) => {
    if (!chatId) {
      throw new Error('chatId is required for loading message history')
    }

    const historyKey = isGroupChat
      ? `chat_history_group_${chatId}`
      : `chat_history_${chatId}`

    const savedHistory = localStorage.getItem(historyKey)
    if (savedHistory) {
      try {
        const history = JSON.parse(savedHistory)
        // Batch process timestamps for better performance
        const normalizedHistory = history.map((msg: any) => ({
          ...msg,
          timestamp: ensureMilliseconds(msg.timestamp),
          isGroup: isGroupChat,
        }))

        // Return only last 100 messages for performance
        return normalizedHistory.slice(-100)
      } catch (parseError) {
        console.error('Failed to parse message history:', parseError)
        return []
      }
    }

    return []
  }
)

export const initializeChatInfoAsync = createAsyncThunk(
  'chatRoom/initializeChatInfo',
  async ({
    chatId,
    chatName,
    isGroupChat,
    unreadResetDone,
  }: {
    chatId: string
    chatName: string
    isGroupChat: boolean
    unreadResetDone: boolean
  }) => {
    if (!chatId || !chatName) {
      throw new Error(
        'chatId and chatName are required for initializing chat info'
      )
    }

    const existingChat = await getChatInfo(chatId)
    if (!existingChat) {
      // Only create new chat entry if it doesn't exist
      await updateChatInfo(chatId, chatName, '', Date.now(), isGroupChat)
    } else if (!unreadResetDone) {
      // Reset unread count when user opens the chat (only once)

      await resetUnreadCount(chatId)

      // Trigger refresh after a short delay
      setTimeout(() => {
        window.dispatchEvent(new Event('focus'))
      }, 50)
    }

    return { chatId, chatName, isGroupChat }
  }
)

export const sendMessageAsync = createAsyncThunk(
  'chatRoom/sendMessage',
  async ({
    content,
    isGroupChat,
    peer,
    group,
  }: {
    content: string
    isGroupChat: boolean
    peer: Peer | null
    group: SerializableGroup | null
  }) => {
    const timestamp = Date.now()
    let result: any

    if (isGroupChat && group) {
      result = await MessagingClient.sendGroupMessage(group.id, content.trim())
    } else if (!isGroupChat && peer) {
      result = await MessagingClient.sendMessageToNickname(
        peer.nickname,
        content.trim()
      )
    } else {
      throw new Error('Invalid chat state for sending message')
    }

    return { result, timestamp }
  }
)

const chatRoomSlice = createSlice({
  name: 'chatRoom',
  initialState,
  reducers: {
    setPeer: (state, action: PayloadAction<Peer | null>) => {
      state.peer = action.payload
    },

    setGroup: (state, action: PayloadAction<Group | null>) => {
      state.group = toSerializableGroup(action.payload)
    },

    setIsGroupChat: (state, action: PayloadAction<boolean>) => {
      state.isGroupChat = action.payload
    },

    setMessages: (state, action: PayloadAction<Message[]>) => {
      state.messages = action.payload
    },

    addMessage: (state, action: PayloadAction<Message>) => {
      state.messages.push(action.payload)
    },

    addImageMessage: (state, action: PayloadAction<Message>) => {
      state.messages.push(action.payload)
    },

    addGroupImageMessage: (state, action: PayloadAction<Message>) => {
      state.messages.push(action.payload)
    },

    removeMessage: (state, action: PayloadAction<string>) => {
      state.messages = state.messages.filter(msg => msg.id !== action.payload)
    },

    setNewMessage: (state, action: PayloadAction<string>) => {
      state.newMessage = action.payload
    },

    setSending: (state, action: PayloadAction<boolean>) => {
      state.sending = action.payload
    },

    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload
    },

    setUnreadResetDone: (state, action: PayloadAction<boolean>) => {
      state.unreadResetDone = action.payload
    },

    // Handle incoming direct message
    handleDirectMessage: (
      state,
      action: PayloadAction<{
        from_peer_id: string
        from_nickname: string
        content: string
        timestamp: number
      }>
    ) => {
      const { from_peer_id, from_nickname, content, timestamp } = action.payload

      const newMessage: Message = {
        id: generateMessageId(),
        from_peer_id,
        from_nickname,
        content,
        timestamp: ensureMilliseconds(timestamp),
        isOutgoing: false,
        isGroup: false,
      }

      state.messages.push(newMessage)
    },

    // Handle incoming group message
    handleGroupMessage: (
      state,
      action: PayloadAction<{
        id?: string
        from_peer_id: string
        from_nickname: string
        content: string
        timestamp: number
        group_id: string
      }>
    ) => {
      const { id, from_peer_id, from_nickname, content, timestamp } =
        action.payload

      const newMessage: Message = {
        id: id || generateMessageId(),
        from_peer_id,
        from_nickname,
        content,
        timestamp: ensureMilliseconds(timestamp),
        isOutgoing: false,
        isGroup: true,
      }

      state.messages.push(newMessage)
    },

    updateMessage: (
      state,
      action: PayloadAction<{
        id?: string
        downloadId?: string
        shareCode?: string
        content?: string
        imageData?: string
        newId?: string
        isDownloading?: boolean
        downloadProgress?: number
        isUploading?: boolean
      }>
    ) => {
      const {
        id,
        shareCode,
        downloadId,
        content,
        imageData,
        newId,
        isDownloading,
        downloadProgress,
        isUploading,
      } = action.payload

      // Find message by id, downloadId, or shareCode (search from newest to oldest)
      // Priority: id > downloadId > shareCode (to ensure we update the right message)
      const message = [...state.messages]
        .reverse()
        .find(
          msg =>
            (id !== undefined && msg.id === id) ||
            (downloadId !== undefined && msg.downloadId === downloadId) ||
            (shareCode !== undefined && msg.shareCode === shareCode)
        )

      console.log('🔧 updateMessage called:', {
        payload: action.payload,
        stateMessages: state.messages.map(m => ({
          id: m.id,
          downloadId: m.downloadId,
        })),
        searchingFor: { id, downloadId, shareCode },
        foundMessage: message,
        messageFound: !!message,
        foundMessageDetails: message
          ? {
              id: message.id,
              downloadId: message.downloadId,
              shareCode: message.shareCode,
            }
          : null,
        totalMessages: state.messages.length,
        allMessagesIds: state.messages.map(m => m.id),
      })

      console.log('🔍 About to check if (message):', {
        messageIsTruthy: !!message,
        messageValue: message,
        messageType: typeof message,
      })

      console.log(
        '🧪 Executing if(message) check. Result:',
        message ? 'TRUE' : 'FALSE'
      )

      console.log('✨ NEW VERSION OF CODE - this confirms code is updated')

      if (message) {
        // Redux Toolkit uses Immer - direct mutations are allowed but let's be explicit
        if (content !== undefined) message.content = content
        if (imageData !== undefined) message.imageData = imageData
        if (newId !== undefined) message.id = newId
        if (isDownloading !== undefined) message.isDownloading = isDownloading
        if (downloadProgress !== undefined)
          message.downloadProgress = downloadProgress
        if (downloadId !== undefined) message.downloadId = downloadId
        if (isUploading !== undefined) message.isUploading = isUploading
        console.log('✅ Message updated in Redux:', {
          id: message.id,
          messageType: message.messageType,
          hasImageData: !!message.imageData,
          imageDataLength: message.imageData?.length,
          content: message.content?.substring(0, 50),
        })
      } else {
        console.warn('⚠️ Message not found for update:', {
          id,
          shareCode,
          downloadId,
          messageValue: message,
          messagesInState: state.messages.length,
        })
      }
    },

    updateGroupMessage: (
      state,
      action: PayloadAction<{
        id?: string
        shareCode?: string
        downloadId?: string
        content?: string
        imageData?: string
        newId?: string
        isDownloading?: boolean
        downloadProgress?: number
      }>
    ) => {
      const {
        id,
        shareCode,
        downloadId,
        content,
        imageData,
        newId,
        isDownloading,
        downloadProgress,
      } = action.payload
      // Find message by id, downloadId, or shareCode (search from newest to oldest)
      // Priority: downloadId > id > shareCode (to ensure we update the right message)
      const message = [...state.messages]
        .reverse()
        .find(
          msg =>
            msg.downloadId === downloadId ||
            msg.id === id ||
            msg.shareCode === shareCode
        )
      if (message) {
        if (content !== undefined) message.content = content
        if (imageData !== undefined) message.imageData = imageData
        if (newId !== undefined) message.id = newId
        if (isDownloading !== undefined) message.isDownloading = isDownloading
        if (downloadProgress !== undefined)
          message.downloadProgress = downloadProgress
        if (downloadId !== undefined) message.downloadId = downloadId
      }
    },

    clearMessages: state => {
      state.messages = []
    },

    resetChatRoomState: () => initialState,
  },
  extraReducers: builder => {
    builder
      // Initialize chat room
      .addCase(initializeChatRoomAsync.pending, state => {
        state.isLoading = true
        state.error = null
      })
      .addCase(initializeChatRoomAsync.fulfilled, (state, action) => {
        state.isLoading = false
        state.peer = action.payload.peer
        state.group = action.payload.group
        state.isGroupChat = action.payload.isGroupChat
        state.chatId = action.payload.chatId
        state.chatName = action.payload.chatName
      })
      .addCase(initializeChatRoomAsync.rejected, (state, action) => {
        state.isLoading = false
        state.peer = null
        state.group = null
        state.error = action.error.message || 'Failed to initialize chat room'
      })

      // Load message history
      .addCase(loadMessageHistoryAsync.fulfilled, (state, action) => {
        state.messages = action.payload
      })
      .addCase(loadMessageHistoryAsync.rejected, (state, action) => {
        state.error = action.error.message || 'Failed to load message history'
      })

      // Initialize chat info
      .addCase(initializeChatInfoAsync.fulfilled, (state, _action) => {
        state.unreadResetDone = true
      })
      .addCase(initializeChatInfoAsync.rejected, (state, action) => {
        state.error = action.error.message || 'Failed to initialize chat info'
      })

      // Send message
      .addCase(sendMessageAsync.pending, state => {
        state.sending = true
        state.error = null
      })
      .addCase(sendMessageAsync.fulfilled, (state, _action) => {
        state.sending = false
        state.newMessage = ''
      })
      .addCase(sendMessageAsync.rejected, (state, action) => {
        state.sending = false
        state.error = action.error.message || 'Failed to send message'
      })
  },
})

export const {
  setPeer,
  setGroup,
  setIsGroupChat,
  setMessages,
  addMessage,
  addImageMessage,
  addGroupImageMessage,
  removeMessage,
  updateMessage,
  updateGroupMessage,
  setNewMessage,
  setSending,
  setError,
  setUnreadResetDone,
  handleDirectMessage,
  handleGroupMessage,
  clearMessages,
  resetChatRoomState,
} = chatRoomSlice.actions

export default chatRoomSlice.reducer
