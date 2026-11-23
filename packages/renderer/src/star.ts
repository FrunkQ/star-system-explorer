// Stars drawn from their physics (R5a): a limb-darkened photosphere in the
// star's own colour, spots where the activity cycle puts them, and the corona
// in X-rays. The numbers come from the kernel: effective temperature, radius,
// mass and age from the system's facts, and spot coverage, cycle phase, X-ray
// power, spin and flares from the light track's `read_activity` (O10). This
// file turns them into pixels; where it has to choose something the kernel
// does not publish (spot places, spot temperature, coronal temperature) the
// choice is named here and in docs/fudges.md.
import { constant, planck, thermalColour } from "../../ui-contract/src/index.ts";

export const STAR_REFERENCES = [
  { text: "Eddington, A. S. (1926). The Internal Constitution of the Stars. Cambridge University Press (grey atmosphere limb darkening, I(mu)/I(1) = (2 + 3 mu) / 5).", url: "https://archive.org/details/internalconstitu0000eddi" },
  { text: "Hathaway, D. H. (2015). The Solar Cycle. Living Reviews in Solar Physics 12, 4 (Sporer's law: spots start near 25 to 30 degrees and drift to about 8 degrees).", doi: "10.1007/lrsp-2015-4" },
  { text: "Berdyugina, S. V. (2005). Starspots: a key to the stellar dynamo. Living Reviews in Solar Physics 2, 8 (spot temperature contrasts; polar spots on active stars).", doi: "10.12942/lrsp-2005-8" },
  { text: "Aschwanden, M. J. (2005). Physics of the Solar Corona. Springer (hydrostatic scale height kT/(mu m_H g); optically thin emission measure).", doi: "10.1007/3-540-30766-4" },
  { text: "Nordlund, A., Stein, R. F. and Asplund, M. (2009). Solar Surface Convection. Living Reviews in Solar Physics 6, 2 (granules: bright hot upflows in dark cool lanes).", doi: "10.12942/lrsp-2009-2" },
  { text: "Wedemeyer-Bohm, S. and Rouppe van der Voort, L. (2009). On the continuum intensity distribution of the solar photosphere. A&A 503, 225 (granulation rms intensity contrast about 14% at 630 nm, Hinode).", doi: "10.1051/0004-6361/200911983" },
  { text: "Chiavassa, A. et al. (2010). Radiative hydrodynamics simulations of red supergiant stars II. A&A 515, A12 (Betelgeuse: a few giant convective cells).", doi: "10.1051/0004-6361/200913907" },
];

const K_B = constant("k_B");
const M_H = 1.6735575e-27;
const G = constant("G");
/** Spot umbra this much cooler than the photosphere (Berdyugina 2005: about 1500 to 2000 K for Sun-like stars). Stand-in. */
export const SPOT_DELTA_K = 1700;
/** Coronal temperature (quiet Sun 1 to 2 MK). Stand-in until the kernel publishes one. */
export const CORONA_K = 1.5e6;
/**
 * Granule temperature spread, as a share of the surface temperature (rms). Set so the Sun's granulation has
 * the observed rms intensity contrast of about 14% in green light (Wedemeyer-Bohm and Rouppe van der Voort
 * 2009): dI/I = x dT/T with x = hc / (lambda k T) about 4.5 at 550 nm, so dT/T = 0.031 (about 180 K).
 * The same share is used for every star; the kernel publishes sizes and speeds, not contrasts. Stand-in.
 */
export const GRANULE_DT = 0.031;
/** Smallest granule, in pixels across, worth drawing; below it the pattern averages out to the plain disc. */
export const GRANULE_MIN_PX = 6;
/** Most cells drawn over a whole star. */
const GRANULE_MAX = 60000;
const HC_K = 0.014387769;
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

/** The kernel's surface convection for a star (read_activity, O10), the parts the picture uses. */
export interface Convection {
  granule_size_m: number;
  granule_lifetime_s: number;
  supergranule_size_m: number;
  supergranule_lifetime_s: number;
  speed_m_s: number;
  sound_speed_m_s: number;
  granules_across: number;
  network_cells: number;
}

/** Granule cells over the whole star: centres on the unit sphere, with a lookup grid. */
export interface Granules {
  n: number;
  /** Mean spacing between neighbouring centres, as a chord on the unit sphere. */
  spacing: number;
  pts: Float64Array;
  grid: Map<number, number[]>;
  cell: number;
  /** Mean and spread of the raw pattern, so the picture's mean light equals the plain disc's. */
  mean: number;
  rms: number;
}

/** How many granules cover a star with `across` granules over its diameter: the sphere's area over a hexagon's. */
export const granuleCount = (across: number) => Math.round((Math.PI * across * across) / (Math.sqrt(3) / 2));

/**
 * Whether a disc `disc_px` across shows granules (`across` over the diameter): each must be at least
 * GRANULE_MIN_PX wide on the screen, and the star must not need more than GRANULE_MAX cells.
 */
export const granulesShow = (across: number, disc_px: number) => across > 0 && disc_px / across >= GRANULE_MIN_PX && granuleCount(across) <= GRANULE_MAX;

const gkey = (i: number, j: number, k: number) => (i + 512) * 1048576 + (j + 512) * 1024 + (k + 512);

/**
 * Granule centres for a star at one generation of cells. Cells live `granule_lifetime_s`, so a new
 * generation (a new `epoch`) is a fresh set; the same star and epoch always give the same cells. Centres
 * are a Fibonacci lattice, jittered by a share of the spacing so they read as convection, not a grid.
 */
export function granulesFor(address: string, across: number, epoch: number): Granules {
  const n = Math.max(12, granuleCount(across));
  const r = rng(`${address}|granules|${epoch}`);
  const spacing = Math.sqrt((4 * Math.PI) / n) * 1.07;
  const pts = new Float64Array(n * 3);
  const golden = Math.PI * (3 - Math.sqrt(5));
  const turn = r() * 2 * Math.PI;
  for (let i = 0; i < n; i++) {
    let z = 1 - (2 * (i + 0.5)) / n;
    let phi = i * golden + turn;
    // Jitter: up to about 0.35 of a cell in each direction.
    z = Math.max(-1, Math.min(1, z + (r() - 0.5) * 1.0 * (spacing * Math.sqrt(Math.max(0, 1 - z * z)) || spacing)));
    phi += ((r() - 0.5) * 1.0 * spacing) / Math.max(0.05, Math.sqrt(Math.max(0, 1 - z * z)));
    const q = Math.sqrt(Math.max(0, 1 - z * z));
    pts[i * 3] = q * Math.cos(phi);
    pts[i * 3 + 1] = z;
    pts[i * 3 + 2] = q * Math.sin(phi);
  }
  const cell = spacing * 1.5;
  const grid = new Map<number, number[]>();
  for (let i = 0; i < n; i++) {
    const k = gkey(Math.floor(pts[i * 3]! / cell), Math.floor(pts[i * 3 + 1]! / cell), Math.floor(pts[i * 3 + 2]! / cell));
    const b = grid.get(k);
    if (b) b.push(i);
    else grid.set(k, [i]);
  }
  const g: Granules = { n, spacing, pts, grid, cell, mean: 0, rms: 1 };
  // The raw pattern's mean and spread, from a fixed sample of the sphere.
  let s1 = 0;
  let s2 = 0;
  const m = 4000;
  for (let i = 0; i < m; i++) {
    const z = 1 - (2 * (i + 0.5)) / m;
    const q = Math.sqrt(1 - z * z);
    const v = granulePattern(g, q * Math.cos(i * golden), z, q * Math.sin(i * golden));
    s1 += v;
    s2 += v * v;
  }
  g.mean = s1 / m;
  g.rms = Math.sqrt(Math.max(1e-9, s2 / m - g.mean * g.mean));
  return g;
}

/**
 * The raw granulation pattern at a point on the unit sphere: bright, domed cell interiors (hot upflow)
 * falling to dark lanes where two cells meet (cool downflow), from the two nearest centres.
 */
function granulePattern(g: Granules, x: number, y: number, z: number): number {
  let f1 = Infinity;
  let f2 = Infinity;
  const i0 = Math.floor(x / g.cell);
  const j0 = Math.floor(y / g.cell);
  const k0 = Math.floor(z / g.cell);
  for (let i = i0 - 1; i <= i0 + 1; i++)
    for (let j = j0 - 1; j <= j0 + 1; j++)
      for (let k = k0 - 1; k <= k0 + 1; k++) {
        const b = g.grid.get(gkey(i, j, k));
        if (!b) continue;
        for (const p of b) {
          const dx = x - g.pts[p * 3]!;
          const dy = y - g.pts[p * 3 + 1]!;
          const dz = z - g.pts[p * 3 + 2]!;
          const d2 = dx * dx + dy * dy + dz * dz;
          if (d2 < f1) {
            f2 = f1;
            f1 = d2;
          } else if (d2 < f2) f2 = d2;
        }
      }
  if (!Number.isFinite(f2)) return 0.5;
  // Lanes take about a third of a cell, rising smoothly into a domed, brightest-in-the-middle cell.
  const a = Math.min(1, Math.sqrt(f1) / (0.75 * g.spacing));
  const u = Math.min(1, (Math.sqrt(f2) - Math.sqrt(f1)) / (0.45 * g.spacing));
  const lane = u * u * (3 - 2 * u);
  return lane * (1 - 0.7 * a * a);
}

/** The granulation as a temperature change at a point on the star, kelvin (mean zero over the star). */
export function granuleDeltaT(g: Granules, teff_k: number, x: number, y: number, z: number): number {
  return ((granulePattern(g, x, y, z) - g.mean) / g.rms) * GRANULE_DT * teff_k;
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
  /** Granule cells (from granulesFor), drawn in visible light when the disc resolves them. */
  granules?: Granules | null;
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

/** A point on the visible disc (x right, y up, z towards the viewer) back to the star's own frame: the inverse of toView. */
function toStar(x: number, y: number, z: number, cLat: number, cLon: number): [number, number, number] {
  const t = (cLat * Math.PI) / 180;
  const y0 = y * Math.cos(t) + z * Math.sin(t);
  const z0 = -y * Math.sin(t) + z * Math.cos(t);
  const lo = (cLon * Math.PI) / 180;
  // Star frame used by granulesFor: x and z in the equator, y to the pole; longitude turns about y.
  return [x * Math.cos(lo) + z0 * Math.sin(lo), y0, -x * Math.sin(lo) + z0 * Math.cos(lo)];
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
  // Granules change each channel by d(ln B)/d(ln T) = x / (1 - e^-x), x = hc / (lambda k T): linear in dT, so the
  // disc's mean light is the plain disc's.
  const gran = s.light === "visible" ? (s.granules ?? null) : null;
  const slope = RGB_M.map((l) => {
    const x = HC_K / (l * s.teff_k);
    return x / (1 - Math.exp(-x)) / s.teff_k;
  });
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
        if (gran) {
          const [sx, sy, sz] = toStar(x, y, mu, s.lat_deg, s.lon_deg);
          const dT = granuleDeltaT(gran, s.teff_k, sx, sy, sz);
          f = slope.map((k) => Math.max(0, 1 + k * dT));
        }
        const fg = f;
        for (const sp of spots) {
          const cosang = x * sp.v[0] + y * sp.v[1] + mu * sp.v[2];
          if (cosang > sp.cosr) f = spotRatio;
          else if (cosang > sp.pen && f === fg) f = spotRatio.map((q) => 0.5 + 0.5 * q);
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

const km = (m: number) => (m >= 1e9 ? `${(m / 1e9).toPrecision(2)} million km` : `${Math.round(m / 1000).toLocaleString("en-GB")} km`);
const DAY_S = constant("day");
const span = (s: number) => (s < 3600 ? `${Math.round(s / 60)} minutes` : s < 2 * DAY_S ? `${Math.round(s / 3600)} hours` : s < 2 * 365.25 * DAY_S ? `${Math.round(s / DAY_S)} days` : `${Math.round(s / (365.25 * DAY_S))} years`);

/** The churning surface in words: what the cells are, and whether this picture shows them. */
export function narrateGranules(c: Convection | null | undefined, disc_px: number): string {
  if (!c) return "No granules: the surface is too hot to churn (above about 6,900 K), so heat leaves by radiation alone.";
  const across = Math.round(c.granules_across);
  const shown = granulesShow(c.granules_across, disc_px)
    ? `About ${across.toLocaleString("en-GB")} across the disc, drawn here.`
    : `About ${across.toLocaleString("en-GB")} across the disc, too small to draw at this size, so the disc shows their average.`;
  return `Granules about ${km(c.granule_size_m)} wide, each living about ${span(c.granule_lifetime_s)}, churning at ${(c.speed_m_s / 1000).toFixed(1)} km/s. ${shown} Supergranules about ${km(c.supergranule_size_m)} wide outline the magnetic network.`;
}
