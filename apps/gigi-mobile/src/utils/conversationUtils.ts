/**
 * Conversation utilities for managing conversations via backend store
 */

import {
  get_conversations,
  get_conversation,
  upsert_conversation,
  update_conversation_last_message,
  increment_conversation_unread,
  mark_conversation_as_read,
  delete_conversation,
} from 'tauri-plugin-gigi-api'

// Frontend Conversation type matching backend
export interface Conversation {
  id: string
  name: string
  is_group: boolean
  peer_id: string
  last_message: string | null
  last_message_time: number | null
  last_message_timestamp: number | null
  unread_count: number
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
 * Handles both seconds and milliseconds format, and string timestamps
 */
export function isValidTimestamp(
  timestamp: number | string | undefined | null
): boolean {
  if (!timestamp || timestamp === 0 || timestamp === '') return false

  // Convert string timestamp to number if needed
  let timestampNum: number
  if (typeof timestamp === 'string') {
    // If it's an ISO string like "2026-01-18T09:42:54.487Z", parse it
    const date = new Date(timestamp)
    if (isNaN(date.getTime())) return false
    timestampNum = date.getTime()
  } else {
    // Convert to milliseconds if in seconds format
    timestampNum = ensureMilliseconds(timestamp)
  }

  // Check if it's after year 2000 (946684800000 ms = Sep 9, 2001)
  return timestampNum > 1000000000000
}

/**
 * Get all conversations from backend
 */
export async function getAllConversations(): Promise<Conversation[]> {
  try {
    const conversations = await get_conversations()
    return conversations || []
  } catch (error) {
    console.error('Failed to get conversations:', error)
    return []
  }
}

/**
 * Get a single conversation by ID
 */
export async function getConversationInfo(
  id: string
): Promise<Conversation | undefined> {
  try {
    const conversation = await get_conversation({ id })
    return conversation
  } catch (error) {
    console.error('Failed to get conversation info:', error)
    return undefined
  }
}

/**
 * Create or update a conversation
 */
export async function updateConversationInfo(
  id: string,
  name: string,
  message: string,
  timestamp: number,
  isGroup: boolean = false
): Promise<void> {
  try {
    const timestampMs = ensureMilliseconds(timestamp)
    const peerId = isGroup ? id : id // For groups, peer_id is same as id
    await upsert_conversation({
      id,
      name,
      isGroup,
      peerId,
      lastMessage: message,
      lastMessageTimestamp: timestampMs,
    })
  } catch (error) {
    console.error('Failed to update conversation info:', error)
  }
}

/**
 * Update last message for a conversation
 */
export async function updateLatestMessage(
  conversationId: string,
  message: string,
  timestamp: number,
  isOutgoing: boolean = false,
  isGroup: boolean = false,
  incrementUnread: boolean = false
): Promise<void> {
  try {
    const timestampMs = ensureMilliseconds(timestamp)

    // Update the last message
    await update_conversation_last_message({
      id: conversationId,
      lastMessage: message,
      lastMessageTimestamp: timestampMs,
    })

    // Handle unread count
    if (incrementUnread && !isOutgoing) {
      await increment_conversation_unread({ id: conversationId })
    } else if (isOutgoing) {
      await mark_conversation_as_read({ id: conversationId })
    }
  } catch (error) {
    console.error('Failed to update latest message:', error)
  }
}

/**
 * Reset unread count (mark as read)
 */
export async function resetUnreadCount(conversationId: string): Promise<void> {
  try {
    await mark_conversation_as_read({ id: conversationId })

    // Force UI refresh by dispatching a custom event
    window.dispatchEvent(
      new CustomEvent('unreadCountReset', {
        detail: { conversationId, previousCount: 0 },
      })
    )
  } catch (error) {
    console.error('Failed to reset unread count:', error)
  }
}

/**
 * Delete a conversation
 */
export async function removeConversation(
  conversationId: string
): Promise<void> {
  try {
    await delete_conversation({ id: conversationId })
  } catch (error) {
    console.error('Failed to delete conversation:', error)
  }
}

/**
 * Ensure conversation entries exist for all groups
 * This migrates group data to conversations
 */
export async function ensureConversationsForGroups(
  groups: { id: string; name: string }[]
): Promise<void> {
  try {
    const allConversations = await getAllConversations()
    const conversationIds = new Set(allConversations.map(c => c.id))

    for (const group of groups) {
      if (!conversationIds.has(group.id)) {
        await upsert_conversation({
          id: group.id,
          name: group.name,
          isGroup: true,
          peerId: group.id,
          lastMessage: '',
          lastMessageTimestamp: 0,
        })
      }
    }
  } catch (error) {
    console.error('Failed to ensure conversations for groups:', error)
  }
}
