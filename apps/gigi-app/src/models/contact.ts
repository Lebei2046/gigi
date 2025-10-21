import { db } from './db';
import { useLiveQuery } from 'dexie-react-hooks';

export async function addContact(name: string, address: string) {
  await db.contacts.add({ name, id: address });
}

export function useAllContacts() {
  return useLiveQuery(() => db.contacts.toArray(), []);
}
