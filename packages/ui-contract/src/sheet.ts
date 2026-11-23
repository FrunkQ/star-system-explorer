// The master data sheet's constants (packs/base-reference, MP22), for the interface. One door to a constant's value:
// `constant("au")` in its SI unit, with `constantOf` for its unit and source. Nothing here is typed: the values come
// from ../generated/reference-constants.json, which scripts/build-ui-reference.py writes from the pack.
import doc from "../generated/reference-constants.json" with { type: "json" };

export interface SheetConstant {
  v: number;
  unit: string;
  src: string;
}

const CONSTANTS = doc.constants as Record<string, SheetConstant>;

/** The sheet this copy was made from. */
export const SHEET_PACK: { id: string; version: string } = doc.pack;

/** A constant with its unit and source, or throws for an id the sheet does not hold. */
export function constantOf(id: string): SheetConstant {
  const c = CONSTANTS[id];
  if (!c) throw new Error(`the master data sheet has no constant "${id}"`);
  return c;
}

/** A constant's value in its SI unit. */
export const constant = (id: string): number => constantOf(id).v;

/** Every constant id the sheet holds. */
export const constantIds = (): string[] => Object.keys(CONSTANTS);

const UNITS = doc.units as Record<string, number>;

/** A unit's factor to SI (`sheetUnitFactor("km")` is 1000), or throws for an id the sheet does not hold. */
export function sheetUnitFactor(id: string): number {
  const f = UNITS[id];
  if (f === undefined) throw new Error(`the master data sheet has no unit "${id}"`);
  return f;
}

/**
 * A sheet reference in a document: `{ "sheet": "constants/au" }` or `{ "sheet": "units/day" }`, with an optional
 * `"in": "<unit id>"` to give the value in that unit. The same form packs use (sse-reference refs), so a document
 * never types a copy of a sheet value (MP22). Other keys allowed beside them: `note`.
 */
export interface SheetRef {
  sheet: string;
  in?: string;
  note?: string;
}

const REF_KEYS = new Set(["sheet", "in", "note"]);

const isRef = (v: unknown): v is SheetRef =>
  typeof v === "object" && v !== null && !Array.isArray(v) && typeof (v as SheetRef).sheet === "string" && Object.keys(v).every((k) => REF_KEYS.has(k));

/** The number a sheet reference names. */
export function sheetValue(r: SheetRef): number {
  const [table, id] = r.sheet.split("/", 2) as [string, string | undefined];
  if (!id) throw new Error(`sheet reference "${r.sheet}" must be table/id`);
  const si = table === "constants" ? constant(id) : table === "units" ? sheetUnitFactor(id) : NaN;
  if (Number.isNaN(si)) throw new Error(`sheet reference "${r.sheet}": the interface's copy holds constants and units only`);
  return r.in ? si / sheetUnitFactor(r.in) : si;
}

/** A number, or a sheet reference standing for one. */
export const numberOf = (v: number | SheetRef): number => (typeof v === "number" ? v : sheetValue(v));
