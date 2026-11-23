// One Open door (V3 gap 28): whatever file is given, say what it is by looking inside it, and offer the one way to
// open it. V3 named a wrong file ("this is a campaign, open it as one?"); SSE4 never asks which door to use.
import { CAMPAIGN_SCHEMA, type CampaignSave } from "./campaign.ts";
import { CLIP_VERSION } from "./clip.ts";

export type FileKind =
  | "v4-campaign" // a campaign from Save campaign
  | "v4-clip" // bodies copied from a chart (Ctrl+C), saved as a file
  | "v3-clip" // a clip from Star System Explorer 3
  | "v3-save" // a Star System Explorer 3 save: starmap.json, system.json or .sse.zip
  | "pack" // an sse.pack/1 document
  | "diagnostic" // a diagnostic file from Your data
  | "other-app" // Universe Sandbox or SpaceEngine: through the converter
  | "unknown";

export interface Sniffed {
  kind: FileKind;
  /** What the file is, in a sentence a newcomer follows. */
  what: string;
  /** The parsed JSON, when it was JSON. */
  doc?: unknown;
}

export const CONVERTER_URL = "https://convert.starsystemx.com/";

const isObj = (x: unknown): x is Record<string, unknown> => !!x && typeof x === "object" && !Array.isArray(x);

/** Name and leading bytes are enough: zips and other apps' files are told by name and magic, the rest by content. */
export function sniffFile(name: string, text: string | null, bytes?: Uint8Array): Sniffed {
  const lower = name.toLowerCase();
  const zip = !!bytes && bytes.length > 4 && bytes[0] === 0x50 && bytes[1] === 0x4b && bytes[2] === 0x03 && bytes[3] === 0x04;
  if (lower.endsWith(".ubox")) return { kind: "other-app", what: "A Universe Sandbox simulation (.ubox). The converter turns it into a file SSE4 opens, in your own browser." };
  if (lower.endsWith(".sc") || lower.endsWith(".pak")) return { kind: "other-app", what: `A SpaceEngine ${lower.endsWith(".pak") ? "package (.pak)" : "script (.sc)"}. The converter turns it into a file SSE4 opens, in your own browser.` };
  if (zip || lower.endsWith(".sse.zip")) return { kind: "v3-save", what: "A Star System Explorer 3 save (.sse.zip). It comes in as new systems in this campaign." };
  let doc: unknown;
  try {
    doc = JSON.parse((text ?? "").trim());
  } catch {
    return { kind: "unknown", what: "Not a file SSE4 can read: it is not JSON, a .sse.zip, a .ubox or a SpaceEngine file." };
  }
  if (!isObj(doc)) return { kind: "unknown", what: "A JSON file, but not one SSE4 recognises.", doc };
  if (doc.schema === CAMPAIGN_SCHEMA) {
    const s = doc as unknown as CampaignSave;
    const ev = isObj(s.export) && Array.isArray((s.export as { events?: unknown[] }).events) ? (s.export as { events: unknown[] }).events.length : null;
    return { kind: "v4-campaign", what: `A campaign saved from SSE4${s.title ? `: ${s.title}` : ""}${s.saved_at ? `, kept ${new Date(s.saved_at).toLocaleString("en-GB", { dateStyle: "medium", timeStyle: "short" })}` : ""}${ev !== null ? `, ${ev} ${ev === 1 ? "event" : "events"}` : ""}. Opening it replaces the campaign open now.`, doc };
  }
  if (doc.sseClip === CLIP_VERSION) {
    const n = Array.isArray(doc.nodes) ? doc.nodes.length : 0;
    return { kind: "v4-clip", what: `${n === 1 ? "A body" : `${n} bodies`} copied from an SSE4 chart. It pastes into the system on the chart, on the Build desk.`, doc };
  }
  if (doc.sseClip === 1) return { kind: "v3-clip", what: "A clip from Star System Explorer 3. V3 clips come in through the V3 importer, with the next part of paste.", doc };
  if (doc.format === "sse.pack/1") return { kind: "pack", what: `A pack: ${String(doc.title ?? doc.id ?? "untitled")}${doc.version ? ` ${String(doc.version)}` : ""}. Adding it shows every value it would change first.`, doc };
  if (doc.kind === "sse4-diagnostic") return { kind: "diagnostic", what: "A diagnostic file from Your data. Tars reads these; it opens nothing here.", doc };
  if ("bundleFormat" in doc || (Array.isArray(doc.systems) && !("schema" in doc)) || (Array.isArray(doc.bodies) && ("name" in doc || "id" in doc))) {
    const n = Array.isArray(doc.systems) ? doc.systems.length : 1;
    return { kind: "v3-save", what: `A Star System Explorer 3 save${typeof doc.name === "string" ? `: ${doc.name}` : ""}, ${n} ${n === 1 ? "system" : "systems"}. It comes in as new systems in this campaign.`, doc };
  }
  return { kind: "unknown", what: "A JSON file, but not one SSE4 recognises.", doc };
}

/** Hex SHA-256 of bytes, with the platform's own digest. */
export async function sha256Hex(bytes: Uint8Array): Promise<string> {
  const d = await crypto.subtle.digest("SHA-256", bytes as BufferSource);
  return Array.from(new Uint8Array(d), (b) => b.toString(16).padStart(2, "0")).join("");
}

/** One Explorers V4 listing (GET /api/v4/<slug>): a pack or a campaign, the sha256 of its bytes and its source. */
export interface ExplorersListing {
  sha256: string;
  source: { site: string; title: string; url: string; creditChain: string[] };
  pack?: { id: string; version: string };
  campaign?: { root: string; stateHash?: string; events?: number };
}

export type ExplorersFetched =
  | { ok: true; listing: ExplorersListing; bytes: Uint8Array; text: string }
  | { ok: false; why: string; noShelf?: boolean };

/** Only Explorers itself may be opened from a link. */
export function explorersUrl(raw: string): URL | null {
  try {
    const u = new URL(raw);
    if (u.protocol !== "https:" || !/(^|\.)starsystemx\.com$/.test(u.hostname) || !u.hostname.startsWith("explorers.")) return null;
    if (!/^\/api\/v4\/[A-Za-z0-9._-]+\/?$/.test(u.pathname)) return null;
    return u;
  } catch {
    return null;
  }
}

/**
 * Fetch a listing and its file from Explorers and check the bytes are the ones listed: the listing's sha256, the
 * file's x-sse-sha256 header and our own digest must all agree. Nothing is opened here.
 */
export async function fetchExplorers(raw: string, get: typeof fetch = fetch): Promise<ExplorersFetched> {
  const u = explorersUrl(raw);
  if (!u) return { ok: false, why: "That link is not an Explorers V4 listing." };
  const base = u.href.replace(/\/$/, "");
  let listing: ExplorersListing;
  try {
    const r = await get(base, { cache: "no-store" });
    if (r.status === 404) return { ok: false, noShelf: true, why: "Explorers has nothing at that link yet: no V4 shelf holds it." };
    if (!r.ok) return { ok: false, why: `Explorers answered ${r.status}.` };
    listing = (await r.json()) as ExplorersListing;
  } catch (e) {
    return { ok: false, why: `Explorers could not be reached (${(e as Error).message}).` };
  }
  if (!isObj(listing) || typeof listing.sha256 !== "string" || !isObj(listing.source)) return { ok: false, noShelf: true, why: "Explorers has no V4 shelf for that yet." };
  let bytes: Uint8Array;
  let header: string | null;
  try {
    const r = await get(`${base}/file`, { cache: "no-store" });
    if (!r.ok) return { ok: false, why: `Explorers would not give the file (${r.status}).` };
    header = r.headers.get("x-sse-sha256");
    bytes = new Uint8Array(await r.arrayBuffer());
  } catch (e) {
    return { ok: false, why: `The file could not be fetched (${(e as Error).message}).` };
  }
  const ours = await sha256Hex(bytes);
  if (ours !== listing.sha256.toLowerCase() || (header && header.toLowerCase() !== ours)) return { ok: false, why: "The file Explorers sent is not the one it lists (its sha256 differs), so it is not opened." };
  return { ok: true, listing, bytes, text: new TextDecoder().decode(bytes) };
}

/** "Made by A, then B" from a credit chain, or null. */
export function creditWords(chain: readonly string[] | undefined): string | null {
  return chain?.length ? `Made by ${chain.join(", then ")}` : null;
}
