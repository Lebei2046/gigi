/**
 * Safe localStorage wrapper with validation
 */

type StorageData = {
  version: string
  data: unknown
}

const STORAGE_VERSION = 'v1'

export function getStorageItem<T>(key: string): T | null {
  try {
    const item = localStorage.getItem(key)
    if (!item) return null

    const parsed = JSON.parse(item) as StorageData
    if (parsed.version !== STORAGE_VERSION) {
      console.warn(`Storage version mismatch for ${key}, clearing...`)
      localStorage.removeItem(key)
      return null
    }

    return parsed.data as T
  } catch (error) {
    console.error(`Failed to parse ${key} from localStorage:`, error)
    localStorage.removeItem(key)
    return null
  }
}

export function setStorageItem<T>(key: string, value: T): void {
  try {
    const data: StorageData = {
      version: STORAGE_VERSION,
      data: value
    }
    localStorage.setItem(key, JSON.stringify(data))
  } catch (error) {
    console.error(`Failed to store ${key} in localStorage:`, error)
  }
}

export function clearStorageItem(key: string): void {
  localStorage.removeItem(key)
}