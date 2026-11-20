// World data sheet (X3a): the pure half. Everything the view shows is built
// here from the kernel's `codes` response and the pack's own data (the code
// sheet, the class names, the vocabulary), so no game is written into the
// view. A Traveller pack makes it a UWP sheet; any other pack with a code
// sheet makes it its own.

// ---- The kernel's `codes` response (sse-codes `Reading`, serialized) ----

export type FactValue = number | string | boolean | null;

export interface Evidence {
  field: string;
  value: FactValue;
}

/** A rule's tag with its evidence (sse-sift `TagState`). */
export interface TagState {
  tag: string;
  rule: string;
  evidence: Evidence[];
}

export interface AxisFit {
  axis: string;
  fit: number;
  value: FactValue;
  change_to_core: number | null;
}

export interface ClassFit {
  class: string;
  fit: number;
  depth: number;
  limiting: AxisFit | null;
  axes: AxisFit[];
}

/** A scheme's label and its working (sse-sift `Label`). */
export interface Label {
  scheme: string;
  base: string | null;
  fit: number;
  modifiers: string[];
  nearest: ClassFit[];
  kept_by_stickiness: boolean;
}

export type Why =
  | { kind: "classified"; label: Label; defaulted?: string[] }
  | { kind: "unclassified"; label: Label }
  | { kind: "authored"; field: string; physics: string | null }
  | { kind: "leading"; field: string; value: number; exponent: number }
  | { kind: "counted"; tag: string; members: string[] }
  | { kind: "no-system"; tag: string };

export interface SegmentReading {
  id: string;
  symbol: string;
  value: number | null;
  why: Why;
}

export interface GroupReading {
  id: string;
  /** The short code (tag less the group's prefix) and the rule that gave it. */
  tags: [string, TagState][];
}

export interface Reading {
  sheet: string;
  lines: Record<string, string>;
  segments: SegmentReading[];
  groups: GroupReading[];
  prepared: TagState[];
}

export interface AddressedReading {
  address: string;
  reading: Reading;
}

export interface CodesResponse {
  type: "codes";
  sheet: string;
  readings: AddressedReading[];
}

// ---- The pack's code sheet (only the parts the view reads) ----

export interface SheetSymbol {
  class: string;
  symbol: string;
  value?: number;
}

export interface SheetSegment {
  id: string;
  label?: string;
  source: { from: "classify"; scheme: string; symbols: SheetSymbol[] } | { from: "leading"; field: string; segment: string } | { from: "count"; tag: string; max: number };
  authored?: string;
}

export interface SheetGroup {
  id: string;
  label?: string;
  prefix: string;
}

export interface CodeSheetDef {
  id: string;
  label?: string;
  about?: string;
  segments: SheetSegment[];
  layouts: Record<string, string>;
  groups?: SheetGroup[];
}

/** A pack's `provides.other.vocabulary`: `terms` plus code-to-words tables. */
export type Vocabulary = Record<string, Record<string, string>>;

/** Class id to its names (`physics`, `plain`, `story`), from the packs' classes. */
export type ClassNames = Record<string, Record<string, string>>;

/** What the view is given: the response, the sheet it was read through, and the pack's words. */
export interface WorldCodesInput {
  response: CodesResponse;
  sheet: CodeSheetDef;
  vocabulary?: Vocabulary;
  classes?: ClassNames;
  /** Body names by address; a body without one shows its address. */
  names?: Record<string, string>;
  /** The body whose sheet is open; none shows the list when there are several. */
  selected?: string;
  title?: string;
}

// ---- Reading the packs ----

type PackLike = { provides?: { other?: Record<string, unknown> } } | null | undefined;

const other = (p: unknown): Record<string, unknown> => ((p as PackLike)?.provides?.other ?? {}) as Record<string, unknown>;

/** The code sheet with this id in the stack (the last pack that has it wins), or the first sheet when no id is given. */
export function sheetOf(packs: readonly unknown[], id?: string): CodeSheetDef | null {
  let found: CodeSheetDef | null = null;
  for (const p of packs) {
    const sheets = other(p).code_sheets;
    if (!Array.isArray(sheets)) continue;
    for (const s of sheets as CodeSheetDef[]) {
      if (id === undefined) {
        if (found === null) found = s;
      } else if (s.id === id) found = s;
    }
  }
  return found;
}

/** Every pack's vocabulary tables merged in stack order (later packs win per entry). */
export function vocabularyOf(packs: readonly unknown[]): Vocabulary {
  const out: Vocabulary = {};
  for (const p of packs) {
    const v = other(p).vocabulary;
    if (!v || typeof v !== "object" || Array.isArray(v)) continue;
    for (const [table, entries] of Object.entries(v as Record<string, unknown>)) {
      if (!entries || typeof entries !== "object" || Array.isArray(entries)) continue;
      const t = (out[table] ??= {});
      for (const [k, w] of Object.entries(entries as Record<string, unknown>)) if (typeof w === "string") t[k] = w;
    }
  }
  return out;
}

/** Class names from every pack's `provides.other.classes`. */
export function classNamesOf(packs: readonly unknown[]): ClassNames {
  const out: ClassNames = {};
  for (const p of packs) {
    const cs = other(p).classes;
    if (!Array.isArray(cs)) continue;
    for (const c of cs as { id?: unknown; names?: unknown }[]) {
      if (typeof c?.id !== "string" || !c.names || typeof c.names !== "object") continue;
      const n: Record<string, string> = {};
      for (const [k, w] of Object.entries(c.names as Record<string, unknown>)) if (typeof w === "string") n[k] = w;
      out[c.id] = n;
    }
  }
  return out;
}

/** The view's input from a stack and a `codes` response: the sheet the response names, its words and class names. */
export function inputFromPacks(packs: readonly unknown[], response: CodesResponse, extra: Partial<Pick<WorldCodesInput, "names" | "selected" | "title">> = {}): WorldCodesInput | null {
  const sheet = sheetOf(packs, response.sheet);
  if (!sheet) return null;
  return { response, sheet, vocabulary: vocabularyOf(packs), classes: classNamesOf(packs), ...extra };
}

// ---- Words ----

/** A class's name: "size 8 (about 12,800 km across)" when the physics and plain names differ. */
export function className(classes: ClassNames | undefined, id: string): string {
  const n = classes?.[id];
  const physics = n?.physics;
  const plain = n?.plain;
  if (physics && plain && plain !== physics) return `${physics} (${plain})`;
  return physics ?? plain ?? id;
}

/** The class a symbol stands for in a classify segment, named. */
export function symbolName(seg: SheetSegment | undefined, symbol: string, classes?: ClassNames): string | null {
  if (!seg || seg.source.from !== "classify") return null;
  const s = seg.source.symbols.find((x) => x.symbol === symbol);
  return s ? className(classes, s.class) : null;
}

const fitWords = (fit: number): string => (fit >= 1 ? "full fit" : `fit ${fit.toFixed(2)}`);

const shown = (v: FactValue): string => (v === null ? "absent" : typeof v === "number" ? trimNumber(v) : String(v));

function trimNumber(v: number): string {
  if (Number.isInteger(v)) return String(v);
  return String(Number(v.toPrecision(4)));
}

/** "8.1 × 10^9" for a leading-digit reading. */
function scaled(value: number, exponent: number): string {
  const m = value / Math.pow(10, exponent);
  return `${Number(m.toPrecision(3))} × 10^${exponent}`;
}

/** True when an authored symbol differs from what the physics gives (SegmentReading::disagrees). */
export function disagrees(seg: SegmentReading): boolean {
  return seg.why.kind === "authored" && seg.why.physics !== null && seg.why.physics !== seg.symbol;
}

/** One plain sentence on why a segment shows its symbol. */
export function whySentence(seg: SegmentReading, sheet?: CodeSheetDef, classes?: ClassNames, names?: Record<string, string>): string {
  const def = sheet?.segments.find((s) => s.id === seg.id);
  const w = seg.why;
  switch (w.kind) {
    case "classified": {
      const base = w.label.base === null ? "no class" : className(classes, w.label.base);
      let s = `Classified as ${base}, ${fitWords(w.label.fit)}`;
      if (w.label.modifiers.length) s += `, with ${w.label.modifiers.map((m) => className(classes, m)).join(", ")}`;
      if (w.label.kept_by_stickiness) s += `; kept from before although another class now fits better`;
      s += ".";
      if (w.defaulted?.length) s += ` Defaulted (absent, read at the axis's stated value): ${w.defaulted.join(", ")}.`;
      return s;
    }
    case "unclassified": {
      const n = w.label.nearest[0];
      if (!n) return "Unclassified (?): no class fits and none is near.";
      let s = `Unclassified (?): no class fits. Nearest is ${className(classes, n.class)}, ${fitWords(n.fit)}`;
      const lim = n.limiting;
      if (lim && lim.change_to_core !== null && lim.change_to_core !== 0) s += `; ${lim.axis} is ${shown(lim.value)} and needs ${lim.change_to_core > 0 ? "+" : ""}${trimNumber(lim.change_to_core)} to reach it`;
      return s + ".";
    }
    case "authored": {
      if (w.physics === null) return `Set by hand (${w.field}); the physics gives no reading here.`;
      const named = symbolName(def, w.physics, classes);
      const what = named ? `${w.physics}, ${named}` : w.physics;
      if (w.physics === seg.symbol) return `Set by hand (${w.field}); physics says ${what} too.`;
      return `Set by hand (${w.field}); physics says ${what}. The hand-set code and the physics disagree.`;
    }
    case "leading":
      return `Leading digit of ${w.field}, ${scaled(w.value, w.exponent)}, at the power of ten the sheet settled.`;
    case "counted": {
      const n = w.members.length;
      const who = w.members.map((a) => names?.[a] ?? a).join(", ");
      return `Counted: ${n} member${n === 1 ? "" : "s"} of this system carr${n === 1 ? "ies" : "y"} ${w.tag}${n ? `: ${who}` : ""}.`;
    }
    case "no-system":
      return `No system: ${w.tag} is counted over the whole system, and this body was read without it.`;
  }
}

// ---- Rows ----

export interface LineRow {
  id: string;
  label: string;
  text: string;
}

/** The code lines in the sheet's layout order (lines the sheet does not list come after). */
export function lineRows(reading: Reading, sheet: CodeSheetDef): LineRow[] {
  const ids = [...Object.keys(sheet.layouts).filter((k) => k in reading.lines), ...Object.keys(reading.lines).filter((k) => !(k in sheet.layouts))];
  return ids.map((id) => ({ id, label: id.toUpperCase(), text: reading.lines[id] ?? "" }));
}

export interface SegmentRow {
  id: string;
  label: string;
  symbol: string;
  kind: Why["kind"];
  why: string;
  disagrees: boolean;
  /** What the physics gives, when the code was set by hand. */
  physics: string | null;
  /** Fields that stood in at their default value. */
  defaulted: string[];
}

export function segmentRows(reading: Reading, sheet: CodeSheetDef, classes?: ClassNames, names?: Record<string, string>): SegmentRow[] {
  const order = new Map(sheet.segments.map((s, i) => [s.id, i]));
  const segs = [...reading.segments].sort((a, b) => (order.get(a.id) ?? 1e9) - (order.get(b.id) ?? 1e9));
  return segs.map((s) => ({
    id: s.id,
    label: sheet.segments.find((d) => d.id === s.id)?.label || s.id,
    symbol: s.symbol,
    kind: s.why.kind,
    why: whySentence(s, sheet, classes, names),
    disagrees: disagrees(s),
    physics: s.why.kind === "authored" ? s.why.physics : null,
    defaulted: s.why.kind === "classified" ? (s.why.defaulted ?? []) : [],
  }));
}

/** The vocabulary table for a group: the prefix less its dot ("traveller.trade"), else the sheet's namespace and the group id ("traveller.bases"). */
export function vocabFor(vocabulary: Vocabulary | undefined, sheet: CodeSheetDef, group: SheetGroup | undefined, groupId: string): Record<string, string> | undefined {
  if (!vocabulary) return undefined;
  const ns = sheet.id.split(".")[0] ?? sheet.id;
  const keys = [group?.prefix.replace(/\.$/, ""), `${ns}.${groupId}`].filter((k): k is string => !!k);
  for (const k of keys) if (vocabulary[k]) return vocabulary[k];
  return undefined;
}

export interface Chip {
  code: string;
  /** The pack's words for the code, if it has any. */
  words: string | null;
  rule: string;
  evidence: Evidence[];
  /** Tooltip: words, the rule and what it read. */
  title: string;
}

export interface ChipGroup {
  id: string;
  label: string;
  chips: Chip[];
}

/** The rule groups as chips, in the sheet's group order; empty groups are left out. */
export function chipGroups(reading: Reading, sheet: CodeSheetDef, vocabulary?: Vocabulary): ChipGroup[] {
  const defs = sheet.groups ?? [];
  const order = new Map(defs.map((g, i) => [g.id, i]));
  const gs = [...reading.groups].sort((a, b) => (order.get(a.id) ?? 1e9) - (order.get(b.id) ?? 1e9));
  const out: ChipGroup[] = [];
  for (const g of gs) {
    if (!g.tags.length) continue;
    const def = defs.find((d) => d.id === g.id);
    const words = vocabFor(vocabulary, sheet, def, g.id);
    out.push({
      id: g.id,
      label: def?.label || g.id,
      chips: g.tags.map(([code, st]) => {
        const w = words?.[code] ?? null;
        const read = st.evidence.map((e) => `${e.field} = ${shown(e.value)}`).join(", ");
        return { code, words: w, rule: st.rule, evidence: st.evidence, title: `${w ? `${code}: ${w}. ` : `${code}. `}Rule ${st.rule}${read ? `, read ${read}` : ""}.` };
      }),
    });
  }
  return out;
}

export interface BodyRow {
  address: string;
  name: string;
  lines: LineRow[];
  /** Group id to its codes, in sheet order (empty groups included, so columns line up). */
  groups: { id: string; label: string; codes: string[] }[];
  /** How many hand-set digits disagree with the physics. */
  disagreements: number;
}

/** One row per body for the list mode. */
export function bodyRows(input: WorldCodesInput): BodyRow[] {
  const defs = input.sheet.groups ?? [];
  return input.response.readings.map(({ address, reading }) => ({
    address,
    name: input.names?.[address] ?? address,
    lines: lineRows(reading, input.sheet),
    groups: defs.map((d) => ({ id: d.id, label: d.label || d.id, codes: reading.groups.find((g) => g.id === d.id)?.tags.map(([c]) => c) ?? [] })),
    disagreements: reading.segments.filter(disagrees).length,
  }));
}

/** The text tier: one body's sheet as plain sentences, for small screens and screen readers. */
export function textSheet(input: WorldCodesInput, address: string): string[] {
  const r = input.response.readings.find((x) => x.address === address);
  if (!r) return [`No reading for ${address}.`];
  const out = [`${input.names?.[address] ?? address}: ${input.sheet.label || input.sheet.id}.`];
  for (const l of lineRows(r.reading, input.sheet)) out.push(`${l.label} ${l.text}.`);
  for (const s of segmentRows(r.reading, input.sheet, input.classes, input.names)) out.push(`${s.label} ${s.symbol}${s.disagrees ? " (differs from physics)" : ""}: ${s.why}`);
  for (const g of chipGroups(r.reading, input.sheet, input.vocabulary)) out.push(`${g.label}: ${g.chips.map((c) => (c.words ? `${c.code} (${c.words})` : c.code)).join(", ")}.`);
  return out;
}

/** The text tier for several bodies: one line each. */
export function textList(input: WorldCodesInput): string[] {
  return bodyRows(input).map((b) => {
    const codes = b.groups.filter((g) => g.codes.length).map((g) => `${g.label} ${g.codes.join(" ")}`);
    return [`${b.name}: ${b.lines.map((l) => `${l.label} ${l.text}`).join(", ")}`, ...codes].join("; ") + (b.disagreements ? `; ${b.disagreements} hand-set digit${b.disagreements === 1 ? " differs" : "s differ"} from physics` : "") + ".";
  });
}
