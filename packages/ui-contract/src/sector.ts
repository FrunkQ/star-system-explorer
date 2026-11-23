// Code sheets and sectors on the chart room (X3a, X3c): the kernel's `codes`,
// `sector_map`, `import_sector` and `export_sector` answers, typed for the
// views, and the pure step that turns a reading into the fields a hex map's
// glyph rules read. Nothing here names a game: the code sheet, its segment
// ids and its groups come from whichever pack is stacked.

/** A sse-codes `Reading`, as the kernel serialises it (the world sheet view types it in full). */
export interface CodeReading {
  sheet: string;
  lines: Record<string, string>;
  segments: { id: string; symbol: string; value: number | null; why: { kind: string } & Record<string, unknown> }[];
  groups: { id: string; tags: [string, unknown][] }[];
  prepared: unknown[];
}

export interface CodesRead {
  type: "codes";
  sheet: string;
  readings: { address: string; reading: CodeReading }[];
}

export interface MappedWorldRow {
  system: string;
  body: string;
  hex: string;
  name: string;
  sector: string;
  reading: CodeReading;
}

export interface SectorMap {
  type: "sector_mapped";
  sheet: string;
  worlds: MappedWorldRow[];
  /** Mapped systems with no main world, by address. */
  skipped: string[];
}

export type SectorFormat = "t5_tab" | "sec" | "uwp_lines";

/** Words for each file format, for a picker. */
export const SECTOR_FORMATS: readonly { id: SectorFormat; label: string; ext: string }[] = [
  { id: "t5_tab", label: "Tab-delimited (T5 columns)", ext: "tab" },
  { id: "sec", label: "Legacy SEC columns", ext: "sec" },
  { id: "uwp_lines", label: "One line per world", ext: "txt" },
];

export interface SectorImportAsk {
  text: string;
  format?: SectorFormat;
  sheet?: string;
  /** Write the worlds as history; without it the import is a preview. */
  apply?: boolean;
  /** With apply: grow each world's whole system first (heavy for a whole sector). */
  grow?: boolean;
  at_s?: number | bigint;
}

export interface ImportedWorldRow {
  hex: string;
  name: string;
  system: string;
  main_world: string;
  raw: string;
  refused?: string;
  guessed: string[];
  differs: { group: string; listed_only: string[]; read_only: string[] }[];
  plan?: unknown;
  world_edits: { field: string; value: unknown; impossible?: boolean; [k: string]: unknown }[];
}

export interface SectorImported {
  type: "sector_import";
  import: {
    sheet: string;
    sector: string | null;
    worlds: ImportedWorldRow[];
    warnings: string[];
    summary: { lines: number; imported: number; refused: number; impossible: number; guessed: number; differ: number };
  };
  summary: string;
  events: string[];
  state_hash?: string;
}

export interface SectorExported {
  type: "sector_export";
  text: string;
  written: number;
  skipped: string[];
}

/**
 * The fields a hex map's glyph rules read, from one reading: every segment by
 * its id (its number when it has one, else its symbol), every group by its id
 * as the list of its short codes, each group's first code under
 * `<group>_first` (a zone is one code), every code line by its id, and the name.
 * A group with no codes reads "none", so a rule can ask for the unzoned.
 */
export function mapFields(reading: CodeReading, name: string): Record<string, string | number | string[]> {
  const out: Record<string, string | number | string[]> = { name };
  for (const s of reading.segments) out[s.id] = s.value ?? s.symbol;
  for (const g of reading.groups) {
    const codes = g.tags.map(([t]) => t);
    out[g.id] = codes;
    out[`${g.id}_first`] = codes[0] ?? "none";
  }
  for (const [id, line] of Object.entries(reading.lines)) out[id] = line;
  return out;
}

/** True when some pack in the stack gives `provides.other.<key>` (a need a screen's rail entry may name). */
export function stackProvides(packs: readonly unknown[], key: string): boolean {
  return packs.some((p) => {
    const v = (p as { provides?: { other?: Record<string, unknown> } } | null)?.provides?.other?.[key];
    return v !== undefined && v !== null && !(Array.isArray(v) && v.length === 0);
  });
}
