// Orac makes Atlas editions, margin note sets and interface kits (AT11 and
// AT16, atlas design 11A.7 and 12.8). Each target's check is the loader's own
// (ui-contract/src/atlas: checkEdition with the test render of check 6,
// checkNoteSet, checkKit with palette contrast), so a draft from Orac, a pack
// or a hand passes the same gate and nothing trusts the model. A new document
// gets a `mine.` id, so a shipped starter, edition or note set is never
// overwritten; "Hauler but amber" is a kit that extends Hauler. None of these
// can state a fact or widen what a role knows: the loader refuses a figure
// typed into their words, and a reveal is a different change (orac-known.ts).
// Accept keeps the document in the campaign's own pack, `campaign.atlas`,
// under `provides.other.atlas`, stacked like any pack the GM adds.
import {
  ARRANGEMENTS,
  DELIVERIES,
  MOMENTS,
  NOTE_TOPICS,
  SHIPPED_EDITIONS,
  SHIPPED_NOTE_SETS,
  SHIPPED_PALETTES,
  SITUATIONS,
  STARTER_KITS,
  TIERS,
  VIEW_STYLES,
  atlasDocsOf,
  checkEdition,
  checkKit,
  checkNoteSet,
  stackPacks,
  testRender,
  type EditionDoc,
  type KitDoc,
  type NoteSet,
  type Palette,
  type PackDoc,
  type PartInfo,
  type PlayerScene,
  type ViewSession,
  type Vocab,
  type Workspace,
} from "../../ui-contract/src/index.ts";
import type { Entry, OracTarget } from "./orac.ts";

export const ATLAS_PACK = "campaign.atlas";
export type AtlasKind = "editions" | "note_sets" | "kits";
/** The three lists of `atlasDocsOf` (the atlas thread's loader), without its refusals. */
export type AtlasDocs = Omit<ReturnType<typeof atlasDocsOf>, "refused">;

/** What the targets read: the documents the stack holds beside the shipped ones, and what may be named. */
export interface AtlasHost {
  /** Editions, note sets and kits from the pack stack (not the shipped ones). */
  docs(): AtlasDocs;
  looks(): readonly string[];
  vocabs(): Readonly<Record<string, Vocab>>;
  parts(): readonly PartInfo[];
  /** Scenes for the test render (check 6): the GM's and a seat's with partial knowledge. Empty skips it, and the summary says so. */
  scenes(): Promise<readonly PlayerScene[]>;
  /** Keep an accepted document. Returns a problem, or null when kept. */
  keep(kind: AtlasKind, doc: EditionDoc | NoteSet | KitDoc): Promise<string | null>;
}

const obj = (x: unknown): x is Record<string, unknown> => !!x && typeof x === "object" && !Array.isArray(x);
const bump = (v: string) => {
  const [a = 0, b = 0, c = 0] = v.split(".").map(Number);
  return `${a}.${b}.${c + 1}`;
};

/** A new document's id: `mine.` first, never a shipped one; an existing `mine.` one is an edit. */
function idProblems(id: unknown, shipped: readonly { id: string }[], what: string): string[] {
  const an = /^[aeiou]/.test(what) ? "an" : "a";
  if (typeof id !== "string" || !id) return [`${an} ${what} needs an id`];
  if (shipped.some((s) => s.id === id)) return [`${id} is a shipped ${what}; make your own with an id starting "mine." that extends it`];
  const tail = id.split(".").pop()!.toLowerCase().replace(/[^a-z0-9-]+/g, "-").replace(/^-+|-+$/g, "") || what.replace(/\s+/g, "-");
  if (!/^mine\.[a-z0-9][a-z0-9.-]*$/.test(id)) return [`${an} ${what} Orac makes has an id starting "mine." in lower case, so nothing shipped is overwritten (for example "mine.${tail}")`];
  return [];
}

const sentence = (s: string) => s.replace(/\.$/, "");

export function editionTarget(host: AtlasHost): OracTarget<{ doc: EditionDoc; rendered: number }> {
  return {
    kind: "edition",
    what: "An Atlas edition: how the Atlas reads and looks for a role (words, order of sections, look, gap style, margin notes, letterhead, cover). It never decides a fact: figures come from the page. Start from a shipped one with extends and change only what differs. Read one first (need edition:<id>) and the guide (need guide:interfaces).",
    form: [
      '{"schema": "sse.edition/1", "id": "mine.<lower-case-id>", "name": "<name>", "description": "<one line people choose by>", "extends": "<edition id>", "look": "<look>", "vocab": "plain" | "guide", "gaps": "plain" | "voice" | "bar" | "omit", "voice": {"labels": {"<section or line.<field>>": "<words>"}, "gap": "<words for a gap>"}, "letterhead": {"use": "brand" | "masthead" | "none", "lines": ["<words with {slots}>"], "stamp": "<words>"}, "margin": {"sets": ["<note set id>"], "max": <n>}, "templates": {"world": [{"key": "<section>", "lines": ["<field>"]}]}}',
      "Never type a figure (a number with a unit) into an edition's words. Slots go only in letterhead, stamp and print lines: {brand.name}, {brand.motto}, {audience}, {date}, {place}, {chapter}, {doc.number}, {page}, {pages}.",
    ].join("\n"),
    async check(doc) {
      if (!obj(doc)) return { problems: ["an edition is one JSON object"] };
      const d = doc as unknown as EditionDoc;
      const ids = idProblems(d.id, SHIPPED_EDITIONS, "edition");
      if (ids.length) return { problems: ids };
      const docs = host.docs();
      const stack = { editions: docs.editions, looks: host.looks(), vocabs: Object.keys(host.vocabs()), noteSets: [...SHIPPED_NOTE_SETS, ...docs.note_sets].map((s) => s.id) };
      const problems = checkEdition(d, stack);
      if (problems.length) return { problems };
      const scenes = await host.scenes();
      const rendered = testRender(d, scenes, host.vocabs(), stack);
      return rendered.length ? { problems: rendered } : { problems: [], value: { doc: d, rendered: scenes.length } };
    },
    summary: ({ doc: d, rendered }) =>
      `${d.name}: ${sentence(d.description)}.${d.extends ? ` Starts from ${d.extends}` : ""}${d.look ? `, wears ${d.look}` : ""}${d.gaps ? `, gaps shown ${d.gaps}` : ""}${d.margin?.sets?.length ? `, margin notes from ${d.margin.sets.join(", ")}` : ""}. ${rendered ? `Drawn for ${rendered} ${rendered === 1 ? "audience" : "audiences"} beside the Survey atlas: every figure the same, nothing shown that they were not given.` : "Not test-drawn: no scene was open."} Accept keeps it in the campaign's own pack; pick it as a role's edition.`,
    accept: ({ doc }) => host.keep("editions", doc),
  };
}

export function noteSetTarget(host: AtlasHost): OracTarget<NoteSet> {
  return {
    kind: "note_set",
    what: "A set of margin notes for an edition (the Guide's jokes, a company's warnings, a GM's asides): opinion, not fact. Each note says what kind of page it is about and when it fits, read only from what the audience knows; a number in a note is a {field} slot the page fills, never typed. Read the shipped set first (need notes:sse.notes.guide).",
    form: [
      '{"schema": "sse.margin-notes/1", "id": "mine.<lower-case-id>", "name": "<name>", "notes": [{"id": "<lower-case-id>", "about": "world" | "moon" | "star" | "system" | "sector" | "station" | "ship" | "other" | "any", "when": {"fact": "<field>", "at_least": <SI number>} | {"fact": "<field>", "present": true} | {"fact": "<field>", "word": "<a whole word, any case>"} | {"tag": "<tag>"} | {"all": [...]} | {"any": [...]}, "text": "<words, with {field} for a figure>", "topic": "<topic>", "always": true, "by": "<who said it>", "source": "<where>", "weight": <n>}]}',
      "Numbers in when are SI (kelvin, pascals, metres). Notes are at most 200 characters. Add the set to an edition's margin.sets to show it.",
      `Topics (a page takes one note per topic): ${NOTE_TOPICS.join(", ")}. "always" needs a "when" and puts the note first. "by" makes the line a quotation (up to 320 characters, no {slots}); "source" only with "by". topic, always, by, source and weight are optional.`,
    ].join("\n"),
    async check(doc) {
      if (!obj(doc)) return { problems: ["a note set is one JSON object"] };
      const d = doc as unknown as NoteSet;
      const ids = idProblems(d.id, SHIPPED_NOTE_SETS, "note set");
      if (ids.length) return { problems: ids };
      const problems = checkNoteSet(d);
      return problems.length ? { problems } : { problems: [], value: d };
    },
    summary: (d) => {
      const about = [...new Set(d.notes.map((n) => n.about))].join(", ");
      return `${d.name}: ${d.notes.length} ${d.notes.length === 1 ? "note" : "notes"} about ${about}, each shown only when what the audience knows fits it. Accept keeps it in the campaign's own pack; name it in an edition's margin to show it.`;
    },
    accept: (d) => host.keep("note_sets", d),
  };
}

export function kitTarget(host: AtlasHost): OracTarget<KitDoc> {
  return {
    kind: "kit",
    what: "An interface kit: everything a role's screens need, chosen once (parts and where they sit, which controls are on, a look and palette, a view style, an Atlas edition, how screens arrive and leave, the device tier). Make a new one by extending a starter (Hauler, Corporate, Deck, The Guide) and stating only what differs. A kit cannot state a fact or widen what a role knows. Read the starter first (need kit:<id>), the parts and their controls (need parts) and the guide (need guide:interfaces).",
    form: [
      '{"schema": "sse.kit/1", "id": "mine.<lower-case-id>", "name": "<name>", "description": "<one line>", "extends": "<kit id>", "look": "<look>", "palettes": ["<palette id>"] | [{"id": "mine.<id>", "name": "<name>", "tokens": {"ground": "#rrggbb", "ink": "#rrggbb", "ink-muted": "#rrggbb", "accent": "#rrggbb", "accent-ink": "#rrggbb"}}], "view_style": "<view style>", "atlas": {"edition": "<edition id>"}, "delivery": {"<moment>": {"use": "<delivery>", "seconds": <s>}}, "needs": {"tier": "<tier>", "falls_back_to": "<kit id>"}, "banners": {"at": "top" | "bottom" | "both", "sets": ["<note set id>"]}, "layouts": {"<situation>": {"arrangement": "<arrangement>", "items": [{"view": "<part>", "area": "<area>", "controls": "none" | "all" | {"<control>": true | false}}]}}}',
      `Situations: ${SITUATIONS.join(", ")}. Moments: ${MOMENTS.join(", ")}. Deliveries: ${DELIVERIES.join(", ")}. Tiers: ${TIERS.join(", ")}. View styles: ${VIEW_STYLES.join(", ")}. Arrangements and their areas: ${Object.entries(ARRANGEMENTS).map(([k, v]) => `${k} (${v.areas.join(", ") || "no areas"})`).join("; ")}.`,
      "A palette needs 4.5 to 1 contrast for ink, muted ink and accent ink on its ground. Switch only the controls you want different from the part's default.",
    ].join("\n"),
    async check(doc) {
      if (!obj(doc)) return { problems: ["a kit is one JSON object"] };
      const d = doc as unknown as KitDoc;
      const ids = idProblems(d.id, STARTER_KITS, "kit");
      if (ids.length) return { problems: ids };
      const docs = host.docs();
      const problems = checkKit(d, { kits: docs.kits, parts: host.parts(), looks: host.looks(), editions: [...SHIPPED_EDITIONS, ...docs.editions].map((e) => e.id), vocabs: Object.keys(host.vocabs()), noteSets: [...SHIPPED_NOTE_SETS, ...docs.note_sets].map((s) => s.id), palettes: docs.kits.flatMap((k) => (k.palettes ?? []).filter((p): p is Palette => typeof p === "object")) });
      return problems.length ? { problems } : { problems: [], value: d };
    },
    summary: (d) => {
      const pal = (d.palettes ?? []).map((p) => (typeof p === "string" ? (SHIPPED_PALETTES.find((x) => x.id === p)?.name ?? p) : p.name));
      const bits = [d.extends ? `starts from ${STARTER_KITS.find((k) => k.id === d.extends)?.name ?? d.extends}` : "", d.look ? `wears ${d.look}` : "", pal.length ? `in ${pal.join(", ")}` : "", d.view_style ? `diagrams start ${d.view_style}` : "", d.atlas?.edition ? `Atlas edition ${d.atlas.edition}` : "", Object.keys(d.layouts ?? {}).length ? `its own ${Object.keys(d.layouts!).join(", ")} ${Object.keys(d.layouts!).length === 1 ? "layout" : "layouts"}` : "", Object.keys(d.delivery ?? {}).length ? `its own ${Object.keys(d.delivery!).join(", ")}` : ""].filter(Boolean);
      return `${d.name}: ${sentence(d.description)}. ${bits.length ? `${bits.join("; ")[0]!.toUpperCase()}${bits.join("; ").slice(1)}. ` : ""}Passed every check the kit loader has (parts, controls, contrast, deliveries, tier). Accept keeps it in the campaign's own pack; choose it on a role card.`;
    },
    accept: (d) => host.keep("kits", d),
  };
}

/** Keeping through the stack: the document joins `campaign.atlas` (an edit replaces its own id) and the stack is stacked, chronicled. */
export function atlasPackKeep(session: ViewSession, ws: Workspace): AtlasHost["keep"] {
  return async (kind, doc) => {
    const st = session.stack();
    const mine = st.added.find((p) => p.id === ATLAS_PACK);
    const { refused: _, ...have } = atlasDocsOf(mine ? [mine] : []);
    const next: AtlasDocs = { ...have, [kind]: [...(have[kind] as { id: string }[]).filter((x) => x.id !== doc.id), doc] } as AtlasDocs;
    const pack = { format: "sse.pack/1", id: ATLAS_PACK, version: bump(mine?.version ?? "0.0.0"), kind: ["presentation"], kernel_range: "^0.1", licence: "Apache-2.0", provides: { other: { atlas: next } } } as unknown as PackDoc;
    const r = await stackPacks(session, ws, [...st.added.filter((p) => p.id !== ATLAS_PACK), pack]);
    return r.status === "applied" ? null : r.status === "refused" ? r.reason : r.status === "error" ? r.message : "waiting for approval";
  };
}

/** The Atlas and interface catalogue for Orac: editions, note sets, kits, palettes and parts with their controls. */
export function atlasEntries(host: Pick<AtlasHost, "docs" | "parts" | "looks">): Entry[] {
  const docs = host.docs();
  const eds = [...SHIPPED_EDITIONS, ...docs.editions];
  const sets = [...SHIPPED_NOTE_SETS, ...docs.note_sets];
  const kits = [...STARTER_KITS, ...docs.kits];
  const parts = host.parts();
  return [
    ...eds.map((e) => ({ key: `edition:${e.id}`, line: `Atlas edition ${e.name}: ${e.description}`, full: JSON.stringify(e) })),
    ...sets.map((s) => ({ key: `notes:${s.id}`, line: `margin note set ${s.name} (${s.notes.length} notes)`, full: JSON.stringify({ ...s, notes: s.notes.slice(0, 12) }) + (s.notes.length > 12 ? `\n(${s.notes.length - 12} more notes in the same form)` : "") })),
    ...kits.map((k) => ({ key: `kit:${k.id}`, line: `interface kit ${k.name}: ${k.description}`, full: JSON.stringify(k) })),
    { key: "palettes", line: `the ${SHIPPED_PALETTES.length} shipped palettes`, full: SHIPPED_PALETTES.map((p) => `${p.id}: ${p.name} (ground ${p.tokens.ground}, ink ${p.tokens.ink}, accent ${p.tokens.accent})`).join("\n") },
    { key: "parts", line: `the ${parts.length} player parts a kit may place, with their controls`, full: parts.map((p) => `${p.id}: controls ${p.controls.join(", ") || "none"}${p.gm_only ? " (GM only: never on a player's screen)" : ""}`).join("\n") || "none" },
    { key: "looks", line: "the looks a kit or edition may wear", full: host.looks().join(", ") },
  ];
}

/** The host for the GM's chart room: documents from the pack stack, scenes for the test render read from the kernel. */
export function sessionAtlasHost(session: ViewSession, ws: Workspace, o: { looks: () => readonly string[]; vocabs: Readonly<Record<string, Vocab>>; system: () => string | null; parts: () => readonly PartInfo[] }): AtlasHost {
  return {
    docs: () => {
      const s = session.stack();
      const { refused: _, ...docs } = atlasDocsOf([...s.base, ...(s.campaign ? [s.campaign] : [])]);
      return docs;
    },
    looks: o.looks,
    vocabs: () => o.vocabs,
    parts: o.parts,
    async scenes() {
      const sys = o.system();
      if (!sys) return [];
      const out: PlayerScene[] = [];
      // The GM's own knowledge and the table's: a figure that differs from the Survey atlas, or a fact the table was not given, shows in one of them.
      for (const audience of ["gm", "everyone"]) {
        const r = await session.scene(sys, { audience, at_yr: ws.now_yr });
        if (r.ok) out.push(r.scene);
      }
      return out;
    },
    keep: atlasPackKeep(session, ws),
  };
}
