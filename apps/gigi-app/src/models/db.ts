import Dexie, { type EntityTable } from 'dexie';

interface Contact {
  id?: number;
  name: string;
  address: string;
}

interface Chat {
  id?: number;
  name: string;
  isGroup?: boolean;
  lastMessage?: string;
  lastMessageTime?: string;
  unreadCount?: number;
  originalId?: string; // 用于存储原始字符串ID
}

interface Message {
  id?: number;
  chatId: number;
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

const db = new Dexie('GigiDatabase') as Dexie & {
  contacts: EntityTable<Contact, 'id'>;
  chats: EntityTable<Chat, 'id'>;
  messages: EntityTable<Message, 'id'>;
  images: EntityTable<Image, 'id'>;
  avatars: EntityTable<Avatar, 'id'>; // 添加avatars表
};

// 版本1：只包含contacts表
db.version(1).stores({
  contacts: '++id, name, &address'
});

// 版本2：保留contacts表并添加chats表，chats表只将id作为自增主键
db.version(2).stores({
  contacts: '++id, name, &address',
  chats: '++id' // 只将id作为自增主键，移除其他索引字段
});

db.version(3).stores({
  contacts: '++id, name, &address',
  chats: '++id', // 只将id作为自增主键，移除其他索引字段
  messages: '++id, chatId, timestamp'
});

// 版本4：添加images表
db.version(4).stores({
  contacts: '++id, name, &address',
  chats: '++id',
  messages: '++id, chatId, timestamp',
  images: 'id, createdAt'
});

// 版本5：添加avatars表
db.version(5).stores({
  contacts: '++id, name, &address',
  chats: '++id',
  messages: '++id, chatId, timestamp',
  images: 'id, createdAt',
  avatars: 'id, imageId, createdAt, updatedAt' // id对应用户address
});

export type { Contact, Chat, Message, Image, Avatar };
export { db };
