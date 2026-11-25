// The Playbook (PB3, design SSE4-playbook-and-switchboard-design.md section 4): the GM's hand of presets, in chapters,
// pages and sections, kept with the campaign. No Next and no script (Alex, 24 Nov): any preset fires at any time, and
// pressing it again while its picture is up takes it back. A preset names a role (or Everyone), never a screen, and
// holds no knowledge: a Show step names a bookmark, and each screen draws it as its own role knows it.
import { back, hold, holdCardFrom, lift, release, resolve, send, type Board, type Dest, type HoldCard } from "./board.ts";
import { cardLabel } from "./holding.ts";
import type { Bookmark } from "./bookmark.ts";
import type { Shown } from "./seat.ts";
import type { Json } from "./json.ts";

export const PLAYBOOK_SCHEMA = "sse.playbook/1";

/** One thing a preset does. Steps in one preset go together, at once (no conditions, no timers in 4.0). */
export type Step =
  /** `layout` (PB13): the layout the role's screens use for this send (a shipped file or a made `mine.` id); going back returns them to their own. */
  | { show: { bookmark: string; layout?: string } }
  | { hold: { card: HoldCard } }
  /** Words for the GM to read; nothing goes to any screen (a card on screens comes with the Looks work). */
  /** Words for the GM to read; `on_screens` also puts them on the role's screens as a card over what they show (4.3, PB6). */
  | { say: { text: string; on_screens?: true } }
  /**
   * Tell the role's audience about a thing (secrets design; design 4.2): the one step that changes what anyone knows.
   * It is not a send: every screen of the role redraws where it stands. It is never on the board's undo (the eye takes
   * it back). `name` is for the row's words only.
   */
  | { reveal: { address: string; landmark: "known" | "heard_of"; name?: string } };

export interface Preset {
  id: string;
  title: string;
  /** "everyone" or a role (seat) id. */
  to: string;
  steps: Step[];
  note?: string;
}
export interface Section {
  id: string;
  title: string;
  presets: Preset[];
}
export interface Page {
  id: string;
  title: string;
  sections: Section[];
}
export interface Chapter {
  id: string;
  title: string;
  pages: Page[];
}
export interface Playbook {
  schema: typeof PLAYBOOK_SCHEMA;
  chapters: Chapter[];
}

const clone = <T>(x: T): T => JSON.parse(JSON.stringify(x)) as T;

/** A new playbook: one chapter, one page, one section, ready for the first preset. */
export function emptyPlaybook(): Playbook {
  return { schema: PLAYBOOK_SCHEMA, chapters: [{ id: "c1", title: "Chapter 1", pages: [{ id: "p1", title: "First page", sections: [{ id: "s1", title: "Presets", presets: [] }] }] }] };
}

/** Every id in the playbook, so a new one is never reused. */
function ids(pb: Playbook): string[] {
  const out: string[] = [];
  for (const c of pb.chapters) {
    out.push(c.id);
    for (const p of c.pages) {
      out.push(p.id);
      for (const s of p.sections) (out.push(s.id), out.push(...s.presets.map((x) => x.id)));
    }
  }
  return out;
}
/** The next free id with this letter: c, p, s or k (presets). */
export function nextId(pb: Playbook, letter: "c" | "p" | "s" | "k"): string {
  let n = 0;
  const re = new RegExp(`^${letter}(\\d+)$`);
  for (const id of ids(pb)) {
    const m = re.exec(id);
    if (m) n = Math.max(n, Number(m[1]));
  }
  return `${letter}${n + 1}`;
}

export function pages(pb: Playbook): { chapter: Chapter; page: Page }[] {
  return pb.chapters.flatMap((chapter) => chapter.pages.map((page) => ({ chapter, page })));
}
export function findPage(pb: Playbook, pageId: string | null | undefined): { chapter: Chapter; page: Page } | null {
  return pages(pb).find((x) => x.page.id === pageId) ?? pages(pb)[0] ?? null;
}
export function findPreset(pb: Playbook, id: string): { preset: Preset; section: Section; page: Page } | null {
  for (const c of pb.chapters) for (const p of c.pages) for (const s of p.sections) for (const k of s.presets) if (k.id === id) return { preset: k, section: s, page: p };
  return null;
}
/** The presets of a page in the order shown, skipping folded sections (keys 1 to 9 count only these). */
export function presetsInView(page: Page, folded: ReadonlySet<string> = new Set()): Preset[] {
  return page.sections.filter((s) => !folded.has(s.id)).flatMap((s) => s.presets);
}

export function addChapter(pb: Playbook, title: string): { pb: Playbook; page: string } {
  const n = clone(pb);
  const c = nextId(n, "c");
  const p = nextId(n, "p");
  const s = nextId(n, "s");
  n.chapters.push({ id: c, title, pages: [{ id: p, title: "First page", sections: [{ id: s, title: "Presets", presets: [] }] }] });
  return { pb: n, page: p };
}
export function addPage(pb: Playbook, chapterId: string, title: string): { pb: Playbook; page: string } {
  const n = clone(pb);
  const p = nextId(n, "p");
  const s = nextId(n, "s");
  const ch = n.chapters.find((c) => c.id === chapterId) ?? n.chapters[n.chapters.length - 1];
  if (!ch) return addChapter(pb, "Chapter 1");
  ch.pages.push({ id: p, title, sections: [{ id: s, title: "Presets", presets: [] }] });
  return { pb: n, page: p };
}
export function addSection(pb: Playbook, pageId: string, title: string): { pb: Playbook; section: string } {
  const n = clone(pb);
  const s = nextId(n, "s");
  findPage(n, pageId)?.page.sections.push({ id: s, title, presets: [] });
  return { pb: n, section: s };
}
/** Rename a chapter, page, section or preset by id; an empty name changes nothing. */
export function rename(pb: Playbook, id: string, title: string): Playbook {
  const t = title.trim();
  if (!t) return pb;
  const n = clone(pb);
  for (const c of n.chapters) {
    if (c.id === id) c.title = t;
    for (const p of c.pages) {
      if (p.id === id) p.title = t;
      for (const s of p.sections) {
        if (s.id === id) s.title = t;
        for (const k of s.presets) if (k.id === id) k.title = t;
      }
    }
  }
  return n;
}
/** Remove a section (with its presets), a page or a chapter; the last page of the last chapter stays. */
export function removePart(pb: Playbook, id: string): Playbook {
  const n = clone(pb);
  for (const c of n.chapters) {
    for (const p of c.pages) p.sections = p.sections.filter((s) => s.id !== id);
    c.pages = c.pages.filter((p) => p.id !== id);
  }
  n.chapters = n.chapters.filter((c) => c.id !== id && c.pages.length);
  return n.chapters.length ? n : emptyPlaybook();
}

/** Put a preset at the end of a section (the page's last section when the section is gone). Returns its id. */
export function addPreset(pb: Playbook, sectionId: string, p: Omit<Preset, "id">): { pb: Playbook; id: string } {
  const n = clone(pb);
  const id = nextId(n, "k");
  const all = n.chapters.flatMap((c) => c.pages.flatMap((pg) => pg.sections));
  const s = all.find((x) => x.id === sectionId) ?? all[all.length - 1];
  s?.presets.push({ ...clone(p), id });
  return { pb: n, id };
}
export function updatePreset(pb: Playbook, id: string, change: Partial<Omit<Preset, "id">>): Playbook {
  const n = clone(pb);
  const f = findPreset(n, id);
  if (f) Object.assign(f.preset, clone(change));
  return n;
}
export function removePreset(pb: Playbook, id: string): Playbook {
  const n = clone(pb);
  for (const c of n.chapters) for (const p of c.pages) for (const s of p.sections) s.presets = s.presets.filter((k) => k.id !== id);
  return n;
}
/** Drag reorder: the preset goes to `index` in `sectionId` (any page). */
export function movePreset(pb: Playbook, id: string, sectionId: string, index: number): Playbook {
  const f = findPreset(pb, id);
  if (!f) return pb;
  const n = removePreset(pb, id);
  const s = n.chapters.flatMap((c) => c.pages.flatMap((pg) => pg.sections)).find((x) => x.id === sectionId);
  if (!s) return pb;
  s.presets.splice(Math.max(0, Math.min(index, s.presets.length)), 0, clone(f.preset));
  return n;
}
/** A preset dropped on a page's name goes to the end of that page's last section. */
export function movePresetToPage(pb: Playbook, id: string, pageId: string): Playbook {
  const pg = findPage(pb, pageId)?.page;
  const s = pg?.sections[pg.sections.length - 1];
  return s ? movePreset(pb, id, s.id, s.presets.length) : pb;
}

/** Where a preset lands: Everyone, or its role's line. */
export function destOf(p: Preset): Dest {
  return p.to === "everyone" ? "everyone" : `role:${p.to}`;
}
/** What a Show step sends: the bookmark's place under the preset's name. A missing bookmark sends nothing. */
export function showOf(p: Preset, step: { show: { bookmark: string; layout?: string } }, bookmarks: readonly Bookmark[]): Shown | null {
  const b = bookmarks.find((x) => x.id === step.show.bookmark);
  // The bookmark's camera goes with it (PC5), so a screen that can frame it shows the view as it was kept.
  return b ? { scope: b.scope, title: p.title, ...(b.selected ? { focus: b.selected } : {}), ...(b.pose ? { pose: JSON.parse(JSON.stringify(b.pose)) as Json } : {}), ...(step.show.layout ? { screen: step.show.layout } : {}) } : null;
}

/** Only words for the GM: nothing goes to any screen, so its button marks it read (4.3). */
export function wordsOnly(p: Preset): boolean {
  return p.steps.every((s) => "say" in s && !s.say.on_screens);
}
/** The words a preset puts on screens (its Read aloud steps with "also show on screens"), or null. */
export function sayOf(p: Preset): string | null {
  const t = p.steps.flatMap((s) => ("say" in s && s.say.on_screens && s.say.text.trim() ? [s.say.text.trim()] : []));
  return t.length ? t.join("\n\n") : null;
}
const sends = (p: Preset) => p.steps.some((s) => "show" in s) || sayOf(p) !== null;

/**
 * Is the preset's picture up (5.4: its button is a switch while it is)? A Show is up while its destination's line, or
 * Everyone's top layer, came from it; a Hold while the destination is held. A preset with only words is never "up".
 */
export function isUp(b: Board, p: Preset, at: Dest = destOf(p)): boolean {
  const dest = at;
  const line = b.lines[dest];
  if (sends(p)) {
    if (dest === "everyone" && b.over.length) return (b.over_from ?? [])[b.over.length - 1] === p.id;
    return line?.from === p.id;
  }
  if (p.steps.some((s) => "hold" in s)) return !!line?.held;
  return false;
}

/**
 * Fire a preset: every step at once, at its own role or (dragged there) at `at`. Pressed while up, it takes itself back
 * instead (lift, back, release).
 */
export function fire(b: Board, p: Preset, bookmarks: readonly Bookmark[], roles: readonly string[] = [], at: Dest = destOf(p)): Board {
  const dest = at;
  if (isUp(b, p, dest)) {
    if (sends(p)) return dest === "everyone" && b.over.length ? lift(b) : back(b, dest);
    return release(b, dest);
  }
  let n = b;
  const say = sayOf(p);
  let shown = false;
  for (const s of p.steps) {
    if ("show" in s) {
      const sh = showOf(p, s, bookmarks);
      if (sh && "scope" in sh) (n = send(n, dest, say ? { ...sh, say } : sh, p.id)), (shown = true);
    } else if ("hold" in s) n = hold(n, dest, s.hold.card, roles);
  }
  // Words on screens with no picture of their own go over what the destination shows now (4.3).
  if (say && !shown) {
    const here = resolve(n, dest === "everyone" ? "" : dest.startsWith("role:") ? dest.slice(5) : (n.screens[dest.slice(7)]?.role ?? ""), dest.startsWith("screen:") ? dest.slice(7) : undefined).shown;
    if ("scope" in here) n = send(n, dest, { ...here, say }, p.id);
  }
  return n;
}

/** What a preset does, in words, for its row and its title ("Show Tarn, the approach", "Hold: a joke", "Read aloud"). */
export function presetWords(p: Preset, bookmarks: readonly Bookmark[]): string {
  return p.steps
    .map((s) => ("show" in s ? `Show ${bookmarks.find((b) => b.id === s.show.bookmark)?.name ?? "a bookmark that has gone"}` : "reveal" in s ? `${s.reveal.landmark === "known" ? "Reveal" : "Hint at"} ${s.reveal.name ?? "a thing"}` : "hold" in s ? `Hold: ${cardLabel(s.hold.card)}` : s.say.on_screens ? "Read aloud, on screens" : "Read aloud"))
    .join(", ");
}

/** A playbook read from a save or a pack: unknown steps are dropped, never guessed at; nothing usable gives a new one. */
export function playbookFrom(raw: unknown): Playbook {
  const r = raw as { schema?: unknown; chapters?: unknown } | null;
  if (!r || r.schema !== PLAYBOOK_SCHEMA || !Array.isArray(r.chapters)) return emptyPlaybook();
  const str = (x: unknown, d: string) => (typeof x === "string" && x.trim() ? x : d);
  const arr = (x: unknown): Record<string, unknown>[] => (Array.isArray(x) ? x.filter((y) => y && typeof y === "object") : []);
  const step = (s: Record<string, unknown>): Step | null => {
    const v = (k: string) => s[k] as Record<string, unknown> | undefined;
    if (v("show") && typeof v("show")!.bookmark === "string") return { show: { bookmark: v("show")!.bookmark as string, ...(typeof v("show")!.layout === "string" && v("show")!.layout ? { layout: v("show")!.layout as string } : {}) } };
    if (v("hold")) {
      const c = holdCardFrom(v("hold")!.card);
      if (c) return { hold: { card: c } };
    }
    if (v("say") && typeof v("say")!.text === "string") return { say: { text: v("say")!.text as string, ...(v("say")!.on_screens === true ? { on_screens: true as const } : {}) } };
    const rv = v("reveal");
    if (rv && typeof rv.address === "string" && (rv.landmark === "known" || rv.landmark === "heard_of")) return { reveal: { address: rv.address, landmark: rv.landmark, ...(typeof rv.name === "string" ? { name: rv.name } : {}) } };
    return null;
  };
  const pb: Playbook = {
    schema: PLAYBOOK_SCHEMA,
    chapters: arr(r.chapters).map((c, ci) => ({
      id: str(c.id, `c${ci + 1}`),
      title: str(c.title, "Chapter"),
      pages: arr(c.pages).map((p, pi) => ({
        id: str(p.id, `p${ci + 1}x${pi + 1}`),
        title: str(p.title, "Page"),
        sections: arr(p.sections).map((s, si) => ({
          id: str(s.id, `s${ci + 1}x${pi + 1}x${si + 1}`),
          title: str(s.title, "Presets"),
          presets: arr(s.presets)
            .map((k) => ({ id: str(k.id, ""), title: str(k.title, "Preset"), to: str(k.to, "everyone"), steps: arr(k.steps).map(step).filter((x): x is Step => !!x), ...(typeof k.note === "string" ? { note: k.note } : {}) }))
            .filter((k) => k.id && k.steps.length),
        })),
      })),
    })),
  };
  pb.chapters = pb.chapters.filter((c) => c.pages.length);
  return pb.chapters.length ? pb : emptyPlaybook();
}

/** The reveals a preset makes when fired (they go to the kernel as `set_known`, not on the board). */
export function revealsOf(p: Preset): { address: string; landmark: "known" | "heard_of" }[] {
  return p.steps.flatMap((s) => ("reveal" in s ? [{ address: s.reveal.address, landmark: s.reveal.landmark }] : []));
}
/** Does the preset tell anyone anything? Its row wears the eye (design 4.2). */
export function reveals(p: Preset): boolean {
  return p.steps.some((s) => "reveal" in s);
}
