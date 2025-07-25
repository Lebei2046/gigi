import { db, type Message } from './db';
import { useLiveQuery } from 'dexie-react-hooks';

export async function addMessage(message: Omit<Message, 'id'>) {
  return await db.messages.add(message);
}

export async function deleteMessage(id: number) {
  await db.messages.delete(id);
}

// Read all messages from a specific chat
export function useMessagesByChatId(chatId: number) {
  return useLiveQuery(() => db.messages.where('chatId').equals(chatId).toArray(), [chatId]);
}
