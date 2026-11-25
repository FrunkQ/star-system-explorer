// Postcards (PC1, SSE4-postcards-design.md sections 3, 4, 7 and 12A). A postcard is a bookmark that can travel: the
// bookmark (where, when, which instrument, how it is looked at), its camera (`pose`), a fixed date, and enough of the
// world to draw it somewhere else. A few kilobytes of plain data, never a picture file: every screen draws its own
// still from it. Inside a campaign there is no separate postcard object (one store, agreed with the Run desk thread):
// a bookmark with a pose and a fixed date is a postcard. `sse.postcard/1` is only the envelope one travels in.
// Nothing about players is in it (no role, look or knowledge). Presentation only: never read by physics, never in the
// state hash.
import { BOOKMARK_SCHEMA, bookmarksFrom, makeBookmark, type Bookmark, type Whereabouts } from "./bookmark.ts";
import { readClip, type Clip, type ClipEvent, type ClipNode } from "./clip.ts";
import { checkPicture } from "./picture.ts";

export const POSTCARD_SCHEMA = "sse.postcard/1";
/** The kept picture's longer side (design section 2: "about 640 px wide"). */
export const STAMP_SIDE = 640;
/** The file a postcard goes as when it carries a clip or a stamp. */
export const POSTCARD_FILE_SUFFIX = ".ssepostcard.json";
/** The link's fragment key: `v4.starsystemx.com/#postcard=…`. */
export const POSTCARD_LINK_KEY = "postcard";
export const POSTCARD_SITE = "https://v4.starsystemx.com/";
/** The message box opens holding this (Alex, 25 Nov 01:15). A universe pack may give its own. */
export const DEFAULT_MESSAGE = "Wish you were here!";
/** What the back says where an expired message was (Alex, 25 Nov 01:26). */
export const MESSAGE_EXPIRED = "Message expired, but look at that view!";
export const MESSAGE_MAX = 280;
export const FROM_MAX = 60;
export const NAME_MAX = 120;
export const CAPTION_MAX = 400;
/** A postcard without a clip is a few kilobytes; one with a clip may be bigger, but not a campaign. */
export const POSTCARD_MAX = 512_000;

/** The instruments a still can be drawn from (screen document ids). The globe ("body") draws one today; the rest
 * as the renderer's still tier reaches them (PC2). */
export const STILL_INSTRUMENTS = ["body", "system", "surface", "galaxy", "cluster", "web", "explore"] as const;

export interface PackNamed {
  id: string;
  version: string;
}

/** How to get the world (design section 4). */
export type PostcardWorld =
  /** A place in the generated universe: its address, the universe's root and the packs it was made under. */
  | { how: "address"; root: string; packs: PackNamed[]; made_with: string }
  /** An authored place: a clip of the bodies in frame and what they orbit, as authored facts. Never for the shop. */
  | { how: "clip"; clip: Clip; packs: PackNamed[]; made_with: string }
  /** Kept inside its own campaign: the world is already there. */
  | { how: "here" };

/** Words on the back for one sending (12A point 8): plain text only, never published to the shop. */
export interface PostcardMessage {
  text: string;
  from: string;
}

export interface Postcard {
  schema: typeof POSTCARD_SCHEMA;
  id: string;
  name: string;
  /** The maker's words, optional. */
  caption?: string;
  /** The bookmark, with a fixed date and an instrument that can draw a still. Its pose, when present, is the camera. */
  bookmark: Bookmark & { at_yr: number };
  world: PostcardWorld;
  /** Who made it, oldest first, as a clip's creditChain. */
  credit: string[];
  /** A small kept picture (about 640 px), shown at once while the real one is drawn. */
  stamp?: string;
  /** This sending's message, when it has one. */
  message?: PostcardMessage;
}

const isObj = (x: unknown): x is Record<string, unknown> => !!x && typeof x === "object" && !Array.isArray(x);
/** Plain text: no control characters other than a line break. */
const plain = (s: string) => s.replace(/\r\n?/g, "\n").replace(/[\u0000-\u0009\u000b-\u001f\u007f]/g, "").trim();

function packsFrom(raw: unknown): PackNamed[] | null {
  if (!Array.isArray(raw)) return null;
  const out: PackNamed[] = [];
  for (const p of raw) {
    if (!isObj(p) || typeof p.id !== "string" || !p.id || typeof p.version !== "string") return null;
    out.push({ id: p.id, version: p.version });
  }
  return out;
}

function worldFrom(raw: unknown): PostcardWorld | string {
  if (!isObj(raw)) return "it does not say how to get its world";
  if (raw.how === "here") return { how: "here" };
  const packs = packsFrom(raw.packs ?? []);
  if (!packs) return "its packs are not a list of names and versions";
  const made_with = typeof raw.made_with === "string" ? raw.made_with : "";
  if (raw.how === "address") {
    if (typeof raw.root !== "string" || !raw.root) return "it does not name its universe";
    return { how: "address", root: raw.root, packs, made_with };
  }
  if (raw.how === "clip") {
    const c = readClip(JSON.stringify(raw.clip ?? null));
    if (!c.ok) return `its clip does not read: ${c.why}`;
    return { how: "clip", clip: c.clip, packs, made_with };
  }
  return "it does not say how to get its world";
}

/** What is wrong with a message, one line; null when it is fine. */
export function checkMessage(m: unknown): string | null {
  if (!isObj(m) || typeof m.text !== "string" || typeof m.from !== "string") return "the message is not words and a name";
  if (plain(m.text).length > MESSAGE_MAX) return `the message is longer than ${MESSAGE_MAX} letters`;
  if (plain(m.from).length > FROM_MAX) return `who it is from is longer than ${FROM_MAX} letters`;
  return null;
}

/** A message for one sending: plain text, trimmed; empty words become the default line. */
export function makeMessage(text: string, from: string, defaultLine = DEFAULT_MESSAGE): PostcardMessage {
  const t = plain(text).slice(0, MESSAGE_MAX);
  return { text: t || defaultLine, from: plain(from).slice(0, FROM_MAX) };
}

export type PostcardRead = { ok: true; postcard: Postcard } | { ok: false; why: string };

const no = (why: string): PostcardRead => ({ ok: false, why: `This is not a postcard SSE4 can open: ${why}.` });

/** Read and check a postcard envelope. Anything wrong refuses it with one plain line; a bad stamp, caption or
 * message is dropped instead, since the postcard still draws without them. */
export function checkPostcard(raw: unknown): PostcardRead {
  if (!isObj(raw)) return no("it is not an object");
  if (raw.schema !== POSTCARD_SCHEMA) return no(typeof raw.schema === "string" && raw.schema.startsWith("sse.postcard/") ? `it is ${raw.schema}, newer than this version reads` : "it has no postcard schema");
  if (typeof raw.id !== "string" || !raw.id) return no("it has no id");
  if (typeof raw.name !== "string" || !plain(raw.name)) return no("it has no name");
  const [b] = bookmarksFrom([raw.bookmark]);
  if (!b) return no("its bookmark does not read");
  if (b.at_yr === null) return no("it has no fixed date");
  if (!(STILL_INSTRUMENTS as readonly string[]).includes(b.instrument)) return no(`its instrument "${b.instrument}" cannot draw a picture`);
  if (isObj(raw.bookmark) && raw.bookmark.pose !== undefined && !b.pose) return no("its camera does not read");
  const world = worldFrom(raw.world);
  if (typeof world === "string") return no(world);
  const credit = Array.isArray(raw.credit) ? raw.credit.filter((x): x is string => typeof x === "string").map(plain).filter(Boolean) : [];
  const caption = typeof raw.caption === "string" ? plain(raw.caption).slice(0, CAPTION_MAX) : "";
  const stamp = typeof raw.stamp === "string" && checkPicture(raw.stamp) === null ? raw.stamp : undefined;
  const message = raw.message !== undefined && checkMessage(raw.message) === null ? makeMessage((raw.message as PostcardMessage).text, (raw.message as PostcardMessage).from) : undefined;
  return {
    ok: true,
    postcard: {
      schema: POSTCARD_SCHEMA,
      id: raw.id,
      name: plain(raw.name).slice(0, NAME_MAX),
      ...(caption ? { caption } : {}),
      bookmark: b as Bookmark & { at_yr: number },
      world,
      credit,
      ...(stamp ? { stamp } : {}),
      ...(message ? { message } : {}),
    },
  };
}

/** Inside a campaign, a bookmark with a camera and a fixed date is a postcard (one store). */
export function isPostcard(b: Bookmark): boolean {
  return b.at_yr !== null && !!b.pose && (STILL_INSTRUMENTS as readonly string[]).includes(b.instrument);
}

/** Why a bookmark cannot be a postcard yet, one line; null when it can. */
export function postcardBlocker(b: Bookmark): string | null {
  if (!(STILL_INSTRUMENTS as readonly string[]).includes(b.instrument)) return "This screen cannot draw a picture, so it cannot be a postcard.";
  if (!b.pose) return "No camera was kept with this bookmark: open it, then Update from live.";
  return null;
}

/** "Make a postcard" on a bookmark: the same bookmark with its date fixed (a bookmark that follows the campaign's now
 * is fixed at `now_yr`). Kept in the campaign's bookmarks; the world is `here`. */
export function fixDate(b: Bookmark, now_yr: number): Bookmark & { at_yr: number } {
  return { ...b, at_yr: b.at_yr ?? now_yr };
}

let seq = 0;
/** A fresh postcard id. Ids name a postcard on a wall or shelf; they carry nothing. */
export function postcardId(rand: () => number = Math.random): string {
  seq = (seq + 1) % 1296;
  const r = Math.floor(rand() * 2_176_782_336).toString(36).padStart(6, "0");
  return `pc-${r}${seq.toString(36).padStart(2, "0")}`;
}

/** Wrap a bookmark in the envelope it travels in. The bookmark's date must be fixed. */
export function envelope(b: Bookmark & { at_yr: number }, world: PostcardWorld, opts: { id?: string; name?: string; caption?: string; credit?: readonly string[]; stamp?: string | null } = {}): Postcard {
  const caption = opts.caption ? plain(opts.caption).slice(0, CAPTION_MAX) : "";
  const stamp = opts.stamp && checkPicture(opts.stamp) === null ? opts.stamp : undefined;
  return {
    schema: POSTCARD_SCHEMA,
    id: opts.id ?? postcardId(),
    name: plain(opts.name ?? b.name).slice(0, NAME_MAX) || b.name,
    ...(caption ? { caption } : {}),
    bookmark: { ...b, schema: BOOKMARK_SCHEMA },
    world,
    credit: (opts.credit ?? []).map(plain).filter(Boolean),
    ...(stamp ? { stamp } : {}),
  };
}

/** "Make a postcard" from where I am: a bookmark here, its date fixed at now, in its envelope. */
export function postcardHere(list: readonly Bookmark[], at: Whereabouts, what: string, instrumentWords: string, world: PostcardWorld, opts: { credit?: readonly string[]; caption?: string } = {}): Postcard {
  const b = makeBookmark(list, at, what, instrumentWords);
  return envelope(fixDate(b, at.now_yr), world, opts);
}

/** What a seat can see, for packing a clip (design section 4 rule 1). A GM sending "as I see it" sees everything. */
export interface SeatSees {
  /** May this body go at all? */
  body(node: ClipNode): boolean;
  /** May this fact on it go? (A field held back from the table stays home.) */
  fact(node: ClipNode, field: string): boolean;
}

/** A clip cut to what a seat can see: bodies it cannot see are left out with everything under them, and facts it
 * cannot see are left out of the bodies that go. Notes, secrets and Known-to records are never in a clip. */
export function clipThroughSeat(c: Clip, sees: SeatSees): Clip | null {
  const kept = new Set<string>();
  const nodes: ClipNode[] = [];
  for (const n of c.nodes) {
    if (n.parent !== null && !kept.has(n.parent)) continue;
    if (!sees.body(n)) continue;
    kept.add(n.id);
    nodes.push({ ...n });
  }
  if (!kept.has(c.root)) return null;
  const byId = new Map(nodes.map((n) => [n.id, n]));
  const events: ClipEvent[] = c.events.filter((e) => kept.has(e.node) && sees.fact(byId.get(e.node)!, e.field)).map((e) => ({ ...e }));
  return { ...c, source: { ...c.source, creditChain: [...c.source.creditChain] }, packs: c.packs.map((p) => ({ ...p })), nodes, events };
}

/** "What goes with it" (design section 4 rule 1), in plain words: the bodies by name and the count of facts. */
export function goesWith(p: Postcard): { bodies: string[]; facts: number; words: string } {
  const w = p.world;
  if (w.how === "here") return { bodies: [], facts: 0, words: "Nothing goes with it: it is kept in this campaign." };
  if (w.how === "address") {
    const packs = w.packs.length ? `, made under ${listWords(w.packs.map((x) => x.id))}` : "";
    return { bodies: [], facts: 0, words: `Only its address goes${packs}. The universe makes the rest again where it is opened.` };
  }
  const c = w.clip;
  const bodies = c.nodes.map((n) => {
    const e = c.events.find((x) => x.node === n.id && x.field === "name");
    return typeof e?.value === "string" ? e.value : `an unnamed ${n.kind}`;
  });
  const facts = c.events.length;
  return { bodies, facts, words: `${listWords(bodies)}: ${facts === 1 ? "one fact" : `${facts} facts`}, nothing else.` };
}

function listWords(xs: readonly string[]): string {
  return xs.length > 1 ? `${xs.slice(0, -1).join(", ")} and ${xs.at(-1)}` : (xs[0] ?? "");
}

/** The state of one pack a postcard needs on this device (12A point 3). */
export type PackState = "have" | "other-version" | "free" | "paid" | "unknown";
export interface PackNeeded {
  id: string;
  version: string;
  state: PackState;
  /** The version this device has, when it has another one. */
  have?: string;
}

/** The packs a postcard needs, each with its state here. `shop` says whether the shop has one free or paid. */
export function packsNeeded(p: Postcard, stack: readonly PackNamed[], shop: (id: string) => "free" | "paid" | null = () => null): PackNeeded[] {
  if (p.world.how === "here") return [];
  return p.world.packs.map((n) => {
    const h = stack.find((s) => s.id === n.id);
    if (h && h.version === n.version) return { ...n, state: "have" as const };
    if (h) return { ...n, state: "other-version" as const, have: h.version };
    return { ...n, state: shop(n.id) ?? "unknown" };
  });
}

/** Why the shop would refuse this postcard, one line; null when it can go up (12A point 4): a published postcard is
 * an address plus named packs, and everything it shows comes from the base game or something on the shop. */
export function shopRefusal(p: Postcard, published: (id: string) => boolean): string | null {
  if (p.world.how === "here") return "A postcard kept in a campaign has to be packed for sending first.";
  if (p.world.how === "clip") return "The shop takes postcards of published places only: publish this world first, then make the postcard there.";
  const missing = p.world.packs.filter((x) => !published(x.id)).map((x) => x.id);
  if (missing.length) return `The shop does not hold ${listWords(missing)}, so nobody could draw this postcard.`;
  return null;
}

/** The envelope as it goes to the shop: no message (that belongs to a sending), credit kept. */
export function forShop(p: Postcard): Postcard {
  const { message: _m, ...rest } = p;
  return rest;
}

/** With a message for this sending, or without (null). */
export function withMessage(p: Postcard, m: PostcardMessage | null): Postcard {
  const { message: _m, ...rest } = p;
  return m ? { ...rest, message: makeMessage(m.text, m.from) } : rest;
}

// The ways a postcard travels (design section 7): clipboard text, a link, a file.

/** Clipboard text: the envelope as JSON. */
export function postcardText(p: Postcard): string {
  return JSON.stringify(p);
}

/** The file it goes as: a plain name and pretty JSON. */
export function postcardFile(p: Postcard): { name: string; text: string } {
  const slug = p.name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 60) || "postcard";
  return { name: `${slug}${POSTCARD_FILE_SUFFIX}`, text: `${JSON.stringify(p, null, 2)}\n` };
}

function toBase64Url(s: string): string {
  const bytes = new TextEncoder().encode(s);
  let bin = "";
  for (const b of bytes) bin += String.fromCharCode(b);
  return btoa(bin).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}
function fromBase64Url(s: string): string | null {
  if (!/^[A-Za-z0-9_-]*$/.test(s)) return null;
  try {
    const bin = atob(s.replace(/-/g, "+").replace(/_/g, "/"));
    const bytes = Uint8Array.from(bin, (c) => c.charCodeAt(0));
    return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    return null;
  }
}

export type PostcardLink = { ok: true; url: string } | { ok: false; why: string };

/** A link that opens the postcard with nothing installed. Only one that travels as an address goes in a link; the
 * stamp never does (a web address cannot hold a picture). A message goes with it. */
export function postcardLink(p: Postcard, site = POSTCARD_SITE): PostcardLink {
  if (p.world.how === "clip") return { ok: false, why: "This postcard carries its own world, too much for a link: send it as a file." };
  if (p.world.how === "here") return { ok: false, why: "This postcard is kept in this campaign: pack it for sending first." };
  const { stamp: _s, ...rest } = p;
  return { ok: true, url: `${site.replace(/#.*$/, "")}#${POSTCARD_LINK_KEY}=${toBase64Url(JSON.stringify(rest))}` };
}

/** Read whatever was pasted or opened: a postcard's JSON, a postcard link, or just its `#postcard=` part. */
export function readPostcard(text: string): PostcardRead {
  const t = text.trim();
  if (t.length > POSTCARD_MAX * 2) return no("it is far too big");
  const m = new RegExp(`(?:^|[#&])${POSTCARD_LINK_KEY}=([A-Za-z0-9_-]+)`).exec(t);
  const json: string | null = m ? fromBase64Url(m[1]!) : t.startsWith("{") ? t : null;
  if (json === null) return no(m ? "the link is damaged" : "it is not a postcard or a postcard link");
  if (json.length > POSTCARD_MAX) return no("it is far too big");
  let raw: unknown;
  try {
    raw = JSON.parse(json);
  } catch {
    return no("it is not readable data");
  }
  return checkPostcard(raw);
}

/** Does this text look like a postcard (for a paste or a file that has to be told apart from a clip or a save)? */
export function looksLikePostcard(text: string): boolean {
  const t = text.trim();
  return new RegExp(`(?:^|[#&])${POSTCARD_LINK_KEY}=`).test(t) || (t.startsWith("{") && t.includes(`"${POSTCARD_SCHEMA}"`));
}
