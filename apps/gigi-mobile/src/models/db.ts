import Dexie, { type EntityTable } from 'dexie'

interface Contact {
  id: string // peer-id as unique identifier
  name: string
}

interface Group {
  id: string // group peer-id
  name: string
  joined: boolean // false = group creator/owner, true = invited member who joined
  createdAt: Date
}

interface Chat {
  id: string // peer-id or group-id for both direct chats and groups
  name: string // peer nickname for direct chat, group name for groups
  isGroup?: boolean // false = direct chat, true = group chat
  lastMessage?: string
  lastMessageTime?: string
  lastMessageTimestamp?: number // for sorting
  unreadCount?: number
}

// Add image storage interface
interface Image {
  id: string
  data: Blob
  type: string
  createdAt: Date
}

// Add avatar interface
interface Avatar {
  id: string // peer-id as unique identifier
  imageId: string // Corresponds to imageId in images table
  createdAt: Date
  updatedAt: Date
}

// Add settings interface
interface Settings {
  key: string
  value: string
  updatedAt: Date
}

const db = new Dexie('GigiDatabase') as Dexie & {
  contacts: EntityTable<Contact, 'id'>
  groups: EntityTable<Group, 'id'>
  chats: EntityTable<Chat, 'id'>
  images: EntityTable<Image, 'id'>
  avatars: EntityTable<Avatar, 'id'>
  settings: EntityTable<Settings, 'key'>
}

db.version(2)
  .stores({
    contacts: 'id, name',
    groups: 'id, name, joined, createdAt',
    chats:
      'id, name, isGroup, lastMessageTime, lastMessageTimestamp, unreadCount',
    images: 'id, createdAt',
    avatars: 'id, imageId, createdAt, updatedAt',
    settings: 'key, updatedAt',
  })
  .upgrade(tx => {
    // Initialize unreadCount for existing chats
    return tx
      .table('chats')
      .toCollection()
      .modify(chat => {
        if (chat.unreadCount === undefined) {
          chat.unreadCount = 0
        }
      })
  })

export type { Contact, Group, Chat, Image, Avatar, Settings }
export { db }
