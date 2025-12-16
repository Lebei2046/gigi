/**
 * Chat utilities for managing chat data in IndexedDB
 */

import { db } from '@/models/db'
import type { Chat, Group } from '@/models/db'

/**
 * Convert timestamp to milliseconds if it's in seconds
 * Backend might send timestamps in seconds, but JavaScript Date expects milliseconds
 */
function ensureMilliseconds(timestamp: number): number {
  // If timestamp is less than year 2000 (about 946684800000), assume it's in seconds
  if (timestamp < 1000000000000) {
    return timestamp * 1000
  }
  return timestamp
}

/**
 * Check if a date string represents an invalid timestamp (like 1970 dates)
 */
function isInvalidDateString(dateString: string): boolean {
  if (!dateString) return false
  return dateString.includes('1970') || dateString.includes('1970/1/21')
}

/**
 * Clean up existing chat entries with incorrect timestamps
 */
export async function cleanupInvalidTimestamps(): Promise<void> {
  try {
    const allChats = await db.chats.toArray()
    let updatedCount = 0

    for (const chat of allChats) {
      if (chat.lastMessageTime && isInvalidDateString(chat.lastMessageTime)) {
        // If there's a valid timestamp, use it to recalculate the time
        if (
          chat.lastMessageTimestamp &&
          chat.lastMessageTimestamp > 1000000000000
        ) {
          await db.chats.update(chat.id, {
            lastMessageTime: new Date(
              chat.lastMessageTimestamp
            ).toLocaleString(),
          })
          updatedCount++
        } else {
          // Clear the invalid time if no valid timestamp available
          await db.chats.update(chat.id, {
            lastMessageTime: undefined,
          })
          updatedCount++
        }
      }
    }

    if (updatedCount > 0) {
      console.log(
        `🕐 Cleaned up ${updatedCount} chat entries with invalid timestamps`
      )
    }
  } catch (error) {
    console.error('Failed to cleanup invalid timestamps:', error)
  }
}

export async function updateChatInfo(
  id: string,
  name: string,
  message: string,
  timestamp: number,
  isGroup: boolean = false
): Promise<void> {
  try {
    const timestampMs = ensureMilliseconds(timestamp)
    console.log(
      '🕐 updateChatInfo - raw timestamp:',
      timestamp,
      'converted:',
      timestampMs,
      'date:',
      new Date(timestampMs)
    )
    await db.chats.put({
      id,
      name,
      isGroup,
      lastMessage: message,
      lastMessageTime: new Date(timestampMs).toLocaleString(),
      lastMessageTimestamp: timestampMs,
      unreadCount: 0, // Reset unread count when user is in chat
    })
  } catch (error) {
    console.error('Failed to update chat info:', error)
  }
}

export async function getChatInfo(id: string): Promise<Chat | undefined> {
  try {
    return await db.chats.get(id)
  } catch (error) {
    console.error('Failed to get chat info:', error)
    return undefined
  }
}

export async function resetUnreadCount(chatId: string): Promise<void> {
  try {
    const existingChat = await db.chats.get(chatId)
    if (existingChat) {
      await db.chats.update(chatId, {
        unreadCount: 0,
      })
      console.log('🔔 Reset unread count for chat:', chatId)
    }
  } catch (error) {
    console.error('Failed to reset unread count:', error)
  }
}

export async function getAllGroups(): Promise<Group[]> {
  try {
    return await db.groups.orderBy('createdAt').reverse().toArray()
  } catch (error) {
    console.error('Failed to get all groups:', error)
    return []
  }
}

export async function saveGroup(
  group: Omit<Group, 'id'> & { id?: string }
): Promise<void> {
  try {
    if (group.id) {
      await db.groups.put(group)
    } else {
      await db.groups.add({
        id: group.id || crypto.randomUUID(),
        name: group.name,
        joined: group.joined,
        createdAt: group.createdAt,
      })
    }
  } catch (error) {
    console.error('Failed to save group:', error)
  }
}

export async function getAllChats(): Promise<Chat[]> {
  try {
    return await db.chats.orderBy('lastMessageTimestamp').reverse().toArray()
  } catch (error) {
    console.error('Failed to get all chats:', error)
    return []
  }
}

export async function updateLatestMessage(
  chatId: string,
  message: string,
  timestamp: number,
  isOutgoing: boolean = false
): Promise<void> {
  try {
    const timestampMs = ensureMilliseconds(timestamp)
    console.log(
      '🕐 updateLatestMessage - raw timestamp:',
      timestamp,
      'converted:',
      timestampMs,
      'date:',
      new Date(timestampMs)
    )
    const existingChat = await db.chats.get(chatId)

    if (existingChat) {
      await db.chats.update(chatId, {
        lastMessage: message,
        lastMessageTime: new Date(timestampMs).toLocaleString(),
        lastMessageTimestamp: timestampMs,
        unreadCount: isOutgoing ? 0 : (existingChat.unreadCount || 0) + 1,
      })
    } else {
      console.log('🕐 No existing chat found for:', chatId)
    }
  } catch (error) {
    console.error('Failed to update latest message:', error)
  }
}
