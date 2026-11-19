// Photographs kept on this device (R3). The design's Media view (handoff
// 8.2) is not built yet, so photographs live in this browser's IndexedDB,
// listed per world on the Body screen, and can be saved as PNG files. Every
// call tolerates storage being unavailable (private windows, previews):
// the photograph is then kept for this page only.
export interface MediaItem {
  id: string;
  title: string;
  /** The world's address. */
  subject: string;
  caption: string;
  created: string;
  png: Blob;
  width: number;
  height: number;
}

const DB = "sse-media";
const STORE = "photographs";
const memory: MediaItem[] = [];

function open(): Promise<IDBDatabase | null> {
  return new Promise((resolve) => {
    try {
      const r = indexedDB.open(DB, 1);
      r.onupgradeneeded = () => r.result.createObjectStore(STORE, { keyPath: "id" });
      r.onsuccess = () => resolve(r.result);
      r.onerror = () => resolve(null);
    } catch {
      resolve(null);
    }
  });
}

export async function keepPhoto(item: MediaItem): Promise<"device" | "page"> {
  const db = await open();
  if (!db) {
    memory.push(item);
    return "page";
  }
  return new Promise((resolve) => {
    try {
      const tx = db.transaction(STORE, "readwrite");
      tx.objectStore(STORE).put(item);
      tx.oncomplete = () => resolve("device");
      tx.onerror = () => (memory.push(item), resolve("page"));
    } catch {
      memory.push(item);
      resolve("page");
    }
  });
}

export async function listPhotos(subject?: string): Promise<MediaItem[]> {
  const db = await open();
  const fromDb: MediaItem[] = !db
    ? []
    : await new Promise((resolve) => {
        try {
          const r = db.transaction(STORE, "readonly").objectStore(STORE).getAll();
          r.onsuccess = () => resolve(r.result as MediaItem[]);
          r.onerror = () => resolve([]);
        } catch {
          resolve([]);
        }
      });
  return [...fromDb, ...memory].filter((m) => !subject || m.subject === subject).sort((a, b) => (a.created < b.created ? 1 : -1));
}

export async function removePhoto(id: string): Promise<void> {
  const i = memory.findIndex((m) => m.id === id);
  if (i >= 0) memory.splice(i, 1);
  const db = await open();
  if (!db) return;
  await new Promise<void>((resolve) => {
    try {
      const tx = db.transaction(STORE, "readwrite");
      tx.objectStore(STORE).delete(id);
      tx.oncomplete = () => resolve();
      tx.onerror = () => resolve();
    } catch {
      resolve();
    }
  });
}
