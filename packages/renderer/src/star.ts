// Stars drawn from their physics (R5a): a limb-darkened photosphere in the
// star's own colour, spots where the activity cycle puts them, and the corona
// in X-rays. The numbers come from the kernel: effective temperature, radius,
// mass and age from the system's facts, and spot coverage, cycle phase, X-ray
// power, spin and flares from the light track's `read_activity` (O10). This
// file turns them into pixels; where it has to choose something the kernel
// does not publish (spot places, spot temperature, coronal temperature) the
// choice is named here and in docs/fudges.md.
import { planck, thermalColour } from "../../ui-contract/src/index.ts";

export const STAR_REFERENCES = [
  { text: "Eddington, A. S. (1926). The Internal Constitution of the Stars. Cambridge University Press (grey atmosphere limb darkening, I(mu)/I(1) = (2 + 3 mu) / 5).", url: "https://archive.org/details/internalconstitu0000eddi" },
  { text: "Hathaway, D. H. (2015). The Solar Cycle. Living Reviews in Solar Physics 12, 4 (Sporer's law: spots start near 25 to 30 degrees and drift to about 8 degrees).", doi: "10.1007/lrsp-2015-4" },
  { text: "Berdyugina, S. V. (2005). Starspots: a key to the stellar dynamo. Living Reviews in Solar Physics 2, 8 (spot temperature contrasts; polar spots on active stars).", doi: "10.12942/lrsp-2005-8" },
  { text: "Aschwanden, M. J. (2005). Physics of the Solar Corona. Springer (hydrostatic scale height kT/(mu m_H g); optically thin emission measure).", doi: "10.1007/3-540-30766-4" },
];

const K_B = 1.380649e-23;
const M_H = 1.6735575e-27;
const G = 6.6743e-11;
/** Spot umbra this much cooler than the photosphere (Berdyugina 2005: about 1500 to 2000 K for Sun-like stars). Stand-in. */
export const SPOT_DELTA_K = 1700;
/** Coronal temperature (quiet Sun 1 to 2 MK). Stand-in until the kernel publishes one. */
export const CORONA_K = 1.5e6;
/** Wavelengths for the red, green and blue channels, metres. */
const RGB_M = [610e-9, 550e-9, 465e-9] as const;

/** Grey-atmosphere limb darkening (Eddington): intensity at mu = cos(angle from disc centre) over the centre's. */
export const limbDarkening = (mu: number) => (2 + 3 * Math.max(0, mu)) / 5;

export interface Spot {
  lat_deg: number;
  lon_deg: number;
  /** Angular radius on the star, degrees. */
  radius_deg: number;
}

/** A small deterministic generator (mulberry32) from a text key: the same star and cycle draw the same spots. */
function rng(key: string) {
  let h = 2166136261;
  for (let i = 0; i < key.length; i++) h = Math.imul(h ^ key.charCodeAt(i), 16777619);
  let a = h >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/**
 * Spots covering `coverage` of the surface. With a cycle, they sit in two
 * belts whose mean latitude follows Sporer's law, from 28 degrees at cycle
 * minimum to 8 degrees at the next (Hathaway 2015). Without a cycle (saturated
 * stars) they spread up to high latitudes, as on very active stars
 * (Berdyugina 2005). Places and sizes are drawn from the star's address and
 * the cycle number: the kernel publishes coverage, not maps.
 */
export function spotsFor(address: string, coverage: number, cycle: { phase: number; number: number } | null): Spot[] {
  if (!(coverage > 0)) return [];
  const r = rng(`${address}|${cycle ? cycle.number : "sat"}`);
  const n = Math.max(1, Math.min(48, Math.round(coverage / 0.0004)));
  const each = coverage / n;
  const out: Spot[] = [];
  for (let i = 0; i < n; i++) {
    const size = each * (0.4 + 1.2 * r());
    const radius_deg = (Math.acos(1 - 2 * size) * 180) / Math.PI;
    const hemi = r() < 0.5 ? -1 : 1;
    const lat = cycle ? 28 - 20 * cycle.phase + (r() - 0.5) * 10 : 15 + r() * 60;
    out.push({ lat_deg: hemi * Math.max(2, lat), lon_deg: r() * 360, radius_deg });
  }
  return out;
}

/** Coronal scale height in stellar radii: kT / (mu m_H g) with mu = 0.6 (ionised solar mix). */
export function coronaScaleHeight(mass_kg: number, radius_m: number, t_k = CORONA_K): number {
  const g = (G * mass_kg) / (radius_m * radius_m);
  return (K_B * t_k) / (0.6 * M_H * g) / radius_m;
}

export interface StarDraw {
  teff_k: number;
  spots: Spot[];
  /** Central meridian longitude, degrees (the star turns under the viewer). */
  lon_deg: number;
  /** Tilt of the pole towards the viewer, degrees. */
  lat_deg: number;
  light: "visible" | "xray";
  /** Coronal scale height in stellar radii (X-ray view). */
  corona_h: number;
  /** Flares going on now: where and how bright (peak power over the star's X-ray power). */
  flares?: { lat_deg: number; lon_deg: number; strength: number }[];
}

const decode = (c: number) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
const encode = (c: number) => (c <= 0.0031308 ? 12.92 * c : 1.055 * c ** (1 / 2.4) - 0.055);

/** Unit vector on the star for latitude and longitude, turned to the viewer's frame (z towards the viewer). */
function toView(lat: number, lon: number, cLat: number, cLon: number): [number, number, number] {
  const la = (lat * Math.PI) / 180;
  const lo = ((lon - cLon) * Math.PI) / 180;
  const x = Math.cos(la) * Math.sin(lo);
  const y0 = Math.sin(la);
  const z0 = Math.cos(la) * Math.cos(lo);
  const t = (cLat * Math.PI) / 180;
  return [x, y0 * Math.cos(t) - z0 * Math.sin(t), y0 * Math.sin(t) + z0 * Math.cos(t)];
}

/**
 * Draw a star into RGBA pixels, `size` across, the disc filling `disc` of
 * that (the rest shows the corona in X-rays). Visible: the photosphere's
 * colour from its temperature, darkened to the limb, spots by their own
 * blackbody at each channel. X-ray: the optically thin corona's emission
 * measure along each line of sight (a hydrostatic shell), brighter over active
 * regions, on a mapped colour scale.
 */
export function drawStar(out: { width: number; data: Uint8ClampedArray }, s: StarDraw, disc = 0.8): void {
  const n = out.width;
  const d = out.data;
  const R = (n / 2) * disc;
  const c = n / 2;
  const base = (thermalColour(s.teff_k).match(/[0-9a-f]{2}/gi) ?? ["ff", "ff", "ff"]).map((h) => decode(parseInt(h, 16) / 255));
  const spotT = Math.max(1000, s.teff_k - SPOT_DELTA_K);
  const spotRatio = RGB_M.map((l) => planck(l, spotT) / planck(l, s.teff_k));
  const spots = s.spots.map((p) => ({ v: toView(p.lat_deg, p.lon_deg, s.lat_deg, s.lon_deg), cosr: Math.cos((p.radius_deg * Math.PI) / 180), pen: Math.cos((p.radius_deg * 1.8 * Math.PI) / 180) }));
  const flares = (s.flares ?? []).map((f) => ({ v: toView(f.lat_deg, f.lon_deg, s.lat_deg, s.lon_deg), k: f.strength }));
  const H = Math.max(0.01, s.corona_h);
  for (let py = 0; py < n; py++) {
    for (let px = 0; px < n; px++) {
      const o = (py * n + px) * 4;
      const x = (px + 0.5 - c) / R;
      const y = -(py + 0.5 - c) / R;
      const b2 = x * x + y * y;
      const b = Math.sqrt(b2);
      let r = 0;
      let g = 0;
      let bl = 0;
      let a = 0;
      if (s.light === "visible") {
        if (b2 >= 1) {
          d[o + 3] = 0;
          continue;
        }
        const mu = Math.sqrt(1 - b2);
        const ld = limbDarkening(mu);
        let f = [1, 1, 1];
        for (const sp of spots) {
          const cosang = x * sp.v[0] + y * sp.v[1] + mu * sp.v[2];
          if (cosang > sp.cosr) f = spotRatio;
          else if (cosang > sp.pen && f[0] === 1) f = spotRatio.map((q) => 0.5 + 0.5 * q);
        }
        for (const fl of flares) {
          const cosang = x * fl.v[0] + y * fl.v[1] + mu * fl.v[2];
          if (cosang > 0.995) f = f.map((q) => q + Math.min(3, fl.k));
        }
        r = base[0]! * ld * f[0]!;
        g = base[1]! * ld * f[1]!;
        bl = base[2]! * ld * f[2]!;
        a = 1;
        // Soft rim (pixel coverage).
        a = Math.min(1, (1 - b) * R + 0.5);
      } else {
        // Optically thin corona: column of n^2 through a shell n = n0 exp(-(r - 1)/H), Chapman's approximation
        // for the emission scale H/2; behind the disc the star hides the far half.
        let em: number;
        if (b >= 1) em = Math.exp(-(2 * (b - 1)) / H) * Math.sqrt((Math.PI * b * H) / 2);
        else {
          const mu = Math.sqrt(1 - b2);
          em = Math.min(H / (2 * Math.max(mu, 1e-3)), Math.sqrt((Math.PI * H) / 2));
        }
        // Active regions: X-ray bright loops above spots (and flares far brighter).
        let act = 0;
        const z = b < 1 ? Math.sqrt(1 - b2) : 0;
        const nx = b < 1 ? x : x / b;
        const ny = b < 1 ? y : y / b;
        for (const sp of spots) {
          const cosang = nx * sp.v[0] + ny * sp.v[1] + z * sp.v[2];
          act += Math.exp(-(1 - cosang) / 0.01) * 6;
        }
        for (const fl of flares) {
          const cosang = nx * fl.v[0] + ny * fl.v[1] + z * fl.v[2];
          act += Math.exp(-(1 - cosang) / 0.004) * 20 * fl.k;
        }
        em *= 1 + act;
        // Mapped colour: log emission measure on a black-violet-white ramp.
        const v = Math.max(0, Math.min(1, 1 + Math.log10(em / Math.sqrt((Math.PI * H) / 2) + 1e-6) / 3));
        r = v ** 1.2 * 0.95;
        g = v ** 2.2 * 0.85;
        bl = Math.min(1, v ** 0.8 * 1.1);
        // An X-ray picture is a detector image on black: opaque within the round field, clear outside it.
        const fr = Math.hypot(px + 0.5 - c, py + 0.5 - c) / (n / 2);
        a = Math.max(0, Math.min(1, (1 - fr) * (n / 2) + 0.5));
      }
      d[o] = 255 * encode(Math.min(1, r));
      d[o + 1] = 255 * encode(Math.min(1, g));
      d[o + 2] = 255 * encode(Math.min(1, bl));
      d[o + 3] = 255 * a;
    }
  }
}

/** The star in words, for the text tier and screen readers. */
export function narrateStar(label: string, teff_k: number, spots: number, coverage: number, regime: string, light: StarDraw["light"]): string {
  const part = light === "visible" ? `${label}'s photosphere at ${Math.round(teff_k).toLocaleString("en-GB")} K, darker towards the edge` : `${label}'s corona in X-rays, brightest over active regions and at the rim`;
  return `${part}. ${spots ? `${spots} spot${spots === 1 ? "" : "s"} cover about ${(coverage * 100).toFixed(coverage < 0.01 ? 2 : 1)}% of the surface` : "No spots"}; activity ${regime}.`;
}
