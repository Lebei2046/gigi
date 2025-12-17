import Dexie, { type EntityTable } from 'dexie';

interface Contact {
  id: string; // address as unique identifier
  name: string;
}

interface Chat {
  id: string; // address as unique identifier
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

// Add image storage interface
interface Image {
  id: string;
  data: Blob;
  type: string;
  createdAt: Date;
}

// Add avatar interface
interface Avatar {
  id: string; // address as unique identifier
  imageId: string; // Corresponds to imageId in the images table
  createdAt: Date;
  updatedAt: Date;
}

// Add settings interface
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
