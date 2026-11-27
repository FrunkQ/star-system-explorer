// The designer's state and the kernel ask it makes (RW16a). Pure: no physics here, only which pins are set. Each
// "Let it decide" leaves a pin out, so the kernel's own field picks it, as it would for a body found in a belt.
import type { SmallDesign } from "../../../ui-contract/src/index.ts";

export const KINDS = [
  { id: "asteroid", words: "Asteroid", hint: "One body of rock, metal or carbon" },
  { id: "comet-nucleus", words: "Comet", hint: "An icy nucleus that wakes near its star" },
  { id: "contact-binary", words: "Contact binary", hint: "Two lobes resting against each other" },
  { id: "binary-pair", words: "Pair", hint: "A body with a small moon of its own" },
] as const;
export type Kind = (typeof KINDS)[number]["id"];

/** Starting points for what it is made of: mass shares of metal, rock, carbon and ice. */
export const MAKEUPS = [
  { id: "stony", words: "Stony", shares: [0.08, 0.87, 0.05, 0] },
  { id: "carbon", words: "Carbon-rich", shares: [0.03, 0.62, 0.3, 0.05] },
  { id: "iron", words: "Iron", shares: [0.85, 0.15, 0, 0] },
  { id: "icy", words: "Icy", shares: [0, 0.3, 0.15, 0.55] },
] as const;

export interface Design {
  kind: Kind;
  diameter_km: number;
  /** Null: made of what the system has there (or the kind's own makeup where it has no record). */
  makeup: [number, number, number, number] | null;
  /** Null: the field decides. */
  structure: "rubble-pile" | "monolith" | "relaxed" | null;
  spin_hours: number | null;
  battered: number | null;
  /** A pair's or contact binary's second body over the first, by radius. */
  size_ratio: number | null;
  /** Distance from the star, AU (a comet's closest approach). */
  r_au: number;
  /** Which of the lumps this design could be: a new number is another body made the same way. */
  variant: number;
}

export const DEFAULT_DESIGN: Design = { kind: "asteroid", diameter_km: 2, makeup: null, structure: null, spin_hours: null, battered: null, size_ratio: null, r_au: 2.7, variant: 1 };

/** A comet's orbit: eccentric, its closest approach at `r_au`. */
const COMET_E = 0.7;

/** Where the design lives while it is looked at: a small body of the system in view, one per variant. */
export function designAddress(system: string, d: Design): string {
  return `${system}/small:design-${d.variant}`;
}

/** The kernel ask for a design (`make_small_body` with the designer's pins). */
export function designAsk(system: string, d: Design): SmallDesign & { system: string; address: string } {
  const comet = d.kind === "comet-nucleus";
  return {
    system,
    address: designAddress(system, d),
    kind: d.kind,
    diameter_km: d.diameter_km,
    a_au: comet ? d.r_au / (1 - COMET_E) : d.r_au,
    ...(comet ? { e: COMET_E } : {}),
    ...(d.kind === "asteroid" || d.kind === "comet-nucleus" ? { pair: "single" as const } : {}),
    ...(d.makeup ? { makeup: d.makeup } : {}),
    ...(d.structure ? { structure: d.structure } : {}),
    ...(d.spin_hours ? { spin_hours: d.spin_hours } : {}),
    ...(d.battered !== null ? { battered: d.battered } : {}),
    ...(d.size_ratio !== null && (d.kind === "contact-binary" || d.kind === "binary-pair") ? { size_ratio: d.size_ratio } : {}),
  };
}

/** A log slider's position (0..1) for a value between lo and hi, and back. */
export const logPos = (v: number, lo: number, hi: number): number => Math.log(v / lo) / Math.log(hi / lo);
export const logVal = (p: number, lo: number, hi: number): number => lo * Math.pow(hi / lo, p);
