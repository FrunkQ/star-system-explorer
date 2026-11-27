// Where the chart room keeps the open campaign between visits (U10): one
// save in this browser's local storage, written after every change, plus the
// same save as a file the GM downloads or opens. Storage can be absent or
// full (private windows, quotas): every call says what happened instead of
// throwing, and the campaign always opens, fresh if it has to.
import { checkCampaignSave, type CampaignSave, type Json } from "../../ui-contract/src/index.ts";

export const CAMPAIGN_KEY = "sse4.campaign";
/** A save that could not be reopened is set aside here, never thrown away. */
export const SET_ASIDE_KEY = "sse4.campaign.set-aside";

export function readStored(root: string): { save: CampaignSave | null; problem: string | null } {
  let raw: string | null = null;
  try {
    raw = localStorage.getItem(CAMPAIGN_KEY);
  } catch {
    return { save: null, problem: null };
  }
  if (!raw) return { save: null, problem: null };
  try {
    const doc = JSON.parse(raw) as unknown;
    const bad = checkCampaignSave(doc, root);
    return bad.length ? { save: null, problem: bad.join("; ") } : { save: doc as CampaignSave, problem: null };
  } catch {
    return { save: null, problem: "the kept save is not readable" };
  }
}

/** Keep a save; null when kept, else a sentence on why not. */
export function writeStored(save: CampaignSave): string | null {
  try {
    localStorage.setItem(CAMPAIGN_KEY, JSON.stringify(save));
    return null;
  } catch (e) {
    return /quota/i.test(String(e)) ? "this browser's storage is full" : "this browser does not allow storage here";
  }
}

/** Move the kept save aside (it failed to reopen), so the next visit starts fresh but nothing is lost. */
export function setAside(): void {
  try {
    const raw = localStorage.getItem(CAMPAIGN_KEY);
    if (raw) localStorage.setItem(SET_ASIDE_KEY, raw);
    localStorage.removeItem(CAMPAIGN_KEY);
  } catch {
    // Nothing kept, nothing to move.
  }
}

export function clearStored(): void {
  try {
    localStorage.removeItem(CAMPAIGN_KEY);
  } catch {
    // Nothing kept.
  }
}

/** Offer a save as a file download. */
export function download(save: CampaignSave, name: string): void {
  const url = URL.createObjectURL(new Blob([JSON.stringify(save)], { type: "application/json" }));
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

// Big packs kept beside the save (CM0). Local storage holds about five million characters for the whole site, and a
// pack added to the campaign is part of its save; Sol today alone is over four million. So a pack bigger than
// KEEP_INLINE is kept once in this browser's IndexedDB, by the SHA-256 of its text, and the kept save names it with a
// `sse.pack.kept/1` stub. Reopening puts the pack back before anything else reads the save. The file a GM downloads
// (Save campaign) always carries every pack in full, so it opens anywhere. Without IndexedDB (a private window, say)
// the pack stays in the save, exactly as before.
export const PACK_DB = "sse-packs";
const PACK_STORE = "packs";
export const KEPT_PACK = "sse.pack.kept/1";
/** Characters of pack text above which a pack is kept beside the save rather than in it. */
export const KEEP_INLINE = 256 * 1024;

export interface KeptPack {
  format: typeof KEPT_PACK;
  id: string;
  version: string;
  sha256: string;
}

const isKept = (x: unknown): x is KeptPack => !!x && typeof x === "object" && (x as KeptPack).format === KEPT_PACK && typeof (x as KeptPack).sha256 === "string";

function packDb(): Promise<IDBDatabase | null> {
  return new Promise((resolve) => {
    try {
      const r = indexedDB.open(PACK_DB, 1);
      r.onupgradeneeded = () => r.result.createObjectStore(PACK_STORE);
      r.onsuccess = () => resolve(r.result);
      r.onerror = () => resolve(null);
      r.onblocked = () => resolve(null);
    } catch {
      resolve(null);
    }
  });
}

function step<T>(db: IDBDatabase, mode: IDBTransactionMode, go: (s: IDBObjectStore) => IDBRequest<T>): Promise<T | null> {
  return new Promise((resolve) => {
    try {
      const tx = db.transaction(PACK_STORE, mode);
      const r = go(tx.objectStore(PACK_STORE));
      tx.oncomplete = () => resolve(r.result ?? null);
      tx.onerror = tx.onabort = () => resolve(null);
    } catch {
      resolve(null);
    }
  });
}

async function sha256(text: string): Promise<string> {
  const d = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text));
  return Array.from(new Uint8Array(d), (b) => b.toString(16).padStart(2, "0")).join("");
}

// The same pack object is saved after every change: its stub is worked out once.
const stubs = new WeakMap<object, KeptPack>();

/** The save as it is kept in this browser: big packs replaced by stubs, each pack kept once in IndexedDB. */
export async function slimSave(save: CampaignSave): Promise<CampaignSave> {
  const added = save.stack?.added ?? [];
  if (!added.some((p) => p && typeof p === "object" && !isKept(p))) return save;
  let db: IDBDatabase | null | undefined;
  const out: unknown[] = [];
  for (const p of added) {
    if (!p || typeof p !== "object" || isKept(p)) {
      out.push(p);
      continue;
    }
    let stub = stubs.get(p);
    if (!stub) {
      const text = JSON.stringify(p);
      if (text.length <= KEEP_INLINE) {
        out.push(p);
        continue;
      }
      db ??= await packDb();
      if (!db) {
        out.push(p);
        continue;
      }
      const doc = p as { id?: unknown; version?: unknown };
      const s: KeptPack = { format: KEPT_PACK, id: String(doc.id ?? ""), version: String(doc.version ?? ""), sha256: await sha256(text) };
      const have = await step(db, "readonly", (st) => st.count(s.sha256));
      if (!have && (await step(db, "readwrite", (st) => st.put(text, s.sha256))) === null) {
        out.push(p);
        continue;
      }
      stub = s;
      stubs.set(p, s);
    }
    out.push(stub);
  }
  db?.close();
  return { ...save, stack: { ...save.stack!, added: out as Json[] } };
}

/** A kept save with its packs put back; a problem in words when a pack it names is no longer in this browser. */
export async function fullSave(save: CampaignSave): Promise<{ save: CampaignSave | null; problem: string | null }> {
  const added = save.stack?.added ?? [];
  if (!added.some(isKept)) return { save, problem: null };
  const db = await packDb();
  const out: unknown[] = [];
  for (const p of added) {
    if (!isKept(p)) {
      out.push(p);
      continue;
    }
    const text = db ? await step<string>(db, "readonly", (st) => st.get(p.sha256) as IDBRequest<string>) : null;
    let doc: unknown = null;
    try {
      doc = text && (await sha256(text)) === p.sha256 ? JSON.parse(text) : null;
    } catch {
      doc = null;
    }
    if (!doc) {
      db?.close();
      return { save: null, problem: `the pack ${p.id} ${p.version} it uses is no longer kept in this browser` };
    }
    stubs.set(doc as object, p);
    out.push(doc);
  }
  db?.close();
  return { save: { ...save, stack: { ...save.stack!, added: out as Json[] } }, problem: null };
}

/** Let go of kept packs that neither the kept save nor its set-aside copy names any more. */
export async function pruneKeptPacks(): Promise<void> {
  const named = new Set<string>();
  for (const k of [CAMPAIGN_KEY, SET_ASIDE_KEY]) {
    try {
      const s = JSON.parse(localStorage.getItem(k) ?? "null") as CampaignSave | null;
      for (const p of s?.stack?.added ?? []) if (isKept(p)) named.add(p.sha256);
    } catch {
      return; // Cannot tell what is named: keep everything.
    }
  }
  const db = await packDb();
  if (!db) return;
  const keys = (await step(db, "readonly", (st) => st.getAllKeys())) ?? [];
  for (const k of keys) if (typeof k === "string" && !named.has(k)) await step(db, "readwrite", (st) => st.delete(k));
  db.close();
}
