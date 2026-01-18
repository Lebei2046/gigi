import Dexie, { type EntityTable } from 'dexie'

interface Contact {
  id: string // peer-id as unique identifier
  name: string
}

interface Group {
  id: string // group peer-id
  name: string
  joined: boolean // false = group creator/owner, true = invited member who joined
  createdAt: Date
}

// Add image storage interface
interface Image {
  id: string
  data: Blob
  type: string
  createdAt: Date
}

// Add avatar interface
interface Avatar {
  id: string // peer-id as unique identifier
  imageId: string // Corresponds to imageId in images table
  createdAt: Date
  updatedAt: Date
}

// Add settings interface
interface Settings {
  key: string
  value: string
  updatedAt: Date
}

const db = new Dexie('GigiDatabase') as Dexie & {
  contacts: EntityTable<Contact, 'id'>
  groups: EntityTable<Group, 'id'>
  images: EntityTable<Image, 'id'>
  avatars: EntityTable<Avatar, 'id'>
  settings: EntityTable<Settings, 'key'>
}

db.version(2)
  .stores({
    contacts: 'id, name',
    groups: 'id, name, joined, createdAt',
    images: 'id, createdAt',
    avatars: 'id, imageId, createdAt, updatedAt',
    settings: 'key, updatedAt',
  })

export type { Contact, Group, Image, Avatar, Settings }
export { db }
