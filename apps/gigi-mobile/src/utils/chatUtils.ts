/**
 * Chat utilities for managing chat data in IndexedDB
 */

import { db } from '@/models/db'
import type { Chat } from '@/models/db'
import type { GroupInfo } from '@/utils/tauriCommands'
import {
  groupGetAll,
  groupCreate,
  groupUpdate,
  groupGet,
} from '@/utils/tauriCommands'

// Local Group type (no longer stored in IndexedDB)
export interface Group {
  id: string
  name: string
  joined: boolean
  createdAt: Date
}

/**
 * Convert timestamp to milliseconds if it's in seconds
 * Backend might send timestamps in seconds, but JavaScript Date expects milliseconds
 */
export function ensureMilliseconds(timestamp: number): number {
  // If timestamp is less than year 2000 (about 946684800000), assume it's in seconds
  if (timestamp < 1000000000000) {
    return timestamp * 1000
  }
  return timestamp
}

/**
 * Check if a timestamp represents a valid date (after year 2000)
 * Helps identify invalid timestamps like 1970 dates
 */
export function isValidTimestamp(timestamp: number | undefined): boolean {
  return !!(timestamp && timestamp > 1000000000000)
}

/**
 * Check if a date string represents an invalid timestamp (like 1970 dates)
 */
function isInvalidDateString(dateString: any): boolean {
  if (!dateString || typeof dateString !== 'string') return false
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
        if (isValidTimestamp(chat.lastMessageTimestamp)) {
          await db.chats.update(chat.id, {
            lastMessageTime: new Date(
              chat.lastMessageTimestamp!
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
      const previousCount = existingChat.unreadCount || 0

      await db.chats.update(chatId, {
        unreadCount: 0,
      })

      if (previousCount > 0) {
        // Force UI refresh by dispatching a custom event
        window.dispatchEvent(
          new CustomEvent('unreadCountReset', {
            detail: { chatId, previousCount: 0 },
          })
        )
      }
    }
  } catch (error) {
    console.error('Failed to reset unread count:', error)
  }
}

export async function getAllGroups(): Promise<Group[]> {
  try {
    // Fetch groups from backend group store
    const backendGroups = await groupGetAll()

    // Map backend groups to local Group type and return sorted by createdAt
    return backendGroups
      .map(g => ({
        id: g.group_id,
        name: g.name,
        joined: g.joined,
        createdAt: new Date(g.created_at),
      }))
      .sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime())
  } catch (error) {
    console.error('Failed to get all groups from backend:', error)
    return []
  }
}

export async function ensureChatEntriesForGroups(): Promise<void> {
  try {
    const allGroups = await getAllGroups()
    const allChats = await getAllChats()
    const chatIds = new Set(allChats.map(chat => chat.id))

    for (const group of allGroups) {
      if (!chatIds.has(group.id)) {
        await db.chats.put({
          id: group.id,
          name: group.name,
          isGroup: true,
          lastMessage: '',
          lastMessageTime: '',
          lastMessageTimestamp: 0,
          unreadCount: 0,
        })
      }
    }
  } catch (error) {
    console.error('Failed to ensure chat entries for groups:', error)
  }
}

export async function getGroup(id: string): Promise<Group | undefined> {
  try {
    // Fetch from backend
    const backendGroup = await groupGet(id)
    if (backendGroup) {
      // Map backend group to local Group type
      return {
        id: backendGroup.group_id,
        name: backendGroup.name,
        joined: backendGroup.joined,
        createdAt: new Date(backendGroup.created_at),
      }
    }
    return undefined
  } catch (error) {
    console.error('Failed to get group from backend:', error)
    return undefined
  }
}

export async function saveGroup(
  group: Omit<Group, 'id'> & { id?: string }
): Promise<void> {
  try {
    const groupId = group.id || crypto.randomUUID()

    // Check if group already exists in backend
    const existingGroup = await groupGet(groupId)
    if (existingGroup) {
      // Update existing group in backend
      await groupUpdate(groupId, group.name, group.joined)
    } else {
      // Create new group in backend
      await groupCreate(groupId, group.name, group.joined)
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
  isOutgoing: boolean = false,
  isGroup: boolean = false,
  incrementUnread: boolean = false // New parameter to control unread count increment
): Promise<void> {
  try {
    const timestampMs = ensureMilliseconds(timestamp)
    const existingChat = await db.chats.get(chatId)

    if (existingChat) {
      let newUnreadCount = existingChat.unreadCount || 0

      // Only increment unread count for incoming messages when explicitly requested
      if (incrementUnread && !isOutgoing) {
        newUnreadCount += 1
        console.log(
          `📈 Incrementing unread count for ${chatId} (${existingChat.name}) to ${newUnreadCount}`
        )
      } else if (isOutgoing) {
        newUnreadCount = 0
      }

      await db.chats.update(chatId, {
        lastMessage: message,
        lastMessageTime: new Date(timestampMs).toLocaleString(),
        lastMessageTimestamp: timestampMs,
        unreadCount: newUnreadCount,
      })
    } else {
      // Create new chat entry when one doesn't exist
      const groupName = isGroup
        ? `Group ${chatId.substring(0, 6)}...`
        : `Peer ${chatId.substring(0, 6)}...`
      await db.chats.put({
        id: chatId,
        name: groupName,
        isGroup,
        lastMessage: message,
        lastMessageTime: new Date(timestampMs).toLocaleString(),
        lastMessageTimestamp: timestampMs,
        unreadCount: isOutgoing ? 0 : 1, // Start with 1 for incoming messages
      })
    }
  } catch (error) {
    console.error('Failed to update latest message:', error)
  }
}
