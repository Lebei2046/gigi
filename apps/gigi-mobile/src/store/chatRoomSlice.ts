import { createSlice, createAsyncThunk } from '@reduxjs/toolkit'
import type { PayloadAction } from '@reduxjs/toolkit'
import type { Peer } from '@/utils/messaging'
import type { Group } from '@/models/db'
import {
  getChatInfo,
  updateChatInfo,
  resetUnreadCount,
  getGroup,
} from '@/utils/chatUtils'
import { MessagingClient } from '@/utils/messaging'

// Serializable version of Group for Redux state (Date converted to string)
interface SerializableGroup {
  id: string
  name: string
  joined: boolean
  createdAt: string
}

// Generate unique IDs to prevent collisions
let messageIdCounter = 0
export const generateMessageId = () => {
  const now = Date.now()
  const counter = ++messageIdCounter
  return `${now}-${counter}`
}

export interface Message {
  id: string
  from_peer_id: string
  from_nickname: string
  content: string
  timestamp: number
  isOutgoing: boolean
  isGroup?: boolean
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
      group = await getGroup(id)
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
      group: group
        ? {
            ...group,
            createdAt: group.createdAt.toISOString(),
          }
        : null,
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
          timestamp:
            msg.timestamp < 1000000000000
              ? msg.timestamp * 1000
              : msg.timestamp,
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
      await updateChatInfo(chatId, chatName, '', Date.now(), false, isGroupChat)
    } else if (!unreadResetDone) {
      // Reset unread count when user opens the chat (only once)
      console.log(`🏠 Entering chat room for ${chatId}, resetting unread count`)
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
      state.group = action.payload
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
        timestamp: timestamp < 1000000000000 ? timestamp * 1000 : timestamp,
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
        timestamp: timestamp < 1000000000000 ? timestamp * 1000 : timestamp,
        isOutgoing: false,
        isGroup: true,
      }

      state.messages.push(newMessage)
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
      .addCase(initializeChatInfoAsync.fulfilled, (state, action) => {
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
      .addCase(sendMessageAsync.fulfilled, (state, action) => {
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
  removeMessage,
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
