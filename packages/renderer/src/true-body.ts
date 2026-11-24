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
}

/** The GPU fields of a body's True face. */
export type TrueLook = Pick<GpuItem, "colour" | "shines" | "surface" | "rings" | "face" | "mesh" | "clouds" | "air" | "flatten" | "pole">;

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
  if (!b.star && b.flatten && b.flatten > 0) ((out.flatten = b.flatten), (out.pole = b.pole));
  if (!b.star && b.clouds) out.clouds = { key: `${b.key}|clouds`, texture: b.clouds };
  if (!b.star && b.air) {
    const key = `${b.key}|air|${b.air.beta.join(",")}|${b.air.scale_height}`;
    if (!tables.has(key)) tables.set(key, airLuts(b.air));
    out.air = { key, shape: b.air, luts: tables.get(key)! };
  }
  const r = b.star ? null : ringsOf(b);
  if (r) out.rings = { key: r.key, inner: r.inner, outer: r.outer, normal: b.pole, width: r.width, data: r.data };
  if (b.figure && !b.star) {
    out.mesh = smallBodyMesh(b.figure, b.key);
    if (!b.surface) out.colour = smallBodyColour(b.figure);
  }
  return out;
}
