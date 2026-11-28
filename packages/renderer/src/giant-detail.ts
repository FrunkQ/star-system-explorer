// A giant's True face at the size its storms need (visual polish 5.4: storms that turn, curls at band edges, VP-A2).
//
// The weather op's six level-0 tiles give a giant's bands at about 4° a cell: enough for the belts and zones, too
// coarse for a storm (Jupiter's largest oval came out four pixels wide) or a polar polygon (Saturn's hexagon was two
// rows at the pole). This file makes the face finer without making anything up:
//
// 1. Facts, finer. The weather op's `patch` read gives the same cells closer, round each place that needs them: a
//    pole with features on it, and each storm too large to vanish at this size. They are laid over the face (the
//    tiles' face, stretched) with the stacking `paint` uses, feathered at their edges. Every shape in them is the
//    kernel's: an oval's ellipse, a vortex's eye and wall, a dark spot's bright companion, the polygon's edge.
// 2. Turning, from the patch's own winds. Inside each storm the picture is wound round its centre by the angle its
//    winds carry it in one shear time (how long the band's shear takes to renew what the storm winds up), capped at
//    one turn of its core: bands drawn into spiral arms round an oval, a spiral round a polar vortex. Which way and
//    how fast are the cells' winds less the band's: no list of kinds says which way a storm turns.
// 3. Curls at the band edges, below the facts: the detail ladder's `giant.curls` (VP6), curl noise whose largest
//    size is the Rhines scale, stretched along the jets by the Rhines scale over the deformation radius, its strength
//    the eddy speed against the band's shear. Drawn from the numbers, not computed: the HUD says so.
//
// Grade (design section 3): 1 is fact; 2 and 3 are drawn from the numbers. Nothing here names a body.

import { DEEP_AIR, composeAt, stackOf, type AirFlow, type ColourOf, type Rgb, type Stack } from "./true-air.ts";
import { detailLine, discScale, ladderAt, ladderLayer, ladderSeed, resolveLayer, rungs, type LadderFacts, type Rung } from "./ladder.ts";
import type { TrueTexture } from "./true-look.ts";
import type { WeatherCell, WeatherSystem } from "./weather.ts";
import type { V3 } from "./ladder-noise.ts";
import { constant } from "../../ui-contract/src/index.ts";

/** The weather op's `want: "patch"` (kernel `FieldPatch`): n × n cells on the azimuthal equidistant plane round a place, rows south to north. */
export interface FieldPatch {
  lat_deg: number;
  lon_deg: number;
  half_km: number;
  n: number;
  cells: WeatherCell[];
}

/** The weather op's `climate` read of a banded giant, its `jets` (K-a): one entry per band, at its centre, south to north. */
export interface GiantJets {
  lat_deg: number[];
  jet_speed_m_s: number[];
  jet_shear_m_s: number[];
  eddy_speed_m_s: number[];
  rhines_scale_m: number[];
  deformation_radius_m: number[];
}

/** Everything a giant's finer face is made from, all read from the kernel. */
export interface GiantFacts {
  address: string;
  radius_m: number;
  /** Universe seconds the face is for (the curls' clock). */
  t_s: number;
  systems: readonly WeatherSystem[];
  patches: readonly FieldPatch[];
  jets: GiantJets | null;
}

/** The finer face's size: 0.35° a pixel, so Jupiter's largest ovals are some forty pixels long and Saturn's hexagon some sixty across. */
export const GIANT_W = 1024;
export const GIANT_H = 512;

const DEG = Math.PI / 180;
const clamp = (x: number, a: number, b: number) => (x < a ? a : x > b ? b : x);
const wrap01 = (u: number) => u - Math.floor(u);
const toLin = (c: number) => {
  const x = c / 255;
  return x <= 0.04045 ? x / 12.92 : Math.pow((x + 0.055) / 1.055, 2.4);
};
const toByte = (x: number) => {
  const v = clamp(x, 0, 1);
  return Math.round(255 * (v <= 0.0031308 ? 12.92 * v : 1.055 * Math.pow(v, 1 / 2.4) - 0.055));
};

// ------------------------------------------------------------------ the patch's plane (kernel rules::patch_dir)

interface Frame {
  c: V3<number>;
  e: V3<number>;
  n: V3<number>;
}

/** A place's own east and north; at a pole, north is towards longitude lon + 180° (as the kernel's `patch_dir`). */
function frameAt(lat_deg: number, lon_deg: number): Frame {
  const p = lat_deg * DEG;
  const l = lon_deg * DEG;
  return {
    c: [Math.cos(p) * Math.cos(l), Math.cos(p) * Math.sin(l), Math.sin(p)],
    e: [-Math.sin(l), Math.cos(l), 0],
    n: [-Math.sin(p) * Math.cos(l), -Math.sin(p) * Math.sin(l), Math.cos(p)],
  };
}

const dot = (a: V3<number>, b: V3<number>) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const dirOf = (lat: number, lon: number): V3<number> => {
  const p = lat * DEG;
  const l = lon * DEG;
  return [Math.cos(p) * Math.cos(l), Math.cos(p) * Math.sin(l), Math.sin(p)];
};

/**
 * A wind given in its own place's east and north (m/s), less the band's eastward `u0`, in a frame's east and north:
 * the two agree near the frame's centre, not at a pole, where each longitude's east points another way.
 */
function relWind(f: Frame, lat: number, lon: number, we: number, wn: number, u0: number): [number, number] {
  const l = frameAt(lat, lon);
  const v = [0, 1, 2].map((k) => (we - u0) * l.e[k as 0]! + wn * l.n[k as 0]!) as V3<number>;
  return [dot(v, f.e), dot(v, f.n)];
}

/** A direction's place on a frame's plane, metres east and north (the inverse of `fromPlane`). */
function toPlane(f: Frame, d: V3<number>, a: number): [number, number] {
  const ang = Math.acos(clamp(dot(d, f.c), -1, 1));
  const x = dot(d, f.e);
  const y = dot(d, f.n);
  const s = Math.hypot(x, y);
  return s > 1e-15 ? [(ang * a * x) / s, (ang * a * y) / s] : [0, 0];
}

/** The latitude and longitude (degrees) x metres east and y north of a frame's centre on its plane. */
function fromPlane(f: Frame, x: number, y: number, a: number): [number, number] {
  const d = Math.hypot(x, y) / a;
  if (d <= 0) return [Math.asin(clamp(f.c[2], -1, 1)) / DEG, Math.atan2(f.c[1], f.c[0]) / DEG];
  const sx = x / a / d;
  const sy = y / a / d;
  const cd = Math.cos(d);
  const sd = Math.sin(d);
  const v = [0, 1, 2].map((k) => cd * f.c[k as 0]! + sd * (sx * f.e[k as 0]! + sy * f.n[k as 0]!)) as V3<number>;
  return [Math.asin(clamp(v[2], -1, 1)) / DEG, Math.atan2(v[1], v[0]) / DEG];
}

// ------------------------------------------------------------------ which patches to read

/** One patch to ask the weather op for. */
export interface PatchAsk {
  lat: number;
  lon: number;
  half_km: number;
  n: number;
}

/** Cells across a polar patch and a storm's: a polar patch holds a polygon's six sides or a ring of cyclones, a storm's one ellipse. */
export const POLE_N = 48;
export const STORM_N = 24;
/** At most this many storms get their own patch (the largest first): the rest are too small to need one, or too many to be worth the reads. */
export const MAX_STORM_PATCHES = 12;

/**
 * Where a system is centred and how far its effect reaches (metres): the weather block's Gaussian falls to e⁻⁹ at
 * three of its radii, half its size each, stretched by its shear's aspect. A polygon jet's record gives its jet's
 * latitude and first vertex, its size the polygon's width: it is centred on its pole.
 */
function reachOf(s: WeatherSystem & { aspect?: number }): { lat: number; lon: number; reach_m: number } {
  const lam = s.aspect && s.aspect > 0 ? Math.max(s.aspect, 1 / s.aspect) : 1;
  if (s.kind === "polygon-jet") return { lat: s.lat_deg >= 0 ? 90 : -90, lon: s.lon_deg, reach_m: 0.5 * s.size_km * 1000 * 1.3 };
  return { lat: s.lat_deg, lon: s.lon_deg, reach_m: 1.5 * s.size_km * 1000 * Math.sqrt(lam) };
}

/**
 * The patches a face of `width` pixels needs: one round each pole that has anything on it reaching the pole (its
 * polygon, its cyclones, its vortex, its cap), then one round each other storm larger than three pixels, largest first.
 */
export function patchesFor(systems: readonly WeatherSystem[], radius_m: number, width = GIANT_W): PatchAsk[] {
  const px_m = (2 * Math.PI * radius_m) / width;
  const out: PatchAsk[] = [];
  const covered = new Set<WeatherSystem>();
  for (const pole of [90, -90]) {
    let half = 0;
    for (const s of systems) {
      const r = reachOf(s);
      const colat = (90 - Math.abs(r.lat)) * DEG * radius_m;
      if (Math.sign(r.lat) !== Math.sign(pole) || colat > r.reach_m) continue;
      half = Math.max(half, colat + r.reach_m);
      covered.add(s);
    }
    if (half > 0) out.push({ lat: pole, lon: 0, half_km: Math.min(half, 0.5 * Math.PI * radius_m) / 1000, n: POLE_N });
  }
  const storms = systems
    .filter((s) => !covered.has(s) && s.size_km * 1000 >= 3 * px_m)
    .sort((a, b) => b.size_km - a.size_km || (a.id < b.id ? -1 : 1))
    .slice(0, MAX_STORM_PATCHES);
  for (const s of storms) {
    const r = reachOf(s);
    out.push({ lat: r.lat, lon: r.lon, half_km: r.reach_m / 1000, n: STORM_N });
  }
  return out;
}

// ------------------------------------------------------------------ the finer face

/** A face's pixel centre as latitude and longitude (true-look.ts `equirect`: row 0 at the south). */
const latOf = (j: number, h: number) => (180 * (j + 0.5)) / h - 90;
const lonOf = (i: number, w: number) => (360 * (i + 0.5)) / w - 180;

/** Bilinear read of a linear RGB float picture (3 a pixel) at a latitude and longitude, wrapping in longitude. */
function readLin(src: Float32Array, w: number, h: number, lat: number, lon: number, out: number[]): void {
  const x = wrap01((lon + 180) / 360) * w - 0.5;
  const y = clamp(((clamp(lat, -90, 90) + 90) / 180) * h - 0.5, 0, h - 1);
  const i0 = Math.floor(x);
  const j0 = Math.floor(y);
  const fx = x - i0;
  const fy = y - j0;
  const j1 = Math.min(h - 1, j0 + 1);
  const a = ((i0 % w) + w) % w;
  const b = (a + 1) % w;
  for (let ch = 0; ch < 3; ch++) {
    const top = src[(j0 * w + a) * 3 + ch]! * (1 - fx) + src[(j0 * w + b) * 3 + ch]! * fx;
    const bot = src[(j1 * w + a) * 3 + ch]! * (1 - fx) + src[(j1 * w + b) * 3 + ch]! * fx;
    out[ch] = top * (1 - fy) + bot * fy;
  }
}

/** A patch ready to read: its cells' stack and its frame. */
interface ReadyPatch {
  p: FieldPatch;
  st: Stack;
  f: Frame;
  half_m: number;
}

/**
 * The patch's blended values at plane point (x, y) metres: bilinear over its grid, each slot's reflectance and
 * colour, the haze's absorption, the wind; and how far inside it is (1 well inside, falling to 0 at its edge).
 */
function patchAt(rp: ReadyPatch, x: number, y: number, acc: Float32Array, ab: number[], wind: number[]): number {
  const n = rp.p.n;
  const gx = ((x / rp.half_m + 1) * n) / 2 - 0.5;
  const gy = ((y / rp.half_m + 1) * n) / 2 - 0.5;
  if (gx < -0.5 || gy < -0.5 || gx > n - 0.5 || gy > n - 0.5) return 0;
  const edge = Math.min(gx + 0.5, n - 0.5 - gx, gy + 0.5, n - 0.5 - gy) / (0.2 * n);
  const inside = clamp(edge, 0, 1);
  if (inside <= 0) return 0;
  const cx = clamp(gx, 0, n - 1);
  const cy = clamp(gy, 0, n - 1);
  const i0 = Math.min(n - 2, Math.floor(cx));
  const j0 = Math.min(n - 2, Math.floor(cy));
  const fx = cx - i0;
  const fy = cy - j0;
  const st = rp.st;
  const slots = st.slots * 4;
  acc.fill(0);
  ab[0] = ab[1] = ab[2] = 0;
  wind[0] = wind[1] = 0;
  const corners: [number, number][] = [
    [j0 * n + i0, (1 - fx) * (1 - fy)],
    [j0 * n + i0 + 1, fx * (1 - fy)],
    [(j0 + 1) * n + i0, (1 - fx) * fy],
    [(j0 + 1) * n + i0 + 1, fx * fy],
  ];
  for (const [c, wgt] of corners) {
    for (let q = 0; q < slots; q++) acc[q]! += wgt * st.vals[c * slots + q]!;
    for (let ch = 0; ch < 3; ch++) ab[ch]! += wgt * st.absorb[c * 3 + ch]!;
    wind[0]! += wgt * st.wind[c * 2]!;
    wind[1]! += wgt * st.wind[c * 2 + 1]!;
  }
  // Smooth in, so the seam with the tiles' face does not show.
  return inside * inside * (3 - 2 * inside);
}

/** A storm to turn: the patch that holds it, its aspect, its core's angular speed, and the angle-per-angular-speed it is wound by. */
interface Turning {
  /** The storm's own frame, at its centre. */
  f: Frame;
  lam: number;
  R: number;
  /** The angle its core is wound through, radians (anticlockwise seen from outside positive). */
  core: number;
}

/** Interpolate a per-band list (south to north) at a latitude, holding the end values. */
function byLat(lats: readonly number[], vals: readonly number[], lat: number): number {
  if (!lats.length) return 0;
  if (lat <= lats[0]!) return vals[0]!;
  for (let k = 1; k < lats.length; k++)
    if (lat <= lats[k]!) {
      const t = (lat - lats[k - 1]!) / (lats[k]! - lats[k - 1]!);
      return vals[k - 1]! * (1 - t) + vals[k]! * t;
    }
  return vals[vals.length - 1]!;
}

/**
 * The relative vorticity of the bands at a latitude, s⁻¹, from the jets at the band centres:
 * ζ = -(1 / a cos φ) d(u cos φ)/dφ, differenced between neighbouring centres.
 */
function bandVorticity(j: GiantJets, a: number, lat: number): number {
  const L = j.lat_deg;
  if (L.length < 2) return 0;
  let k = 1;
  while (k < L.length - 1 && L[k]! < lat) k++;
  const uc = (i: number) => j.jet_speed_m_s[i]! * Math.cos(L[i]! * DEG);
  const dphi = (L[k]! - L[k - 1]!) * DEG;
  return -(uc(k) - uc(k - 1)) / dphi / (a * Math.max(0.02, Math.cos(lat * DEG)));
}

/** The giant's facts for the curls' declaration, by band: the kernel's jets, with the eddies' share against the shear. */
export function curlFacts(j: GiantJets): LadderFacts {
  return {
    rhines_scale_m: j.rhines_scale_m,
    deformation_radius_m: j.deformation_radius_m,
    eddy_speed_m_s: j.eddy_speed_m_s,
    // How far an eddy carries the cloud before the band's shear draws it out: its speed over its speed plus the
    // shear across the band, so a calm band curls and a strongly sheared one is drawn into threads.
    curl_share: j.eddy_speed_m_s.map((u, i) => Math.abs(u) / (Math.abs(u) + Math.abs(j.jet_shear_m_s[i] ?? 0))),
  };
}

/**
 * A giant's face made finer (see the head of this file): `base` is `trueGiant`'s face from the tiles, `flow` its
 * winds, `deep` and `tint` what it was painted over and through. Returns the face at `width` × `width/2`, its winds
 * at the same size, and the HUD's words.
 */
export function detailGiant(
  base: TrueTexture,
  flow: AirFlow | undefined,
  g: GiantFacts,
  colourOf: ColourOf,
  opts: { deep?: Rgb | null; tint?: Rgb | null; width?: number; source?: boolean } = {},
): { tex: TrueTexture; flow?: AirFlow; said: string[] } {
  const W = opts.width ?? GIANT_W;
  const H = W / 2;
  const a = g.radius_m;
  const deep = opts.deep ?? DEEP_AIR;
  const tint = opts.tint ?? null;
  // 1. The tiles' face and winds, stretched.
  const lin = new Float32Array(W * H * 3);
  const east = new Float32Array(W * H);
  const north = new Float32Array(W * H);
  const blin = new Float32Array(base.width * base.height * 3);
  for (let p = 0; p < base.width * base.height; p++) for (let ch = 0; ch < 3; ch++) blin[p * 3 + ch] = toLin(base.data[p * 4 + ch]!);
  const px = [0, 0, 0];
  for (let j = 0; j < H; j++) {
    const lat = latOf(j, H);
    const fj = flow ? clamp(Math.floor(((lat + 90) / 180) * flow.height), 0, flow.height - 1) : 0;
    for (let i = 0; i < W; i++) {
      const lon = lonOf(i, W);
      readLin(blin, base.width, base.height, lat, lon, px);
      const o = j * W + i;
      lin.set(px, o * 3);
      if (flow) {
        const fi = Math.floor(wrap01((lon + 180) / 360) * flow.width) % flow.width;
        east[o] = flow.east[fj * flow.width + fi]!;
        north[o] = flow.north[fj * flow.width + fi]!;
      }
    }
  }
  // 2. The patches, laid over it.
  const ready: ReadyPatch[] = g.patches.filter((p) => p.cells.length === p.n * p.n && p.n >= 2).map((p) => ({ p, st: stackOf(p.cells, colourOf), f: frameAt(p.lat_deg, p.lon_deg), half_m: p.half_km * 1000 }));
  const acc = new Float32Array(Math.max(1, ...ready.map((r) => r.st.slots)) * 4);
  const ab = [0, 0, 0];
  const wind = [0, 0];
  // How much of each pixel the patches' finer facts cover: the curls, drawn below the facts, stay out of them.
  const cover = new Float32Array(W * H);
  for (const rp of ready) {
    const reach = rp.half_m * Math.SQRT2;
    const span = reach / a / DEG;
    const lat0 = rp.p.lat_deg;
    const jLo = clamp(Math.floor(((lat0 - span + 90) / 180) * H), 0, H - 1);
    const jHi = clamp(Math.ceil(((lat0 + span + 90) / 180) * H), 0, H - 1);
    for (let j = jLo; j <= jHi; j++) {
      const lat = latOf(j, H);
      const c = Math.cos(lat * DEG);
      const dlon = Math.abs(lat) + span >= 89 || c < 1e-3 ? 181 : Math.min(181, span / c + 1);
      const iLo = Math.floor(((rp.p.lon_deg - dlon + 180) / 360) * W);
      const iHi = Math.ceil(((rp.p.lon_deg + dlon + 180) / 360) * W);
      for (let ii = iLo; ii <= Math.min(iHi, iLo + W - 1); ii++) {
        const i = ((ii % W) + W) % W;
        const [x, y] = toPlane(rp.f, dirOf(lat, lonOf(i, W)), a);
        const w = patchAt(rp, x, y, acc, ab, wind);
        if (w <= 0) continue;
        const c3 = composeAt(rp.st, acc, ab, 1, [...deep], tint);
        const o = j * W + i;
        for (let ch = 0; ch < 3; ch++) lin[o * 3 + ch] = lin[o * 3 + ch]! * (1 - w) + c3[ch]! * w;
        cover[o] = Math.max(cover[o]!, w);
        east[o] = east[o]! * (1 - w) + wind[0]! * w;
        north[o] = north[o]! * (1 - w) + wind[1]! * w;
      }
    }
  }
  // Each row's mean eastward wind: the bands' own, what a storm's winds are measured against.
  const rowEast = new Float64Array(H);
  for (let j = 0; j < H; j++) {
    let s = 0;
    for (let i = 0; i < W; i++) s += east[j * W + i]!;
    rowEast[j] = s / W;
  }
  // The band's wind a storm turns against: the kernel's jets where it gives them (a row's mean would take a polar
  // vortex's own winds for the band's), else the row's mean.
  const rowAt = (lat: number) => (g.jets && g.jets.lat_deg.length ? byLat(g.jets.lat_deg, g.jets.jet_speed_m_s, lat) : rowEast[clamp(Math.floor(((lat + 90) / 180) * H), 0, H - 1)]!);
  // 3. The turning: every system inside a patch that is a vortex of its own (a record that holds a smaller one
  // round its centre, a ring round its middle cyclone or a polygon round its vortex, is a group, not a vortex).
  const turning: Turning[] = [];
  const places = g.systems.map((x) => ({ s: x as WeatherSystem & { aspect?: number }, ...reachOf(x) }));
  for (const q of places) {
    const R = 0.5 * q.s.size_km * 1000;
    if (!(R > 0)) continue;
    const cq = dirOf(q.lat, q.lon);
    const holds = places.some((o) => o !== q && o.s.size_km < q.s.size_km && Math.acos(clamp(dot(cq, dirOf(o.lat, o.lon)), -1, 1)) * a < 0.5 * R);
    if (holds) continue;
    const rp = ready.find((r) => {
      const [x, y] = toPlane(r.f, cq, a);
      return patchAt(r, x, y, acc, ab, wind) >= 0.99;
    });
    if (!rp) continue;
    const lam = q.s.aspect && q.s.aspect > 0 ? q.s.aspect : 1;
    const f = frameAt(q.lat, q.lon);
    // The core's angular speed, from the patch's winds a third of a radius out, less the band's.
    let om = 0;
    for (let k = 0; k < 8; k++) {
      const t = (k * Math.PI) / 4;
      const x = (R / 3) * Math.cos(t) * Math.sqrt(lam);
      const y = ((R / 3) * Math.sin(t)) / Math.sqrt(lam);
      const [la, lo] = fromPlane(f, x, y, a);
      const [px2, py2] = toPlane(rp.f, dirOf(la, lo), a);
      if (patchAt(rp, px2, py2, acc, ab, wind) <= 0) continue;
      const [vx, vy] = relWind(f, la, lo, wind[0]!, wind[1]!, rowAt(la));
      om += (x * vy - y * vx) / (x * x + y * y) / 8;
    }
    if (!(Math.abs(om) > 1e-12)) continue;
    // Wound for one shear time of the band, at most one turn of its core.
    const zeta = g.jets ? Math.abs(bandVorticity(g.jets, a, q.lat)) : 0;
    const tau_s = Math.min(zeta > 0 ? 1 / zeta : Infinity, (2 * Math.PI) / Math.abs(om));
    turning.push({ f, lam, R, core: om * tau_s });
  }
  // 4. The curls, through the detail ladder, worked on a grid a quarter the face's size (their finest rung is eight
  // of its pixels) and read bilinearly. The noise is scaled so its typical displacement is how far an eddy carries
  // the cloud before the band's shear draws it out: u_e / (Δu / L) = L u_e / Δu for a band of width L, here
  // L u_e / (u_e + |Δu|) (the curl share times the band's width), so a calm band curls by about its width and a
  // strongly sheared one only wrinkles.
  const layer = ladderLayer("giant.curls");
  const facts = g.jets ? curlFacts(g.jets) : {};
  const res = layer ? resolveLayer(layer, facts) : null;
  let rs: Rung[] = [];
  const CW = W / 4;
  const CH = H / 4;
  const curlE = new Float32Array(CW * CH);
  const curlN = new Float32Array(CW * CH);
  let curled = false;
  if (res && res.drawn && g.jets && g.jets.lat_deg.length > 1) {
    const jt = g.jets;
    rs = rungs(res, discScale(a, CW / Math.PI, CW / Math.PI), "gpu-high", g.t_s, ladderSeed(g.address));
    const shares = facts.curl_share as number[];
    // A band's width about a latitude: the spacing of the band centres there.
    const widths = jt.lat_deg.map((l, k) => {
      const lo = jt.lat_deg[Math.max(0, k - 1)]!;
      const hi = jt.lat_deg[Math.min(jt.lat_deg.length - 1, k + 1)]!;
      return ((hi - lo) / (Math.min(jt.lat_deg.length - 1, k + 1) - Math.max(0, k - 1))) * DEG * a;
    });
    let sum = 0;
    let cnt = 0;
    for (let j = 0; j < CH; j++) {
      const lat = latOf(j, CH);
      for (let i = 0; i < CW; i++) {
        const lon = lonOf(i, CW);
        const d = dirOf(lat, lon);
        const fr = frameAt(lat, lon);
        const v = ladderAt("curl", rs, [d[0] * a, d[1] * a, d[2] * a], { normal: d, axis: fr.n, stretch: res.stretch }) as V3<number>;
        const o = j * CW + i;
        curlE[o] = dot(v, fr.e);
        curlN[o] = dot(v, fr.n);
        const wgt = Math.cos(lat * DEG);
        sum += wgt * (curlE[o]! ** 2 + curlN[o]! ** 2);
        cnt += wgt;
      }
    }
    const rms = Math.sqrt(sum / Math.max(1e-12, cnt));
    if (rms > 0) {
      curled = true;
      for (let j = 0; j < CH; j++) {
        const lat = latOf(j, CH);
        const k = byLat(jt.lat_deg, shares, lat) * byLat(jt.lat_deg, widths, lat) / rms;
        for (let i = 0; i < CW; i++) {
          curlE[j * CW + i]! *= k;
          curlN[j * CW + i]! *= k;
        }
      }
    }
  }
  /** The curls' displacement at a face pixel, metres east and north, bilinear on the quarter grid. */
  const curlAt = (lat: number, lon: number, out: number[]) => {
    const x = wrap01((lon + 180) / 360) * CW - 0.5;
    const y = clamp(((lat + 90) / 180) * CH - 0.5, 0, CH - 1);
    const i0 = Math.floor(x);
    const j0 = Math.floor(y);
    const fx = x - i0;
    const fy = y - j0;
    const j1 = Math.min(CH - 1, j0 + 1);
    const ia = ((i0 % CW) + CW) % CW;
    const ib = (ia + 1) % CW;
    for (const [arr, k] of [[curlE, 0], [curlN, 1]] as const) {
      const top = arr[j0 * CW + ia]! * (1 - fx) + arr[j0 * CW + ib]! * fx;
      const bot = arr[j1 * CW + ia]! * (1 - fx) + arr[j1 * CW + ib]! * fx;
      out[k] = top * (1 - fy) + bot * fy;
    }
  };
  const cv = [0, 0];
  // Draw: each pixel reads the stretched, patched face where its cloud came from.
  const data = new Uint8Array(W * H * 4);
  let mr = 0;
  let mg = 0;
  let mb = 0;
  let mw = 0;
  for (let j = 0; j < H; j++) {
    const lat = latOf(j, H);
    const cl = Math.cos(lat * DEG);
    for (let i = 0; i < W; i++) {
      const lon = lonOf(i, W);
      let sl = lat;
      let so = lon;
      const free = 1 - cover[j * W + i]!;
      // The stirred still's source (WS5, `source`) is the face before any winding or curls: the carrying does both.
      if (curled && free > 0 && !opts.source) {
        curlAt(lat, lon, cv);
        sl = lat + (free * cv[1]!) / a / DEG;
        so = lon + (free * cv[0]!) / (a * Math.max(0.02, cl)) / DEG;
      }
      for (const t of opts.source ? [] : turning) {
        const [x, y] = toPlane(t.f, dirOf(sl, so), a);
        const k = Math.sqrt(t.lam);
        const xr = x / k;
        const yr = y * k;
        const rho2 = (xr * xr + yr * yr) / (t.R * t.R);
        if (rho2 > 12) continue;
        // The weather block's Gaussian vortex turns at Ω(r) ∝ exp(-r²/2R²): the core most, nothing by three radii.
        // Each point is read from where the winds carried it from, in the ellipse's own round frame.
        const th = -t.core * Math.exp(-0.5 * rho2);
        const xs = (xr * Math.cos(th) - yr * Math.sin(th)) * k;
        const ys = (xr * Math.sin(th) + yr * Math.cos(th)) / k;
        [sl, so] = fromPlane(t.f, xs, ys, a);
      }
      readLin(lin, W, H, sl, so, px);
      const o = (j * W + i) * 4;
      data[o] = toByte(px[0]!);
      data[o + 1] = toByte(px[1]!);
      data[o + 2] = toByte(px[2]!);
      data[o + 3] = 255;
      mr += data[o]! * cl;
      mg += data[o + 1]! * cl;
      mb += data[o + 2]! * cl;
      mw += cl;
    }
  }
  const tex: TrueTexture = { width: W, height: H, data, mean: [Math.round(mr / mw), Math.round(mg / mw), Math.round(mb / mw)] };
  const said: string[] = [];
  if (ready.length) said.push(`Its storms and poles are drawn from the weather's own cells read closer (${ready.length} ${ready.length === 1 ? "patch" : "patches"})${opts.source ? "" : "; inside each storm the bands are wound round by its own winds"}.`);
  if (res && !opts.source) {
    const line = detailLine([{ layer: res, shown: rs }]);
    if (line) said.push(line);
  }
  const out: { tex: TrueTexture; flow?: AirFlow; said: string[] } = { tex, said };
  if (flow) out.flow = { width: W, height: H, east, north, rowEast, ...(flow.storms ? { storms: flow.storms } : {}) };
  return out;
}

// ------------------------------------------------------------------ reading it from the kernel

/** The kernel doors the finer face needs (a ViewSession has them). */
export interface DetailReader {
  weather(ask: Record<string, unknown>): Promise<{ ok: true; data: unknown } | { ok: false; message: string }>;
  resolvedSystem?(system: string): Promise<{ ok: true; result: { facts: Record<string, Record<string, unknown>> } } | { ok: false; message: string }>;
}

/** A patch's radius from its own cells: the corner cell's distance on its plane over its angle from the centre. */
function radiusFromPatch(p: FieldPatch): number | null {
  const c = p.cells[0];
  if (!c || p.n < 2) return null;
  const d = (p.half_km * 1000 * (1 - 1 / p.n)) * Math.SQRT2;
  const ang = Math.acos(clamp(dot(dirOf(p.lat_deg, p.lon_deg), dirOf(c.lat_deg, c.lon_deg)), -1, 1));
  return ang > 0 ? d / ang : null;
}

/**
 * Read what a giant's finer face is made from: its systems on the day, its jets, and the patches `patchesFor` asks
 * for. Null when the weather op gives no systems (the face stays as the tiles made it).
 */
export async function readGiantDetail(k: DetailReader, system: string, body: string, when: { at_s?: number; day: number }, width = GIANT_W): Promise<GiantFacts | null> {
  const ask = { system, body, ...(when.at_s !== undefined ? { at_s: when.at_s } : {}), day: when.day };
  const sy = await k.weather({ want: "systems", ...ask });
  if (!sy.ok) return null;
  const all = sy.data as WeatherSystem[];
  const systems = all.filter((s) => s.born <= when.day && when.day < s.ends);
  const cl = await k.weather({ want: "climate", ...ask });
  const jets = cl.ok ? (((cl.data as { jets?: GiantJets }).jets ?? null) as GiantJets | null) : null;
  // The radius: the system's facts, else the first patch's own geometry.
  let radius_m: number | null = null;
  if (k.resolvedSystem) {
    const rs = await k.resolvedSystem(system);
    const r = rs.ok ? rs.result.facts[body]?.["body.radius"] : undefined;
    if (typeof r === "number" && r > 0) radius_m = r;
  }
  // Without a radius yet, ask one pole's patch first to learn it, then size the rest.
  const asks = patchesFor(systems, radius_m ?? 7e7, width);
  const patches: FieldPatch[] = [];
  for (const p of asks) {
    const r = await k.weather({ want: "patch", ...ask, lat: p.lat, lon: p.lon, half_km: p.half_km, n: p.n });
    if (r.ok) patches.push(r.data as FieldPatch);
    radius_m ??= patches[0] ? radiusFromPatch(patches[0]) : null;
  }
  if (!radius_m) return null;
  return { address: body, radius_m, t_s: when.at_s ?? when.day * constant("day"), systems, patches, jets };
}
