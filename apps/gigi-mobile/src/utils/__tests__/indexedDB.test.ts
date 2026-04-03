//! Unit tests for IndexedDB utilities

describe('IndexedDB Utilities', () => {
  let db: IDBDatabase | null = null
  const DB_NAME = 'test-gigi-db'
  const DB_VERSION = 1

  beforeEach(async () => {
    return new Promise<void>((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION)

      request.onerror = () => reject(request.error)
      request.onsuccess = () => {
        db = request.result
        resolve()
      }

      request.onupgradeneeded = event => {
        const database = (event.target as IDBOpenDBRequest).result
        if (!database.objectStoreNames.contains('messages')) {
          database.createObjectStore('messages', { keyPath: 'id' })
        }
        if (!database.objectStoreNames.contains('chatHistory')) {
          database.createObjectStore('chatHistory', { keyPath: 'id' })
        }
      }
    })
  })

  afterEach(() => {
    if (db) {
      db.close()
      db = null
    }
  })

  test('database opens successfully', () => {
    expect(db).not.toBeNull()
    expect(db!.name).toBe(DB_NAME)
    expect(db!.version).toBe(DB_VERSION)
  })

  test('object stores exist', () => {
    expect(db).not.toBeNull()
    const transaction = db!.transaction(['messages', 'chatHistory'], 'readonly')
    expect(transaction.objectStore('messages')).toBeDefined()
    expect(transaction.objectStore('chatHistory')).toBeDefined()
  })

  test('can add message to store', async () => {
    const message = {
      id: 'test-message-1',
      chatId: 'chat-1',
      content: 'Hello, world!',
      timestamp: Date.now(),
      direction: 'incoming',
    }

    return new Promise<void>((resolve, reject) => {
      const transaction = db!.transaction(['messages'], 'readwrite')
      const store = transaction.objectStore('messages')
      const request = store.add(message)

      request.onsuccess = () => resolve()
      request.onerror = () => reject(request.error)
    })
  })

  test('can retrieve message from store', async () => {
    const message = {
      id: 'test-message-2',
      chatId: 'chat-1',
      content: 'Test message',
      timestamp: Date.now(),
      direction: 'outgoing',
    }

    // Add message
    await new Promise<void>((resolve, reject) => {
      const transaction = db!.transaction(['messages'], 'readwrite')
      const store = transaction.objectStore('messages')
      const request = store.add(message)

      request.onsuccess = () => resolve()
      request.onerror = () => reject(request.error)
    })

    // Retrieve message
    return new Promise<void>((resolve, reject) => {
      const transaction = db!.transaction(['messages'], 'readonly')
      const store = transaction.objectStore('messages')
      const request = store.get('test-message-2')

      request.onsuccess = () => {
        const retrieved = request.result
        expect(retrieved).toBeDefined()
        expect(retrieved.content).toBe('Test message')
        resolve()
      }
      request.onerror = () => reject(request.error)
    })
  })

  test('can update message in store', async () => {
    const message = {
      id: 'test-message-3',
      chatId: 'chat-1',
      content: 'Original message',
      timestamp: Date.now(),
      direction: 'outgoing',
    }

    // Add message
    await new Promise<void>((resolve, reject) => {
      const transaction = db!.transaction(['messages'], 'readwrite')
      const store = transaction.objectStore('messages')
      const request = store.add(message)

      request.onsuccess = () => resolve()
      request.onerror = () => reject(request.error)
    })

    // Update message
    const updatedMessage = { ...message, content: 'Updated message' }

    await new Promise<void>((resolve, reject) => {
      const transaction = db!.transaction(['messages'], 'readwrite')
      const store = transaction.objectStore('messages')
      const request = store.put(updatedMessage)

      request.onsuccess = () => resolve()
      request.onerror = () => reject(request.error)
    })

    // Verify update
    return new Promise<void>((resolve, reject) => {
      const transaction = db!.transaction(['messages'], 'readonly')
      const store = transaction.objectStore('messages')
      const request = store.get('test-message-3')

      request.onsuccess = () => {
        const retrieved = request.result
        expect(retrieved.content).toBe('Updated message')
        resolve()
      }
      request.onerror = () => reject(request.error)
    })
  })

  test('can delete message from store', async () => {
    const message = {
      id: 'test-message-4',
      chatId: 'chat-1',
      content: 'Message to delete',
      timestamp: Date.now(),
      direction: 'outgoing',
    }

    // Add message
    await new Promise<void>((resolve, reject) => {
      const transaction = db!.transaction(['messages'], 'readwrite')
      const store = transaction.objectStore('messages')
      const request = store.add(message)

      request.onsuccess = () => resolve()
      request.onerror = () => reject(request.error)
    })

    // Delete message
    await new Promise<void>((resolve, reject) => {
      const transaction = db!.transaction(['messages'], 'readwrite')
      const store = transaction.objectStore('messages')
      const request = store.delete('test-message-4')

      request.onsuccess = () => resolve()
      request.onerror = () => reject(request.error)
    })

    // Verify deletion
    return new Promise<void>((resolve, reject) => {
      const transaction = db!.transaction(['messages'], 'readonly')
      const store = transaction.objectStore('messages')
      const request = store.get('test-message-4')

      request.onsuccess = () => {
        const retrieved = request.result
        expect(retrieved).toBeUndefined()
        resolve()
      }
      request.onerror = () => reject(request.error)
    })
  })

  test('can add chat history', async () => {
    const chatHistory = {
      id: 'chat-history-1',
      peerId: 'peer-1',
      messages: [],
      unreadCount: 0,
      lastMessage: null,
      lastUpdated: Date.now(),
    }

    return new Promise<void>((resolve, reject) => {
      const transaction = db!.transaction(['chatHistory'], 'readwrite')
      const store = transaction.objectStore('chatHistory')
      const request = store.add(chatHistory)

      request.onsuccess = () => resolve()
      request.onerror = () => reject(request.error)
    })
  })

  test('can retrieve all messages for a chat', async () => {
    const messages = [
      {
        id: 'msg-1',
        chatId: 'chat-test',
        content: 'Message 1',
        timestamp: Date.now(),
        direction: 'outgoing',
      },
      {
        id: 'msg-2',
        chatId: 'chat-test',
        content: 'Message 2',
        timestamp: Date.now(),
        direction: 'incoming',
      },
    ]

    // Add messages
    for (const message of messages) {
      await new Promise<void>((resolve, reject) => {
        const transaction = db!.transaction(['messages'], 'readwrite')
        const store = transaction.objectStore('messages')
        const request = store.add(message)

        request.onsuccess = () => resolve()
        request.onerror = () => reject(request.error)
      })
    }

    // Retrieve all messages for chat
    return new Promise<void>((resolve, reject) => {
      const transaction = db!.transaction(['messages'], 'readonly')
      const store = transaction.objectStore('messages')
      const index = store.index('chatId')
      const request = index.getAll('chat-test')

      request.onsuccess = () => {
        const retrieved = request.result
        expect(retrieved.length).toBe(2)
        resolve()
      }
      request.onerror = () => reject(request.error)
    })
  })

  test('handles transaction errors', async () => {
    return new Promise<void>((resolve, reject) => {
      const transaction = db!.transaction(['messages'], 'readwrite')
      const store = transaction.objectStore('messages')

      // Try to add invalid data (missing required field)
      const request = store.add({ id: 'invalid' })

      request.onerror = () => {
        expect(request.error).toBeDefined()
        resolve()
      }

      request.onsuccess = () => {
        reject(new Error('Should have failed'))
      }
    })
  })

  test('can clear all messages', async () => {
    // Add some messages first
    const messages = [
      {
        id: 'msg-clear-1',
        chatId: 'chat-clear',
        content: 'Message 1',
        timestamp: Date.now(),
        direction: 'outgoing',
      },
      {
        id: 'msg-clear-2',
        chatId: 'chat-clear',
        content: 'Message 2',
        timestamp: Date.now(),
        direction: 'incoming',
      },
    ]

    for (const message of messages) {
      await new Promise<void>((resolve, reject) => {
        const transaction = db!.transaction(['messages'], 'readwrite')
        const store = transaction.objectStore('messages')
        const request = store.add(message)

        request.onsuccess = () => resolve()
        request.onerror = () => reject(request.error)
      })
    }

    // Clear all messages
    await new Promise<void>((resolve, reject) => {
      const transaction = db!.transaction(['messages'], 'readwrite')
      const store = transaction.objectStore('messages')
      const request = store.clear()

      request.onsuccess = () => resolve()
      request.onerror = () => reject(request.error)
    })

    // Verify all cleared
    return new Promise<void>((resolve, reject) => {
      const transaction = db!.transaction(['messages'], 'readonly')
      const store = transaction.objectStore('messages')
      const request = store.getAll()

      request.onsuccess = () => {
        const retrieved = request.result
        expect(retrieved.length).toBe(0)
        resolve()
      }
      request.onerror = () => reject(request.error)
    })
  })
})
