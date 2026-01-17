import { createSlice, createAsyncThunk } from '@reduxjs/toolkit'
import type { PayloadAction } from '@reduxjs/toolkit'
import type { Peer, GroupShareMessage } from '@/utils/messaging'
import type { Chat } from '@/models/db'
import { getAllChats, getAllGroups, updateChatInfo } from '@/utils/chatUtils'
import { MessagingClient } from '@/utils/messaging'

// Redux-compatible Group type with serializable dates
interface ReduxGroup {
  id: string
  name: string
  joined: boolean
  createdAt: string // ISO string instead of Date
}

export interface ChatState {
  peers: Peer[]
  loading: boolean
  error: string | null
  chats: Chat[]
  groups: ReduxGroup[]
  latestMessages: Record<string, string>
  groupShareNotifications: GroupShareMessage[]
  showShareDrawer: boolean
  selectedGroup: ReduxGroup | null
  componentError: Error | null
}

const initialState: ChatState = {
  peers: [],
  loading: true,
  error: null,
  chats: [],
  groups: [],
  latestMessages: {},
  groupShareNotifications: [],
  showShareDrawer: false,
  selectedGroup: null,
  componentError: null,
}

// Async thunks for complex operations
export const loadChatsAsync = createAsyncThunk('chat/loadChats', async () => {
  const allChats = await getAllChats()
  return allChats
})

export const loadGroupsAsync = createAsyncThunk('chat/loadGroups', async () => {
  const allGroups = await getAllGroups()
  // Convert Date objects to ISO strings for Redux compatibility
  return allGroups.map(group => ({
    ...group,
    createdAt:
      group.createdAt instanceof Date
        ? group.createdAt.toISOString()
        : group.createdAt,
  }))
})

export const subscribeToGroupsAsync = createAsyncThunk(
  'chat/subscribeToGroups',
  async (_, { rejectWithValue }) => {
    try {
      const allGroups = await getAllGroups()
      const joinedGroups = allGroups.filter(group => group.joined)

      for (const group of joinedGroups) {
        try {
          await MessagingClient.joinGroup(group.id)
        } catch (error) {
          console.error(`Failed to subscribe to group ${group.name}:`, error)
        }
      }
      // Convert Date objects to ISO strings for Redux compatibility
      return joinedGroups.map(group => ({
        ...group,
        createdAt:
          group.createdAt instanceof Date
            ? group.createdAt.toISOString()
            : group.createdAt,
      }))
    } catch (error) {
      return rejectWithValue(error)
    }
  }
)

export const loadPeersAsync = createAsyncThunk('chat/loadPeers', async () => {
  const timeoutPromise = new Promise((_, reject) => {
    setTimeout(() => reject(new Error('Timeout loading peers')), 5000)
  })

  const peersPromise = MessagingClient.getPeers()
  const connectedPeers = (await Promise.race([
    peersPromise,
    timeoutPromise,
  ])) as Peer[]

  return connectedPeers
})

export const clearChatMessagesAsync = createAsyncThunk(
  'chat/clearMessages',
  async ({ chatId, isGroupChat }: { chatId: string; isGroupChat: boolean }) => {
    console.log('🗑️ clearChatMessagesAsync called:', { chatId, isGroupChat })

    // Clear from localStorage
    const historyKey = isGroupChat
      ? `chat_history_group_${chatId}`
      : `chat_history_${chatId}`
    localStorage.removeItem(historyKey)

    console.log('✅ Cleared localStorage for:', historyKey)

    // Clear messages from backend database and delete thumbnail files for incoming images
    const { MessagingClient } = await import('@/utils/messaging')
    try {
      console.log('📞 Calling clearMessagesWithThumbnails with chatId:', chatId)
      const result = await MessagingClient.clearMessagesWithThumbnails(chatId)
      console.log('✅ clearMessagesWithThumbnails succeeded, cleared', result, 'messages')
    } catch (error) {
      console.error('❌ Failed to clear messages from backend:', error)
      throw error
    }

    // Reset chat info in IndexedDB
    await updateChatInfo(chatId, '', '', 0, isGroupChat)

    console.log('✅ Chat info reset for chatId:', chatId)

    return { chatId, isGroupChat }
  }
)

const chatSlice = createSlice({
  name: 'chat',
  initialState,
  reducers: {
    setPeers: (state, action: PayloadAction<Peer[]>) => {
      state.peers = action.payload
    },

    addPeer: (state, action: PayloadAction<Peer>) => {
      const exists = state.peers.find(p => p.id === action.payload.id)
      if (!exists) {
        state.peers.push(action.payload)
      }
    },

    removePeer: (state, action: PayloadAction<string>) => {
      state.peers = state.peers.filter(p => p.id !== action.payload)
    },

    setLoading: (state, action: PayloadAction<boolean>) => {
      state.loading = action.payload
    },

    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload
    },

    setChats: (state, action: PayloadAction<Chat[]>) => {
      state.chats = action.payload
    },

    setGroups: (state, action: PayloadAction<ReduxGroup[]>) => {
      state.groups = action.payload
    },

    setLatestMessages: (
      state,
      action: PayloadAction<Record<string, string>>
    ) => {
      state.latestMessages = action.payload
    },

    updateSingleLatestMessage: (
      state,
      action: PayloadAction<{ id: string; message: string }>
    ) => {
      state.latestMessages[action.payload.id] = action.payload.message
    },

    addGroupShareNotification: (
      state,
      action: PayloadAction<GroupShareMessage>
    ) => {
      const exists = state.groupShareNotifications.find(
        msg => msg.from_peer_id === action.payload.from_peer_id
      )
      if (!exists) {
        state.groupShareNotifications.push(action.payload)
      }
    },

    removeGroupShareNotification: (state, action: PayloadAction<string>) => {
      state.groupShareNotifications = state.groupShareNotifications.filter(
        msg => msg.from_peer_id !== action.payload
      )
    },

    setShowShareDrawer: (state, action: PayloadAction<boolean>) => {
      state.showShareDrawer = action.payload
    },

    setSelectedGroup: (state, action: PayloadAction<ReduxGroup | null>) => {
      state.selectedGroup = action.payload
    },

    setComponentError: (state, action: PayloadAction<Error | null>) => {
      state.componentError = action.payload
    },

    // Handle direct message received
    updateDirectMessage: (
      state,
      action: PayloadAction<{
        from_peer_id: string
        content: string
        timestamp: number
      }>
    ) => {
      const { from_peer_id, content, timestamp } = action.payload

      // Update latest message
      state.latestMessages[from_peer_id] = content

      // Update in chats as well
      const chatIndex = state.chats.findIndex(chat => chat.id === from_peer_id)
      if (chatIndex !== -1) {
        state.chats[chatIndex].lastMessage = content
        state.chats[chatIndex].lastMessageTime = new Date(
          timestamp
        ).toLocaleString()
        state.chats[chatIndex].unreadCount =
          (state.chats[chatIndex].unreadCount || 0) + 1
      }
    },

    // Handle group message received
    updateGroupMessage: (
      state,
      action: PayloadAction<{
        group_id: string
        content: string
        timestamp: number
      }>
    ) => {
      const { group_id, content, timestamp } = action.payload

      // Update latest message
      state.latestMessages[group_id] = content

      // Update in chats as well
      const chatIndex = state.chats.findIndex(chat => chat.id === group_id)
      if (chatIndex !== -1) {
        state.chats[chatIndex].lastMessage = content
        state.chats[chatIndex].lastMessageTime = new Date(
          timestamp
        ).toLocaleString()
        state.chats[chatIndex].unreadCount =
          (state.chats[chatIndex].unreadCount || 0) + 1
      }
    },

    // Reset component state
    resetChatState: () => initialState,

    // Clear chat messages
    clearChatMessages: (state, action: PayloadAction<{ chatId: string }>) => {
      const { chatId } = action.payload

      // Remove from latestMessages
      delete state.latestMessages[chatId]

      // Update chat info to remove last message
      const chatIndex = state.chats.findIndex(chat => chat.id === chatId)
      if (chatIndex !== -1) {
        state.chats[chatIndex].lastMessage = ''
        state.chats[chatIndex].lastMessageTime = ''
        state.chats[chatIndex].lastMessageTimestamp = 0
        state.chats[chatIndex].unreadCount = 0
      }
    },
  },
  extraReducers: builder => {
    builder
      .addCase(loadChatsAsync.fulfilled, (state, action) => {
        state.chats = action.payload

        // Update latestMessages from chats
        const messagesFromChats: Record<string, string> = {}
        action.payload.forEach(chat => {
          if (chat.lastMessage) {
            messagesFromChats[chat.id] = chat.lastMessage
          }
        })
        state.latestMessages = messagesFromChats
      })
      .addCase(loadChatsAsync.rejected, (_state, action) => {
        console.error('Failed to load chats:', action.error)
      })

      .addCase(loadGroupsAsync.fulfilled, (state, action) => {
        state.groups = action.payload
      })
      .addCase(loadGroupsAsync.rejected, (_state, action) => {
        console.error('Failed to load groups:', action.error)
      })

      .addCase(loadPeersAsync.fulfilled, (state, action) => {
        state.peers = action.payload
        state.loading = false
        state.error = null
      })
      .addCase(loadPeersAsync.rejected, (state, action) => {
        state.error = `Failed to load peers: ${action.error.message || 'Unknown error'}`
        state.peers = []
        state.loading = false
      })
      .addCase(clearChatMessagesAsync.fulfilled, (state, action) => {
        const { chatId } = action.payload

        // Remove from latestMessages
        delete state.latestMessages[chatId]

        // Update chat info to remove last message
        const chatIndex = state.chats.findIndex(chat => chat.id === chatId)
        if (chatIndex !== -1) {
          state.chats[chatIndex].lastMessage = ''
          state.chats[chatIndex].lastMessageTime = ''
          state.chats[chatIndex].lastMessageTimestamp = 0
          state.chats[chatIndex].unreadCount = 0
        }
      })
      .addCase(clearChatMessagesAsync.rejected, (_state, action) => {
        console.error('Failed to clear chat messages:', action.error)
      })
  },
})

export const {
  setPeers,
  addPeer,
  removePeer,
  setLoading,
  setError,
  setChats,
  setGroups,
  setLatestMessages,
  updateSingleLatestMessage,
  addGroupShareNotification,
  removeGroupShareNotification,
  setShowShareDrawer,
  setSelectedGroup,
  setComponentError,
  updateDirectMessage,
  updateGroupMessage,
  resetChatState,
  clearChatMessages,
} = chatSlice.actions

export default chatSlice.reducer
