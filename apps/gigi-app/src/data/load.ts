import { initialMessages } from "./messages";
import { allChats } from "./users";
import { senders } from "./senders";
import { db, type Contact, type Chat, type Message } from '../models/db';


export async function loadData() {
  const contacts: Contact[] = [];
  senders.forEach((sender) => {
    contacts.push({
      id: sender.id,
      name: sender.name,
    });
  });
  await db.contacts.bulkAdd(contacts);

  const chats: Chat[] = [];
  allChats.forEach((chat) => {
    chats.push({
      id: chat.id,
      name: chat.name,
      isGroup: chat.isGroup || false,
      lastMessage: chat.lastMessage,
      lastMessageTime: chat.lastMessageTime,
      unreadCount: chat.unreadCount ?? 0,
    });
  });
  await db.chats.bulkAdd(chats);

  const messages: Message[] = [];
  chats.forEach((chat) => {
    initialMessages.forEach((message) => {
      messages.push({
        chatId: chat.id,
        sender: message.senderId,
        content: message.content,
        timestamp: message.timestamp,
      });
    });
  });
  await db.messages.bulkAdd(messages);
}
