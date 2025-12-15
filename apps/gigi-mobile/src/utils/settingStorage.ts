/**
 * Safe storage wrapper with validation using IndexedDB
 */

import { db } from '../models/db'

type StorageData = {
  version: string
  data: unknown
}

const STORAGE_VERSION = 'v1'

export async function getStorageItem<T>(key: string): Promise<T | null> {
  try {
    const setting = await db.settings.get(key)
    if (!setting) return null

    const parsed = JSON.parse(setting.value) as StorageData
    if (parsed.version !== STORAGE_VERSION) {
      console.warn(`Storage version mismatch for ${key}, clearing...`)
      return null
    }

    return parsed.data as T
  } catch (error) {
    console.error(`Failed to parse ${key} from IndexedDB:`, error)
    return null
  }
}

export async function setStorageItem<T>(key: string, value: T): Promise<void> {
  try {
    const data: StorageData = {
      version: STORAGE_VERSION,
      data: value,
    }
    await db.settings.put({
      key,
      value: JSON.stringify(data),
      updatedAt: new Date(),
    })
  } catch (error) {
    console.error(`Failed to store ${key} in IndexedDB:`, error)
  }
}

export async function clearStorageItem(key: string): Promise<void> {
  try {
    await db.settings.delete(key)
  } catch (error) {
    console.error(`Failed to remove ${key} from IndexedDB:`, error)
  }
}
