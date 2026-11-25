// The corona in visible light (VP-S3): the star's outer atmosphere, a million times fainter than its disc, seen only
// when the disc is hidden (by a world in front, or a coronagraph). Its brightness with height is Baumbach's (1937) fit
// to eclipse photometry; its shape follows the activity cycle, as eclipse photographs show it: at a quiet point long
// streamers lie along the equator and the poles are dark with thin plumes; at an active point it is bright all round
// (Golub and Pasachoff 2010). The cycle's phase is the kernel's (`cycleOf`); a star with no clean cycle (saturated,
// always active) is drawn round.
import { thermalColour } from "../../ui-contract/src/index.ts";
import { cycleOf, type LiveStar } from "./live-star.ts";

export const CORONA_REFERENCES = [
  { text: "Baumbach, S. (1937). Strahlung, Ergiebigkeit und Elektronendichte der Sonnenkorona. Astron. Nachr. 263, 121 (I/I☉ = 10⁻⁶ (0.0532 r^−2.5 + 1.425 r^−7 + 2.565 r^−17), r in solar radii, mean of the disc).", url: "https://ui.adsabs.harvard.edu/abs/1937AN....263..121B" },
  { text: "Golub, L. and Pasachoff, J. M. (2010). The Solar Corona, 2nd ed. Cambridge University Press (streamers along the equator at minimum, a round corona at maximum).", url: "https://www.cambridge.org/core/books/solar-corona/" },
];

/** Baumbach's corona at r stellar radii from the centre, over the disc's mean brightness. */
export const baumbach = (r: number) => 1e-6 * (0.0532 * r ** -2.5 + 1.425 * r ** -7 + 2.565 * r ** -17);

/** How active the star is in its cycle, 0 at the quiet point to 1 at the peak; 1 with no cycle (always active). */
export function cycleActivity(s: LiveStar): number {
  const c = cycleOf(s);
  return c ? (1 - Math.cos(2 * Math.PI * c.phase)) / 2 : 1;
}

/** Shape across position angle: at a quiet point most light lies along the equator; at the peak it is round. */
export function coronaShape(pa_from_equator_rad: number, activity: number): number {
  const quiet = 1 - Math.max(0, Math.min(1, activity));
  const s2 = Math.sin(pa_from_equator_rad) ** 2;
  // Streamers stretch the light toward the equator the further out one looks: at a quiet point the poles keep about
  // a fifth of the equator's light (stand-in from eclipse pictures).
  return 1 - 0.8 * quiet * s2;
}

const decode = (c: number) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
const encode = (c: number) => (c <= 0.0031308 ? 12.92 * c : 1.055 * c ** (1 / 2.4) - 0.055);

/** The corona is shown on a log scale this many decades deep from its brightest point at the limb, as eclipse photographs are processed. */
export const CORONA_DECADES = 3;

/**
 * Draw the hidden-disc view: the disc black (occulted), the corona round it out to the square's edge, `disc` the
 * disc's share of the square, the pole tilted `tilt_deg` in the picture (0: pole up). Returns the cycle activity used.
 */
export function drawCorona(out: { width: number; data: Uint8ClampedArray }, s: LiveStar, disc = 0.3, tilt_deg = 0): number {
  const n = out.width;
  const d = out.data;
  const c = n / 2;
  const R = c * disc;
  const act = cycleActivity(s);
  // Scattered starlight keeps the star's colour (the K corona is Thomson scattering by free electrons).
  const col = (thermalColour(s.teff).match(/[0-9a-f]{2}/gi) ?? ["ff", "ff", "ff"]).map((h) => decode(parseInt(h, 16) / 255));
  const top = baumbach(1);
  const tilt = (tilt_deg * Math.PI) / 180;
  for (let py = 0; py < n; py++)
    for (let px = 0; px < n; px++) {
      const o = (py * n + px) * 4;
      const x = (px + 0.5 - c) / R;
      const y = -(py + 0.5 - c) / R;
      const r = Math.hypot(x, y);
      if (r < 1) {
        d[o] = d[o + 1] = d[o + 2] = 0;
        d[o + 3] = 255;
        continue;
      }
      // Position angle from the equator, with the pole tilted in the picture.
      const pa = Math.atan2(y * Math.cos(tilt) - x * Math.sin(tilt), x * Math.cos(tilt) + y * Math.sin(tilt));
      const i = baumbach(r) * coronaShape(pa, act) ** Math.min(3, r);
      const v = Math.max(0, 1 + Math.log10(i / top) / CORONA_DECADES);
      d[o] = 255 * encode(col[0]! * v);
      d[o + 1] = 255 * encode(col[1]! * v);
      d[o + 2] = 255 * encode(col[2]! * v);
      d[o + 3] = 255;
    }
  return act;
}

/** The hidden-disc view in words. */
export function narrateCorona(activity: number, hasCycle: boolean): string {
  const shape = !hasCycle ? "round, as the star is always active" : activity < 0.35 ? "stretched into long streamers along the equator, with dark poles, as at a quiet point in its cycle" : activity > 0.65 ? "bright all round, as at the peak of its cycle" : "partly stretched along the equator, between the quiet point and the peak of its cycle";
  return `The disc is hidden, as a world in front or a coronagraph would hide it, so the corona shows: about a millionth of the disc's brightness, falling away with height as eclipses measure it (Baumbach), drawn on a log scale ${CORONA_DECADES} decades deep. It is ${shape}.`;
}
