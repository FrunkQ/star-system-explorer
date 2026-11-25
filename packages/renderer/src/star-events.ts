// Flares and mass ejections you can see (SG3). Every number is the kernel's (read_activity, O10): each flare's peak
// time, rise and fall, and its white-light part (a 10,000 K patch of a stated area, `light.parts`); for the larger
// ones an ejection with its speed, mass, direction and cone. This file places the patch on the disc where the
// ejection leaves, lets the ejection's front travel outward at its speed, and works out how bright it is by Thomson
// scattering of the star's light off its electrons. It also holds the one safety rule for anything that flickers:
// never more than three flashes a second on the screen (exotic objects design, section 2.4), and no flashing at all
// when the viewer asks for reduced motion.
import { constant, planck } from "../../ui-contract/src/index.ts";
import { flareProfile, type Flare, type LiveStar } from "./live-star.ts";

export const EVENT_REFERENCES = [
  { text: "Benz, A. O. (2017). Flare Observations. Living Reviews in Solar Physics 14, 2 (the impulsive rise and gradual fall; white-light flare kernels near 10,000 K).", doi: "10.1007/s41116-016-0004-3" },
  { text: "Howard, T. (2011). Coronal Mass Ejections: An Introduction. Springer (Thomson-scattered brightness of an ejection, its fall with distance, the three-part bulb seen by coronagraphs).", doi: "10.1007/978-1-4419-8789-1" },
  { text: "Billings, D. E. (1966). A Guide to the Solar Corona. Academic Press (Thomson scattering by coronal electrons: brightness from the electron column and the star's light).", url: "https://archive.org/details/guidetosolarcoro0000bill" },
  { text: "W3C (2023). Web Content Accessibility Guidelines 2.2, success criterion 2.3.1, Three Flashes or Below Threshold.", url: "https://www.w3.org/TR/WCAG22/#three-flashes-or-below-threshold" },
];

/** Most flashes a second anything may show on the screen, whatever the real rate. */
export const FLASHES_PER_SECOND_MAX = 3;
/** Electrons per kilogram of ejected matter: fully ionised solar mix, μ_e about 1.17 proton masses per electron. */
const ELECTRONS_PER_KG = 1 / (1.17 * constant("m_p"));
const SIGMA_T = constant("sigma_T");
const SIGMA_SB = constant("sigma_SB");
/**
 * An ejection is drawn as a coronagraph shows it: its scattered light over the disc's, through a radial filter that
 * takes out the (r/R)² fall a coronagraph's filter removes, on a log scale from 10⁻¹¹ (nothing) over three decades.
 * In the eye an ejection is a billion times fainter than its star and could never be seen beside it.
 */
export const EJECTION_SCALE = { floor: 1e-11, decades: 3, peak: 0.8 };

/** A flare's white-light part from its light description: temperature and area, or null when it has none. */
export function flareWhiteLight(f: Flare): { t_k: number; area_m2: number } | null {
  const parts = (f.light?.parts ?? []) as { kind?: string; t_k?: number; area_m2?: number }[];
  const p = parts.find((q) => q.kind === "thermal" && Number(q.t_k) > 0 && Number(q.area_m2) > 0);
  return p ? { t_k: Number(p.t_k), area_m2: Number(p.area_m2) } : null;
}

/** The patch's angular radius on the star, degrees: a cap of the flare's area on a sphere of the star's radius. */
export function patchRadiusDeg(area_m2: number, radius_m: number): number {
  const c = 1 - area_m2 / (2 * Math.PI * radius_m * radius_m);
  return (Math.acos(Math.max(-1, Math.min(1, c))) * 180) / Math.PI;
}

/** How much brighter the patch is than the photosphere in red, green and blue: black bodies at the two temperatures. */
export const patchContrast = (t_k: number, teff_k: number) => [610e-9, 550e-9, 465e-9].map((l) => planck(l, t_k) / planck(l, teff_k));

export interface ShownFlare {
  flare: Flare;
  index: number;
  /** Its light now over its peak, on the screen's clock (0 to 1). */
  k: number;
}

/**
 * The flares to show at star time `t`, with the safety cap. `clock` is star seconds per screen second. Flares are
 * grouped by the third of a screen second their rise begins in, and only the largest of each group is shown; each
 * shown flare rises over at least a quarter of that and fades over at least all of it, so no flare is a single-frame
 * flash and no more than three begin each second. Under reduced motion nothing flashes: no flare is shown (the words
 * give the rate instead).
 */
/** One flashing event for the cap: when its rise starts, how big it is (the largest of a window wins), its rise and fall. */
export interface FlashEvent {
  start_s: number;
  size: number;
  rise_s: number;
  decay_s: number;
}

/**
 * The safety cap for anything that flashes, on any clock (stars' flares, aurora pulses, pulsars): `clock` is the
 * event's seconds per screen second. Events are grouped by the third of a screen second their rise begins in and only
 * the largest of each is kept; each kept one rises over at least a quarter of that and fades over at least all of it
 * (linear rise, exponential fall, gone below a fiftieth). Returns the kept events' indices with their light now over
 * their peak. Under reduced motion nothing flashes: an empty list.
 */
export function capFlashes(events: readonly FlashEvent[], t: number, clock: number, calm = false): { index: number; k: number }[] {
  if (calm || !events.length) return [];
  const w = Math.max(1e-9, clock / FLASHES_PER_SECOND_MAX);
  const best = new Map<number, number>();
  events.forEach((e, i) => {
    const win = Math.floor(e.start_s / w);
    const was = best.get(win);
    if (was === undefined || e.size > events[was]!.size) best.set(win, i);
  });
  const out: { index: number; k: number }[] = [];
  for (const i of best.values()) {
    const e = events[i]!;
    const rise = Math.max(e.rise_s, 0.25 * w);
    const k = flareProfile({ at_s: e.start_s + rise, rise_s: rise, decay_s: Math.max(e.decay_s, w), peak_w: 1, energy_j: 1 }, t);
    if (k > 0) out.push({ index: i, k });
  }
  return out.sort((a, b) => a.index - b.index);
}

/** The flares to show at star time `t`, through the safety cap (`capFlashes`), largest by energy. */
export function shownFlares(s: LiveStar, t: number, clock: number, calm = false): ShownFlare[] {
  return capFlashes(
    s.flares.map((f) => ({ start_s: f.at_s - f.rise_s, size: f.energy_j, rise_s: f.rise_s, decay_s: f.decay_s })),
    t,
    clock,
    calm,
  ).map(({ index, k }) => ({ flare: s.flares[index]!, index, k }));
}

/** The star's light now over its quiet light: one plus each shown flare's white light at its strength now. */
export function flareBoost(s: LiveStar, shown: ShownFlare[]): number {
  let extra = 0;
  for (const { flare, k } of shown) {
    const wl = flareWhiteLight(flare);
    if (wl) extra += k * wl.area_m2 * SIGMA_SB * wl.t_k ** 4;
  }
  return 1 + extra / Math.max(1, s.lum);
}

export interface EjectionNow {
  flare: Flare;
  /** Where the cone points, in the viewer's frame before the tilt: latitude on the star and longitude from the viewer. */
  lat_deg: number;
  lon_deg: number;
  half_width_deg: number;
  /** Its front's distance from the star's centre, in stellar radii. */
  r: number;
  /** Electrons per square metre through the front shell, seen face on. */
  column_m2: number;
}

/** Ejections under way at star time `t`, out to `reach` stellar radii (beyond that the picture cannot show them). */
export function ejectionsAt(s: LiveStar, t: number, reach: number, calm = false): EjectionNow[] {
  if (calm) return [];
  const out: EjectionNow[] = [];
  for (const f of s.flares) {
    const e = f.ejection;
    if (!e || t < f.at_s) continue;
    const r = 1 + (e.speed_m_s * (t - f.at_s)) / s.radius;
    if (r > reach) continue;
    const [dx, dy, dz] = e.direction;
    const cap = 2 * Math.PI * (1 - Math.cos(e.half_width_rad)) * (r * s.radius) ** 2;
    out.push({
      flare: f,
      lat_deg: (Math.asin(Math.max(-1, Math.min(1, dz))) * 180) / Math.PI,
      lon_deg: (Math.atan2(dy, dx) * 180) / Math.PI,
      half_width_deg: (e.half_width_rad * 180) / Math.PI,
      r,
      column_m2: (e.mass_kg * ELECTRONS_PER_KG) / cap,
    });
  }
  return out;
}

/**
 * An ejection's surface brightness over the star disc's mean, seen face on: the star's flux at the front, scattered by
 * the electron column, over the disc's intensity, σ_T N (R/r)² / 4 (optically thin, scattering taken as even in all
 * directions, a stand-in for Thomson's (1 + cos²χ)).
 */
export const ejectionShare = (e: Pick<EjectionNow, "column_m2" | "r">) => (SIGMA_T * e.column_m2) / (4 * e.r * e.r);

/** The value drawn for a share of the disc's light at r stellar radii (coronagraph scale, EJECTION_SCALE). */
export function ejectionShown(share: number, r: number): number {
  const filtered = share * r * r;
  const v = (Math.log10(Math.max(1e-30, filtered)) - Math.log10(EJECTION_SCALE.floor)) / EJECTION_SCALE.decades;
  return Math.max(0, Math.min(1, v)) * EJECTION_SCALE.peak;
}

const decode = (c: number) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
const encode = (c: number) => (c <= 0.0031308 ? 12.92 * c : 1.055 * c ** (1 / 2.4) - 0.055);

/**
 * Add the ejections to a square glow picture whose half-width is `reach` stellar radii, the star tilted `lat_view`
 * degrees toward the viewer. Each front is a thin shell at its distance, `shell` of it thick, inside its cone; the line
 * of sight crosses it at most twice, and the path through the shell brightens its rim, which gives the bright loop a
 * coronagraph sees. The far side is hidden behind the disc. White: scattered light keeps the star's colour, which the
 * glow already carries, so the shell is drawn in the star's own tint `rgb` (linear 0 to 1).
 */
export function addEjections(out: { width: number; data: Uint8ClampedArray }, ejections: EjectionNow[], reach: number, rgb: [number, number, number], lat_view = 7, shell = 0.15): void {
  if (!ejections.length) return;
  const n = out.width;
  const d = out.data;
  const c = n / 2;
  const t = (lat_view * Math.PI) / 180;
  const cones = ejections.map((e) => {
    const la = (e.lat_deg * Math.PI) / 180;
    const lo = (e.lon_deg * Math.PI) / 180;
    const x = Math.cos(la) * Math.sin(lo);
    const y0 = Math.sin(la);
    const z0 = Math.cos(la) * Math.cos(lo);
    return { e, v: [x, y0 * Math.cos(t) - z0 * Math.sin(t), y0 * Math.sin(t) + z0 * Math.cos(t)] as const, cosw: Math.cos((e.half_width_deg * Math.PI) / 180), face: ejectionShown(ejectionShare(e), e.r) };
  });
  for (let py = 0; py < n; py++)
    for (let px = 0; px < n; px++) {
      const x = ((px + 0.5 - c) / c) * reach;
      const y = (-(py + 0.5 - c) / c) * reach;
      const b2 = x * x + y * y;
      let v = 0;
      for (const k of cones) {
        const r = k.e.r;
        if (b2 >= r * r) continue;
        const zf = Math.sqrt(r * r - b2);
        for (const z of [zf, -zf]) {
          // Hidden behind the star's disc.
          if (z < 0 && b2 < 1) continue;
          const cosang = (x * k.v[0] + y * k.v[1] + z * k.v[2]) / r;
          if (cosang < k.cosw) continue;
          // Path through a shell `shell · r` thick, crossed at slant z / r, over the face-on path (capped at grazing).
          const slant = Math.min(6, r / Math.max(1e-3, Math.abs(z)));
          // Soft edge to the cone.
          const edge = Math.min(1, (cosang - k.cosw) / Math.max(1e-3, 1 - k.cosw) / 0.2);
          v += k.face * Math.min(1, (slant * shell) / 1) * edge;
        }
      }
      if (v <= 0) continue;
      const o = (py * n + px) * 4;
      v = Math.min(1, v);
      for (let ch = 0; ch < 3; ch++) d[o + ch] = 255 * encode(Math.min(1, decode((d[o + ch] ?? 0) / 255) + rgb[ch]! * v));
      d[o + 3] = 255 * encode(Math.min(1, decode((d[o + 3] ?? 0) / 255) + v));
    }
}

/** The star's flares and ejections in words: what shows now, and the rate when motion is reduced. */
export function narrateEvents(s: LiveStar, shown: ShownFlare[], ejections: EjectionNow[], calm: boolean): string {
  const span = Math.max(1, (s.flares.at(-1)?.at_s ?? s.age_s) - (s.flares[0]?.at_s ?? s.age_s));
  const perDay = s.flares.length ? (s.flares.length / span) * constant("day") : 0;
  const rate = !perDay ? "no flares above 10²⁴ J" : perDay >= 1 ? `${perDay.toFixed(perDay < 10 ? 1 : 0)} flares above 10²⁴ J a day` : `a flare above 10²⁴ J about every ${Math.round(1 / perDay).toLocaleString("en-GB")} days`;
  if (calm) return `Motion is reduced, so flares are not shown flashing: the star has ${rate}.`;
  const now = shown.length ? `${shown.length === 1 ? "A flare is" : `${shown.length} flares are`} going off now (white-hot patches from the kernel's light, at most three new ones a second on screen)` : `It has ${rate}`;
  const ej = ejections.length ? `; ${ejections.length === 1 ? "a mass ejection is" : `${ejections.length} mass ejections are`} leaving at ${Math.round(Math.max(...ejections.map((e) => e.flare.ejection!.speed_m_s)) / 1000).toLocaleString("en-GB")} km/s, drawn as a coronagraph sees it (about a billion times fainter than the star to the eye)` : "";
  return `${now}${ej}.`;
}
