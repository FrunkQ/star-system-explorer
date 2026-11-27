// Your data (V3 gap 27): what this browser keeps for SSE4, asking it to keep it, clearing it all, and a diagnostic
// file. Everything SSE4 keeps on a device is under the "sse4." prefix in local storage (the campaign, its set-aside
// copy, settings, agent keys) plus the renderer's media store and the big packs a campaign uses (campaign-store.ts) in IndexedDB. Every call says what happened instead of
// throwing: private windows and blocked storage are normal.
import { CAMPAIGN_KEY, PACK_DB, SET_ASIDE_KEY } from "./campaign-store.ts";
import { roundTo } from "../../ui-contract/src/index.ts";

const PREFIX = "sse4.";
const MEDIA_DB = "sse-media";
/** Set while a kept campaign is being reopened; still set on the next visit means that open never finished. */
export const OPENING_KEY = "sse4.campaign.opening";

export interface StorageUse {
  /** Bytes SSE4 keeps in local storage, by part. */
  campaign: number;
  setAside: number;
  other: number;
  /** The browser's whole figure for this site (all stores), when it gives one. */
  usage: number | null;
  quota: number | null;
  /** True when the browser has agreed to keep this site's data; null when it cannot say. */
  persisted: boolean | null;
}

const size = (s: string | null) => (s ? s.length * 2 : 0); // local storage holds UTF-16

function keys(): string[] {
  try {
    return Array.from({ length: localStorage.length }, (_, i) => localStorage.key(i)).filter((k): k is string => !!k && k.startsWith(PREFIX));
  } catch {
    return [];
  }
}

function read(k: string): string | null {
  try {
    return localStorage.getItem(k);
  } catch {
    return null;
  }
}

export async function storageUse(): Promise<StorageUse> {
  let campaign = 0;
  let setAside = 0;
  let other = 0;
  for (const k of keys()) {
    const n = size(read(k)) + size(k);
    if (k === CAMPAIGN_KEY) campaign += n;
    else if (k === SET_ASIDE_KEY) setAside += n;
    else other += n;
  }
  let usage: number | null = null;
  let quota: number | null = null;
  let persisted: boolean | null = null;
  try {
    const e = await navigator.storage?.estimate?.();
    usage = e?.usage ?? null;
    quota = e?.quota ?? null;
  } catch {
    /* no estimate here */
  }
  try {
    persisted = (await navigator.storage?.persisted?.()) ?? null;
  } catch {
    /* cannot say */
  }
  return { campaign, setAside, other, usage, quota, persisted };
}

/** Ask the browser to keep this site's data when space runs low. The browser may decide without asking the person. */
export async function askToKeep(): Promise<boolean | null> {
  try {
    return (await navigator.storage?.persist?.()) ?? null;
  } catch {
    return null;
  }
}

/** "1.2 MB" and the like. */
export function bytesWords(n: number): string {
  if (n < 1000) return `${n} bytes`;
  const units = ["kB", "MB", "GB", "TB"];
  let v = n / 1000;
  let u = 0;
  while (v >= 1000 && u < units.length - 1) (v /= 1000), u++;
  return `${v >= 100 ? Math.round(v) : roundTo(v, 2)} ${units[u]}`;
}

/** The campaign set aside after a failed open, as text, or null. */
export function setAsideText(): string | null {
  return read(SET_ASIDE_KEY);
}

/** Put the set-aside campaign back as the kept one, to try opening it again. */
export function restoreSetAside(): boolean {
  try {
    const raw = localStorage.getItem(SET_ASIDE_KEY);
    if (!raw) return false;
    localStorage.setItem(CAMPAIGN_KEY, raw);
    localStorage.removeItem(SET_ASIDE_KEY);
    localStorage.removeItem(OPENING_KEY);
    return true;
  } catch {
    return false;
  }
}

/** Mark the start and the end of reopening the kept campaign. */
export function opening(on: boolean): void {
  try {
    if (on) localStorage.setItem(OPENING_KEY, new Date().toISOString());
    else localStorage.removeItem(OPENING_KEY);
  } catch {
    /* nothing kept */
  }
}

/** When the last reopen started and never finished (the tab closed or hung part way), its start time. */
export function unfinishedOpen(): string | null {
  return read(OPENING_KEY);
}

/** Forget everything SSE4 keeps on this device: the campaign, its set-aside copy, settings, agent keys, media and kept packs. */
export async function clearAll(): Promise<void> {
  for (const k of keys()) {
    try {
      localStorage.removeItem(k);
    } catch {
      /* already gone */
    }
  }
  for (const db of [MEDIA_DB, PACK_DB])
    await new Promise<void>((done) => {
      try {
        const r = indexedDB.deleteDatabase(db);
        r.onsuccess = r.onerror = r.onblocked = () => done();
      } catch {
        done();
      }
    });
}

/** Text a download can carry. */
export function downloadText(text: string, name: string, type = "application/json"): void {
  const url = URL.createObjectURL(new Blob([text], { type }));
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

// The last few problems the page met, for the diagnostic file. Collected from the start; never sent anywhere.
const problems: { at: string; message: string }[] = [];
export function watchProblems(): void {
  const keep = (message: string) => {
    problems.push({ at: new Date().toISOString(), message: message.slice(0, 500) });
    if (problems.length > 20) problems.shift();
  };
  addEventListener("error", (e) => keep(String(e.message || e.error)));
  addEventListener("unhandledrejection", (e) => keep(`unhandled: ${String(e.reason)}`));
}

export interface DiagnosticState {
  kernel: string;
  tier: string;
  state_hash: string;
  stack_hash: string;
  campaign: { title: string; events: number | null; saved_at: string | null };
  notes: string[];
}

/**
 * A diagnostic file the person saves to their own machine and can hand to Tars or attach to a problem report. It
 * says what ran and what went wrong; it never holds agent keys or the campaign itself (Save campaign carries that).
 */
export async function diagnostic(state: DiagnosticState): Promise<string> {
  const use = await storageUse();
  return JSON.stringify(
    {
      kind: "sse4-diagnostic",
      version: 1,
      made_at: new Date().toISOString(),
      page: location.pathname + location.hash,
      browser: navigator.userAgent,
      screen: { width: innerWidth, height: innerHeight, ratio: devicePixelRatio },
      kernel: state.kernel,
      device_tier: state.tier,
      state_hash: state.state_hash,
      stack_hash: state.stack_hash,
      campaign: state.campaign,
      storage: use,
      unfinished_open: unfinishedOpen(),
      notes: state.notes,
      problems: [...problems],
    },
    null,
    2,
  );
}
