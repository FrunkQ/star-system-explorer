// The Atlas's page model (track AT, package AT1; atlas design section 10): a pure function from the audience's own
// scene, a subject, an edition and a date to a list of blocks. The view draws blocks on screen, on paper and as text;
// it formats no figure itself. The same scene, edition and date give the same block list on every device, and a
// seat's list can only hold what its scene holds: nothing here reads the truth, a placeholder, or the clock.
import type { Depth } from "../depth.ts";
import { fact, parentOf, subjectName, wordsFor, type PlayerScene, type PlayerSubject, type Vocab } from "../player-scene.ts";
import { lineLabel, showFact, type FactShown } from "./catalogue.ts";
import { editionWord, type Edition, type GapStyle, type PageKind, type PlateStyle, type TemplateSection } from "./edition.ts";

// ------------------------------------------------------------------------------------------------------ blocks

/** One line of a facts section. `gap` rows carry no value and no hint of one: their look comes from the edition. */
export interface FactRow {
  field: string;
  label: string;
  state: "known" | "band" | "gap";
  /** What is printed: the figure or words; for a gap, the edition's words (empty for a bar). */
  text: string;
  /** Drawn as a fixed-length bar (gap style `bar`). */
  bar?: boolean;
  /** For a figure: the SI value(s) and what the figures door was asked, so the view can open its Why. */
  figure?: Extract<FactShown, { figure?: unknown }>["figure"];
}

export interface TableCell {
  text: string;
  state: "known" | "band" | "gap";
  bar?: boolean;
}

export interface TableRow {
  /** The page this row links to. */
  link: string;
  cells: TableCell[];
}

export type Block =
  | { type: "title"; text: string; subtitle: string; kind: PageKind }
  | { type: "prose"; key: string; title: string; text: string }
  | { type: "facts"; key: string; title: string; rows: FactRow[] }
  | { type: "table"; key: string; title: string; columns: string[]; rows: TableRow[] }
  | { type: "plate"; key: string; title: string; style: PlateStyle; subject: string; addresses: string[]; names: string[] }
  | { type: "story"; key: string; title: string; lines: { label: string; text: string }[] }
  | { type: "note"; text: string }
  | { type: "rule" };

export interface AtlasOptions {
  vocab: Vocab;
  /** The reader's depth (Curious, Science, Engine): how the figures door says each figure. Science when absent. */
  depth?: Depth;
}

// --------------------------------------------------------------------------------------------------- the book

export interface ContentsLine {
  address: string;
  title: string;
  kind: PageKind;
  /** Page number, from 1, in reading order. */
  number: number;
  /** How far down the chapter it is: 0 for the top of the book. */
  level: number;
  parent: string | null;
}

/** Address order a person expects: "body:2" before "body:10". */
function addressOrder(a: string, b: string): number {
  const pa = a.split("/");
  const pb = b.split("/");
  for (let i = 0; i < Math.min(pa.length, pb.length); i++) {
    if (pa[i] === pb[i]) continue;
    const [ka, na] = pa[i]!.split(":");
    const [kb, nb] = pb[i]!.split(":");
    if (ka !== kb) return ka! < kb! ? -1 : 1;
    const x = Number(na);
    const y = Number(nb);
    if (Number.isFinite(x) && Number.isFinite(y) && x !== y) return x - y;
    return pa[i]! < pb[i]! ? -1 : 1;
  }
  return pa.length - pb.length;
}

/** The kind of page a subject gets, from what the audience knows of it and where it sits. */
export function pageKindOf(scene: PlayerScene, s: PlayerSubject): PageKind {
  const k = fact(s, "kind")?.value;
  const top = s.address === scene.scope;
  if (k === "sector" || /^u:[^/]+$/.test(s.address) || /\/sector:[^/]+$/.test(s.address)) return "sector";
  if (k === "system") return "system";
  if (k === "star") return top ? "system" : "star";
  if (k === "moon") return "moon";
  if (k === "planetary-body" || k === "planet") return "world";
  if (k === "station" || k === "ship" || k === "construct") return "station";
  // Not yet known what it is: a world if it sits under a star, else an entry.
  return parentOf(s.address, scene.scope) === null && !top ? "world" : top ? "system" : "other";
}

/**
 * Where a subject sits in this audience's book: its nearest ancestor the audience knows of, else the top. A moon of a
 * world they have not found sits under the star, so nothing about the unknown world shows.
 */
export function hostIn(scene: PlayerScene, address: string): string {
  let p = parentOf(address, scene.scope);
  while (p && !scene.subjects.some((x) => x.address === p)) p = parentOf(p, scene.scope);
  return p ?? scene.scope;
}

/** The subjects directly under `address` in the audience's book, in address order. */
export function childrenIn(scene: PlayerScene, address: string): PlayerSubject[] {
  const kids = scene.subjects.filter((s) => s.address !== scene.scope && hostIn(scene, s.address) === address);
  return kids.sort((a, b) => addressOrder(a.address, b.address));
}

/**
 * The Atlas's contents for this scene: the top of the book (the scope), then each subject under it, depth first,
 * in address order. Only subjects in the audience's scene have a line: a world they have not found has no line and
 * no number.
 */
export function atlasContents(scene: PlayerScene): ContentsLine[] {
  const out: ContentsLine[] = [];
  const top = scene.subjects.find((s) => s.address === scene.scope);
  const walk = (addr: string, level: number, parent: string | null) => {
    for (const s of childrenIn(scene, addr)) {
      out.push({ address: s.address, title: subjectName(s), kind: pageKindOf(scene, s), number: out.length + 1, level, parent });
      walk(s.address, level + 1, s.address);
    }
  };
  if (top) out.push({ address: top.address, title: subjectName(top), kind: pageKindOf(scene, top), number: 1, level: 0, parent: null });
  walk(scene.scope, top ? 1 : 0, top ? top.address : null);
  return out;
}

/** A chapter: the page and every page under it, in reading order (what "Print this chapter" prints). */
export function chapterOf(lines: readonly ContentsLine[], address: string): ContentsLine[] {
  const i = lines.findIndex((l) => l.address === address);
  if (i < 0) return [];
  const out = [lines[i]!];
  for (let j = i + 1; j < lines.length && lines[j]!.level > lines[i]!.level; j++) out.push(lines[j]!);
  return out;
}

// --------------------------------------------------------------------------------------------------- one page

export interface AtlasPage {
  address: string;
  kind: PageKind;
  number: number;
  /** The edition this page was built in. */
  edition: string;
  title: string;
  blocks: Block[];
  prev: string | null;
  next: string | null;
}

function gapText(ed: Edition, short: boolean): { text: string; bar?: boolean } | null {
  const style: GapStyle = ed.gaps;
  if (style === "omit") return null;
  if (style === "bar") return { text: "", bar: true };
  return { text: short ? ed.voice.gap_short : ed.voice.gap };
}

function rowFor(s: PlayerSubject, field: string, ed: Edition, opts: AtlasOptions, isGm: boolean): FactRow | null {
  const shown = showFact(s, field, opts);
  const lab = lineLabel(ed.voice.labels, opts.vocab, field);
  if (shown.state === "gap") {
    // The GM's scene holds everything there is: a line with nothing behind it does not apply to this subject.
    if (isGm) return null;
    const g = gapText(ed, false);
    return g ? { field, label: lab, state: "gap", ...g } : null;
  }
  return { field, label: lab, state: shown.state, text: shown.text, ...("figure" in shown && shown.figure ? { figure: shown.figure } : {}) };
}

function cellFor(s: PlayerSubject, field: string, ed: Edition, opts: AtlasOptions, isGm: boolean): TableCell {
  if (field === "name") return { text: subjectName(s), state: "known" };
  const shown = showFact(s, field, opts);
  if (shown.state !== "gap") return { text: shown.text, state: shown.state };
  if (isGm) return { text: "", state: "gap" };
  const g = gapText(ed, true) ?? { text: "" };
  return { state: "gap", ...g };
}

/** The summary: the vocabulary's sentence for each known line, with figures said by the figures door. */
function proseFor(s: PlayerSubject, lines: readonly string[], ed: Edition, opts: AtlasOptions): string {
  const said: string[] = [];
  for (const field of lines) {
    const shown = showFact(s, field, opts);
    if (shown.state === "gap") continue;
    const w = wordsFor(opts.vocab, field);
    if (!w.sentence) continue;
    said.push(w.sentence.replace("{name}", subjectName(s)).replace("{text}", shown.text));
  }
  return said.length ? said.join(" ") : editionWord(ed, "prose.empty", "Little is known of it yet.");
}

function subtitleFor(scene: PlayerScene, s: PlayerSubject, kind: PageKind, ed: Edition, opts: AtlasOptions): string {
  const parts: string[] = [editionWord(ed, `page.${kind}`, kind)];
  // What sort of thing it is, in the first words the audience has: its class, else its size, else its star type.
  for (const field of ["class", "size_class", "spectral"]) {
    const shown = showFact(s, field, opts);
    if (shown.state === "known") {
      parts.push(shown.text);
      break;
    }
  }
  const parent = s.address !== scene.scope ? hostIn(scene, s.address) : null;
  const p = parent ? scene.subjects.find((x) => x.address === parent) : undefined;
  if (p && kind !== "system" && kind !== "sector") parts.push(`of ${subjectName(p)}`);
  return parts.join(" · ");
}

function sectionBlocks(scene: PlayerScene, s: PlayerSubject, sec: TemplateSection, ed: Edition, opts: AtlasOptions, isGm: boolean): Block[] {
  const title = editionWord(ed, sec.key, sec.key.replace(/^section\./, ""));
  switch (sec.block ?? "facts") {
    case "prose":
      return [{ type: "prose", key: sec.key, title, text: proseFor(s, sec.lines ?? [], ed, opts) }];
    case "plate": {
      if (ed.plates === "none") return [];
      const kids = childrenIn(scene, s.address);
      return kids.length ? [{ type: "plate", key: sec.key, title, style: ed.plates, subject: s.address, addresses: kids.map((k) => k.address), names: kids.map(subjectName) }] : [];
    }
    case "table": {
      const t = sec.table!;
      const kids = childrenIn(scene, s.address);
      if (!kids.length) return [];
      return [
        {
          type: "table",
          key: sec.key,
          title,
          columns: t.columns.map((c) => editionWord(ed, c, c.replace(/^column\./, ""))),
          rows: kids.map((k) => ({ link: k.address, cells: t.lines.map((f) => cellFor(k, f, ed, opts, isGm)) })),
        },
      ];
    }
    case "story": {
      const lines = s.facts
        .filter((f) => f.field.startsWith("story.") && typeof f.value === "string")
        .map((f) => ({ label: lineLabel(ed.voice.labels, opts.vocab, f.field), text: String(f.value) }));
      return lines.length ? [{ type: "story", key: sec.key, title, lines }] : [];
    }
    case "facts": {
      const rows = (sec.lines ?? []).map((f) => rowFor(s, f, ed, opts, isGm)).filter((r): r is FactRow => r !== null);
      return rows.length ? [{ type: "facts", key: sec.key, title, rows }] : [];
    }
  }
}

/**
 * One page of the Atlas: the subject at `address` in this audience's scene, in this edition. Null when the scene has
 * no such subject (the audience does not know it exists). Pure: no clock, no randomness, no reads beyond its inputs.
 */
export function buildPage(scene: PlayerScene, address: string, ed: Edition, opts: AtlasOptions): AtlasPage | null {
  const s = scene.subjects.find((x) => x.address === address);
  if (!s) return null;
  const lines = atlasContents(scene);
  const i = lines.findIndex((l) => l.address === address);
  const kind = pageKindOf(scene, s);
  const isGm = scene.audience === "gm";
  const blocks: Block[] = [{ type: "title", text: subjectName(s), subtitle: subtitleFor(scene, s, kind, ed, opts), kind }];
  for (const sec of ed.templates[kind] ?? []) blocks.push(...sectionBlocks(scene, s, sec, ed, opts, isGm));
  // A barred page says once what a bar means: it is the same sentence whatever is behind the bars.
  if (blocks.some((b) => (b.type === "facts" && b.rows.some((r) => r.bar)) || (b.type === "table" && b.rows.some((r) => r.cells.some((c) => c.bar))))) {
    blocks.push({ type: "note", text: editionWord(ed, "note.bars", "Lines marked with a bar are above this document's clearance or await survey.") });
  }
  return {
    address,
    kind,
    number: i + 1,
    edition: ed.id,
    title: subjectName(s),
    blocks,
    prev: i > 0 ? lines[i - 1]!.address : null,
    next: i >= 0 && i + 1 < lines.length ? lines[i + 1]!.address : null,
  };
}

/** Every fact field a block list shows as known, by subject: what a test compares with the scene. */
export function shownFields(page: AtlasPage): string[] {
  const out: string[] = [];
  for (const b of page.blocks) if (b.type === "facts") for (const r of b.rows) if (r.state !== "gap") out.push(r.field);
  return out;
}

/** Every field the edition's templates name (loader check 2 reads this). */
export function templateFields(ed: Edition): string[] {
  const out = new Set<string>();
  for (const secs of Object.values(ed.templates)) for (const s of secs) for (const f of [...(s.lines ?? []), ...(s.table?.lines ?? [])]) out.add(f);
  return [...out].sort();
}

