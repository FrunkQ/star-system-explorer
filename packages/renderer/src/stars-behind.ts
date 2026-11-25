// The sky's stars behind a True view (VP2, visual polish design 5.2): the kernel's sky from the system
// (`sky_from_here`, galactic directions, luminosity and distance per star) turned into the system's own frame, each
// star's flux taken as a share of the flux that lights the exposure's white surface. True is exposed so a white
// surface under the Sun's light at 1 au reads white (the "solar constant" exposure); at that exposure most stars are far
// too faint to show beside a sunlit world, which is real (question 9). A long exposure lifts them and says it is a
// composite (`longGain`).
import { constant, thermalColour } from "../../ui-contract/src/index.ts";
import { toSystem, type SystemFrame } from "./sky.ts";

export interface SkyStar {
  /** Galactic unit direction. */
  dir: [number, number, number];
  luminosity_lsun: number;
  distance_pc: number;
  teff_k: number;
}

const decode = (c: number) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);

/** A temperature's colour as linear light, scaled so its channels sum to 3 (the light, not its brightness). */
export function starTint(teff_k: number): [number, number, number] {
  const h = (thermalColour(teff_k).match(/[0-9a-f]{2}/gi) ?? ["ff", "ff", "ff"]).map((x) => decode(parseInt(x, 16) / 255));
  const s = (h[0]! + h[1]! + h[2]!) / 3 || 1;
  return [h[0]! / s, h[1]! / s, h[2]! / s];
}

/**
 * The stars in the system's frame, with each one's flux over the reference: (L / L_sun) / (d / au)², per channel by
 * its colour. Bolometric, as the night sky on the ground (fudge register).
 */
export function starsBehind(stars: readonly SkyStar[], frame: Pick<SystemFrame, "to_galactic">): { dirs: Float32Array; share: Float32Array } {
  const dirs = new Float32Array(stars.length * 3);
  const share = new Float32Array(stars.length * 3);
  const auPerPc = constant("pc") / constant("au");
  stars.forEach((s, i) => {
    const v = toSystem(frame as SystemFrame, s.dir);
    dirs.set(v, i * 3);
    const d = s.distance_pc * auPerPc;
    const f = d > 0 ? s.luminosity_lsun / (d * d) : 0;
    const c = starTint(s.teff_k);
    share.set([f * c[0], f * c[1], f * c[2]], i * 3);
  });
  return { dirs, share };
}

/**
 * The gain a long exposure takes: enough that the `n`th brightest star fills its pixel to white (π · share / Ω_pixel
 * times the gain is 1), never below 1. A camera choice, not the eye's (camera effect grade).
 */
export function longGain(share: Float32Array, pixel_rad: number, n = 40): number {
  const lum: number[] = [];
  for (let i = 0; i < share.length; i += 3) lum.push((share[i]! + share[i + 1]! + share[i + 2]!) / 3);
  lum.sort((a, b) => b - a);
  const f = lum[Math.min(n, lum.length) - 1] ?? 0;
  const k = Math.PI / (pixel_rad * pixel_rad);
  return f > 0 ? Math.max(1, 1 / (k * f)) : 1;
}
