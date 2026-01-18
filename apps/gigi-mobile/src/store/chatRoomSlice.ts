import { createSlice, createAsyncThunk } from '@reduxjs/toolkit'
import type { PayloadAction } from '@reduxjs/toolkit'
import type { Peer } from '@/utils/messaging'
import type { Group } from '@/models/db'
import {
  getConversationInfo,
  updateConversationInfo,
  resetUnreadCount,
} from '@/utils/conversationUtils'
import { getGroup, ensureMilliseconds } from '@/utils/chatUtils'
import { MessagingClient } from '@/utils/messaging'

// LRU Cache for thumbnails to prevent memory leaks
const MAX_CACHED_THUMBNAILS = 20
const thumbnailCache: Map<string, string> = new Map()

function getCachedThumbnail(shareCode: string): string | undefined {
  const value = thumbnailCache.get(shareCode)
  if (value !== undefined) {
    // Move to end (most recently used)
    thumbnailCache.delete(shareCode)
    thumbnailCache.set(shareCode, value)
    return value
  }
  return undefined
}

function setCachedThumbnail(shareCode: string, thumbnail: string): void {
  // Remove if exists
  thumbnailCache.delete(shareCode)
  // Add to end (most recently used)
  thumbnailCache.set(shareCode, thumbnail)
  // Evict oldest if over limit
  while (thumbnailCache.size > MAX_CACHED_THUMBNAILS) {
    const oldestKey = thumbnailCache.keys().next().value
    thumbnailCache.delete(oldestKey)
  }
}

function clearThumbnailCache(): void {
  thumbnailCache.clear()
}

// Never store full images in Redux - only thumbnails
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
  thumbnailData?: string // Base64 thumbnail (from backend, cached in Redux)
  thumbnailPath?: string // Path to thumbnail file from backend (used to load thumbnail)
  filePath?: string // Local file path (used to get thumbnail from thumbnail_store)
  imageData?: string // NEVER stored in Redux - only temporary in component state
  filename?: string
  fileSize?: number
  fileType?: string
  shareCode?: string
  isDownloading?: boolean
  downloadProgress?: number
  downloadId?: string // Unique ID to track specific download
  isUploading?: boolean // For showing upload progress
}

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

    const existingChat = await getConversationInfo(chatId)
    if (!existingChat) {
      // Only create new conversation entry if it doesn't exist
      await updateConversationInfo(chatId, chatName, '', Date.now(), isGroupChat)
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

// Load messages from backend
export const loadMessagesFromBackendAsync = createAsyncThunk(
  'chatRoom/loadMessagesFromBackend',
  async ({
    peerId,
    limit = 50,
    offset = 0,
  }: {
    peerId: string
    limit?: number
    offset?: number
  }) => {
    try {
      const data = await MessagingClient.getMessages(peerId, { limit, offset })
      return { messages: data.messages || [], prepend: offset > 0 }
    } catch (error) {
      console.error('Failed to load messages from backend:', error)
      throw error
    }
  }
)

// Search messages in backend
export const searchMessagesAsync = createAsyncThunk(
  'chatRoom/searchMessages',
  async ({ query, peerId }: { query: string; peerId?: string }) => {
    try {
      const data = await MessagingClient.searchMessages(query, peerId)
      return { query, peerId, messages: data.messages || [] }
    } catch (error) {
      console.error('Failed to search messages:', error)
      throw error
    }
  }
)

// Load thumbnail from backend
export const loadThumbnailAsync = createAsyncThunk(
  'chatRoom/loadThumbnail',
  async (filePath: string, { rejectWithValue }) => {
    try {
      const thumbnail = await MessagingClient.getFileThumbnail(filePath)
      return { filePath, thumbnail }
    } catch (error) {
      console.error('Failed to load thumbnail:', error)
      return rejectWithValue({ filePath, error: String(error) })
    }
  }
)

// Load full image from backend
export const loadFullImageAsync = createAsyncThunk(
  'chatRoom/loadFullImage',
  async (shareCode: string, { rejectWithValue }) => {
    try {
      const imageData = await MessagingClient.getFullImage(shareCode)
      return { shareCode, imageData }
    } catch (error) {
      console.error('Failed to load full image:', error)
      return rejectWithValue({ shareCode, error: String(error) })
    }
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
      // Check if message already exists to prevent duplicates
      const existingIndex = state.messages.findIndex(
        msg => msg.id === action.payload.id
      )
      if (existingIndex === -1) {
        console.log('➕ Adding message to state:', action.payload.id, action.payload.content)
        state.messages.push(action.payload)
      } else {
        console.log('🔁 Skipping duplicate message in addMessage:', action.payload.id)
      }
    },

    addImageMessage: (state, action: PayloadAction<Message>) => {
      // Check if message already exists to prevent duplicates
      const existingIndex = state.messages.findIndex(
        msg => msg.id === action.payload.id
      )
      if (existingIndex === -1) {
        state.messages.push(action.payload)
      } else {
        console.log('🔁 Skipping duplicate image message in addImageMessage:', action.payload.id)
      }
    },

    addGroupImageMessage: (state, action: PayloadAction<Message>) => {
      // Check if message already exists to prevent duplicates
      const existingIndex = state.messages.findIndex(
        msg => msg.id === action.payload.id
      )
      if (existingIndex === -1) {
        state.messages.push(action.payload)
      } else {
        console.log('🔁 Skipping duplicate group image message in addGroupImageMessage:', action.payload.id)
      }
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
        thumbnailData?: string
        filePath?: string
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
        thumbnailData,
        filePath,
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
        if (thumbnailData !== undefined) message.thumbnailData = thumbnailData
        if (filePath !== undefined) message.filePath = filePath
        if (newId !== undefined) message.id = newId
        if (isDownloading !== undefined) message.isDownloading = isDownloading
        if (downloadProgress !== undefined)
          message.downloadProgress = downloadProgress
        if (downloadId !== undefined) message.downloadId = downloadId
        if (isUploading !== undefined) message.isUploading = isUploading
        console.log('✅ Message updated in Redux:', {
          id: message.id,
          messageType: message.messageType,
          hasThumbnailData: !!message.thumbnailData,
          thumbnailDataLength: message.thumbnailData?.length,
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
        thumbnailData?: string
        filePath?: string
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
        thumbnailData,
        filePath,
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
        if (thumbnailData !== undefined) message.thumbnailData = thumbnailData
        if (filePath !== undefined) message.filePath = filePath
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

      // Load messages from backend
      .addCase(loadMessagesFromBackendAsync.pending, state => {
        state.isLoading = true
      })
      .addCase(loadMessagesFromBackendAsync.fulfilled, (state, action) => {
        state.isLoading = false
        const { messages, prepend } = action.payload

        console.log('📥 loadMessagesFromBackendAsync.fulfilled:', {
          messageCount: messages.length,
          currentMessageCount: state.messages.length,
          currentMessages: state.messages.map(m => ({ id: m.id, content: m.content })),
          newMessages: messages.map(m => ({ id: m.id, content: m.content })),
          prepend,
        })

        // Deduplicate messages by ID to prevent duplicates
        const uniqueMessages = prepend
          ? [...messages, ...state.messages]
          : messages

        console.log('🔍 Before deduplication:', uniqueMessages.length, 'messages')
        console.log('🔍 Messages before dedup:', uniqueMessages.map(m => ({ id: m.id, content: m.content })))

        const seenIds = new Set<string>()
        const deduplicatedMessages: typeof uniqueMessages = []

        for (const msg of uniqueMessages) {
          if (!seenIds.has(msg.id)) {
            seenIds.add(msg.id)
            deduplicatedMessages.push(msg)
          } else {
            console.log('🔁 Skipping duplicate message:', msg.id, msg.content)
          }
        }

        console.log('🔍 After deduplication:', deduplicatedMessages.length, 'messages')
        console.log('🔍 Messages after dedup:', deduplicatedMessages.map(m => ({ id: m.id, content: m.content })))

        state.messages = deduplicatedMessages
      })
      .addCase(loadMessagesFromBackendAsync.rejected, (state, action) => {
        state.isLoading = false
        state.error =
          action.error.message || 'Failed to load messages from backend'
      })

      // Search messages
      .addCase(searchMessagesAsync.pending, state => {
        state.isLoading = true
      })
      .addCase(searchMessagesAsync.fulfilled, (state, action) => {
        state.isLoading = false
        state.messages = action.payload.messages
      })
      .addCase(searchMessagesAsync.rejected, (state, action) => {
        state.isLoading = false
        state.error = action.error.message || 'Failed to search messages'
      })

      // Load thumbnail - with LRU cache
      .addCase(loadThumbnailAsync.fulfilled, (state, action) => {
        const { filePath, thumbnail } = action.payload
        // Only cache non-empty thumbnails
        if (thumbnail) {
          // Cache thumbnail using LRU (use filePath as key)
          setCachedThumbnail(filePath, thumbnail)
          // Update message with thumbnail
          const message = state.messages.find(m => m.filePath === filePath)
          if (message) {
            message.thumbnailData = thumbnail
          }
        }
      })
      .addCase(loadThumbnailAsync.rejected, (state, action) => {
        console.error('Failed to load thumbnail:', action.payload)
      })

      // Load full image - NEVER store in Redux!
      // This thunk only exists for component-level temporary usage
      .addCase(loadFullImageAsync.fulfilled, (state, _action) => {
        // DO NOT store imageData in Redux - only return to component
        console.warn(
          '⚠️ loadFullImageAsync fulfilled - image data not stored in Redux (by design)'
        )
      })
      .addCase(loadFullImageAsync.rejected, (state, action) => {
        console.error('Failed to load full image:', action.payload)
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

// Export cache control functions
export { clearThumbnailCache }

export default chatRoomSlice.reducer
