// True in the system view (V10a): each world wrapped in its own surface, read from the same level-0 tiles the Body
// globe draws (surface.ts `prepare`), so a world looks the same from the system as close up. This turns a prepared
// globe into a small equirectangular picture for a lit sphere. Nothing is invented: a world whose tiles are not read
// yet, or that has none (a giant, until cloud colours come), is drawn plain and says so.
import { planck, type Globe, type SurfaceTile } from "../../ui-contract/src/index.ts";
import { starRgb } from "./sky.ts";
import { fastFace, lookupFast, prepare, type Prepared } from "./surface.ts";

export interface TrueTexture {
  width: number;
  height: number;
  /** RGBA, row 0 at the south pole (three.js DataTexture order). */
  data: Uint8Array;
  /** The disc's mean colour (sRGB 0..255), for the canvas tier and the plain fallback. */
  mean: [number, number, number];
}

/**
 * The prepared globe as an equirectangular picture, laid out for a three.js sphere whose poles are turned onto z
 * (SphereGeometry rotated +90° about x): pixel (i, j) looks along (-cos 2πu · sin θ, -sin 2πu · sin θ, cos θ), with
 * u = (i + ½)/w and θ = π(1 - (j + ½)/h). The globe's own pole is +z (ui-contract globe.ts).
 */
export function equirect(p: Prepared, width = 128, height = 64): TrueTexture {
  const data = new Uint8Array(width * height * 4);
  let r = 0;
  let g = 0;
  let b = 0;
  let n = 0;
  for (let j = 0; j < height; j++) {
    const theta = Math.PI * (1 - (j + 0.5) / height);
    const s = Math.sin(theta);
    const z = Math.cos(theta);
    for (let i = 0; i < width; i++) {
      const u = (i + 0.5) / width;
      const x = -Math.cos(2 * Math.PI * u) * s;
      const y = -Math.sin(2 * Math.PI * u) * s;
      const k = lookupFast(p, x, y, z);
      const o = (j * width + i) * 4;
      // A face with no tile stays mid grey: shown, not invented.
      const f = k >= 0 ? lastFace(p) : null;
      const c0 = f ? f.rgb[k * 3]! : 128;
      const c1 = f ? f.rgb[k * 3 + 1]! : 128;
      const c2 = f ? f.rgb[k * 3 + 2]! : 128;
      data[o] = c0;
      data[o + 1] = c1;
      data[o + 2] = c2;
      data[o + 3] = 255;
      // Weight by area on the sphere for the mean.
      r += c0 * s;
      g += c1 * s;
      b += c2 * s;
      n += s;
    }
  }
  return { width, height, data, mean: [Math.round(r / n), Math.round(g / n), Math.round(b / n)] };
}

// `lookupFast` leaves the face it found in the live binding `fastFace`.
const lastFace = (p: Prepared) => (fastFace >= 0 ? p.faces[fastFace] : null);

/** A world's level-0 tiles (kernel `resolved_system` tiles) as its true surface, or null when it has none. */
export function trueSurface(tiles: readonly SurfaceTile[], width = 128, height = 64): TrueTexture | null {
  const globe: Globe = [0, 1, 2, 3, 4, 5].map((f) => tiles.find((t) => t.key.face === f && t.key.level === 0));
  if (!globe.some((t) => t)) return null;
  // The true-colour recipe's ground colours (surface.ts: any colour but "height" reads the tiles' own).
  return equirect(prepare(globe, "survey", 1), width, height);
}

/** A star's own colour as a CSS colour: its surface temperature's (sky.ts `starRgb`, linear) encoded to sRGB. */
export function starCss(t_k: number): string {
  const enc = (v: number) => Math.round(255 * (v <= 0.0031308 ? 12.92 * v : 1.055 * Math.pow(v, 1 / 2.4) - 0.055));
  const [r, g, b] = starRgb(t_k);
  return `rgb(${enc(r)}, ${enc(g)}, ${enc(b)})`;
}

/** Thermal infrared for True's Infrared band: 10 µm, where a world's own heat outshines the starlight it reflects. */
export const THERMAL_IR_M = 10e-6;
/** Decades of surface brightness the mapped grey spans, below the star's. */
export const THERMAL_DECADES = 6;

/**
 * A body's surface brightness in thermal infrared as a mapped grey, 0..1: its blackbody radiance at THERMAL_IR_M
 * over the star's, on a log scale THERMAL_DECADES deep (the star is 1). Mapped, not seen: infrared has no colour.
 */
export function thermalGrey(t_k: number, star_k: number): number {
  if (!(t_k > 0 && star_k > 0)) return 0;
  const r = planck(THERMAL_IR_M, t_k) / planck(THERMAL_IR_M, star_k);
  return Math.max(0, Math.min(1, 1 + Math.log10(Math.max(r, 1e-30)) / THERMAL_DECADES));
}
