// The black hole view's small rules (XO4): which sky sits behind each of the kernel's holes, how each band's layer is
// scaled, and the words. Presentation only; every physical number comes from the kernel's `read_light_paths`.
import { constant, sayNumber, type SeenFrom } from "../../../ui-contract/src/index.ts";
import { BANDS, bandRadiance, linearSrgb, xyz } from "../../../ui-contract/src/index.ts";
import { rgbOf } from "../../../renderer/src/composite.ts";
import { PICTURE_BANDS, SHEET_BANDS_OF, type BandBrightest, type LightPathsRead, type PictureBand } from "../../../renderer/src/light-path.ts";
import type { HoleLayer } from "../../../renderer/src/gpu-light-path.ts";

/**
 * The sky behind each hole. The kernel's three holes are fixtures with no place in the galaxy, so each is drawn in
 * front of the sky in the direction of the real object it stands for, seen from this system's place (fudge register,
 * "The sky behind the black hole view"): Cygnus X-1 for the fed stellar hole, 3C 273 for the quasar, Sagittarius A*
 * for the starved centre. `cone_deg` is the half-width of the patch round that direction laid finer (rays passing near the
 * shadow are turned by tens of degrees, so the patch is wide); the rest of the sky is laid coarsely.
 */
export const BACKDROPS: Record<string, { l_deg: number; b_deg: number; cone_deg: number; why: string }> = {
  "fixture/stellar-hole": { l_deg: 71.33, b_deg: 3.07, cone_deg: 25, why: "The sky behind is the one in the direction of Cygnus X-1, an X-ray binary like this one: the fixture has no place in the galaxy of its own." },
  "fixture/quasar": { l_deg: 289.95, b_deg: 64.36, cone_deg: 25, why: "The sky behind is the one in the direction of the quasar 3C 273: the fixture has no place in the galaxy of its own." },
  "fixture/starved-nucleus": { l_deg: 359.94, b_deg: -0.05, cone_deg: 25, why: "The sky behind is the one towards the galaxy's centre, where this hole sits." },
};

/** Seen from (the one view controls): the viewer's tilt from the disc's axis, degrees. Edge-on is Luminet's view, 10° above the disc. */
export const SEEN_TILT_DEG: Partial<Record<SeenFrom, number>> = { above: 2, tilted: 60, edge: 80 };

export function tiltWords(deg: number): string {
  const above = 90 - deg;
  if (Math.abs(above) < 1) return "edge on";
  return `${sayNumber(Math.abs(above), 2)}° ${above > 0 ? "above" : "below"} the disc's plane`;
}

/** A number in words a GM can read: plain under ten thousand, else "3.2 × 10⁴". */
export function sci(x: number): string {
  if (!(x > 0) || !Number.isFinite(x)) return sayNumber(x, 2);
  const e = Math.floor(Math.log10(x));
  if (e >= -2 && e < 4) return sayNumber(x, 2);
  const sup = String(e).replace(/[-0-9]/g, (c) => "⁻⁰¹²³⁴⁵⁶⁷⁸⁹"["-0123456789".indexOf(c)]!);
  return `${sayNumber(x / 10 ** e, 2)} × 10${sup}`;
}

/**
 * The traced disc's share of the body's light in each band, from the kernel's own numbers: the disc's pieces (seen
 * temperature, area) through Planck over the sheet's bands, against the body's light by band (which adds the hot
 * flow's glow). For the kernel's viewer; used as the layer's weight, so a band whose light is mostly the hot flow (not
 * drawn point by point yet) is drawn as faint as the disc really is in it.
 */
export function bandShares(p: LightPathsRead): Record<PictureBand, number> | null {
  const pieces = (p.image as { pieces?: [number, number][] } | null | undefined)?.pieces;
  const bands = p.light?.bands;
  if (!pieces || !bands) return null;
  const out = {} as Record<PictureBand, number>;
  for (const b of PICTURE_BANDS) {
    let disc = 0;
    let all = 0;
    for (const id of SHEET_BANDS_OF[b]) {
      const sb = BANDS.find((x) => x.id === id);
      if (!sb) continue;
      for (const [t, area] of pieces) disc += area * Math.PI * bandRadiance({ address: "", continuum: [{ kind: "thermal", t_k: t, weight: 1 }], lines: [], stubbed: false, why: "" }, sb.from_m, sb.to_m, 24) * (sb.to_m - sb.from_m);
      all += bands.find((x) => x.band === id)?.power_w ?? 0;
    }
    out[b] = all > 0 ? Math.min(1, disc / all) : 0;
  }
  return out;
}

/** log10 of a white surface in the Sun's light at 1 au, as the band table's visible channels hold light. */
export function whiteLog(): number {
  return Math.log10(Math.max(...linearSrgb(xyz({ address: "", continuum: [{ kind: "thermal", t_k: constant("T_sun"), weight: 1 }], lines: [], stubbed: false, why: "" }))) * (constant("R_sun") / constant("au")) ** 2);
}

/**
 * The layers the GPU pass draws, each shown at 1 at its `top`: the finish's white (VP1: 1 is a white surface in
 * daylight). Visible in natural colour is exposed as the system's True is, a white surface in the Sun's light at 1 au
 * at 1, and dimmed further only when the disc is brighter than that (so a starved disc stays dark, a fed one is dimmed
 * until its brightest point is drawn as bright as white paper in sunlight); every other band is mapped to its own
 * brightest, times the disc's share of that band's light.
 */
/** Visible's own colours, as Composite names them; and the plain white a single mapped band is drawn in (its brightness alone). */
export const NATURAL = "natural";
export const GREY: [number, number, number] = [1, 1, 1];
export const isNatural = (l: { colour: unknown }) => l.colour === NATURAL;

export function layersFor(list: readonly { band: string; colour: string | [number, number, number] }[], bright: BandBrightest | null, shares: Record<PictureBand, number> | null): HoleLayer[] {
  const white = whiteLog();
  const out: HoleLayer[] = [];
  for (const l of list) {
    const band = l.band as PictureBand;
    if (!PICTURE_BANDS.includes(band)) continue;
    if (isNatural(l)) {
      out.push({ band, colour: "natural", top: Math.max(bright?.rgbTop ?? white, white) });
      continue;
    }
    const share = shares?.[band] ?? 1;
    out.push({ band, colour: Array.isArray(l.colour) ? l.colour : rgbOf(l.colour), top: (bright?.log[band] ?? 0) - Math.log10(Math.max(share, 1e-12)) });
  }
  return out;
}
