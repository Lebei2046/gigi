import Dexie, { type EntityTable } from 'dexie'

interface Contact {
  id: string // peer-id作为唯一标识
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

// 添加图片存储接口
interface Image {
  id: string
  data: Blob
  type: string
  createdAt: Date
}

// 添加头像接口
interface Avatar {
  id: string // peer-id作为唯一标识
  imageId: string // 对应images表中的imageId
  createdAt: Date
  updatedAt: Date
}

// 添加设置接口
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

db.version(1).stores({
  contacts: 'id, name',
  groups: 'id, name, joined, createdAt',
  chats: 'id, name, isGroup, lastMessageTime, lastMessageTimestamp',
  images: 'id, createdAt',
  avatars: 'id, imageId, createdAt, updatedAt',
  settings: 'key, updatedAt',
})

export type { Contact, Group, Chat, Image, Avatar, Settings }
export { db }
