// The star chooser (ED6 part 2, editing design section 9): a star's mass strip drawn in the real colour of the star
// at each mass today, the spectral letters under it (the kernel's own class) and the walls where the answer changes kind; the life bar for the
// chosen mass with the system's present age marked; and Kinds chips that find the mass which is that kind now. All of
// it is read off the kernel's life line (`describe_star`, K18): a catalogued star's temperature and luminosity are
// authored facts, so nothing here is worked out from the star's own record, and nothing is typed.
import data from "./star-kinds.json" with { type: "json" };
import { constant } from "./sheet.ts";
import { thermalColour } from "./light.ts";
import { secondsToYears } from "./intent.ts";
import type { Json } from "./json.ts";

export type StarPhase = "pre-main-sequence" | "main-sequence" | "hertzsprung-gap" | "giant" | "wolf-rayet" | "brown-dwarf" | "white-dwarf" | "neutron-star" | "black-hole";

/**
 * A star's formal class as the kernel reads it (sse_formation::spectral): "G2 V", "K1 III", "T5", "DX 4.5", "NS", "BH".
 * `family` is the atlas star scheme's base class (star/<family>), `kind` its child (star/<family>/<kind>).
 */
export interface StarClass {
  code: string;
  letter: string;
  subclass?: number;
  luminosity: string;
  family: string;
  kind: string;
  why: string;
  notes?: string[];
}

/** `describe_star`'s answer (sse_formation::describe::Description). */
export interface StarDescription {
  mass_msun: number;
  feh: number;
  age_yr: number;
  now: { phase: StarPhase; label: string; teff_k: number; luminosity_w: number; radius_m: number; mass_kg: number; mass_msun: number; luminosity_lsun: number; radius_rsun: number; class: StarClass };
  phases: { phase: StarPhase; label: string; from_yr: number; to_yr: number | null }[];
  remnant: { phase: StarPhase; label: string; mass_msun: number; from_yr: number | null };
  walls: { deuterium_msun: number; hydrogen_msun: number; white_dwarf_msun: number; black_hole_msun: number; fits_msun: number };
  notes: string[];
  sources: string[];
}

export type Describe = (mass_msun: number, age_yr: number, feh?: number) => Promise<{ ok: true; star: StarDescription } | { ok: false; code: string; message: string }>;

export interface StarKind {
  id: string;
  label: string;
  phase: StarPhase;
  letters?: string[];
  prefer_msun?: number;
}
export const STAR_KINDS: readonly StarKind[] = data.kinds as StarKind[];

/** The spectral letter of a described star (O to M, then L, T, Y), read off the kernel's class; none for a remnant. */
export function letterOf(d: Pick<StarDescription, "now">): string | null {
  const l = d.now.class?.letter ?? "";
  return /^[OBAFGKMLTY]$/.test(l) ? l : null;
}

/** One mass along the strip, as it is at the asked age. */
export interface StarSample {
  mass_msun: number;
  phase: StarPhase;
  label: string;
  teff_k: number;
  letter: string | null;
  /** The colour of its light (a thermal spectrum at its temperature); null for a black hole, which gives none. */
  colour: string | null;
}

function sample(d: StarDescription): StarSample {
  return {
    mass_msun: d.mass_msun,
    phase: d.now.phase,
    label: d.now.label,
    teff_k: d.now.teff_k,
    letter: letterOf(d),
    colour: d.now.phase === "black-hole" || !(d.now.teff_k > 0) ? null : thermalColour(d.now.teff_k),
  };
}

/** The masses the life line describes, log-spaced from the lightest to the heaviest (both ends included). */
export function massSteps(lo: number, hi: number, n: number): number[] {
  const a = Math.log(lo);
  const b = Math.log(hi);
  return Array.from({ length: n }, (_, i) => Math.exp(a + ((b - a) * i) / (n - 1)));
}

/** The life line's mass range and walls (any description carries them). */
export async function lifeLineRange(describe: Describe, age_yr: number, feh?: number): Promise<StarDescription["walls"] | null> {
  const r = await describe(1, age_yr, feh);
  return r.ok ? r.star.walls : null;
}

/** The strip: `n` masses across the life line's range, each as it is at `age_yr`. */
export async function readMassStrip(describe: Describe, age_yr: number, feh: number | undefined, n = 64): Promise<{ samples: StarSample[]; walls: StarDescription["walls"] } | null> {
  const walls = await lifeLineRange(describe, age_yr, feh);
  if (!walls) return null;
  const out: StarSample[] = [];
  // Just inside each end: the ends themselves are the edges of the fits.
  for (const m of massSteps(walls.deuterium_msun * 1.001, walls.fits_msun * 0.999, n)) {
    const r = await describe(m, age_yr, feh);
    if (r.ok) out.push(sample(r.star));
  }
  return { samples: out, walls };
}

// The order the phases come in along the mass line at one age: heavier stars are further on. A star still forming is
// left out of the search (the lightest stars form slowest, so it is not in order).
const ORDER: StarPhase[] = ["brown-dwarf", "main-sequence", "hertzsprung-gap", "giant", "wolf-rayet", "white-dwarf", "neutron-star", "black-hole"];
const rank = (p: StarPhase) => ORDER.indexOf(p);

const fits = (k: StarKind, s: StarSample) => s.phase === k.phase && (!k.letters || (s.letter !== null && k.letters.includes(s.letter)));

/**
 * The mass that is this kind at `age_yr`, or null when no star is that kind at that age (a blue giant in a system
 * billions of years old). Among the strip's masses that qualify it takes the one nearest the kind's preferred mass,
 * or the middle one; when none does (a short phase falls between two masses) it bisects for the window on the life
 * line and takes its middle.
 */
export async function kindMass(describe: Describe, kind: StarKind, samples: readonly StarSample[], age_yr: number, feh?: number): Promise<number | null> {
  const hits = samples.filter((s) => fits(kind, s));
  if (hits.length) {
    if (kind.prefer_msun === undefined) return hits[Math.floor(hits.length / 2)]!.mass_msun;
    const p = Math.log(kind.prefer_msun);
    return hits.reduce((a, b) => (Math.abs(Math.log(b.mass_msun) - p) < Math.abs(Math.log(a.mass_msun) - p) ? b : a)).mass_msun;
  }
  if (kind.letters) return null;
  const want = rank(kind.phase);
  for (let i = 0; i + 1 < samples.length; i++) {
    const [a, b] = [samples[i]!, samples[i + 1]!];
    if (rank(a.phase) < 0 || rank(b.phase) < 0 || !(rank(a.phase) < want && want < rank(b.phase))) continue;
    const phaseAt = async (m: number) => {
      const r = await describe(m, age_yr, feh);
      return r.ok ? rank(r.star.now.phase) : -1;
    };
    // Find one mass inside the window, then each edge, all on the log of the mass.
    let [lo, hi] = [Math.log(a.mass_msun), Math.log(b.mass_msun)];
    let inside: number | null = null;
    for (let k = 0; k < 40 && inside === null; k++) {
      const mid = (lo + hi) / 2;
      const r = await phaseAt(Math.exp(mid));
      if (r === want) inside = mid;
      else if (r < 0) return null;
      else if (r < want) lo = mid;
      else hi = mid;
    }
    if (inside === null) return null;
    const edge = async (from: number, to: number) => {
      // `from` is inside the window, `to` outside it.
      for (let k = 0; k < 24; k++) {
        const mid = (from + to) / 2;
        if ((await phaseAt(Math.exp(mid))) === want) from = mid;
        else to = mid;
      }
      return from;
    };
    return Math.exp(((await edge(inside, lo)) + (await edge(inside, hi))) / 2);
  }
  return null;
}

/**
 * What choosing a mass writes on the star, as one change: its mass now, radius, luminosity and surface temperature
 * from the life line at the system's present age, and the mass it was born with (`star.mass_msun`, the field a drawn
 * star carries too), so the strip reopens at the same place after the star has shed mass or left a remnant.
 */
export function starEdits(d: StarDescription, facts: Readonly<Record<string, Json>>): { field: string; value: number; before: Json }[] {
  const was = (f: string) => (facts[f] ?? null) as Json;
  return [
    { field: "body.mass", value: d.now.mass_kg, before: was("body.mass") },
    { field: "star.mass_msun", value: d.mass_msun, before: was("star.mass_msun") },
    { field: "body.radius", value: d.now.radius_m, before: was("body.radius") },
    { field: "star.luminosity", value: d.now.luminosity_w, before: was("star.luminosity") },
    { field: "star.effective_temperature", value: d.now.teff_k, before: was("star.effective_temperature") },
  ];
}

/** The mass the star was born with, in solar masses: `star.mass_msun` when it carries one, else its mass now. */
export function birthMass(facts: Readonly<Record<string, Json>>): number | null {
  const m = facts["star.mass_msun"];
  if (typeof m === "number" && m > 0) return m;
  const now = facts["body.mass"];
  return typeof now === "number" && now > 0 ? now / constant("M_sun") : null;
}

/** The system's present age in years (`system.age` is held in seconds), or null when the record has none. */
export function systemAgeYr(facts: Readonly<Record<string, Json>>): number | null {
  const a = facts["system.age"];
  return typeof a === "number" && a > 0 ? secondsToYears(a) : null;
}

/** Where a mass sits along the strip, 0 to 1, on the log of the mass. */
export function massPos(m: number, lo: number, hi: number): number {
  return Math.min(1, Math.max(0, Math.log(m / lo) / Math.log(hi / lo)));
}
/** The mass at a place along the strip, 0 to 1. */
export function massAt(pos: number, lo: number, hi: number): number {
  return lo * Math.exp(Math.min(1, Math.max(0, pos)) * Math.log(hi / lo));
}
