// Atlas editions (track AT, atlas design sections 1, 11 and 12): a data document that says how the Atlas reads and
// looks. It chooses words, order, look, gaps, letterhead and where sources go. It never states a fact, never changes a
// figure's precision and never shows anything the audience's scene does not hold: the page builder (page.ts) is the
// only thing that turns a scene into blocks, and an edition only steers it.
import survey from "./editions/survey.edition.json" with { type: "json" };

export const EDITION_SCHEMA = "sse.edition/1";

/** The kinds of page the Atlas has (design section 4.3). */
export type PageKind = "sector" | "system" | "star" | "world" | "moon" | "station" | "other";
export const PAGE_KINDS: readonly PageKind[] = ["sector", "system", "star", "world", "moon", "station", "other"];

/** How a line the template has, and the audience has not learnt, is shown (design section 5). */
export type GapStyle = "plain" | "voice" | "bar" | "omit";
export const GAP_STYLES: readonly GapStyle[] = ["plain", "voice", "bar", "omit"];

export type PlateStyle = "chart" | "true" | "wireframe" | "none";
export type SourcesStyle = "endnotes" | "section" | "off";

/** A table of the subject's children (the worlds of a system, the moons of a world): one column per line. */
export interface TemplateTable {
  /** Column headings, as label keys. */
  columns: string[];
  /** One fact field per column. */
  lines: string[];
}

/** One section of a page: a key (its title is the voice's label for it) and what it holds. */
export interface TemplateSection {
  key: string;
  /** What the section is: facts rows (the default), prose, a plate, a table of children, or the story lines heard. */
  block?: "facts" | "prose" | "plate" | "table" | "story";
  /** Fact fields: rows for `facts`, the facts spoken for `prose`. */
  lines?: string[];
  table?: TemplateTable;
}

export interface EditionVoice {
  /** New words for section titles, column headings and line labels (`line.<field>`), by key. */
  labels?: Record<string, string>;
  /** The words for a gap in `plain` and `voice` gap styles. */
  gap?: string;
  /** The words for a gap in a table cell. */
  gap_short?: string;
}

export interface EditionDoc {
  schema: typeof EDITION_SCHEMA;
  id: string;
  name: string;
  description: string;
  /** The edition this one starts from; everything not stated comes from it. */
  extends?: string;
  /** A look (player theme) id; its print tokens are used on paper. */
  look?: string;
  plates?: PlateStyle;
  voice?: EditionVoice;
  gaps?: GapStyle;
  letterhead?: { use: "brand" | "masthead" | "none"; lines?: string[]; stamp?: string };
  sources?: SourcesStyle;
  margin?: { sets: string[]; max?: number; signature?: string };
  templates?: Partial<Record<PageKind, TemplateSection[]>>;
  print?: { running_head?: string; footer?: string; save_ink_look?: string };
}

/** An edition with its `extends` chain applied: every field the page builder reads is present. */
export interface Edition {
  id: string;
  name: string;
  description: string;
  /** The chain this edition was made from, itself first. */
  chain: string[];
  look: string;
  plates: PlateStyle;
  gaps: GapStyle;
  sources: SourcesStyle;
  voice: { labels: Record<string, string>; gap: string; gap_short: string };
  letterhead: { use: "brand" | "masthead" | "none"; lines: string[]; stamp?: string };
  margin: { sets: string[]; max: number; signature: string } | null;
  templates: Record<PageKind, TemplateSection[]>;
  print: { running_head?: string; footer?: string; save_ink_look?: string };
}

/** The editions shipped with the base Atlas. */
export const SURVEY_EDITION = survey as unknown as EditionDoc;
export const SHIPPED_EDITIONS: readonly EditionDoc[] = [SURVEY_EDITION];
export const DEFAULT_EDITION = SURVEY_EDITION.id;

/** Why a document is not an edition by its shape alone, or null (loader check 1, before `extends` is followed). */
export function checkEditionShape(x: unknown): string | null {
  const d = x as EditionDoc;
  if (!d || typeof d !== "object") return "an edition is a JSON object";
  if (d.schema !== EDITION_SCHEMA) return `an edition's schema is "${EDITION_SCHEMA}"`;
  for (const k of ["id", "name", "description"] as const) if (typeof d[k] !== "string" || !d[k]) return `an edition needs a ${k}`;
  if (d.extends !== undefined && typeof d.extends !== "string") return "extends names one edition by its id";
  if (d.gaps !== undefined && !GAP_STYLES.includes(d.gaps)) return `gaps is one of ${GAP_STYLES.join(", ")}`;
  for (const [kind, secs] of Object.entries(d.templates ?? {})) {
    if (!PAGE_KINDS.includes(kind as PageKind)) return `there is no page kind "${kind}" (${PAGE_KINDS.join(", ")})`;
    if (!Array.isArray(secs)) return `the ${kind} template is a list of sections`;
    for (const s of secs) {
      if (!s || typeof s.key !== "string") return `a section of the ${kind} template has no key`;
      if (s.block === "table" && (!s.table || s.table.columns.length !== s.table.lines.length)) return `the ${s.key} table needs one column heading per line`;
    }
  }
  return null;
}

/**
 * The edition `id` with its chain applied, from these documents (the shipped ones are always there). Throws with a
 * sentence when the id is unknown, a parent is missing, or the chain loops.
 */
export function resolveEdition(id: string, docs: readonly EditionDoc[] = []): Edition {
  const all = [...SHIPPED_EDITIONS, ...docs];
  const byId = (x: string) => {
    let hit: EditionDoc | undefined;
    for (const d of all) if (d.id === x) hit = d; // the last one given wins, so a pack may replace a shipped edition
    return hit;
  };
  const chain: EditionDoc[] = [];
  let at: string | undefined = id;
  while (at) {
    if (chain.some((c) => c.id === at)) throw new Error(`the edition ${id} extends itself through ${at}`);
    const d = byId(at);
    if (!d) throw new Error(chain.length ? `the edition ${chain[chain.length - 1]!.id} extends ${at}, which is not here` : `there is no edition ${at}`);
    const bad = checkEditionShape(d);
    if (bad) throw new Error(`${d.id}: ${bad}`);
    chain.push(d);
    at = d.extends ?? (d.id === SURVEY_EDITION.id ? undefined : SURVEY_EDITION.id);
  }
  const root = chain[chain.length - 1]!;
  const out: Edition = {
    id: chain[0]!.id,
    name: chain[0]!.name,
    description: chain[0]!.description,
    chain: chain.map((c) => c.id),
    look: root.look ?? "paper",
    plates: root.plates ?? "chart",
    gaps: root.gaps ?? "plain",
    sources: root.sources ?? "endnotes",
    voice: { labels: { ...(root.voice?.labels ?? {}) }, gap: root.voice?.gap ?? "not yet surveyed", gap_short: root.voice?.gap_short ?? "—" },
    letterhead: { use: root.letterhead?.use ?? "none", lines: [...(root.letterhead?.lines ?? [])], ...(root.letterhead?.stamp ? { stamp: root.letterhead.stamp } : {}) },
    margin: null,
    templates: Object.fromEntries(PAGE_KINDS.map((k) => [k, root.templates?.[k] ?? []])) as Record<PageKind, TemplateSection[]>,
    print: { ...(root.print ?? {}) },
  };
  // From the root's child up to the edition itself, each layer overrides what it states.
  for (const d of chain.slice(0, -1).reverse()) {
    if (d.look) out.look = d.look;
    if (d.plates) out.plates = d.plates;
    if (d.gaps) out.gaps = d.gaps;
    if (d.sources) out.sources = d.sources;
    if (d.voice?.labels) out.voice.labels = { ...out.voice.labels, ...d.voice.labels };
    if (d.voice?.gap) out.voice.gap = d.voice.gap;
    if (d.voice?.gap_short) out.voice.gap_short = d.voice.gap_short;
    if (d.letterhead) out.letterhead = { use: d.letterhead.use, lines: [...(d.letterhead.lines ?? out.letterhead.lines)], ...(d.letterhead.stamp ?? out.letterhead.stamp ? { stamp: d.letterhead.stamp ?? out.letterhead.stamp } : {}) };
    for (const k of PAGE_KINDS) if (d.templates?.[k]) out.templates[k] = d.templates[k]!;
    if (d.print) out.print = { ...out.print, ...d.print };
  }
  for (const d of [...chain].reverse()) {
    if (d.margin) out.margin = { sets: [...d.margin.sets], max: d.margin.max ?? 2, signature: d.margin.signature ?? out.margin?.signature ?? "" };
  }
  return out;
}

/** The edition's word for a key, or the fallback. */
export function label(ed: Edition, key: string, fallback: string): string {
  return ed.voice.labels[key] ?? fallback;
}
