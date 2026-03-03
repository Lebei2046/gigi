import { createSlice, createAsyncThunk } from '@reduxjs/toolkit'
import type { PayloadAction } from '@reduxjs/toolkit'
import { indexedDBManager, type Message, type ChatHistory } from '@/utils/indexedDB'

export interface PersistenceState {
  initialized: boolean
  loading: boolean
  error: string | null
  chatHistories: Record<string, ChatHistory>
}

const initialState: PersistenceState = {
  initialized: false,
  loading: false,
  error: null,
  chatHistories: {},
}

// Async thunks
export const initPersistenceAsync = createAsyncThunk('persistence/init', async () => {
  await indexedDBManager.init()
  const histories = await indexedDBManager.getAllChatHistories()
  return histories
})

export const saveMessageAsync = createAsyncThunk(
  'persistence/saveMessage',
  async (message: Message) => {
    await indexedDBManager.saveMessage(message)
    return message
  }
)

export const loadMessagesAsync = createAsyncThunk(
  'persistence/loadMessages',
  async ({ chatId, limit }: { chatId: string; limit?: number }) => {
    const messages = await indexedDBManager.getMessages(chatId, limit)
    return { chatId, messages }
  }
)

export const clearMessagesAsync = createAsyncThunk(
  'persistence/clearMessages',
  async (chatId: string) => {
    await indexedDBManager.clearMessages(chatId)
    return chatId
  }
)

export const markAsDeliveredAsync = createAsyncThunk(
  'persistence/markAsDelivered',
  async (chatId: string) => {
    await indexedDBManager.markAsDelivered(chatId)
    const history = await indexedDBManager.getChatHistory(chatId)
    return { chatId, history }
  }
)

export const clearAllAsync = createAsyncThunk('persistence/clearAll', async () => {
  await indexedDBManager.clearAll()
})

const persistenceSlice = createSlice({
  name: 'persistence',
  initialState,
  reducers: {
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload
    },
    setChatHistory: (state, action: PayloadAction<ChatHistory>) => {
      state.chatHistories[action.payload.chatId] = action.payload
    },
    removeChatHistory: (state, action: PayloadAction<string>) => {
      delete state.chatHistories[action.payload]
    },
  },
  extraReducers: builder => {
    builder
      // initPersistenceAsync
      .addCase(initPersistenceAsync.pending, state => {
        state.loading = true
        state.error = null
      })
      .addCase(initPersistenceAsync.fulfilled, (state, action) => {
        state.loading = false
        state.initialized = true
        // Convert array to object
        state.chatHistories = action.payload.reduce(
          (acc, history) => {
            acc[history.chatId] = history
            return acc
          },
          {} as Record<string, ChatHistory>
        )
      })
      .addCase(initPersistenceAsync.rejected, (state, action) => {
        state.loading = false
        state.error = action.error.message || 'Failed to initialize persistence'
      })
      // saveMessageAsync
      .addCase(saveMessageAsync.fulfilled, (state, action) => {
        const message = action.payload
        const history = state.chatHistories[message.chatId]
        if (history) {
          history.lastMessage = message.content
          history.lastTimestamp = message.timestamp
          if (message.direction === 'received' && !message.delivered) {
            history.unreadCount += 1
          }
          history.messages.push(message)
          if (history.messages.length > 50) {
            history.messages = history.messages.slice(-50)
          }
        } else {
          state.chatHistories[message.chatId] = {
            chatId: message.chatId,
            isGroupChat: message.isGroupChat,
            messages: [message],
            lastMessage: message.content,
            lastTimestamp: message.timestamp,
            unreadCount: message.direction === 'received' ? 1 : 0,
          }
        }
      })
      .addCase(saveMessageAsync.rejected, (state, action) => {
        state.error = action.error.message || 'Failed to save message'
      })
      // loadMessagesAsync
      .addCase(loadMessagesAsync.fulfilled, (state, action) => {
        const { chatId, messages } = action.payload
        const history = state.chatHistories[chatId]
        if (history) {
          history.messages = messages
        }
      })
      // clearMessagesAsync
      .addCase(clearMessagesAsync.fulfilled, (state, action) => {
        delete state.chatHistories[action.payload]
      })
      // markAsDeliveredAsync
      .addCase(markAsDeliveredAsync.fulfilled, (state, action) => {
        const { chatId, history } = action.payload
        if (history) {
          state.chatHistories[chatId] = history
        }
      })
      // clearAllAsync
      .addCase(clearAllAsync.fulfilled, state => {
        state.chatHistories = {}
      })
  },
})

export const { setError, setChatHistory, removeChatHistory } = persistenceSlice.actions

export default persistenceSlice.reducer
