const DB_NAME = 'gigi-wallet';
const DB_VERSION = 1;
const STORE_NAME = 'vault';

let db: IDBDatabase;

export async function initDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);

    request.onerror = () => reject(new Error('Failed to open DB'));
    request.onsuccess = () => {
      db = request.result;
      resolve(db);
    };

    request.onupgradeneeded = (event) => {
      const db = (event.target as IDBOpenDBRequest).result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME, { keyPath: 'id' });
      }
    };
  });
}

export async function encryptAndStore(data: { id: string; value: Uint8Array }, password: string): Promise<void> {
  if (!db) await initDB();

  const encrypted = await crypto.subtle.encrypt(
    { name: 'AES-GCM', iv: new Uint8Array(12) },
    await crypto.subtle.importKey(
      'raw',
      new TextEncoder().encode(password),
      { name: 'AES-GCM' },
      false,
      ['encrypt']
    ),
    data.value
  );

  return new Promise((resolve, reject) => {
    const transaction = db.transaction(STORE_NAME, 'readwrite');
    const store = transaction.objectStore(STORE_NAME);
    const request = store.put({ id: data.id, value: encrypted });

    request.onsuccess = () => resolve();
    request.onerror = () => reject(new Error('Failed to store data'));
  });
}
