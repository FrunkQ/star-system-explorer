// A surface that churns (SG2). The kernel gives each star's convection (read_activity, O10): the pressure scale
// height at the photosphere from its surface gravity and temperature, granules about ten scale heights wide, their
// speed and their life. This file turns those numbers into cells that are born, dome up and fade over their own life,
// each on its own clock, so the surface boils smoothly instead of the whole pattern jumping at once; and into a limb
// lifted and dropped by the cells under it, which on a giant (a few dozen cells across, each a twentieth of the star)
// makes the edge visibly lumpy, while on the Sun (a scale height of 150 km on a radius of 700,000) it stays round.
import { finishGranules, granuleCount, type Convection, type Granules } from "./star.ts";

export const CHURN_REFERENCES = [
  { text: "Nordlund, A., Stein, R. F. and Asplund, M. (2009). Solar Surface Convection. Living Reviews in Solar Physics 6, 2 (granules about ten pressure scale heights wide, living a few turnover times; new cells rise, old ones fragment and fade).", doi: "10.12942/lrsp-2009-2" },
  { text: "Freytag, B., Steffen, M. and Dorch, B. (2002). Spots on the surface of Betelgeuse: results from new 3D stellar convection models. Astron. Nachr. 323, 213 (a red supergiant's few giant cells raise and lower its visible edge).", doi: "10.1002/1521-3994(200208)323:3/4<213::AID-ASNA213>3.0.CO;2-H" },
  { text: "Chiavassa, A. et al. (2010). Radiative hydrodynamics simulations of red supergiant stars II. A&A 515, A12 (Betelgeuse: a few giant convective cells and an uneven limb).", doi: "10.1051/0004-6361/200913907" },
  { text: "Böhm-Vitense, E. (1958). Über die Wasserstoffkonvektionszone in Sternen verschiedener Effektivtemperaturen und Leuchtkräfte. Z. Astrophys. 46, 108 (mixing-length convection: a parcel's buoyancy pays for its speed, so δT/T grows as the square of its Mach number). Unverified here: paper search unavailable until 1 Dec 2026." },
];

/** How far a cell drifts over its life, as a share of the cell spacing (granules move about their own size, Nordlund et al. 2009). Stand-in. */
export const CELL_DRIFT = 0.3;
/** The limb's lift per standard deviation of the pattern, in pressure scale heights (cells overshoot about one scale height). Stand-in. */
export const LUMP_SCALE_HEIGHTS = 1;

/** A small integer hash (lowbias32) of a star key, a cell and a generation: the same three always give the same number. */
function hash(a: number, b: number, c: number): number {
  let h = (a ^ Math.imul(b, 0x9e3779b1) ^ Math.imul(c, 0x85ebca77)) >>> 0;
  h ^= h >>> 16;
  h = Math.imul(h, 0x7feb352d);
  h ^= h >>> 15;
  h = Math.imul(h, 0x846ca68b);
  h ^= h >>> 16;
  return (h >>> 0) / 4294967296;
}

function keyOf(s: string): number {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) h = Math.imul(h ^ s.charCodeAt(i), 16777619);
  return h >>> 0;
}

const lattices = new Map<string, Float64Array>();
/** The even spread of cell homes on the unit sphere (a Fibonacci lattice), kept per cell count. */
function lattice(n: number): Float64Array {
  const k = String(n);
  const hit = lattices.get(k);
  if (hit) return hit;
  const pts = new Float64Array(n * 3);
  const golden = Math.PI * (3 - Math.sqrt(5));
  for (let i = 0; i < n; i++) {
    const z = 1 - (2 * (i + 0.5)) / n;
    const q = Math.sqrt(Math.max(0, 1 - z * z));
    pts[i * 3] = q * Math.cos(i * golden);
    pts[i * 3 + 1] = z;
    pts[i * 3 + 2] = q * Math.sin(i * golden);
  }
  if (lattices.size > 8) lattices.clear();
  lattices.set(k, pts);
  return pts;
}

/**
 * The star's cells at time `t_s` (star time). Each home on the lattice holds one cell at a time; each cell lives
 * `life_s` and its home's clock is offset by a share of a life drawn from the star, so cells are born and die at
 * different moments. A new cell takes a fresh place near its home and drifts across it as it lives; its strength
 * rises and falls as sin(π · age / life). The same star at the same moment always gives the same cells.
 */
export function churningGranules(address: string, across: number, t_s: number, life_s: number): Granules {
  return churn(keyOf(address), Math.max(12, granuleCount(across)), t_s, life_s);
}

/**
 * The star's larger cells at `t_s` (SC1): the kernel's supergranule scale (read_activity's `supergranule_size_m`,
 * `network_cells` and `supergranule_lifetime_s`), churning as the granules do. On the Sun they are the supergranulation,
 * thousands of them and barely seen in white light; on a red supergiant the same scale comes out wider than the star,
 * so a few cells cover it, as three-dimensional models of Betelgeuse show (Freytag et al. 2002, Chiavassa et al. 2010).
 * At least BIG_CELLS_LEAST, the fewest that tile a sphere.
 */
export function churningBigCells(address: string, network_cells: number, t_s: number, life_s: number): Granules {
  return churn(keyOf(address) ^ 0x6a09e667, Math.max(BIG_CELLS_LEAST, Math.round(network_cells)), t_s, life_s);
}

/** The fewest large cells drawn on a star: a sphere is not tiled by fewer. Stand-in (fudge register, "Large cells on giants"). */
export const BIG_CELLS_LEAST = 4;

/** Cells across the diameter for `n` cells over the whole star (the inverse of granuleCount). */
export const cellsAcross = (n: number) => Math.sqrt((n * (Math.sqrt(3) / 2)) / Math.PI);

/** The ratio of specific heats of the photosphere's gas (monatomic, ionising little): 5/3. */
export const GAMMA_PHOTOSPHERE = 5 / 3;
/** The largest temperature contrast drawn for the large cells, as a share of the surface temperature. Stand-in. */
export const BIG_CELL_DT_MAX = 0.2;

/**
 * The large cells' temperature contrast as a share of the surface temperature, from how fast they flow: in mixing-length
 * convection a parcel's buoyancy pays for its speed, v² ≈ g H δT/T, and g H = c_s²/γ, so δT/T ≈ γ (v/c_s)². The flow
 * speed is the kernel's supergranule size over its life; c_s its photosphere's sound speed. The Sun's supergranules
 * come out at about half a per cent (barely seen in white light); a red supergiant's at several per cent. Capped at
 * BIG_CELL_DT_MAX. Zero without convection.
 */
export function bigCellContrast(c: { supergranule_size_m?: number; supergranule_lifetime_s?: number; sound_speed_m_s?: number } | null | undefined): number {
  const size = Number(c?.supergranule_size_m ?? 0);
  const life = Number(c?.supergranule_lifetime_s ?? 0);
  const cs = Number(c?.sound_speed_m_s ?? 0);
  if (!(size > 0 && life > 0 && cs > 0)) return 0;
  const mach = size / life / cs;
  return Math.min(BIG_CELL_DT_MAX, GAMMA_PHOTOSPHERE * mach * mach);
}

function churn(key: number, n: number, t_s: number, life_s: number): Granules {
  const home = lattice(n);
  const spacing = Math.sqrt((4 * Math.PI) / n) * 1.07;
  const life = Math.max(1, life_s);
  const pts = new Float64Array(n * 3);
  const amp = new Float64Array(n);
  for (let i = 0; i < n; i++) {
    const u = t_s / life + hash(key, i, -1);
    const gen = Math.floor(u);
    const age = u - gen;
    amp[i] = Math.sin(Math.PI * age);
    const hx = home[i * 3]!;
    const hy = home[i * 3 + 1]!;
    const hz = home[i * 3 + 2]!;
    // Two directions along the surface at the home: east (round the pole) and north.
    const q = Math.hypot(hx, hz) || 1;
    const ex = -hz / q;
    const ez = hx / q;
    const nx = -hy * (hx / q);
    const ny = q;
    const nz = -hy * (hz / q);
    // A fresh place for this generation (up to half a spacing from home) and its drift over the life.
    const a = (hash(key, i, gen) - 0.5) * spacing;
    const b = (hash(key ^ 0x5bd1e995, i, gen) - 0.5) * spacing;
    const da = (hash(key ^ 0x27d4eb2f, i, gen) - 0.5) * 2 * CELL_DRIFT * spacing * (age - 0.5);
    const db = (hash(key ^ 0x165667b1, i, gen) - 0.5) * 2 * CELL_DRIFT * spacing * (age - 0.5);
    let x = hx + (a + da) * ex + (b + db) * nx;
    let y = hy + (b + db) * ny;
    let z = hz + (a + da) * ez + (b + db) * nz;
    const r = Math.hypot(x, y, z) || 1;
    x /= r;
    y /= r;
    z /= r;
    pts[i * 3] = x;
    pts[i * 3 + 1] = y;
    pts[i * 3 + 2] = z;
  }
  return finishGranules(n, spacing, pts, amp);
}

/**
 * The cells' share of their full contrast in a picture that stands for `span_s` of star time: a picture spanning N
 * cell lives averages N independent patterns, so the contrast falls as 1/√N (a long exposure of the Sun shows its
 * granulation smeared). A picture shorter than a life keeps it all.
 */
export const granuleBlur = (life_s: number, span_s: number) => (span_s > life_s && life_s > 0 ? Math.sqrt(life_s / span_s) : 1);

/**
 * How far the cells lift the limb, in stellar radii per standard deviation of the pattern: the pressure scale height
 * over the radius (kernel convection), times LUMP_SCALE_HEIGHTS. The Sun's is about 0.0002, a red supergiant's a few
 * hundredths. Zero with no convection.
 */
export function limbLumps(c: Pick<Convection, "pressure_scale_height_m"> | null | undefined, radius_m: number): number {
  const h = Number(c?.pressure_scale_height_m ?? 0);
  return h > 0 && radius_m > 0 ? (LUMP_SCALE_HEIGHTS * h) / radius_m : 0;
}

/** The lumpy limb in words, or "" when it stays round at this size. */
export function narrateLumps(lumps: number, disc_px: number): string {
  if (!(lumps * (disc_px / 2) >= 0.25)) return "";
  return `Its edge is uneven: each convection cell is so large (the surface's scale height is ${(lumps * 100).toFixed(lumps < 0.01 ? 2 : 1)}% of the radius) that it lifts or drops the limb where it sits.`;
}
