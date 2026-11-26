// The star class gallery (SC1, Alex 26 Nov: "a gallery for them too... so we can see if they look right... actual size
// to each other or scaled to same size... wanna see the cells on a overinflated stars"). Every star class as a tile
// drawn live by the True renderer: each class's representative star read off the life line (`describe_star`, K18) and
// its surface from the kernel's activity (`read_activity`, O10: spin, spots, flares, and the convection cells sized
// from the photosphere's pressure scale height). Nothing here is painted: a giant's few huge cells are the kernel's
// numbers for its low gravity, drawn by the same face door the system's True uses (star-now.ts `liveFace`).
//
// The class list is the base atlas's "star" scheme (thread "Classify every star and remnant"): families with a child
// per spectral letter, each class's first example carrying its sample as `star.sample_mass_msun`, `star.sample_age_yr`
// and `star.sample_feh`. Until that scheme is on the stack, STAND_IN lists a star per family, aged to the middle of
// the phase that makes it that family.
import { constant, roundTo, sayNumber, type AtlasClass, type AtlasScheme, type Json, type StarDescription, type StarPhase } from "../../../ui-contract/src/index.ts";
import { bigCellContrast, type LiveStar } from "../../../renderer/src/index.ts";

/** The families the star scheme groups by (agreed with "Classify every star and remnant", 26 Nov). */
export type StarFamily = "pre-main-sequence" | "main-sequence" | "subgiant" | "giant" | "bright-giant" | "supergiant" | "brown-dwarf" | "white-dwarf" | "neutron-star" | "black-hole";

/** A star on the life line: its starting mass, its age from birth and its metals. */
export interface StarSampleAsk {
  mass_msun: number;
  age_yr: number;
  feh?: number;
}

/** One tile: a class and the star that shows it, or why there is none. */
export interface StarTile {
  id: string;
  name: string;
  family: StarFamily | string;
  summary?: string;
  /** Where the life line gives a star of this class; null with `why` where it cannot. */
  sample: StarSampleAsk | { mass_msun: number; phase: StarPhase } | null;
  why?: string;
}

/** Remnants whose picture is not drawn yet: they need light bending and their own discs (track XO, "Draw the first black hole picture"). */
export const NOT_DRAWN: Partial<Record<StarPhase, string>> = {
  "neutron-star": "a neutron star is twenty kilometres across and drawn by its own light-bending renderer, which is still being built (exotic objects)",
  "black-hole": "a black hole gives no light of its own; its picture is the bent light of what lies behind it and of its disc, which the exotic objects renderer is still building",
};

/** What the drawing leaves out for a family that is drawn, said on its card. */
export const DRAWN_WITHOUT: Partial<Record<StarPhase, string>> = {
  "brown-dwarf": "Drawn as a glowing ball at its temperature: a brown dwarf this cool has cloud bands and weather of its own, which are not drawn yet.",
};

/**
 * A star per family, until the base atlas's star scheme lands: a mass and the phase it should be in, aged by
 * `sampleAge` to the middle of that phase on the life line, so each really is what its tile says.
 */
export const STAND_IN: readonly StarTile[] = [
  { id: "stand-in/pre-main-sequence", name: "Young star still contracting", family: "pre-main-sequence", sample: { mass_msun: 1, phase: "pre-main-sequence" } },
  { id: "stand-in/main-sequence/O", name: "O main-sequence star", family: "main-sequence", sample: { mass_msun: 40, phase: "main-sequence" } },
  { id: "stand-in/main-sequence/B", name: "B main-sequence star", family: "main-sequence", sample: { mass_msun: 6, phase: "main-sequence" } },
  { id: "stand-in/main-sequence/A", name: "A main-sequence star", family: "main-sequence", sample: { mass_msun: 2, phase: "main-sequence" } },
  { id: "stand-in/main-sequence/F", name: "F main-sequence star", family: "main-sequence", sample: { mass_msun: 1.3, phase: "main-sequence" } },
  { id: "stand-in/main-sequence/G", name: "G main-sequence star", family: "main-sequence", sample: { mass_msun: 1, phase: "main-sequence" } },
  { id: "stand-in/main-sequence/K", name: "K main-sequence star", family: "main-sequence", sample: { mass_msun: 0.75, phase: "main-sequence" } },
  { id: "stand-in/main-sequence/M", name: "M main-sequence star (red dwarf)", family: "main-sequence", sample: { mass_msun: 0.3, phase: "main-sequence" } },
  { id: "stand-in/subgiant", name: "Star leaving the main sequence", family: "subgiant", sample: { mass_msun: 1.5, phase: "hertzsprung-gap" } },
  { id: "stand-in/giant/K", name: "Red giant (Sun-like, old)", family: "giant", sample: { mass_msun: 1, phase: "giant" } },
  { id: "stand-in/giant/G", name: "Giant (intermediate mass)", family: "giant", sample: { mass_msun: 3, phase: "giant" } },
  { id: "stand-in/giant/K-late", name: "Red giant near its tip", family: "giant", sample: { mass_msun: 1, age_yr: 1.23e10 } },
  { id: "stand-in/supergiant/M", name: "Red supergiant", family: "supergiant", sample: { mass_msun: 20, age_yr: 9e6 } },
  { id: "stand-in/brown-dwarf", name: "Brown dwarf", family: "brown-dwarf", sample: { mass_msun: 0.05, age_yr: 1e9 } },
  { id: "stand-in/white-dwarf", name: "White dwarf", family: "white-dwarf", sample: { mass_msun: 3, phase: "white-dwarf" } },
  { id: "stand-in/neutron-star", name: "Neutron star", family: "neutron-star", sample: { mass_msun: 15, phase: "neutron-star" } },
  { id: "stand-in/black-hole", name: "Black hole", family: "black-hole", sample: { mass_msun: 40, phase: "black-hole" } },
];

const num = (v: Json | undefined): number | null => (typeof v === "number" && Number.isFinite(v) ? v : null);

/** The atlas's star scheme as tiles: each class's sample from its first example that is in the class. */
export function tilesFromScheme(scheme: Pick<AtlasScheme, "classes">, nameOf: (c: AtlasClass) => string): StarTile[] {
  // A family with kinds under it is shown by its kinds; one with none (neutron stars) is its own tile.
  const parents = new Set(scheme.classes.map((c) => c.parent).filter((p): p is string => !!p));
  return scheme.classes
    .filter((c) => c.kind !== "modifier" && !parents.has(c.id))
    .map((c) => {
      // The sample's values ride on the example (the atlas reader keeps them, classes.ts).
      const ex = c.examples.find((e) => e.in);
      const v = (ex as { values?: Record<string, Json> } | undefined)?.values ?? {};
      const mass = num(v["star.sample_mass_msun"]);
      const age = num(v["star.sample_age_yr"]);
      const feh = num(v["star.sample_feh"]);
      const family = (c.parent ? c.parent.split("/").pop() : c.id.split("/").pop()) ?? c.id;
      const summary = c.summary || undefined;
      return {
        id: c.id,
        name: nameOf(c),
        family,
        ...(summary ? { summary } : {}),
        sample: mass !== null && age !== null ? { mass_msun: mass, age_yr: age, ...(feh !== null ? { feh } : {}) } : null,
        ...(mass === null || age === null ? { why: summary ?? "the life line makes no star of this class" } : {}),
      };
    });
}

type Describe = (mass_msun: number, age_yr: number, feh?: number) => Promise<{ ok: true; star: StarDescription } | { ok: false; code: string; message: string }>;

const SUN_AGE_YR = 4.6e9;

/** A tile's age: given, or the middle of the phase it should show (in log time for a phase that lasts, a remnant's first billion years). */
export async function sampleAge(describe: Describe, s: NonNullable<StarTile["sample"]>): Promise<StarSampleAsk | string> {
  if ("age_yr" in s) return s;
  const r = await describe(s.mass_msun, 1e6);
  if (!r.ok) return r.message;
  const span = r.star.phases.find((p) => p.phase === s.phase);
  // The middle of the phase, but no older than the Sun where the phase allows (a red dwarf's main sequence outlasts the
  // universe): a star of this kind as one might meet it now.
  if (span && span.to_yr !== null) {
    const mid = (span.from_yr + span.to_yr) / 2;
    return { mass_msun: s.mass_msun, age_yr: span.from_yr < SUN_AGE_YR && SUN_AGE_YR < mid ? SUN_AGE_YR : mid };
  }
  if (span) return { mass_msun: s.mass_msun, age_yr: span.from_yr + Math.max(1e8, span.from_yr * 0.05) };
  if (r.star.remnant.phase === s.phase && r.star.remnant.from_yr !== null) return { mass_msun: s.mass_msun, age_yr: r.star.remnant.from_yr + 1e8 };
  return `the life line gives a ${s.mass_msun} solar-mass star no ${s.phase.replace(/-/g, " ")} phase`;
}

/** What the gallery knows of one tile's star once read. */
export interface GalleryStar {
  tile: StarTile;
  ask: StarSampleAsk;
  desc: StarDescription;
  /** The living star to draw; null where it is not drawn yet (`why`). */
  live: LiveStar | null;
  why?: string;
  /** Its own clock, star seconds per screen second, and what that clock is set by. */
  clock: number;
  clockWhy: string;
  /** What its picture leaves out, if anything. */
  without?: string;
  /** The kernel's convection, when it churns: cells across the diameter, each one's size and life. */
  cells: { across: number; size_m: number; life_s: number; scale_height_m: number; big: { count: number; size_m: number; life_s: number; dt: number } | null } | null;
}

interface Reader {
  describeStar?: Describe;
  readActivity(ask: { stars: Json[]; seed?: string; at_s?: number; window?: { from_s: number; to_s: number; min_energy_j?: number } }): Promise<{ ok: true; stars: Json[] } | { ok: false; code: string; message: string }>;
}

const YEAR = constant("yr_julian");
const DAY = constant("day");
const HOUR = DAY / 24;
/** A cell's life on the screen, seconds: long enough to see one rise and fade, short enough that the page churns. */
export const CELL_LIFE_ON_SCREEN_S = 4;
/** One turn on the screen, seconds, for a star with no cells to watch. */
export const TURN_ON_SCREEN_S = 30;

/**
 * The gallery's stars stand outside any campaign and its clock (CLK1), so each runs on a clock of its own, chosen so
 * the tile shows what the star does and said on its card: an explicit, labelled choice. This is the rate for a star
 * with neither cells nor a known spin: six hours a second.
 */
const STILL_STAR_CLOCK = DAY / 4;

/**
 * Read one tile's star: the life line's answer at its age, then its activity (spin, spots, flares, cells) from the
 * star's own mass, radius, light and temperature. A remnant not drawn yet comes back with its description and why.
 */
export async function readGalleryStar(session: Reader, tile: StarTile): Promise<GalleryStar | string> {
  if (!session.describeStar) return "this seat cannot read the life line";
  if (!tile.sample) return tile.why ?? "the life line makes no star of this class";
  const ask = await sampleAge(session.describeStar.bind(session), tile.sample);
  if (typeof ask === "string") return ask;
  const r = await session.describeStar(ask.mass_msun, ask.age_yr, ask.feh);
  if (!r.ok) return r.message;
  const desc = r.star;
  const n = desc.now;
  const base = { tile, ask, desc, clock: STILL_STAR_CLOCK, clockWhy: "six hours a second", cells: null };
  const notYet = NOT_DRAWN[n.phase];
  if (notYet) return { ...base, live: null, why: notYet };
  if (!(n.teff_k > 0 && n.radius_m > 0 && n.mass_kg > 0)) return { ...base, live: null, why: "the life line gives it no surface to draw" };
  // An address of its own, outside any campaign (the activity's draws are keyed by it): the class id, made safe.
  const address = `u:star-gallery/star:${tile.id.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "")}`;
  const age_s = ask.age_yr * YEAR;
  const star = { address, mass_kg: n.mass_kg, radius_m: n.radius_m, luminosity_w: n.luminosity_w, teff_k: n.teff_k, age_s };
  const a = await session.readActivity({ stars: [star], at_s: age_s, window: { from_s: age_s - 2 * DAY, to_s: age_s + YEAR, min_energy_j: 1e24 } });
  if (!a.ok) return { ...base, live: null, why: a.message };
  const one = a.stars[0] as unknown as { activity: Record<string, Json>; moment: Record<string, Json>; flares?: { flares?: LiveStar["flares"] } };
  const live: LiveStar = { address, teff: n.teff_k, mass: n.mass_kg, radius: n.radius_m, lum: n.luminosity_w, age_s, activity: one.activity, moment: one.moment, flares: one.flares?.flares ?? [] };
  const c = one.activity.convection as Record<string, Json> | null | undefined;
  const across = num(c?.granules_across);
  const life = num(c?.granule_lifetime_s);
  const bigCount = num(c?.network_cells);
  const bigDt = bigCellContrast(c as Parameters<typeof bigCellContrast>[0]);
  const big = bigCount && bigDt > 0 ? { count: bigCount, size_m: num(c?.supergranule_size_m) ?? 0, life_s: num(c?.supergranule_lifetime_s) ?? 0, dt: bigDt } : null;
  const cells = c && across && life ? { across, size_m: num(c.granule_size_m) ?? 0, life_s: life, scale_height_m: num(c.pressure_scale_height_m) ?? 0, big } : null;
  const spin = num(one.activity.rotation_period_s);
  // Each star on its own clock, so every tile shows what it does: a cell rises and fades in a few seconds, or, with no
  // cells, it turns in half a minute. The clock is said on the card.
  const clock = cells ? cells.life_s / CELL_LIFE_ON_SCREEN_S : spin && spin > 0 ? spin / TURN_ON_SCREEN_S : STILL_STAR_CLOCK;
  const clockWhy = cells ? `one cell's life (${span(cells.life_s)}) every ${CELL_LIFE_ON_SCREEN_S} seconds` : spin && spin > 0 ? `one turn (${span(spin)}) every ${TURN_ON_SCREEN_S} seconds` : "six hours a second";
  const without = DRAWN_WITHOUT[n.phase];
  return { ...base, live, clock, clockWhy, cells, ...(without ? { without } : {}) };
}

/** A span of time in words. */
export function span(s: number): string {
  if (s < HOUR) return `${Math.round((s / HOUR) * 60)} minutes`;
  if (s < 2 * DAY) return `${Math.round(s / HOUR)} hours`;
  if (s < 2 * YEAR) return `${Math.round(s / DAY)} days`;
  return `${sayNumber(s / YEAR, 2)} years`;
}

/** Sizes on a page: "same" fills every tile alike; "actual" scales every star by the largest on the page. */
export type SizeMode = "actual" | "same";

/** The smallest disc drawn in "actual", pixels of radius: smaller ones are a point, and the tile says how much smaller. */
export const SMALLEST_R_PX = 1;

/** Each star's drawn radius in pixels, and its size against the largest on the page. */
export function radiiOnPage(radii_m: readonly number[], full_px: number, mode: SizeMode): { r_px: number; share: number; point: boolean }[] {
  const most = Math.max(0, ...radii_m.filter((r) => r > 0));
  return radii_m.map((r) => {
    const share = most > 0 && r > 0 ? r / most : 0;
    if (mode === "same") return { r_px: full_px, share, point: false };
    const px = full_px * share;
    return { r_px: Math.max(SMALLEST_R_PX, px), share, point: px < SMALLEST_R_PX };
  });
}

/** How much smaller than the largest, in words ("1/2,300 of the largest"). */
export function shareWords(share: number): string {
  if (!(share > 0)) return "";
  if (share > 0.995) return "the largest here";
  if (share >= 0.1) return `${Math.round(share * 100)}% of the largest`;
  const n = 1 / share;
  return `1/${sayNumber(n >= 1000 ? roundTo(n, 2) : Math.round(n), 6)} of the largest`;
}
