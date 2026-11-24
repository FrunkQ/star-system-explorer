// A body's face in True: one door for every view that draws a body in True (the system's 3D, the size comparison's
// lineup and gallery), so a look added here (a giant's bands, a haze, plumes, frosts, a star's glow) shows in all of
// them at once with no second code path. It takes what is known of the body and gives the GpuItem fields that draw it;
// the view keeps only where the body is and how big. Pure, apart from a cache of ring profiles by key.
import type { GpuItem } from "./gpu.ts";
import type { Vec3 } from "./frames.ts";
import { ringProfile, type RingBand } from "./rings.ts";
import type { TrueTexture } from "./true-look.ts";

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
}

/** The GPU fields of a body's True face. */
export type TrueLook = Pick<GpuItem, "colour" | "shines" | "surface" | "rings">;

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

/**
 * The rings' profile for a body, worked out once per body, rings and surface: their colour is the world's own mean
 * surface colour when its surface has been read, else a neutral grey (the ring data carries no colour yet: fudge
 * register, "Rings in True").
 */
function ringsOf(b: TrueBody): (Profile & { key: string }) | null {
  if (!b.rings?.length || !(b.radius_m && b.radius_m > 0)) return null;
  const key = `${b.key}|${JSON.stringify(b.rings)}|${b.radius_m}|${b.surface ? "s" : "-"}`;
  if (!profiles.has(key)) {
    const p = ringProfile(b.rings, b.surface ? meanColour(b.surface) : [0.5, 0.5, 0.5]);
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
  if (b.surface) out.surface = { key: b.key, texture: b.surface };
  const r = b.star ? null : ringsOf(b);
  if (r) out.rings = { key: r.key, inner: r.inner, outer: r.outer, normal: b.pole, width: r.width, data: r.data };
  return out;
}
