import { db } from './db'
import { useLiveQuery } from 'dexie-react-hooks'

export async function addContact(name: string, peerId: string) {
  await db.contacts.add({ name, id: peerId })
}

export function useAllContacts() {
  return useLiveQuery(() => db.contacts.toArray(), [])
}
