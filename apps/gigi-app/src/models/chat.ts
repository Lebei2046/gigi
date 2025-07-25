import { db, type Chat } from './db';
import { useLiveQuery } from 'dexie-react-hooks';

// 添加聊天相关函数
export async function addChat(chat: Omit<Chat, 'id'>) {
  await db.chats.add(chat);
}

export async function updateChat(id: number, updates: Partial<Chat>) {
  await db.chats.update(id, updates);
}

export async function deleteChat(id: number) {
  await db.chats.delete(id);
}

export function useAllChats() {
  return useLiveQuery(() => db.chats.toArray(), []);
}

export function useChat(id: number) {
  return useLiveQuery(() => db.chats.get(id), [id]);
}
