import { invoke } from '@tauri-apps/api/core';

// Storage utilities for managing IndexedDB and local storage

/**
 * Clear all IndexedDB databases
 */
export const clearIndexedDB = async (): Promise<void> => {
  try {
    const databases = await indexedDB.databases();
    
    for (const database of databases) {
      if (database.name) {
        indexedDB.deleteDatabase(database.name);
      }
    }
    console.log('All IndexedDB databases cleared');
  } catch (error) {
    console.error('Error clearing IndexedDB:', error);
    throw error;
  }
};

/**
 * Clear all localStorage data
 */
export const clearLocalStorage = (): void => {
  localStorage.clear();
  console.log('Local storage cleared');
};

/**
 * Clear all sessionStorage data
 */
export const clearSessionStorage = (): void => {
  sessionStorage.clear();
  console.log('Session storage cleared');
};

/**
 * Clear all browser storage (IndexedDB, localStorage, sessionStorage)
 */
export const clearAllStorage = async (): Promise<void> => {
  await clearIndexedDB();
  clearLocalStorage();
  clearSessionStorage();
  console.log('All browser storage cleared');
};

/**
 * Clear Tauri app data (backend storage)
 */
export const clearAppData = async (): Promise<void> => {
  try {
    await invoke('clear_app_data');
    console.log('Tauri app data cleared');
  } catch (error) {
    console.error('Error clearing Tauri app data:', error);
    throw error;
  }
};

/**
 * Clear everything (browser storage + Tauri app data)
 */
export const clearEverything = async (): Promise<void> => {
  await clearAllStorage();
  await clearAppData();
  console.log('All storage cleared (browser + Tauri)');
};

/**
 * Get list of all IndexedDB databases
 */
export const getIndexedDBDatabases = async (): Promise<string[]> => {
  try {
    const databases = await indexedDB.databases();
    return databases.map(db => db.name).filter((name): name is string => !!name);
  } catch (error) {
    console.error('Error getting IndexedDB databases:', error);
    return [];
  }
};

/**
 * Delete a specific IndexedDB database
 */
export const deleteDatabase = async (databaseName: string): Promise<void> => {
  return new Promise((resolve, reject) => {
    const request = indexedDB.deleteDatabase(databaseName);
    
    request.onerror = () => {
      console.error(`Error deleting database ${databaseName}:`, request.error);
      reject(request.error);
    };
    
    request.onsuccess = () => {
      console.log(`Database ${databaseName} deleted successfully`);
      resolve();
    };
    
    request.onblocked = () => {
      console.warn(`Database ${databaseName} deletion blocked`);
      // Wait and try again
      setTimeout(() => {
        deleteDatabase(databaseName).then(resolve).catch(reject);
      }, 100);
    };
  });
};

// Make available in console for debugging (only after functions are defined)
if (typeof window !== 'undefined') {
  (window as any).clearDB = clearIndexedDB;
  (window as any).clearAll = clearAllStorage;
  (window as any).deleteDB = deleteDatabase;
}