/**
 * Storage Migration Utility
 *
 * This utility helps migrate from the old dual-storage system to the new unified storage.
 * Old system:
 *   - localStorage: Full message history with embedded image base64 data
 *   - IndexedDB: Image Blobs
 * New system:
 *   - Backend SQLite: All message data
 *   - Backend files: Full-size images
 *   - Backend files: Thumbnails (small ~10KB images)
 *   - Frontend Redux: In-memory cache only (cleared on reload)
 *   - Frontend localStorage: Only metadata (chat list, unread counts)
 */

/**
 * Clear old message history from localStorage
 * This should be called once after migration to clean up old data
 */
export function clearOldMessageHistory() {
  try {
    // Get all localStorage keys
    const keys = Object.keys(localStorage)

    // Remove keys that match the old pattern
    const keysToRemove = keys.filter(
      key =>
        key.startsWith('chat_history_') || key.startsWith('chat_history_group_')
    )

    keysToRemove.forEach(key => {
      localStorage.removeItem(key)
    })

    console.log(`✅ Cleared ${keysToRemove.length} old message history entries`)
    return { success: true, count: keysToRemove.length }
  } catch (error) {
    console.error('❌ Failed to clear old message history:', error)
    return { success: false, error: String(error) }
  }
}

/**
 * Get size of old message history in localStorage
 * Useful for showing migration progress to users
 */
export function getOldMessageHistorySize() {
  try {
    const keys = Object.keys(localStorage)
    const historyKeys = keys.filter(
      key =>
        key.startsWith('chat_history_') || key.startsWith('chat_history_group_')
    )

    let totalSize = 0
    historyKeys.forEach(key => {
      const value = localStorage.getItem(key)
      if (value) {
        totalSize += value.length
      }
    })

    // Convert to human-readable format
    const inKB = (totalSize / 1024).toFixed(2)
    const inMB = (totalSize / 1024 / 1024).toFixed(2)

    return {
      keys: historyKeys.length,
      totalBytes: totalSize,
      sizeKB: Number(inKB),
      sizeMB: Number(inMB),
    }
  } catch (error) {
    console.error('❌ Failed to get old message history size:', error)
    return {
      keys: 0,
      totalBytes: 0,
      sizeKB: 0,
      sizeMB: 0,
      error: String(error),
    }
  }
}

/**
 * Check if migration has been done
 */
export function hasMigrationBeenDone() {
  return localStorage.getItem('storage_migration_done') === 'true'
}

/**
 * Mark migration as done
 */
export function markMigrationDone() {
  localStorage.setItem('storage_migration_done', 'true')
  localStorage.setItem('storage_migration_date', new Date().toISOString())
}

/**
 * Get migration date if it was done
 */
export function getMigrationDate() {
  const date = localStorage.getItem('storage_migration_date')
  return date ? new Date(date) : null
}

/**
 * Summary of old storage data for migration UI
 */
export function getMigrationSummary() {
  const historySize = getOldMessageHistorySize()
  const migrationDone = hasMigrationBeenDone()
  const migrationDate = getMigrationDate()

  return {
    historyKeys: historySize.keys,
    historySizeKB: historySize.sizeKB,
    historySizeMB: historySize.sizeMB,
    migrationDone,
    migrationDate,
  }
}
