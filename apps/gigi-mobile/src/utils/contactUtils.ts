/**
 * Contact utilities for managing contacts in backend store
 */

import {
  contactAdd,
  contactGetAll,
  contactGet,
  contactRemove,
  contactUpdate,
} from '@/utils/tauriCommands'

// Local Contact type
export interface Contact {
  id: string // peer_id
  name: string
  addedAt: Date
}

/**
 * Get all contacts from backend
 */
export async function getAllContacts(): Promise<Contact[]> {
  try {
    const backendContacts = await contactGetAll()

    // Map backend contacts to local Contact type
    return backendContacts
      .map(c => ({
        id: c.peer_id,
        name: c.name,
        addedAt: new Date(c.added_at),
      }))
      .sort((a, b) => b.addedAt.getTime() - a.addedAt.getTime())
  } catch (error) {
    console.error('Failed to get all contacts from backend:', error)
    return []
  }
}

/**
 * Get a contact by peer ID
 */
export async function getContact(id: string): Promise<Contact | undefined> {
  try {
    const backendContact = await contactGet(id)
    if (backendContact) {
      return {
        id: backendContact.peer_id,
        name: backendContact.name,
        addedAt: new Date(backendContact.added_at),
      }
    }
    return undefined
  } catch (error) {
    console.error('Failed to get contact from backend:', error)
    return undefined
  }
}

/**
 * Add a contact
 */
export async function addContact(name: string, peerId: string): Promise<void> {
  try {
    await contactAdd(peerId, name)
  } catch (error) {
    console.error('Failed to add contact:', error)
    throw error
  }
}

/**
 * Remove a contact
 */
export async function removeContact(peerId: string): Promise<void> {
  try {
    await contactRemove(peerId)
  } catch (error) {
    console.error('Failed to remove contact:', error)
    throw error
  }
}

/**
 * Update a contact's name
 */
export async function updateContact(
  peerId: string,
  name: string
): Promise<void> {
  try {
    await contactUpdate(peerId, name)
  } catch (error) {
    console.error('Failed to update contact:', error)
    throw error
  }
}
