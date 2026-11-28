// The `light-path` stage (XO4; exotic objects design 2.1, 3.1 to 3.4, 7): for a pixel near a compact mass, where the
// light it shows really came from, and what it passed on the way. It runs before the pixel looks anything up.
//
// Geometry and lookups only (edits-reach design section 11, ER11a): the physics comes from the kernel.
//
// - the bend table (`sse_gravity::bend`): its layout, `row_of_b`, `u_at` and each row's turn, read from the table the
//   kernel hands over (`table: true`, keyed by its hash). Nothing here traces a ray of its own;
// - where a ray first lands on the disc (`sse_light::disc_image::landing`): the crossing angles φ₀ = π/2 +
//   atan(cos α tan i) and φ₀ + π, the first inside the disc's edges wins; beyond the table a straight ray;
// - the shift and the ring's temperature there: read off the disc table the kernel bakes (`bake`, table
//   `sse.light.disc`, format `sse.table/1`) at the landing radius and the ray's angular momentum over it,
//   b sin i sin α / r. The kernel fills it from the block in force `sse.light.disc_shift` (Luminet 1979) and
//   `sse_light::disc_temperature`, so an edit to the block changes the picture;
// - light seen at a shift g from a surface at T is a blackbody at gT, so each band's recipe colours it.
//
// Rays that miss the disc and escape are turned by the table's turn (the strong bender) inside 64 gravitational
// radii and by 4 r_g / b (block `sse.gravity.weak_bend`) beyond, and look the sky up in the direction they came from.
// The GPU pass (gpu-light-path.ts) does the same reads per pixel; this file is the reference it is tested against,
// and the CPU trace that sets the exposure and each band's brightest.
//
// Units: lengths in gravitational radii (r_g = GM / c²) unless named otherwise; angles in radians.
import { BANDS, bandRadiance, channelOf, constant, linearSrgb, sampleTable, xyz, type LightDescription, type Table } from "../../ui-contract/src/index.ts";

/** The bend table's layout (sse_gravity::bend). The kernel's `table` reference carries the same numbers. */
export const ROWS = 256;
export const CAPTURED_ROWS = 128;
export const COLS = 256;
/** Per row: b, captured (0 or 1), end angle, turn, then the COLS cells of u = r_g / r. */
export const ROW_LENGTH = COLS + 4;
export const B_MAX = 64;
export const PHI_MAX = 3 * Math.PI;
export const U_HORIZON = 0.5;
/** √27: rays inside it are swallowed, and from far away it is the shadow's edge. */
export const B_CRIT = 3 * Math.sqrt(3);
/** Image orders traced (sse_light::disc_image::ORDERS): the direct image with the arch, and the first thin ring. */
export const ORDERS = 2;

/** The bend table as the kernel hands it over (`read_light_paths` with `table: true`). */
export interface BendTable {
  hash: string;
  rows: number;
  cols: number;
  row_length: number;
  flat: ArrayLike<number>;
}

/** The parts of one `read_light_paths` answer the picture reads (sse_light::paths::LightPaths). */
export interface LightPathsRead {
  address: string;
  name?: string;
  about?: string;
  bender: "strong" | "weak";
  gravitational_radius_m: number;
  horizon_m: number;
  photon_orbit_m: number;
  last_stable_orbit_m: number;
  shadow_angle_rad: number;
  spin: { a: number; axis: string };
  table: { hash: string; rows: number; cols: number; b_crit_rg: number; b_max_rg: number; phi_max_rad: number };
  feed?: {
    rate_kg_s: number;
    eddington_share: number;
    flows: { kind: "thin" | "hot" | "thick"; strength: number }[];
    disc: { inner_m: number; outer_m: number; inner_set_by: string };
    luminosity_w: number;
    why?: string[];
    stand_ins?: string[];
  } | null;
  image?: {
    tilt_rad: number;
    side_ratio: number;
    brightest: Spot;
    approaching: Spot;
    receding: Spot;
    exposure: { times_sun_surface: number | null; note: string };
    stand_ins?: string[];
    why?: string[];
  } | null;
  light?: { peak_band: string; bands: { band: string; power_w: number }[] } | null;
  viewer: { tilt_to_disc_rad: number; distance_m: number };
  marks?: unknown[];
  why?: string[];
}

export interface Spot {
  order: number;
  impact_rg: number;
  image_angle_rad: number;
  r_rg: number;
  shift: number;
  t_emitted_k: number;
  t_seen_k: number;
  intensity_w_m2_sr: number;
}

/** Fractional row of an impact parameter, kept on its own side of √27 (sse_gravity::bend::row_of_b). */
export function rowOfB(b: number): number | null {
  if (!(b >= 0 && b <= B_MAX)) return null;
  if (b < B_CRIT) {
    const t = 1 - Math.sqrt(1 - b / B_CRIT);
    return Math.min(t * CAPTURED_ROWS, CAPTURED_ROWS - 1);
  }
  const s = Math.sqrt((b - B_CRIT) / (B_MAX - B_CRIT));
  return Math.max(s * (ROWS - CAPTURED_ROWS) - 1 + CAPTURED_ROWS, CAPTURED_ROWS);
}

/** The two rows an impact parameter reads and the weight of the second (never across √27). */
function rowPair(at: number): [number, number, number] {
  const i0 = Math.floor(at);
  const i1 = i0 + 1 < ROWS && i0 + 1 < CAPTURED_ROWS === i0 < CAPTURED_ROWS ? i0 + 1 : i0;
  return [i0, i1, at - i0];
}

/** u = r_g / r on the ray of impact parameter `b` after sweeping `phi` (sse_gravity::BendTable::u_at). */
export function uAt(t: BendTable, b: number, phi: number): number | null {
  if (!(phi >= 0 && phi <= PHI_MAX)) return null;
  const at = rowOfB(b);
  if (at === null) return null;
  const [i0, i1, fr] = rowPair(at);
  const ck = (phi / PHI_MAX) * (COLS - 1);
  const k0 = Math.min(Math.floor(ck), COLS - 2);
  const fc = ck - k0;
  const cell = (i: number) => {
    const o = i * ROW_LENGTH + 4;
    const a = t.flat[o + k0]!;
    return a + (t.flat[o + k0 + 1]! - a) * fc;
  };
  const c0 = cell(i0);
  return c0 + (cell(i1) - c0) * fr;
}

/**
 * How far an escaping ray of impact parameter `b` is turned: the table's turn inside 64 r_g (between rows, as the
 * cells are), 4 / b beyond (the weak bender). Null for a ray that is swallowed.
 */
export function turnOf(t: BendTable, b: number): number | null {
  if (b < B_CRIT) return null;
  if (b > B_MAX) return 4 / b;
  const [i0, i1, fr] = rowPair(rowOfB(b)!);
  const a = t.flat[i0 * ROW_LENGTH + 3]!;
  return a + (t.flat[i1 * ROW_LENGTH + 3]! - a) * fr;
}

/**
 * Where a ray of impact parameter `b` at picture angle `alpha` (from the disc's axis as seen, towards the viewer's
 * left) first lands on a disc from `rIn` to `rOut`, seen `tilt` from its axis (sse_light::disc_image::landing).
 */
export function landing(t: BendTable, b: number, alpha: number, tilt: number, rIn: number, rOut: number): { order: number; r: number } | null {
  const phi0 = Math.PI / 2 + Math.atan(Math.cos(alpha) * Math.tan(tilt));
  for (let n = 0; n < ORDERS; n++) {
    const phi = phi0 + n * Math.PI;
    let u: number;
    if (b <= B_MAX) {
      const v = uAt(t, b, phi);
      if (v === null) return null;
      u = v;
    } else if (n === 0) u = Math.sin(phi) / b;
    else return null;
    if (u <= 0 || u >= U_HORIZON * 0.999) return null;
    const r = 1 / u;
    if (r >= rIn && r <= rOut) return { order: n, r };
  }
  return null;
}

/** The disc the picture draws, in gravitational radii: its edges and the table the kernel baked for it. */
export interface Disc {
  rIn: number;
  rOut: number;
  /** The kernel's `sse.light.disc` table (`bake`): 1 + z (the inverse of the shift) and the ring's T⁴ against r and b sin i sin α / r. */
  table: Table;
}

/** What to ask the kernel to bake for a `read_light_paths` answer's disc (the hole's size and its feed), or null for
 * a bare body. The kernel takes the disc's edges and the rate its thin flow carries from the feed itself. */
export function discSubject(p: LightPathsRead): { gravitational_radius_m: number; feed: NonNullable<LightPathsRead["feed"]> } | null {
  if (!p.feed) return null;
  return { gravitational_radius_m: p.gravitational_radius_m, feed: p.feed };
}

/** The disc a baked `sse.light.disc` table describes (its first axis runs from the inner edge to the outer). */
export function discOf(table: Table | null): Disc | null {
  if (!table || table.id !== "sse.light.disc") return null;
  const r = table.axes[0]!;
  return { rIn: r.from, rOut: r.to, table };
}

/** The shift and the ring's own temperature where a ray landed: one read of the disc table. */
export function discAt(disc: Disc, r: number, b: number, tilt: number, alpha: number): { g: number; t_emitted_k: number } {
  const v = sampleTable(disc.table, [r, (b * Math.sin(tilt) * Math.sin(alpha)) / r]);
  const t4 = v[channelOf(disc.table, "t4")]!;
  const z = v[channelOf(disc.table, "redshift")]!;
  return { g: z > 0 ? 1 / z : 0, t_emitted_k: t4 > 0 ? Math.sqrt(Math.sqrt(t4)) : 0 };
}

/** What one ray shows. */
export type Ray =
  | { kind: "disc"; order: number; r: number; g: number; t_emitted_k: number; t_seen_k: number }
  | { kind: "sky"; turn: number }
  | { kind: "shadow" };

/** The light-path stage for one ray: the disc it lands on, the sky it came from, or the shadow. */
export function traceRay(t: BendTable, disc: Disc | null, b: number, alpha: number, tilt: number): Ray {
  if (disc) {
    const hit = landing(t, b, alpha, tilt, disc.rIn, disc.rOut);
    if (hit) {
      const { g, t_emitted_k: te } = discAt(disc, hit.r, b, tilt, alpha);
      if (te > 0 && g > 0) return { kind: "disc", order: hit.order, r: hit.r, g, t_emitted_k: te, t_seen_k: g * te };
    }
  }
  const turn = turnOf(t, b);
  return turn === null ? { kind: "shadow" } : { kind: "sky", turn };
}

// ── Bands ────────────────────────────────────────────────────────────────────────────────────────────────────────

/** The bands the picture is seen in (the view's Band switch) and the sheet's light bands each one sums. */
export const PICTURE_BANDS = ["visible", "ultraviolet", "xray", "infrared", "radio"] as const;
export type PictureBand = (typeof PICTURE_BANDS)[number];
/** Infrared is the sheet's near, mid and far infrared; Radio its millimetre and radio bands (the 1.3 mm of the
 * Event Horizon Telescope sits in the millimetre band). */
export const SHEET_BANDS_OF: Record<PictureBand, readonly string[]> = { visible: ["visible"], ultraviolet: ["uv"], xray: ["xray"], infrared: ["near-ir", "mid-ir", "far-ir"], radio: ["mm", "radio"] };

/** Seen temperatures the band table spans: 10^LOG_T0 to 10^LOG_T1 K, RAMP steps. */
export const LOG_T0 = 1;
export const LOG_T1 = 10;
export const RAMP = 1024;
/** log10 of "nothing" in the band table. */
export const LOG_NONE = -60;

const thermal = (t_k: number): LightDescription => ({ address: "", continuum: [{ kind: "thermal", t_k, weight: 1 }], lines: [], stubbed: false, why: "" });

/** Mean radiance of a blackbody at `t_k` across a picture band, W m⁻³ sr⁻¹ (the sheet's band edges, Planck). */
export function bandMean(band: PictureBand, t_k: number): number {
  const parts = SHEET_BANDS_OF[band].map((id) => BANDS.find((b) => b.id === id)!);
  let sum = 0;
  let width = 0;
  for (const p of parts) {
    sum += bandRadiance(thermal(t_k), p.from_m, p.to_m) * (p.to_m - p.from_m);
    width += p.to_m - p.from_m;
  }
  return sum / width;
}

/**
 * The band table the GPU pass reads, as log10 values over seen temperature (RAMP steps from 10^LOG_T0 to 10^LOG_T1
 * K), two rows of RGBA: row 0 the visible light as linear sRGB (CIE 1931, through xyz) and the visible band's mean
 * radiance; row 1 the mean radiance in ultraviolet, X-rays, infrared and radio. Recomputed from Planck and the sheet's
 * band edges; nothing is tuned.
 */
export function bandTable(): Float32Array {
  const out = new Float32Array(RAMP * 2 * 4);
  const lg = (v: number) => (v > 0 ? Math.max(LOG_NONE, Math.log10(v)) : LOG_NONE);
  for (let i = 0; i < RAMP; i++) {
    const t = 10 ** (LOG_T0 + ((LOG_T1 - LOG_T0) * i) / (RAMP - 1));
    const rgb = linearSrgb(xyz(thermal(t)));
    out.set([lg(rgb[0]), lg(rgb[1]), lg(rgb[2]), lg(bandMean("visible", t))], i * 4);
    out.set([lg(bandMean("ultraviolet", t)), lg(bandMean("xray", t)), lg(bandMean("infrared", t)), lg(bandMean("radio", t))], (RAMP + i) * 4);
  }
  return out;
}

/** Index in the band table of a picture band's radiance: [row, channel]. */
export const BAND_SLOT: Record<PictureBand, [number, number]> = { visible: [0, 3], ultraviolet: [1, 0], xray: [1, 1], infrared: [1, 2], radio: [1, 3] };

// ── The picture ──────────────────────────────────────────────────────────────────────────────────────────────────

/**
 * The camera of a close-up (design 3.7): `distance` r_g from the hole, looking at it, with the disc's axis drawn
 * upright; the viewer's tilt from the axis is `tilt` (0 from above, π/2 edge on). The picture spans `half` r_g of
 * impact parameter from its centre to its top edge. The disc turns anticlockwise as seen when `turning` is 1 (the
 * kernel's convention) and clockwise when −1 (seen from the disc's other face).
 */
export interface HoleView {
  distance: number;
  tilt: number;
  half: number;
  turning: 1 | -1;
}

/** A pixel's ray: its impact parameter and its angle round the picture (from up, towards the left). */
export function rayOf(view: HoleView, x: number, y: number): { b: number; alpha: number } {
  // x, y in −1..1 of the picture's height (y up); the picture plane at the hole spans ±half.
  const px = x * view.half;
  const py = y * view.half;
  const theta = Math.atan(Math.hypot(px, py) / view.distance);
  return { b: view.distance * Math.sin(theta), alpha: view.turning * Math.atan2(-px, py) };
}

export interface BandBrightest {
  /** log10 of the brightest mean radiance in each band, over the disc as this viewer sees it. */
  log: Record<PictureBand, number>;
  /** log10 of the brightest point's largest visible channel (linear sRGB, as the band table's row 0 holds it). */
  rgbTop: number;
  /** The brightest point's seen temperature. */
  t_seen_k: number;
  /** Its visible-band brightness over the Sun's surface (the kernel's exposure note). */
  times_sun_surface: number;
}

/**
 * The disc's brightest point in each band as this viewer sees it, from a trace of the picture on the CPU (the same
 * rays the GPU draws, on a polar grid fine near the shadow): it sets each band's exposure and the dimming note. The
 * kernel's own figure (`image.exposure`) is for its viewer's tilt; at that tilt they agree.
 */
export function brightest(t: BendTable, disc: Disc | null, tilt: number, turning: 1 | -1 = 1): BandBrightest | null {
  if (!disc) return null;
  const log = Object.fromEntries(PICTURE_BANDS.map((b) => [b, LOG_NONE])) as Record<PictureBand, number>;
  let hot = 0;
  const ANG = 192;
  const bHi = Math.min(disc.rOut * 1.02, 4000);
  const steps: number[] = [];
  for (let b = 0.02; b < 12; b += 0.04) steps.push(b);
  for (let b = 12; b < bHi; b *= 10 ** (1 / 48)) steps.push(b);
  // The hottest seen point: the visible mean is monotonic in T, so the band's brightest is the hottest point. The
  // other bands too, since a blackbody brightens in every band as it heats.
  for (const b of steps)
    for (let j = 0; j < ANG; j++) {
      const alpha = turning * (-Math.PI + ((j + 0.5) * 2 * Math.PI) / ANG);
      const r = traceRay(t, disc, b, alpha, tilt);
      if (r.kind === "disc" && r.t_seen_k > hot) hot = r.t_seen_k;
    }
  if (!(hot > 0)) return null;
  for (const b of PICTURE_BANDS) log[b] = Math.log10(Math.max(1e-300, bandMean(b, hot)));
  const sun = bandMean("visible", constant("T_sun"));
  const rgb = linearSrgb(xyz(thermal(hot)));
  return { log, rgbTop: Math.log10(Math.max(...rgb)), t_seen_k: hot, times_sun_surface: bandMean("visible", hot) / sun };
}

/**
 * Where the shadow's edge and the arch's inner edge sit straight up the picture, as fractions of its half-height
 * from the centre: what a frame is measured against. The shadow's edge is √27 for a far viewer; the arch's inner edge
 * is the smallest b straight up the picture (α = 0) whose ray lands on the disc's far side in the direct image (the
 * disc's inner edge seen over the top of the hole), and its foot the smallest b straight down (the near side's inner
 * edge, in front of the hole).
 */
export function marksOf(t: BendTable, disc: Disc | null, view: HoleView): { shadow: number; arch: number | null; below: number | null } {
  const toY = (b: number) => (Math.tan(Math.asin(Math.min(1, b / view.distance))) * view.distance) / view.half;
  const first = (alpha: number) => {
    if (!disc) return null;
    for (let b = B_CRIT; b < view.half * 1.5; b += 0.005) {
      const hit = landing(t, b, alpha, view.tilt, disc.rIn, disc.rOut);
      if (hit && hit.order === 0) return toY(b);
    }
    return null;
  };
  return { shadow: toY(B_CRIT), arch: first(0), below: first(Math.PI) };
}
