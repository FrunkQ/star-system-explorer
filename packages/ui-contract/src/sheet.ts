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
