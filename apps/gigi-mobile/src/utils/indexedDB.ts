//! IndexedDB message persistence for mobile/web clients
//!
//! This module provides IndexedDB-based message storage for offline support.

export interface Message {
  id: string
  chatId: string
  isGroupChat: boolean
  fromPeerId: string
  fromNickname: string
  content: string
  timestamp: number
  direction: 'sent' | 'received'
  isImage?: boolean
  imageThumbnail?: string
  delivered?: boolean
}

export interface ChatHistory {
  chatId: string
  isGroupChat: boolean
  messages: Message[]
  lastMessage?: string
  lastTimestamp?: number
  unreadCount: number
}

const DB_NAME = 'gigi-messages'
const DB_VERSION = 1
const STORE_MESSAGES = 'messages'
const STORE_CHAT_HISTORY = 'chat_history'

class IndexedDBManager {
  private db: IDBDatabase | null = null
  private initPromise: Promise<void> | null = null

  async init(): Promise<void> {
    if (this.initPromise) {
      return this.initPromise
    }

    this.initPromise = new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION)

      request.onerror = () => {
        reject(new Error(`Failed to open IndexedDB: ${request.error}`))
      }

      request.onupgradeneeded = event => {
        const db = (event.target as IDBOpenDBRequest).result

        // Create messages store
        if (!db.objectStoreNames.contains(STORE_MESSAGES)) {
          const messagesStore = db.createObjectStore(STORE_MESSAGES, {
            keyPath: 'id',
          })
          messagesStore.createIndex('chatId', 'chatId', { unique: false })
          messagesStore.createIndex('timestamp', 'timestamp', { unique: false })
        }

        // Create chat history store
        if (!db.objectStoreNames.contains(STORE_CHAT_HISTORY)) {
          const chatHistoryStore = db.createObjectStore(STORE_CHAT_HISTORY, {
            keyPath: 'chatId',
          })
          chatHistoryStore.createIndex('lastTimestamp', 'lastTimestamp', {
            unique: false,
          })
        }
      }

      request.onsuccess = () => {
        this.db = request.result
        resolve()
      }
    })

    return this.initPromise
  }

  private async getTransaction(
    storeName: string,
    mode: IDBTransactionMode = 'readonly'
  ): Promise<IDBObjectStore> {
    if (!this.db) {
      await this.init()
    }
    if (!this.db) {
      throw new Error('Database not initialized')
    }

    const transaction = this.db.transaction([storeName], mode)
    return transaction.objectStore(storeName)
  }

  async saveMessage(message: Message): Promise<void> {
    await this.init()

    return new Promise((resolve, reject) => {
      const request = this.db!.transaction([STORE_MESSAGES], 'readwrite')
        .objectStore(STORE_MESSAGES)
        .put(message)

      request.onsuccess = () => {
        // Update chat history
        this.updateChatHistory(message).then(resolve).catch(reject)
      }

      request.onerror = () => {
        reject(new Error(`Failed to save message: ${request.error}`))
      }
    })
  }

  async getMessages(chatId: string, limit = 100): Promise<Message[]> {
    await this.init()

    return new Promise((resolve, reject) => {
      const store = this.db!.transaction(
        [STORE_MESSAGES],
        'readonly'
      ).objectStore(STORE_MESSAGES)
      const index = store.index('chatId')
      const request = index.getAll(IDBKeyRange.only(chatId))

      request.onsuccess = () => {
        let messages = request.result
        // Sort by timestamp descending
        messages = messages.sort((a, b) => b.timestamp - a.timestamp)
        // Apply limit
        if (limit > 0) {
          messages = messages.slice(0, limit)
        }
        resolve(messages)
      }

      request.onerror = () => {
        reject(new Error(`Failed to get messages: ${request.error}`))
      }
    })
  }

  async clearMessages(chatId: string): Promise<void> {
    await this.init()

    return new Promise((resolve, reject) => {
      const store = this.db!.transaction(
        [STORE_MESSAGES],
        'readwrite'
      ).objectStore(STORE_MESSAGES)
      const index = store.index('chatId')
      const request = index.openCursor(IDBKeyRange.only(chatId))

      request.onsuccess = event => {
        const cursor = (event.target as IDBRequest).result
        if (cursor) {
          cursor.delete()
          cursor.continue()
        } else {
          // Clear chat history
          this.clearChatHistory(chatId).then(resolve).catch(reject)
        }
      }

      request.onerror = () => {
        reject(new Error(`Failed to clear messages: ${request.error}`))
      }
    })
  }

  private async updateChatHistory(message: Message): Promise<void> {
    return new Promise((resolve, reject) => {
      const store = this.db!.transaction(
        [STORE_CHAT_HISTORY],
        'readwrite'
      ).objectStore(STORE_CHAT_HISTORY)
      const getRequest = store.get(message.chatId)

      getRequest.onsuccess = () => {
        const history: ChatHistory = getRequest.result || {
          chatId: message.chatId,
          isGroupChat: message.isGroupChat,
          messages: [],
          unreadCount: 0,
        }

        // Update last message info
        history.lastMessage = message.content
        history.lastTimestamp = message.timestamp

        // Update unread count if received message
        if (message.direction === 'received' && !message.delivered) {
          history.unreadCount += 1
        }

        // Add message to history
        history.messages.push(message)

        // Keep only last 50 messages in history
        if (history.messages.length > 50) {
          history.messages = history.messages.slice(-50)
        }

        const putRequest = store.put(history)
        putRequest.onsuccess = () => resolve()
        putRequest.onerror = () =>
          reject(
            new Error(`Failed to update chat history: ${putRequest.error}`)
          )
      }

      getRequest.onerror = () =>
        reject(new Error(`Failed to get chat history: ${getRequest.error}`))
    })
  }

  private async clearChatHistory(chatId: string): Promise<void> {
    return new Promise((resolve, reject) => {
      const request = this.db!.transaction([STORE_CHAT_HISTORY], 'readwrite')
        .objectStore(STORE_CHAT_HISTORY)
        .delete(chatId)

      request.onsuccess = () => resolve()
      request.onerror = () =>
        reject(new Error(`Failed to clear chat history: ${request.error}`))
    })
  }

  async getChatHistory(chatId: string): Promise<ChatHistory | undefined> {
    await this.init()

    return new Promise((resolve, reject) => {
      const request = this.db!.transaction([STORE_CHAT_HISTORY], 'readonly')
        .objectStore(STORE_CHAT_HISTORY)
        .get(chatId)

      request.onsuccess = () => resolve(request.result)
      request.onerror = () =>
        reject(new Error(`Failed to get chat history: ${request.error}`))
    })
  }

  async getAllChatHistories(): Promise<ChatHistory[]> {
    await this.init()

    return new Promise((resolve, reject) => {
      const request = this.db!.transaction([STORE_CHAT_HISTORY], 'readonly')
        .objectStore(STORE_CHAT_HISTORY)
        .getAll()

      request.onsuccess = () => resolve(request.result)
      request.onerror = () =>
        reject(new Error(`Failed to get all chat histories: ${request.error}`))
    })
  }

  async markAsDelivered(chatId: string): Promise<void> {
    await this.init()

    return new Promise((resolve, reject) => {
      const store = this.db!.transaction(
        [STORE_MESSAGES],
        'readwrite'
      ).objectStore(STORE_MESSAGES)
      const index = store.index('chatId')
      const request = index.openCursor(IDBKeyRange.only(chatId))

      request.onsuccess = event => {
        const cursor = (event.target as IDBRequest).result
        if (cursor) {
          const message = cursor.value as Message
          if (message.direction === 'received' && !message.delivered) {
            message.delivered = true
            cursor.update(message)
          }
          cursor.continue()
        } else {
          // Reset unread count
          this.resetUnreadCount(chatId).then(resolve).catch(reject)
        }
      }

      request.onerror = () =>
        reject(new Error(`Failed to mark as delivered: ${request.error}`))
    })
  }

  private async resetUnreadCount(chatId: string): Promise<void> {
    return new Promise((resolve, reject) => {
      const store = this.db!.transaction(
        [STORE_CHAT_HISTORY],
        'readwrite'
      ).objectStore(STORE_CHAT_HISTORY)
      const getRequest = store.get(chatId)

      getRequest.onsuccess = () => {
        const history = getRequest.result
        if (history) {
          history.unreadCount = 0
          const putRequest = store.put(history)
          putRequest.onsuccess = () => resolve()
          putRequest.onerror = () =>
            reject(
              new Error(`Failed to reset unread count: ${putRequest.error}`)
            )
        } else {
          resolve()
        }
      }

      getRequest.onerror = () =>
        reject(new Error(`Failed to get chat history: ${getRequest.error}`))
    })
  }

  async clearAll(): Promise<void> {
    await this.init()

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction(
        [STORE_MESSAGES, STORE_CHAT_HISTORY],
        'readwrite'
      )

      const clearMessages = transaction.objectStore(STORE_MESSAGES).clear()
      const clearHistory = transaction.objectStore(STORE_CHAT_HISTORY).clear()

      let completed = 0
      const checkComplete = () => {
        completed++
        if (completed === 2) {
          resolve()
        }
      }

      clearMessages.onsuccess = checkComplete
      clearMessages.onerror = () =>
        reject(new Error(`Failed to clear messages: ${clearMessages.error}`))

      clearHistory.onsuccess = checkComplete
      clearHistory.onerror = () =>
        reject(new Error(`Failed to clear history: ${clearHistory.error}`))
    })
  }
}

// Export singleton instance
export const indexedDBManager = new IndexedDBManager()
