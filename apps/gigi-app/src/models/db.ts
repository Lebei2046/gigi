import Dexie, { type EntityTable } from 'dexie';

interface Contact {
  id?: number;
  name: string;
  address: string;
}

const db = new Dexie('GigiDatabase') as Dexie & {
  contacts: EntityTable<Contact, 'id'>;
};

db.version(1).stores({
  contacts: '++id, name, &address'
});

export type { Contact };
export { db };
