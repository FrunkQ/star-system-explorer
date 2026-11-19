// What the relief passes read, prepared once per globe and look: each
// sample's base colour and its shading normal. The normal is the sphere's
// own, tilted by the real slope of the elevation layer (hill shading, Horn
// 1981) with the pass's exaggeration; liquid cover is a flat surface. The
// canvas and GPU tiers both shade from these same numbers, so a recipe looks
// the same on either. Nothing here invents terrain: it only reads the tiles.
import { cubeDirection, faceOf, sampleColour, SURFACE_COVERS, type Globe, type SurfaceTile } from "../../ui-contract/src/index.ts";

export interface Prepared {
  globe: Globe;
  samples: number;
  /** Per face: rgb 0..255 (3 per sample), unit normal (3 per sample), elevation m, cover code. */
  faces: ({ s: number; rgb: Uint8ClampedArray; normal: Float32Array; elev: Float32Array; cover: Uint8Array; albedo: Float32Array; tile: SurfaceTile } | null)[];
  lowest_m: number;
  highest_m: number;
}

/** Hypsometric tint for bare ground, low to high (the height colouring). */
const HEIGHT_RAMP: readonly [number, [number, number, number]][] = [
  [0, [86, 122, 82]],
  [0.25, [150, 158, 104]],
  [0.5, [178, 150, 108]],
  [0.75, [138, 110, 86]],
  [1, [238, 236, 232]],
];

function ramp(x: number): [number, number, number] {
  const t = Math.min(1, Math.max(0, x));
  for (let k = 1; k < HEIGHT_RAMP.length; k++) {
    const [b, cb] = HEIGHT_RAMP[k]!;
    const [a, ca] = HEIGHT_RAMP[k - 1]!;
    if (t <= b) {
      const f = (t - a) / (b - a);
      return [ca[0] + (cb[0] - ca[0]) * f, ca[1] + (cb[1] - ca[1]) * f, ca[2] + (cb[2] - ca[2]) * f];
    }
  }
  return HEIGHT_RAMP[HEIGHT_RAMP.length - 1]![1];
}

const sub = (a: number[], b: number[]) => [a[0]! - b[0]!, a[1]! - b[1]!, a[2]! - b[2]!];
const norm = (a: number[]) => {
  const l = Math.hypot(a[0]!, a[1]!, a[2]!) || 1;
  return [a[0]! / l, a[1]! / l, a[2]! / l];
};

export function prepare(globe: Globe, colour: string, exaggeration: number): Prepared {
  let lo = Infinity;
  let hi = -Infinity;
  for (const t of globe) {
    if (!t) continue;
    for (let i = 0; i < t.elevation_dm.length; i++) {
      if (SURFACE_COVERS[t.cover[i]!] !== "none") continue;
      const e = t.elevation_dm[i]! / 10;
      if (e < lo) lo = e;
      if (e > hi) hi = e;
    }
  }
  if (!Number.isFinite(lo)) [lo, hi] = [0, 1];
  let samples = 0;
  const faces = globe.map((t) => {
    if (!t) return null;
    const s = t.samples;
    const n = s * s;
    samples += n;
    const rgb = new Uint8ClampedArray(n * 3);
    const normal = new Float32Array(n * 3);
    const elev = new Float32Array(n);
    const cover = new Uint8Array(n);
    const albedo = new Float32Array(n);
    for (let i = 0; i < n; i++) {
      elev[i] = t.elevation_dm[i]! / 10;
      cover[i] = t.cover[i]!;
      albedo[i] = (t.albedo[i] ?? 300) / 1000;
    }
    const face = t.key.face;
    const span = 2 / s;
    for (let j = 0; j < s; j++) {
      for (let i = 0; i < s; i++) {
        const k = j * s + i;
        const u = -1 + (i + 0.5) * span;
        const v = -1 + (j + 0.5) * span;
        // Colour.
        let c: readonly number[];
        if (colour === "height" && cover[k] === 0) c = ramp((elev[k]! - lo) / Math.max(1, hi - lo));
        else c = sampleColour(t, k, 1);
        rgb[k * 3] = c[0]!;
        rgb[k * 3 + 1] = c[1]!;
        rgb[k * 3 + 2] = c[2]!;
        // Shading normal: the radial direction, tilted against the slope.
        const r = cubeDirection(face, u, v);
        if (cover[k] !== 0) {
          normal.set(r, k * 3);
          continue;
        }
        const h = (ii: number, jj: number) => {
          const q = Math.min(s - 1, Math.max(0, jj)) * s + Math.min(s - 1, Math.max(0, ii));
          return cover[q] !== 0 ? elev[k]! : elev[q]!;
        };
        const i0 = Math.max(0, i - 1);
        const i1 = Math.min(s - 1, i + 1);
        const j0 = Math.max(0, j - 1);
        const j1 = Math.min(s - 1, j + 1);
        const gi = ((h(i1, j) - h(i0, j)) / ((i1 - i0) * t.spacing_m)) * exaggeration;
        const gj = ((h(i, j1) - h(i, j0)) / ((j1 - j0) * t.spacing_m)) * exaggeration;
        const ei = norm(sub(cubeDirection(face, Math.min(1, u + span / 2), v), cubeDirection(face, Math.max(-1, u - span / 2), v)));
        const ej = norm(sub(cubeDirection(face, u, Math.min(1, v + span / 2)), cubeDirection(face, u, Math.max(-1, v - span / 2))));
        normal.set(norm([r[0] - gi * ei[0]! - gj * ej[0]!, r[1] - gi * ei[1]! - gj * ej[1]!, r[2] - gi * ei[2]! - gj * ej[2]!]), k * 3);
      }
    }
    return { s, rgb, normal, elev, cover, albedo, tile: t };
  });
  return { globe, samples, faces, lowest_m: lo, highest_m: hi };
}

/** The sample a world direction falls in. */
export function lookup(p: Prepared, d: readonly [number, number, number]): { f: NonNullable<Prepared["faces"][number]>; k: number } | null {
  const { face, u, v } = faceOf(d);
  const f = p.faces[face];
  if (!f) return null;
  const i = Math.min(f.s - 1, Math.max(0, Math.floor(((u + 1) / 2) * f.s)));
  const j = Math.min(f.s - 1, Math.max(0, Math.floor(((v + 1) / 2) * f.s)));
  return { f, k: j * f.s + i };
}

/** Normal, right and up of each cube face (atlas.rs FACES, as ui-contract's globe.ts). */
const FACE_AXES: readonly (readonly number[])[] = [
  [1, 0, 0, 0, 1, 0, 0, 0, 1],
  [-1, 0, 0, 0, -1, 0, 0, 0, 1],
  [0, 1, 0, -1, 0, 0, 0, 0, 1],
  [0, -1, 0, 1, 0, 0, 0, 0, 1],
  [0, 0, 1, 0, 1, 0, -1, 0, 0],
  [0, 0, -1, 0, 1, 0, 1, 0, 0],
];
const FOUR_OVER_PI = 4 / Math.PI;
/** The face last found by `lookupFast`. */
export let fastFace = -1;
/**
 * The same sample `lookup` finds, without allocating: returns the sample
 * index and leaves the face in `fastFace` (-1 and -1 when that face has no tile).
 * For the per-pixel loop; checked against `lookup` in the tests.
 */
export function lookupFast(p: Prepared, x: number, y: number, z: number): number {
  const ax = Math.abs(x);
  const ay = Math.abs(y);
  const az = Math.abs(z);
  const face = ax >= ay && ax >= az ? (x >= 0 ? 0 : 1) : ay >= az ? (y >= 0 ? 2 : 3) : z >= 0 ? 4 : 5;
  const f = p.faces[face];
  if (!f) return (fastFace = -1);
  const a = FACE_AXES[face]!;
  const k = x * a[0]! + y * a[1]! + z * a[2]!;
  const u = Math.atan((x * a[3]! + y * a[4]! + z * a[5]!) / k) * FOUR_OVER_PI;
  const v = Math.atan((x * a[6]! + y * a[7]! + z * a[8]!) / k) * FOUR_OVER_PI;
  const s = f.s;
  const i = Math.min(s - 1, Math.max(0, Math.floor(((u + 1) / 2) * s)));
  const j = Math.min(s - 1, Math.max(0, Math.floor(((v + 1) / 2) * s)));
  fastFace = face;
  return j * s + i;
}

/** The view rotation for a globe turned by `lon` about its pole and tipped by `lat` (the same as ui-contract's viewToWorld). */
export function viewRotation(lon_deg: number, lat_deg: number) {
  const lo = (lon_deg * Math.PI) / 180;
  const la = (lat_deg * Math.PI) / 180;
  const so = Math.sin(lo);
  const co = Math.cos(lo);
  const sa = Math.sin(la);
  const ca = Math.cos(la);
  return {
    /** View (x right, y up, z to the viewer) to world (z the pole). */
    toWorld: (x: number, y: number, z: number): [number, number, number] => {
      const vy = y * ca + z * sa;
      const vz = -y * sa + z * ca;
      return [vz * co - x * so, vz * so + x * co, vy];
    },
    /** World to view. */
    toView: (wx: number, wy: number, wz: number): [number, number, number] => {
      const vz = wx * co + wy * so;
      return [-wx * so + wy * co, wz * ca - vz * sa, wz * sa + vz * ca];
    },
    /** Rows of the world-to-view matrix. */
    rows: [
      [-so, co, 0],
      [-sa * co, -sa * so, ca],
      [ca * co, ca * so, sa],
    ] as const,
  };
}

/** The light's direction in view space from the pass's sun azimuth (clockwise from up) and elevation. */
export function lightDir(az_deg: number, el_deg: number): [number, number, number] {
  const a = (az_deg * Math.PI) / 180;
  const e = (el_deg * Math.PI) / 180;
  return [Math.sin(a) * Math.cos(e), Math.cos(a) * Math.cos(e), Math.sin(e)];
}

/** The shading factor both tiers use: flat share, plus ambient, plus Lambert from the light. */
export function lit(ndotl: number, shade: number, ambient: number): number {
  return 1 - shade + shade * (ambient + (1 - ambient) * Math.max(0, ndotl));
}

const DIRS = ["upper right", "right", "lower right", "bottom", "lower left", "left", "upper left", "top"];
function screenDir(x: number, y: number): string {
  if (Math.hypot(x, y) < 0.2) return "near the centre";
  const a = (Math.atan2(x, y) * 180) / Math.PI; // clockwise from up
  return `to the ${DIRS[Math.round((((a - 45) % 360) + 360) % 360 / 45) % 8]}`;
}
const km = (m: number) => (m < 0 ? "−" : "") + (Math.abs(m) >= 1000 ? `${(Math.abs(m) / 1000).toFixed(1)} km` : `${Math.round(Math.abs(m))} m`);
const coverWords: Record<string, string> = { water: "under water", magma: "under magma", hydrocarbon: "under hydrocarbon seas" };

/** The text pass: the face turned to the viewer in words. */
export function narrate(p: Prepared, label: string, lon_deg: number, lat_deg: number, detail: string, light?: [number, number, number]): string {
  const rot = viewRotation(lon_deg, lat_deg);
  const N = 48;
  const counts: Record<string, number> = {};
  let total = 0;
  let hi: { e: number; x: number; y: number } | null = null;
  let lo: { e: number; x: number; y: number; c: string } | null = null;
  for (let a = 0; a < N; a++) {
    for (let b = 0; b < N; b++) {
      const x = -1 + (a + 0.5) * (2 / N);
      const y = -1 + (b + 0.5) * (2 / N);
      const r2 = x * x + y * y;
      if (r2 > 1) continue;
      const hit = lookup(p, rot.toWorld(x, y, Math.sqrt(1 - r2)));
      if (!hit) continue;
      total++;
      const c = SURFACE_COVERS[hit.f.cover[hit.k]!] ?? "none";
      counts[c] = (counts[c] ?? 0) + 1;
      const e = hit.f.elev[hit.k]!;
      if (c === "none" && (!hi || e > hi.e)) hi = { e, x, y };
      if (!lo || e < lo.e) lo = { e, x, y, c };
    }
  }
  const lat = Math.round(lat_deg);
  const lon = Math.round(((lon_deg + 180) % 360 + 360) % 360 - 180);
  const where = `${Math.abs(lat)}° ${lat >= 0 ? "N" : "S"}, ${Math.abs(lon)}° ${lon >= 0 ? "E" : "W"}`;
  if (!total) return `${label}: no surface in view.`;
  const covered = Object.entries(counts).filter(([k]) => k !== "none").sort((x, y) => y[1] - x[1]);
  const pct = (n: number) => `${Math.round((100 * n) / total)}%`;
  const face = covered.length ? `${covered.map(([k, n]) => `${pct(n)} ${coverWords[k] ?? k}`).join(", ")}, the rest bare ground` : "all bare ground";
  const first = `${label}, the face centred on ${where}: ${face}.`;
  if (detail === "brief") return first;
  const parts = [first];
  if (hi) parts.push(`The highest ground in view, ${km(hi.e)}, lies ${screenDir(hi.x, hi.y)}.`);
  if (lo) parts.push(`The lowest, ${km(lo.e)}${lo.c !== "none" ? ` (${coverWords[lo.c] ?? lo.c})` : ""}, lies ${screenDir(lo.x, lo.y)}.`);
  if (light) parts.push(`Light falls from the ${screenDir(light[0], light[1]).replace(/^to the /, "")}.`);
  return parts.join(" ");
}
