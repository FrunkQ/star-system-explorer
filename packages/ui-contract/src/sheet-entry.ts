// "Type it all in" (ED13, editing design section 3 "The sheet"): one plain table of every input a body has, by handle,
// each with its unit, typed in any unit of its kind. It takes a pasted column or rows from a spreadsheet or a paper's
// table, shows what it understood before applying, and applies as one change (one Undo), optionally marked catalogued
// with its source. Rows are the handle sheet's own: each handle, then its box's strips and rings (the place round the
// orbit now is read off the date, so it is left to the box).
import type { Depth } from "./depth.ts";
import type { Json } from "./json.ts";
import { fieldValue, handleValue, type Handle } from "./handles.ts";
import { partHandle } from "./box.ts";
import { readFigure, say, type FigureKind } from "./figures.ts";

export interface SheetRow {
  id: string;
  /** The handle (or box part as a handle) the row writes. */
  handle: Handle;
  label: string;
  /** Other words a pasted row may name it by (its other depth's name, its field). */
  aliases: string[];
}

/** The sheet's rows for a body: each handle in row order, then its box parts (strips and rings), never a bar. */
export function sheetRows(handles: readonly Handle[], depth: Depth): SheetRow[] {
  const out: SheetRow[] = [];
  const name = (h: Handle) => (depth === "curious" ? h.names.curious : h.names.science);
  const add = (h: Handle) => out.push({ id: h.id, handle: h, label: name(h), aliases: [h.names.curious, h.names.science, h.part.field, h.id].map((x) => x.toLowerCase()) });
  for (const h of handles) {
    if (h.part.scale === "bar") continue;
    add(h);
    for (const p of h.box?.parts ?? []) if (!p.part.as) add(partHandle(h, p));
  }
  return out;
}

/**
 * A paste spread over the rows. A line of two or more cells whose first names a row ("Mass", "Day", the field) goes to
 * that row, the last cell as the value; any other line takes the next row down from `start`. Empty lines are skipped.
 */
export function spreadPaste(rows: readonly SheetRow[], text: string, start = 0): Record<string, string> {
  const out: Record<string, string> = {};
  let at = start;
  for (const line of text.split(/\r?\n/)) {
    const cells = line.split(/\t|;|\s{2,}/).map((c) => c.trim()).filter((c) => c.length);
    if (!cells.length) continue;
    const named = cells.length > 1 ? rows.find((r) => r.aliases.includes(cells[0]!.toLowerCase().replace(/:$/, ""))) : undefined;
    if (named) out[named.id] = cells[cells.length - 1]!;
    else if (at < rows.length) out[rows[at++]!.id] = cells[cells.length - 1]!;
  }
  return out;
}

export interface SheetRead {
  /** The fields to write, in row order, with what they were. */
  edits: { field: string; value: number; before: Json; row: string }[];
  /** What it understood, in words: "Mass 0.107 M⊕ → 0.2 M⊕". */
  understood: { row: string; label: string; before: string; after: string }[];
  /** Lines it could not read, and why. */
  problems: { row: string; label: string; why: string }[];
}

/** Read the typed or pasted figures against the body's facts: each in any unit of its row's kind. */
export function readSheet(rows: readonly SheetRow[], typed: Record<string, string>, facts: Record<string, Json>, cls: string, depth: Depth): SheetRead {
  const out: SheetRead = { edits: [], understood: [], problems: [] };
  for (const r of rows) {
    const text = typed[r.id]?.trim();
    if (!text) continue;
    const h = r.handle;
    const kind = h.part.kind as FigureKind;
    // A bare number for a share said as a fraction (a shape of 0.0934) is that fraction.
    const bare = kind === "share" && h.part.reading === "fraction" && /^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$/.test(text);
    const got = bare ? ({ ok: true, si: Number(text) } as const) : readFigure(text, kind);
    if (!got.ok) {
      out.problems.push({ row: r.id, label: r.label, why: got.why });
      continue;
    }
    const old = handleValue(h, facts);
    const value = fieldValue(h, got.si);
    if (old !== null && fieldValue(h, old) === value) continue;
    const sayIt = (si: number) => say(Math.abs(si), kind, { class: cls, ...(h.part.what ? { what: h.part.what } : {}) }, { depth, ...(h.part.reading ? { reading: h.part.reading } : {}) }).text;
    out.edits.push({ field: h.part.field, value, before: facts[h.part.field] ?? null, row: r.id });
    out.understood.push({ row: r.id, label: r.label, before: old === null ? "not set" : sayIt(old), after: sayIt(got.si) });
  }
  return out;
}

/** A body the system sheet can write: its address, name, own facts and scale class. */
export interface SheetBody {
  address: string;
  label: string;
  facts: Record<string, Json>;
  cls: string;
}

export interface TableRead {
  edits: { address: string; field: string; value: number; before: Json }[];
  understood: { body: string; label: string; before: string; after: string }[];
  problems: { where: string; why: string }[];
  /** The table's columns and the input each was matched to (null: not matched, left out). */
  columns: { heading: string; row: string | null }[];
}

const cellsOf = (line: string) => line.split(/\t|;|,(?=\s*\S)|\s{2,}/).map((c) => c.trim());

/**
 * The system sheet (ED13a, design 10B F1): a pasted table for the whole system, a row per body and a column per
 * input. The first line names the columns ("Name, Orbit (AU), Mass, Day"): each is matched to an input by either
 * depth's name, its field or its id, and a unit in brackets is the unit of that column's bare numbers. The first
 * column names the body. Rows naming no body here, and columns naming no input, are said and left out.
 */
export function readTable(text: string, rowsFor: (b: SheetBody) => readonly SheetRow[], bodies: readonly SheetBody[], depth: Depth): TableRead {
  const out: TableRead = { edits: [], understood: [], problems: [], columns: [] };
  const lines = text.split(/\r?\n/).filter((l) => l.trim().length);
  if (!lines.length) return out;
  if (lines.length < 2) return (out.problems.push({ where: "the table", why: "it needs a line of column names, then a line per body" }), out);
  const heads = cellsOf(lines[0]!);
  const unitOf = (h: string) => /[([]\s*([^)\]]+)\s*[)\]]/.exec(h)?.[1]?.trim() ?? null;
  const bare = (h: string) => h.replace(/[([][^)\]]*[)\]]/g, "").trim().toLowerCase();
  const sample = bodies[0] ? rowsFor(bodies[0]) : [];
  for (const [i, h] of heads.entries()) {
    if (i === 0) continue;
    const r = sample.find((x) => x.aliases.includes(bare(h)));
    out.columns.push({ heading: h, row: r?.id ?? null });
    if (!r) out.problems.push({ where: `column "${h}"`, why: "names no input here; left out" });
  }
  const byName = (n: string) => bodies.find((b) => b.label.toLowerCase() === n.trim().toLowerCase());
  for (const line of lines.slice(1)) {
    const cells = cellsOf(line);
    const body = byName(cells[0] ?? "");
    if (!body) {
      out.problems.push({ where: `"${cells[0]}"`, why: "no body of that name in this system; left out" });
      continue;
    }
    const rows = rowsFor(body);
    const typed: Record<string, string> = {};
    for (const [i, h] of heads.entries()) {
      if (i === 0) continue;
      const id = out.columns[i - 1]?.row;
      const v = cells[i]?.trim();
      if (!id || !v || v === "-" || v === "—") continue;
      const u = unitOf(h);
      typed[id] = u && /^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$/.test(v) ? `${v} ${u}` : v;
    }
    const r = readSheet(rows, typed, body.facts, body.cls, depth);
    for (const e of r.edits) out.edits.push({ address: body.address, field: e.field, value: e.value, before: e.before });
    for (const u of r.understood) out.understood.push({ body: body.label, label: u.label, before: u.before, after: u.after });
    for (const p of r.problems) out.problems.push({ where: `${body.label}, ${p.label}`, why: p.why });
  }
  return out;
}
