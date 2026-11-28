// Prominences at the limb (SG6). Every number is the kernel's (read_activity with `prominences`, sse-activity
// prominence.rs): where on the star each one stands, how high and long it is, when it was born and how long it lives,
// whether it ends by erupting and how fast, and how bright it is against the disc. This file only places each one on
// the turning star at a time and draws it into the star's halo picture: an arch of cool gas over the limb, or a
// slingshot cloud riding round a fast spinner at its co-rotation radius.
//
// What is seen is honest to the light: prominences glow in hydrogen lines at about a ten-thousandth of the disc's
// brightness, so the eye never sees them beside the bare star (only in an eclipse or through a hydrogen-alpha filter).
// They are drawn as such a filter shows them, on a stated log scale, and only where they stand off the disc: on the
// disc they are dark threads in hydrogen light and invisible in white light.
import { sayNumber } from "../../ui-contract/src/index.ts";
import { srgb255, linear255 } from "./srgb.ts";
import { rateShare, turnAt, type LiveStar } from "./live-star.ts";
import { rateShareOf } from "./star-regions.ts";
import { nebulaColour, type NebulaShape } from "./star-shells.ts";

export interface Prominence {
  kind: "active" | "quiescent" | "slingshot" | string;
  born_s: number;
  life_s: number;
  lat_deg: number;
  lon_deg: number;
  tilt_deg: number;
  height_m: number;
  length_m: number;
  thickness_m: number;
  mass_kg: number;
  temperature_k: number;
  share_visible: number;
  light?: { lines?: { line: string; power_w: number }[] };
  eruption?: { at_s: number; speed_m_s: number };
}

/**
 * Drawn as a hydrogen-alpha filter shows them: surface brightness over the disc's mean on a log scale from 10⁻⁶
 * (nothing) over two and a half decades, so the Sun's (about 10⁻⁴) show clearly and a faint one faintly. A display
 * choice, like the ejections' coronagraph scale (fudge register, SG6).
 */
export const PROMINENCE_SCALE = { floor: 1e-6, decades: 2.5, peak: 0.9 };

export const PROMINENCE_REFERENCES = [
  { text: "Tandberg-Hanssen, E. (1995). The Nature of Solar Prominences. Kluwer (cool gas held in magnetic loops; seen bright at the limb and dark on the disc in hydrogen-alpha).", doi: "10.1007/978-94-017-3396-0" },
  { text: "Collier Cameron, A. and Robinson, R. D. (1989). Fast H-alpha variations on a rapidly rotating, cool main-sequence star II. MNRAS 238, 657 (slingshot clouds at the co-rotation radius).", doi: "10.1093/mnras/238.2.657" },
];

export interface ProminenceNow {
  p: Prominence;
  /** Its strength now, 0 to 1: grows in, fades out, thins as an eruption carries it off. */
  k: number;
  /** Where its spine's middle stands, in the viewer's frame before the tilt (latitude, longitude from the viewer). */
  lat_deg: number;
  lon_deg: number;
  /** Its top's height over the surface now (risen when erupting), stellar radii. */
  h: number;
  /** Half its length along the spine, as an angle on the star, radians. */
  half_rad: number;
  erupting: boolean;
}

/** Share of a life spent growing in and fading out. */
const EDGE = 0.1;

/** The prominences standing at star time `t`, out to `reach` stellar radii. Under reduced motion, held at the star's present. */
export function prominencesAt(s: LiveStar, t: number, reach: number, calm = false): ProminenceNow[] {
  const at = calm ? s.age_s : t;
  const out: ProminenceNow[] = [];
  const period = Number(s.activity.rotation_period_s ?? 0);
  for (const p of s.prominences ?? []) {
    if (at < p.born_s) continue;
    const end = p.born_s + p.life_s;
    let h = p.height_m / s.radius;
    let k: number;
    let erupting = false;
    if (at <= end) {
      const age = (at - p.born_s) / Math.max(1, p.life_s);
      k = Math.min(1, age / EDGE, p.eruption ? 1 : (1 - age) / EDGE);
    } else if (p.eruption) {
      // Lifting off: risen at its speed, spread out as it goes, so fainter with the square of its distance.
      const r0 = 1 + h;
      const r = r0 + (p.eruption.speed_m_s * (at - end)) / s.radius;
      if (r > reach) continue;
      h = r - 1;
      k = (r0 / r) ** 2;
      erupting = true;
    } else continue;
    if (k <= 0) continue;
    // Carried round at its latitude's own rate from where it was born (drifting against the equator, whose turn
    // `turnAt` follows), then seen against the star's turn now.
    const drift = period > 0 ? ((s.regions ? rateShareOf(s.regions.differential, p.lat_deg) : rateShare(p.lat_deg)) - 1) * 360 * ((at - p.born_s) / period) : 0;
    const lon = p.lon_deg + drift - turnAt(s, at);
    out.push({ p, k, lat_deg: p.lat_deg, lon_deg: ((((lon + 180) % 360) + 360) % 360) - 180, h, half_rad: Math.min(Math.PI / 2, p.length_m / (2 * s.radius)), erupting });
  }
  return out;
}

/** The value drawn for a prominence's share of the disc's light (hydrogen-alpha filter scale). */
export function prominenceShown(share: number): number {
  const v = (Math.log10(Math.max(1e-30, share)) - Math.log10(PROMINENCE_SCALE.floor)) / PROMINENCE_SCALE.decades;
  return Math.max(0, Math.min(1, v)) * PROMINENCE_SCALE.peak;
}

/** A prominence's colour from its own hydrogen lines (linear, brightest channel 1): red hydrogen-alpha, a touch of blue-green hydrogen-beta. */
export const prominenceColour = (p: Prominence): [number, number, number] => nebulaColour({ shells: [], light: { lines: p.light?.lines ?? [] } } as unknown as NebulaShape, "visible");


/** An erupting rope's radius over its height above the surface: the aspect ratio of Thernisien et al.'s croissant
 * model of coronal mass ejections (about 0.3 to 0.5; 0.4 here, from memory, docs/fudges.md). */
export const ROPE_ASPECT = 0.4;

type V3 = [number, number, number];
const unit = (la: number, lo: number): V3 => [Math.cos(la) * Math.sin(lo), Math.sin(la), Math.cos(la) * Math.cos(lo)];

/**
 * Points along one prominence in the viewer's frame (stellar radii, before the tilt), each with its share of the
 * light. A loop prominence is an arch over its spine: a few threads (the field lines holding the gas) side by side
 * across its thickness, the gas sagging into each from its top; a slingshot cloud is a ball of gas at its height.
 */
export function prominencePoints(n: ProminenceNow, radius_m: number, steps = 48): { at: V3; w: number; size?: number }[] {
  const la = (n.lat_deg * Math.PI) / 180;
  const lo = (n.lon_deg * Math.PI) / 180;
  const d = unit(la, lo);
  const east: V3 = [Math.cos(lo), 0, -Math.sin(lo)];
  const north: V3 = [-Math.sin(la) * Math.sin(lo), Math.cos(la), -Math.sin(la) * Math.cos(lo)];
  const tl = (n.p.tilt_deg * Math.PI) / 180;
  const along: V3 = [0, 1, 2].map((i) => Math.cos(tl) * east[i]! + Math.sin(tl) * north[i]!) as V3;
  const across: V3 = [0, 1, 2].map((i) => -Math.sin(tl) * east[i]! + Math.cos(tl) * north[i]!) as V3;
  const out: { at: V3; w: number; size?: number }[] = [];
  const thick = n.p.thickness_m / radius_m;
  if (n.p.kind === "slingshot") {
    const r = 1 + n.h;
    const size = Math.max(thick, 0.02);
    for (let i = 0; i < steps; i++) {
      // A fixed scatter of points through a ball (golden-angle spiral): the cloud keeps its shape from picture to picture.
      const z = 1 - (2 * (i + 0.5)) / steps;
      const ph = i * 2.399963;
      const q = Math.sqrt(1 - z * z) * Math.cbrt((i + 0.5) / steps);
      const o: V3 = [q * Math.cos(ph), q * Math.sin(ph), z * Math.cbrt((i + 0.5) / steps)];
      out.push({ at: [0, 1, 2].map((j) => d[j]! * r + o[j]! * size) as V3, w: 1 / steps });
    }
    return out;
  }
  const threads = 3;
  if (n.erupting) {
    // Lifting off: the field holding the gas balloons out as it rises, its feet still tied to the surface. Its axis
    // is the circle through both feet and its risen top (in the plane of its spine): low down that is the standing
    // arch; risen a radius or more it is as wide as it is tall. Round the axis the rope swells too, its radius a
    // fixed share of its height (the croissant of Thernisien et al.'s model of coronal mass ejections), so from any
    // side it is the light bulb coronagraphs see, never a rod.
    const top = 1 + n.h;
    const sh = Math.sin(n.half_rad);
    const ch = Math.cos(n.half_rad);
    const yc = (top * top - 1) / (2 * (top - ch));
    const rho = top - yc;
    const a0 = Math.atan2(ch - yc, sh);
    const a1 = Math.PI - a0;
    const ring = 8;
    for (let i = 0; i <= steps; i++) {
      const ang = a0 + ((a1 - a0) * i) / steps;
      const u = rho * Math.cos(ang);
      const v = yc + rho * Math.sin(ang);
      const axis: V3 = [0, 1, 2].map((j) => d[j]! * v + along[j]! * u) as V3;
      const out_: V3 = [0, 1, 2].map((j) => d[j]! * Math.sin(ang) + along[j]! * Math.cos(ang)) as V3;
      const lift = Math.max(0, Math.hypot(axis[0], axis[1], axis[2]) - 1);
      const cross = thick / 2 + ROPE_ASPECT * lift;
      for (let k = 0; k < ring; k++) {
        const psi = (2 * Math.PI * k) / ring;
        const o: V3 = [0, 1, 2].map((j) => cross * (Math.cos(psi) * across[j]! + Math.sin(psi) * out_[j]!)) as V3;
        out.push({ at: [axis[0] + o[0], axis[1] + o[1], axis[2] + o[2]], w: 1 / (ring * (steps + 1)), size: Math.max(thick, cross) });
      }
    }
    return out;
  }
  for (let th = 0; th < threads; th++) {
    const off = ((th + 0.5) / threads - 0.5) * thick;
    for (let i = 0; i <= steps; i++) {
      const s = (2 * i) / steps - 1;
      const phi = s * n.half_rad;
      const base: V3 = [0, 1, 2].map((j) => d[j]! * Math.cos(phi) + along[j]! * Math.sin(phi) + across[j]! * off) as V3;
      const bl = Math.hypot(base[0], base[1], base[2]);
      // The arch: highest over the spine's middle, its feet on the surface.
      const r = 1 + n.h * (1 - s * s);
      out.push({ at: [base[0] / bl * r, base[1] / bl * r, base[2] / bl * r], w: 1 / (threads * (steps + 1)) });
    }
  }
  return out;
}

/**
 * Add the prominences to a square glow picture whose half-width is `reach` stellar radii, the star tilted `lat_view`
 * degrees toward the viewer. A point is drawn only where it stands off the disc: behind the disc it is hidden; in
 * front of it, it is a filament, dark in hydrogen light and lost in white light.
 */
export function addProminences(out: { width: number; data: Uint8ClampedArray }, list: ProminenceNow[], radius_m: number, reach: number, lat_view = 7): number {
  if (!list.length) return 0;
  const n = out.width;
  const d = out.data;
  const c = n / 2;
  const t = (lat_view * Math.PI) / 180;
  const px = c / reach;
  let drawn = 0;
  // Each prominence's light is laid down as the brightest of its soft spots at each pixel (not their sum), so a
  // densely sampled arch is as bright as its gas, never brighter; then added to the picture in its own colour.
  const a = new Float32Array(n * n);
  for (const p of list) {
    // A risen rope's gas is spread thinner (k falls as the square of its distance): that is its surface brightness,
    // so it goes through the filter's log scale; growing in and fading out are a fade of the picture.
    const v = p.erupting ? prominenceShown(p.p.share_visible * p.k) : prominenceShown(p.p.share_visible) * p.k;
    if (v <= 0) continue;
    const rgb = prominenceColour(p.p);
    // A soft spot as wide as the gas is thick, at least a pixel and a half.
    const rad0 = Math.max(1.5, (p.p.thickness_m / radius_m) * px * 0.5);
    // Enough points along it that their spots overlap: a risen loop stays a line, never a row of beads.
    const span = p.erupting ? Math.PI * (1 + p.h) : 2 * p.half_rad * (1 + p.h);
    const pts = prominencePoints(p, radius_m, Math.min(384, Math.max(48, Math.ceil((span * px) / rad0))));
    let x0 = n, x1 = -1, y0 = n, y1 = -1;
    for (const { at, size } of pts) {
      // A point of a swollen, risen rope is as wide as the rope is there.
      const rad = size ? Math.min(24, Math.max(rad0, size * px * 0.5)) : rad0;
      const x = at[0];
      const y = at[1] * Math.cos(t) - at[2] * Math.sin(t);
      // Behind the disc or in front of it: hidden, or a filament lost in white light.
      if (x * x + y * y < 1) continue;
      const cx = c + x * px;
      const cy = c - y * px;
      const r = Math.ceil(rad * 2);
      for (let yy = Math.max(0, Math.floor(cy - r)); yy <= Math.min(n - 1, Math.ceil(cy + r)); yy++)
        for (let xx = Math.max(0, Math.floor(cx - r)); xx <= Math.min(n - 1, Math.ceil(cx + r)); xx++) {
          const dd = ((xx + 0.5 - cx) ** 2 + (yy + 0.5 - cy) ** 2) / (rad * rad);
          if (dd > 4) continue;
          // Stand-off: a pixel on the disc stays the disc's.
          const bx = (xx + 0.5 - c) / px;
          const by = -(yy + 0.5 - c) / px;
          if (bx * bx + by * by < 1) continue;
          const i = yy * n + xx;
          const w = v * Math.exp(-dd);
          if (w > a[i]!) a[i] = w;
          if (xx < x0) x0 = xx;
          if (xx > x1) x1 = xx;
          if (yy < y0) y0 = yy;
          if (yy > y1) y1 = yy;
        }
    }
    if (x1 < 0) continue;
    drawn++;
    for (let yy = y0; yy <= y1; yy++)
      for (let xx = x0; xx <= x1; xx++) {
        const i = yy * n + xx;
        const w = a[i]!;
        if (!(w > 0)) continue;
        a[i] = 0;
        const o = i * 4;
        for (let ch = 0; ch < 3; ch++) d[o + ch] = srgb255(linear255(d[o + ch] ?? 0) + rgb[ch]! * w);
        d[o + 3] = srgb255(linear255(d[o + 3] ?? 0) + w);
      }
  }
  return drawn;
}

/**
 * The prominences in words, kept short (it sits in the view's note): how many stand at the limb now and the field
 * holding them. Nothing when none shows. How faint they really are is in the star's card, not here.
 */
export function narrateProminences(s: LiveStar, now: ProminenceNow[], limb: number, reach?: number): string {
  if (!limb) return "";
  const c = s.corona as { surface_field_t?: number } | undefined;
  const gauss = Number(c?.surface_field_t ?? 0) * 1e4;
  // An eruption crosses the picture in minutes to hours: say how fast, and how long it stays in view, so the clock
  // can be slowed to watch it go.
  const e = now.find((p) => p.erupting && p.p.eruption);
  const speed = e ? e.p.eruption!.speed_m_s : 0;
  const left = e && reach && speed > 0 ? ((reach - 1 - e.h) * s.radius) / speed : 0;
  const up = e ? `, one erupting at ${sayNumber(speed / 1e3, 2)} km/s${left > 0 ? ` (out of the picture in ${left < 5400 ? `${Math.max(1, Math.round(left / 60))} minutes` : `${sayNumber(left / 3600, 2)} hours`})` : ""}` : "";
  return `${limb === 1 ? "A prominence" : `${limb} prominences`} at the limb${up}: cool gas held by its ${gauss > 0 ? `${sayNumber(gauss, 2)} gauss ` : ""}field, as a hydrogen-alpha filter shows it.`;
}
