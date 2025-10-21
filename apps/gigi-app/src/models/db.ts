import Dexie, { type EntityTable } from 'dexie';

interface Contact {
  id: string; // address作为唯一标识
  name: string;
}

interface Chat {
  id: string; // address作为唯一标识
  name: string;
  isGroup?: boolean;
  lastMessage?: string;
  lastMessageTime?: string;
  unreadCount?: number;
}

interface Message {
  id?: number;
  chatId: string;
  sender: string;
  content: string;
  timestamp: Date;
}

// 添加图片存储接口
interface Image {
  id: string;
  data: Blob;
  type: string;
  createdAt: Date;
}

// 添加头像接口
interface Avatar {
  id: string; // address作为唯一标识
  imageId: string; // 对应images表中的imageId
  createdAt: Date;
  updatedAt: Date;
}

// 添加设置接口
interface Settings {
  key: string;
  value: string;
  updatedAt: Date;
}

const db = new Dexie('GigiDatabase') as Dexie & {
  contacts: EntityTable<Contact, 'id'>;
  chats: EntityTable<Chat, 'id'>;
  messages: EntityTable<Message, 'id'>;
  images: EntityTable<Image, 'id'>;
  avatars: EntityTable<Avatar, 'id'>;
  settings: EntityTable<Settings, 'key'>;
};

db.version(1).stores({
  contacts: 'id, name',
  chats: 'id, name',
  messages: '++id, chatId, timestamp',
  images: 'id, createdAt',
  avatars: 'id, imageId, createdAt, updatedAt',
  settings: 'key, updatedAt'
});

export type { Contact, Chat, Message, Image, Avatar, Settings };
export { db };
