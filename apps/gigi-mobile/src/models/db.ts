import Dexie, { type EntityTable } from 'dexie'

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

const db = new Dexie('GigiDatabase') as Dexie & {
  images: EntityTable<Image, 'id'>
  avatars: EntityTable<Avatar, 'id'>
}

db.version(2).stores({
  images: 'id, createdAt',
  avatars: 'id, imageId, createdAt, updatedAt',
})

export type { Image, Avatar }
export { db }
