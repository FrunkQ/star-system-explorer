// A body's face in True: one door for every view that draws a body in True (the system's 3D, the size comparison's
// lineup and gallery), so a look added here (a giant's bands, a haze, plumes, frosts, a star's glow) shows in all of
// them at once with no second code path. It takes what is known of the body and gives the GpuItem fields that draw it;
// the view keeps only where the body is and how big. Pure, apart from a cache of ring profiles by key.
import { smallBodyColour, smallBodyMesh, type SmallFigure } from "./small-body.ts";
import type { GpuItem } from "./gpu.ts";
import type { Vec3 } from "./frames.ts";
import { ringProfile, type RingBand } from "./rings.ts";
import { cutRings, ringMarks, type RingMoon } from "./ring-structure.ts";
import type { TrueTexture } from "./true-look.ts";
import { drawnGlow, hotGlow } from "./hot-glow.ts";
import { plumeBrightness, plumeGrains, type PlumeFacts } from "./plumes.ts";
import { flowFace, type AirFlow } from "./true-air.ts";
import { airLuts, type AirLuts, type AirShape } from "./air-luts.ts";

export interface TrueBody {
  /** Unique among what is drawn: keys the textures and ring profiles uploaded once. */
  key: string;
  /** A star draws by its own light. */
  star: boolean;
  /** The colour when nothing better is known (a star's, or the view's plain grey for a world not read yet). */
  colour: string;
  /** The body's mean radius, metres (rings are drawn in units of it). */
  radius_m?: number;
  /** Its own surface, read from the kernel (`trueSurface`). */
  surface?: TrueTexture;
  /** Its rings, as the ring data holds them. */
  rings?: readonly RingBand[];
  /** Its north pole in the drawing's frame: its rings lie in its equator. */
  pole: Vec3;
  /** A star's living face for this moment (star-now.ts `liveFace` or `drawStarNow`): churning cells, spots, flares. */
  face?: GpuItem["face"];
  /**
   * Its own heat (hot-glow.ts): its temperature and its star's (temperature, radius, distance). A world at or over
   * the Draper point glows by Planck against its own daylight.
   */
  heat?: { t_k: number; star?: { t_k: number; r_m: number; d_m: number } | null };
  /** Its plumes from the kernel's surface record (plumes.ts), with the light's and the eye's directions from it. */
  plumes?: { facts: PlumeFacts; toLight: Vec3; toEye: Vec3 };
  /**
   * The winds its face is carried by (a giant's, from `readTrueFace`), and how long after the face was read the
   * drawing is, in the body's own seconds: the bands slide at their jets' speeds and the storms turn (VP-A1).
   */
  flow?: { flow: AirFlow; dt_s: number };
  /** Its clouds and haze alone (true-air.ts `AirLook.clouds`), drawn as a lit layer over `surface` (VP-A4). */
  clouds?: TrueTexture;
  /** Its air in its own radius (true-air.ts `readTrueAir`): the rim and terminator (VP-A3). */
  air?: AirShape;
  /**
   * A small body's own figure from the kernel (`small_body`, sse-small `Figure` with its albedo and weathering): drawn
   * as that shape, seeded by `key` (use the body's address), instead of a sphere.
   */
  figure?: SmallFigure;
  /** The world's mass, kg, and its moons: with both, its rings are cut where the moons' resonances and gaps fall (ring-structure.ts). */
  mass_kg?: number;
  moons?: readonly RingMoon[];
  /** How much its spin flattens it (true-shape.ts `readTrueShape`, TA6). */
  flatten?: number;
  /** Its seas' surface wind (m/s) and the liquid's refractive index, for the sun's glint, when the view has them. */
  sea?: { wind_m_s?: number; index?: number };
}

/** The GPU fields of a body's True face. */
export type TrueLook = Pick<GpuItem, "colour" | "shines" | "surface" | "rings" | "face" | "mesh" | "clouds" | "air" | "glow" | "plumes" | "flatten" | "pole" | "glint">;

// Whether a surface has any liquid ground, worked out once per picture.
const wetSeen = new WeakMap<object, boolean>();
function hasLiquid(t: TrueTexture): boolean {
  const c = t.faces;
  if (!c) return false;
  let w = wetSeen.get(c);
  if (w === undefined) wetSeen.set(c, (w = c.faces.some((f) => !!f && f.liquid.some((x) => x > 0))));
  return w;
}

/** A surface texture's mean colour, 0..1 (sRGB bytes over 255; a tint, not a measurement). */
export function meanColour(t: TrueTexture): [number, number, number] {
  let r = 0;
  let g = 0;
  let b = 0;
  let n = 0;
  for (let i = 0; i < t.data.length; i += 4 * 17) ((r += t.data[i]!), (g += t.data[i + 1]!), (b += t.data[i + 2]!), n++);
  return n ? [r / n / 255, g / n / 255, b / n / 255] : [0.5, 0.5, 0.5];
}

type Profile = { inner: number; outer: number; width: number; data: Uint8ClampedArray; assumed: boolean };
const profiles = new Map<string, Profile | null>();
// The air's look-up tables, worked out once per world and air (about a tenth of a second each).
const tables = new Map<string, AirLuts>();

/**
 * The rings' profile for a body, worked out once per body, rings and surface: their colour is the world's own mean
 * surface colour when its surface has been read, else a neutral grey (the ring data carries no colour yet: fudge
 * register, "Rings in True").
 */
function ringsOf(b: TrueBody): (Profile & { key: string }) | null {
  if (!b.rings?.length || !(b.radius_m && b.radius_m > 0)) return null;
  const shaped = !!(b.moons?.length && b.mass_kg && b.mass_kg > 0);
  const key = `${b.key}|${JSON.stringify(b.rings)}|${b.radius_m}|${b.surface ? "s" : "-"}${shaped ? `|${b.mass_kg}|${JSON.stringify(b.moons)}` : ""}`;
  if (!profiles.has(key)) {
    let rings = b.rings;
    if (shaped) {
      const lo = Math.min(...rings.map((r) => r.inner_m));
      const hi = Math.max(...rings.map((r) => r.outer_m));
      rings = cutRings(rings, ringMarks(b.moons!, b.mass_kg!, lo, hi));
    }
    // Finer when the moons cut it, so a gap a few tens of km wide still shows.
    const p = ringProfile(rings, b.surface ? meanColour(b.surface) : [0.5, 0.5, 0.5], shaped ? 2048 : 512);
    profiles.set(key, p ? { inner: p.inner_m / b.radius_m, outer: p.outer_m / b.radius_m, width: p.width, data: p.data, assumed: p.assumed } : null);
  }
  const p = profiles.get(key);
  return p ? { ...p, key } : null;
}

/** Whether a body's rings are drawn at a fallback depth or brightness (for the view's note). */
export function ringsAssumed(b: TrueBody): boolean {
  return ringsOf(b)?.assumed ?? false;
}

/** A body's face in True. */
export function trueLook(b: TrueBody): TrueLook {
  const out: TrueLook = { colour: b.colour };
  if (b.star) out.shines = true;
  if (b.star && b.face) out.face = b.face;
  if (b.surface) {
    const f = b.flow;
    // Carried by its winds: a new picture each time asked, uploaded into the same texture by its version.
    if (f && f.dt_s && b.radius_m && b.radius_m > 0)
      out.surface = { key: b.key, texture: flowFace(b.surface, f.flow, { radius_m: b.radius_m, dt_s: f.dt_s }), version: f.dt_s };
    else out.surface = { key: b.key, texture: b.surface };
  }
  // Liquid ground glints in its star's light (gpu-shade.ts): drawn on the face pictures, which carry where it is wet.
  if (!b.star && b.surface && hasLiquid(b.surface)) out.glint = { ...(b.sea?.wind_m_s !== undefined ? { wind_m_s: b.sea.wind_m_s } : {}), ...(b.sea?.index !== undefined ? { index: b.sea.index } : {}) };
  if (!b.star && b.flatten && b.flatten > 0) ((out.flatten = b.flatten), (out.pole = b.pole));
  if (!b.star && b.clouds) out.clouds = { key: `${b.key}|clouds`, texture: b.clouds };
  if (!b.star && b.air) {
    const key = `${b.key}|air|${b.air.beta.join(",")}|${b.air.scale_height}`;
    if (!tables.has(key)) tables.set(key, airLuts(b.air));
    out.air = { key, shape: b.air, luts: tables.get(key)! };
  }
  const r = b.star ? null : ringsOf(b);
  if (r) out.rings = { key: r.key, inner: r.inner, outer: r.outer, normal: b.pole, width: r.width, data: r.data };
  if (!b.star && b.heat) {
    const g = drawnGlow(hotGlow(b.heat.t_k, b.heat.star));
    if (g) out.glow = g.rgb;
  }
  if (!b.star && b.plumes && b.radius_m) {
    const p = b.plumes;
    const unit = (v: Vec3) => {
      const l = Math.hypot(v[0], v[1], v[2]) || 1;
      return [v[0] / l, v[1] / l, v[2] / l] as Vec3;
    };
    // The light travels from the star to the plume, then to the eye: back-lit when the star is behind the plume.
    const l = unit(p.toLight);
    const e = unit(p.toEye);
    const cosPhase = -(l[0] * e[0] + l[1] * e[1] + l[2] * e[2]);
    const brightness = plumeBrightness(p.facts, cosPhase);
    if (brightness > 0) out.plumes = { points: turnToPole(plumeKept(b.key, p.facts, b.radius_m), b.pole), brightness };
  }
  if (b.figure && !b.star) {
    out.mesh = smallBodyMesh(b.figure, b.key);
    if (!b.surface) out.colour = smallBodyColour(b.figure);
  }
  return out;
}

// A plume's grains are made once per body and its facts (they are a drawing's, deterministic).
const grains = new Map<string, Float32Array>();
function plumeKept(key: string, f: PlumeFacts, radius_m: number): Float32Array {
  const k = `${key}|${JSON.stringify(f)}|${radius_m}`;
  let g = grains.get(k);
  if (!g) grains.set(k, (g = plumeGrains(f, radius_m)));
  return g;
}

/** Points in the body's own frame (+z its pole) turned so +z lies along `pole` in the drawing's frame. */
function turnToPole(pts: Float32Array, pole: Vec3): Float32Array {
  const l = Math.hypot(pole[0], pole[1], pole[2]) || 1;
  const z: Vec3 = [pole[0] / l, pole[1] / l, pole[2] / l];
  const a: Vec3 = Math.abs(z[0]) < 0.9 ? [1, 0, 0] : [0, 1, 0];
  const xr: Vec3 = [a[1] * z[2] - a[2] * z[1], a[2] * z[0] - a[0] * z[2], a[0] * z[1] - a[1] * z[0]];
  const xl = Math.hypot(xr[0], xr[1], xr[2]) || 1;
  const x: Vec3 = [xr[0] / xl, xr[1] / xl, xr[2] / xl];
  const y: Vec3 = [z[1] * x[2] - z[2] * x[1], z[2] * x[0] - z[0] * x[2], z[0] * x[1] - z[1] * x[0]];
  const out = new Float32Array(pts.length);
  for (let i = 0; i < pts.length; i += 3) {
    const [u, v, w] = [pts[i]!, pts[i + 1]!, pts[i + 2]!];
    for (let k = 0; k < 3; k++) out[i + k] = u * x[k]! + v * y[k]! + w * z[k]!;
  }
  return out;
}
