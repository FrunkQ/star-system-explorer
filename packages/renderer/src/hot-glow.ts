// Hot things glowing in Visible (third audit R11, R12): a world's own thermal light in the true look, by Planck.
//
// True lights every world as if it stood in full daylight from its star (gpu.ts: no fall-off across the frame, the
// distances being drawn on a log scale), so the eye is adjusted to that world's daylight. The world's own glow is
// shown at the same exposure: its blackbody radiance seen by the eye (CIE 1931, 380 to 780 nm) over the radiance of
// a white diffuse surface in its star's light, which is the star's radiance times (R★/d)². Nothing is boosted: a
// 1500 K night side beside a Sun-like star is a few thousandths of its day side and shows only as a dull ember,
// while a brown dwarf far out from its star, or a young giant still hot from forming, outshines its own daylight.
import { DRAPER_K } from "../../ui-contract/src/index.ts";
import { linearSrgb, xyz, type LightDescription } from "../../ui-contract/src/light.ts";

const thermal = (t_k: number): LightDescription => ({ address: "", continuum: [{ kind: "thermal", t_k, weight: 1 }], lines: [], stubbed: false, why: "" });

export interface HotGlow {
  /** Linear sRGB radiance in units of a white diffuse surface in the world's own daylight (1 is that white). */
  rgb: [number, number, number];
  /** The glow's luminance over that white's (CIE Y over Y), unclipped: how bright the glow is against daylight. */
  ratio: number;
  /** One sentence for the note. */
  why: string;
}

/**
 * The world's own visible light at temperature `t_k`, against daylight from a star at `star_k` of radius
 * `star_r_m` seen from `d_m`. With no star (or no distance), there is no daylight to compare with: the glow is shown
 * at its own colour at full brightness, as the only light there is. Below the Draper point a surface gives the eye
 * nothing, so it is null; null too when the temperature is unknown.
 */
export function hotGlow(t_k: number | null | undefined, star?: { t_k: number; r_m: number; d_m: number } | null): HotGlow | null {
  if (!(typeof t_k === "number" && t_k >= DRAPER_K)) return null;
  const own = linearSrgb(xyz(thermal(t_k)));
  const lit = star && star.t_k > 0 && star.r_m > 0 && star.d_m > star.r_m;
  if (!lit) {
    const m = Math.max(...own);
    return { rgb: own.map((v) => Math.max(0, v / m)) as [number, number, number], ratio: Infinity, why: `It glows by its own heat (${Math.round(t_k)} K), the only light it has.` };
  }
  const dil = (star.r_m / star.d_m) ** 2;
  const white = xyz(thermal(star.t_k))[1] * dil;
  const ratio = xyz(thermal(t_k))[1] / white;
  // Over its daylight the glow is clipped as light is: colour kept, the brightest channel at one.
  const scaled = own.map((v) => Math.max(0, v / white)) as [number, number, number];
  const top = Math.max(...scaled);
  const rgb = top > 1 ? (scaled.map((v) => v / top) as [number, number, number]) : scaled;
  const how = ratio >= 1 ? "brighter than its own daylight" : ratio >= 0.01 ? `${pct(ratio)} of its daylight` : `${sci(ratio)} of its daylight, a faint ember on the night side`;
  return { rgb, ratio, why: `It glows by its own heat (${Math.round(t_k)} K), ${how} (Planck, seen by the eye).` };
}

/** Linear radiance under this (about 1/255 once encoded, bottom step) is no glow on any screen: not drawn. */
export const GLOW_FLOOR = 1 / 3300;

/** The glow worth drawing: null when its brightest channel is under GLOW_FLOOR. */
export function drawnGlow(g: HotGlow | null): HotGlow | null {
  return g && Math.max(...g.rgb) >= GLOW_FLOOR ? g : null;
}

const pct = (r: number) => `${Math.round(r * 100)}%`;
const sci = (r: number) => {
  const e = Math.floor(Math.log10(r));
  return `about ${Math.round(r / 10 ** e)}×10^${e}`;
};
