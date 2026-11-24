// Star jets and shed shells in True (SG5; third V3 audit gap R26). The light track owns the physics and publishes
// every one of these as an ordinary nebula descriptor through read_nebulae (sse-nebula): planetary nebulae (shells,
// lobes), and, agreed with "Build light and nebulae" on 24 Nov, a young star's bipolar jet (kind "jet": the spin axis,
// length, speed, opening angle and dated knots) and a cool giant's dusty mass loss (kind "dust-shell": shells from the
// dust condensation radius out to wind speed times time, older episodes as more shells). This file only draws a
// descriptor: the gas's density from its shells' masses and sizes, its light summed along each line of sight
// (density squared for glowing gas, density for dust that shines by its own heat or the star's light), coloured by
// the descriptor's own light: its emission lines at their wavelengths, or its dust temperature.
import { constant, LINES, planck, thermalColour, type ViewBand } from "../../ui-contract/src/index.ts";

export const SHELL_REFERENCES = [
  { text: "Osterbrock, D. E. and Ferland, G. J. (2006). Astrophysics of Gaseous Nebulae and Active Galactic Nuclei, 2nd ed. University Science Books (recombination and forbidden lines scale with the density squared along the line of sight: the emission measure).", url: "https://uscibooks.aip.org/books/astrophysics-of-gaseous-nebulae-and-active-galactic-nuclei-second-edition/" },
  { text: "Reipurth, B. and Bally, J. (2001). Herbig-Haro Flows: Probes of Early Stellar Evolution. ARA&A 39, 403 (bipolar jets from young stars, knots from episodic accretion).", doi: "10.1146/annurev.astro.39.1.403" },
  { text: "Bruton, D. (1996). Color Science: approximate RGB values for visible wavelengths (the piecewise colour of a single wavelength).", url: "https://www.physics.sfasu.edu/astro/color/spectra.html" },
];

/** The descriptor fields this file reads (sse-nebula `Descriptor`, plus the agreed `jet` field). */
export interface NebulaShape {
  address: string;
  kind: string;
  shells: { radius_m: number; thickness_m: number; mass_kg: number }[];
  lobes?: { axis: [number, number, number]; ratio: number } | null;
  elongation?: number;
  jet?: { length_m: number; speed_m_s?: number; opening_rad: number; mass_rate_kg_s?: number; knots?: { distance_m: number; age_s?: number }[] } | null;
  light?: { parts?: { kind?: string; t_k?: number; power_w?: number }[]; lines?: { line: string; power_w: number }[] };
}

/** How many decades below its brightest point a nebula is drawn (a log scale, as nebula photographs are stretched). */
export const SHELL_DECADES = 3;
/** Steps along each line of sight. */
const STEPS = 160;

/** Dust and other continuum-only nebulae shine by density (optically thin); glowing gas by density squared. */
export const byDensitySquared = (s: NebulaShape) => s.kind !== "dust-shell";

/**
 * Colour of one wavelength to the eye, linear 0..1 (Bruton's piecewise approximation, with the fall-off at the ends of
 * sight). Used only for emission lines; continua take their black-body colour.
 */
export function wavelengthRgb(m: number): [number, number, number] {
  const w = m * 1e9;
  let r = 0;
  let g = 0;
  let b = 0;
  if (w >= 380 && w < 440) (r = -(w - 440) / 60), (b = 1);
  else if (w < 490) (g = (w - 440) / 50), (b = 1);
  else if (w < 510) (g = 1), (b = -(w - 510) / 20);
  else if (w < 580) (r = (w - 510) / 70), (g = 1);
  else if (w < 645) (r = 1), (g = -(w - 645) / 65);
  else if (w <= 780) r = 1;
  const f = w < 380 || w > 780 ? 0 : w < 420 ? 0.3 + (0.7 * (w - 380)) / 40 : w > 700 ? 0.3 + (0.7 * (780 - w)) / 80 : 1;
  const lin = (c: number) => (c * f) ** 2.2;
  return [lin(r), lin(g), lin(b)];
}

const decode = (c: number) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
const hexRgb = (hex: string) => (hex.match(/[0-9a-f]{2}/gi) ?? ["ff", "ff", "ff"]).map((h) => decode(parseInt(h, 16) / 255)) as [number, number, number];

/**
 * The nebula's colour in a band, linear, brightest channel 1. Visible: its lines' powers at their wavelengths when it
 * has lines, else its continuum's black-body colour (dust at its own temperature is red to infrared, so in Visible a
 * dust shell shows the starlight it reflects, `starTeff`). Infrared: the dust's own heat, mapped grey.
 */
export function nebulaColour(s: NebulaShape, band: Extract<ViewBand, "visible" | "infrared">, starTeff = constant("T_sun")): [number, number, number] {
  if (band === "infrared") return [1, 1, 1];
  const lines = s.light?.lines ?? [];
  let c: [number, number, number] = [0, 0, 0];
  for (const l of lines) {
    const at = LINES.find((x) => x.id === l.line)?.at_m;
    if (!at || !(l.power_w > 0)) continue;
    const w = wavelengthRgb(at);
    c = [c[0] + w[0] * l.power_w, c[1] + w[1] * l.power_w, c[2] + w[2] * l.power_w];
  }
  if (!(c[0] + c[1] + c[2] > 0)) {
    const t = s.light?.parts?.find((p) => p.kind === "thermal" && Number(p.t_k) > 0)?.t_k;
    // Dust at a thousand kelvin gives almost no visible light of its own: what the eye sees is reflected starlight.
    c = hexRgb(thermalColour(t && t > 2500 ? t : starTeff));
  }
  const m = Math.max(c[0], c[1], c[2]) || 1;
  return [c[0] / m, c[1] / m, c[2] / m];
}

/** Dust's own glow at 10 µm over its glow at 1000 K: the infrared band's brightness for a dust shell. */
export const dustGlow10um = (t_k: number) => planck(10e-6, t_k) / planck(10e-6, 1000);

export interface ShellView {
  /** The picture's half-width, metres from the star. */
  half_m: number;
  /** The lobes' or jet's axis in the viewer's frame (x right, y up, z toward the viewer), unit. */
  axis?: [number, number, number];
  band?: Extract<ViewBand, "visible" | "infrared">;
  starTeff?: number;
}

/**
 * The gas's density at a point (metres from the star, the viewer's frame), in units of its own mean: shells (stretched
 * along the axis by the elongation, or pinched to two lobes), the jet's two cones and its knots.
 */
export function density(s: NebulaShape, p: [number, number, number], axis: [number, number, number]): number {
  const along = p[0] * axis[0] + p[1] * axis[1] + p[2] * axis[2];
  const r = Math.hypot(p[0], p[1], p[2]);
  let n = 0;
  const el = Math.max(1, s.elongation ?? 1);
  const ratio = Math.max(1, s.lobes?.ratio ?? 1);
  for (const sh of s.shells) {
    if (!(sh.radius_m > 0)) continue;
    // A shell's edge at this bearing: stretched along the axis by the elongation, or a waist with two lobes.
    const c = r > 0 ? along / r : 0;
    const edge = sh.radius_m * (s.lobes ? 1 + (ratio - 1) * c * c : 1 / Math.sqrt(1 - (1 - 1 / (el * el)) * c * c) / el ** (1 / 3));
    const t = Math.max(sh.thickness_m, 0.02 * sh.radius_m);
    const mass = sh.mass_kg > 0 ? sh.mass_kg : 1;
    if (t >= sh.radius_m) {
      // A shell thicker than its inner radius is a steady wind from that radius out: density falls as 1/r², and
      // the mass 4π ρ₀ R² t sets ρ₀ (a dust shell from its condensation radius to wind speed × time).
      if (r >= edge && r <= edge + t) n += (mass / (4 * Math.PI * edge * edge * t)) * (edge / r) ** 2;
      continue;
    }
    const d = Math.abs(r - edge);
    if (d < t) n += (mass / (4 * Math.PI * edge * edge * t)) * (1 - d / t);
  }
  const j = s.jet;
  if (j && j.length_m > 0) {
    const a = Math.abs(along);
    const side = Math.sqrt(Math.max(0, r * r - along * along));
    const width = Math.max(1e-9, a * Math.tan(Math.max(0.005, j.opening_rad)));
    if (a <= j.length_m && side < width) {
      // A steady flow spreads over a widening cone: density falls as one over the square of distance.
      const flow = (j.mass_rate_kg_s ?? 1) / Math.max(1, j.speed_m_s ?? 1) / (Math.PI * width * width);
      n += flow * (1 - side / width);
    }
    for (const k of j.knots ?? []) {
      // Each knot is a dated episode: a blob at its distance on both sides, as wide as the cone there.
      const w = Math.max(1e-9, k.distance_m * Math.tan(Math.max(0.005, j.opening_rad)));
      for (const sgn of [1, -1]) {
        const dx = p[0] - sgn * axis[0] * k.distance_m;
        const dy = p[1] - sgn * axis[1] * k.distance_m;
        const dz = p[2] - sgn * axis[2] * k.distance_m;
        const q = (dx * dx + dy * dy + dz * dz) / (w * w);
        if (q < 4) n += ((j.mass_rate_kg_s ?? 1) / Math.max(1, j.speed_m_s ?? 1) / (Math.PI * w * w)) * 3 * Math.exp(-q);
      }
    }
  }
  return n;
}

/**
 * Draw the nebula into a square picture to be added to the scene as light: each pixel's light is the density (dust)
 * or density squared (glowing gas) summed along the line of sight, shown on a log scale SHELL_DECADES deep from the
 * brightest pixel, in the nebula's colour. Returns the brightest line-of-sight sum (for checks).
 */
export function drawShells(out: { width: number; data: Uint8ClampedArray }, s: NebulaShape, v: ShellView): number {
  const n = out.width;
  const d = out.data;
  const axis = v.axis ?? [0, 1, 0];
  const sq = byDensitySquared(s);
  const sums = new Float64Array(n * n);
  let top = 0;
  const dz = (2 * v.half_m) / STEPS;
  for (let py = 0; py < n; py++)
    for (let px = 0; px < n; px++) {
      const x = ((px + 0.5) / n - 0.5) * 2 * v.half_m;
      const y = -((py + 0.5) / n - 0.5) * 2 * v.half_m;
      let sum = 0;
      for (let k = 0; k < STEPS; k++) {
        const z = -v.half_m + (k + 0.5) * dz;
        const rho = density(s, [x, y, z], axis);
        sum += sq ? rho * rho : rho;
      }
      sums[py * n + px] = sum;
      if (sum > top) top = sum;
    }
  const col = nebulaColour(s, v.band ?? "visible", v.starTeff);
  const enc = (c: number) => (c <= 0.0031308 ? 12.92 * c : 1.055 * c ** (1 / 2.4) - 0.055);
  for (let i = 0; i < n * n; i++) {
    const o = i * 4;
    const sum = sums[i]!;
    const val = top > 0 && sum > 0 ? Math.max(0, 1 + Math.log10(sum / top) / SHELL_DECADES) : 0;
    d[o] = 255 * enc(col[0] * val);
    d[o + 1] = 255 * enc(col[1] * val);
    d[o + 2] = 255 * enc(col[2] * val);
    d[o + 3] = 255 * enc(val);
  }
  return top;
}

/** The nebula in words, for the note beside the view. */
export function narrateShells(s: NebulaShape): string {
  const au = constant("au");
  const ly = constant("ly");
  const size = (m: number) => (m >= 0.1 * ly ? `${(m / ly).toPrecision(2)} light-years` : `${Math.round(m / au).toLocaleString("en-GB")} au`);
  if (s.kind === "jet" && s.jet) return `A jet from the young star, ${size(s.jet.length_m)} each way along its spin axis${s.jet.knots?.length ? `, with ${s.jet.knots.length} knots from bursts of feeding` : ""}: drawn from the kernel's flow, on a log scale ${SHELL_DECADES} decades deep.`;
  if (s.kind === "dust-shell") return `Dust the giant has shed, from ${size(s.shells[0]?.radius_m ?? 0)} out${s.shells.length > 1 ? `, in ${s.shells.length} shells from separate episodes` : ""}: drawn from the kernel's shells, on a log scale ${SHELL_DECADES} decades deep.`;
  return `Gas the star has shed, ${size(Math.max(...s.shells.map((x) => x.radius_m), 0))} across its outer shell${s.lobes ? ", pinched into two lobes" : ""}: drawn from the kernel's shells, on a log scale ${SHELL_DECADES} decades deep.`;
}
