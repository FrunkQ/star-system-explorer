// Light and bands (nebulae-and-light design, sections 1 and 2; O1, the UI
// side). Every object that gives off or reflects light carries a light
// description; a band recipe turns it into colour on screen. The recipe is a
// presentation choice on this device only: it never reaches the session, so
// it cannot move the state hash.
//
// Until the kernel publishes light facts, descriptions here are built from
// the facts a scene already has (a star's derived surface temperature, a
// planet's albedo and surface temperature) and are marked "stubbed": a grey
// reflectance and a pure thermal continuum, nothing more. Every view that
// shows them says so.
//
// This is presentation code, not kernel code: it uses Math.exp freely because
// nothing here is hashed or replayed.

import type { SystemScene } from "./scene.ts";
import type { PlayerScene } from "./player-scene.ts";
import { parentOf, subjectName } from "./player-scene.ts";
import { playerSubjectOf } from "./mark.ts";

export const LIGHT_SCHEMA = "sse.band-recipe/0" as const;

// Physical constants (CODATA 2018, exact in SI since 2019).
const H = 6.62607015e-34;
const C = 299792458;
const K = 1.380649e-23;

/** The fixed band set (design 2.1), wavelengths in metres. */
export interface Band {
  id: string;
  name: string;
  from_m: number;
  to_m: number;
}
export const BANDS: readonly Band[] = [
  { id: "gamma", name: "Gamma rays", from_m: 1e-14, to_m: 1e-11 },
  { id: "xray", name: "X-rays", from_m: 1e-11, to_m: 1e-8 },
  { id: "uv", name: "Ultraviolet", from_m: 1e-8, to_m: 3.8e-7 },
  { id: "visible", name: "Visible", from_m: 3.8e-7, to_m: 7.5e-7 },
  { id: "near-ir", name: "Near infrared", from_m: 7.5e-7, to_m: 5e-6 },
  { id: "mid-ir", name: "Mid infrared", from_m: 5e-6, to_m: 2.5e-5 },
  { id: "far-ir", name: "Far infrared", from_m: 2.5e-5, to_m: 3.5e-4 },
  { id: "mm", name: "Millimetre", from_m: 3.5e-4, to_m: 1e-2 },
  { id: "radio", name: "Radio", from_m: 1e-2, to_m: 10 },
];

/** Emission lines the palettes use (rest wavelengths in air, metres). */
export interface Line {
  id: string;
  name: string;
  at_m: number;
}
export const LINES: readonly Line[] = [
  { id: "oiii", name: "[O III] 500.7 nm", at_m: 500.7e-9 },
  { id: "hb", name: "H-beta 486.1 nm", at_m: 486.1e-9 },
  { id: "ha", name: "H-alpha 656.3 nm", at_m: 656.3e-9 },
  { id: "nii", name: "[N II] 658.4 nm", at_m: 658.4e-9 },
  { id: "sii", name: "[S II] 671.6 nm", at_m: 671.6e-9 },
];
/** A narrowband filter's full width, as amateur and Hubble narrowband filters roughly are. */
export const NARROW_M = 3e-9;

export type Continuum =
  | { kind: "thermal"; t_k: number; weight: number }
  /** Light from a named star, reflected with a grey albedo and diluted by distance. */
  | { kind: "reflected"; t_k: number; weight: number };

export interface LightDescription {
  address: string;
  continuum: Continuum[];
  /** Emission lines: [line id, strength in the same units as the continuum per metre]. */
  lines: [string, number][];
  /** True while this is built from scene facts, not from kernel light facts. */
  stubbed: boolean;
  /** In words, what this light is made of and what is guessed. */
  why: string;
}

/** Planck spectral radiance B(λ, T), W sr⁻¹ m⁻³. */
export function planck(lambda_m: number, t_k: number): number {
  if (!(t_k > 0)) return 0;
  const x = (H * C) / (lambda_m * K * t_k);
  if (x > 700) return 0;
  return (2 * H * C * C) / lambda_m ** 5 / Math.expm1(x);
}

/** Spectral radiance of a description at one wavelength. */
export function radiance(d: LightDescription, lambda_m: number): number {
  let r = 0;
  for (const c of d.continuum) r += c.weight * planck(lambda_m, c.t_k);
  return r;
}

/** Mean radiance over [from, to], integrated in log wavelength. */
export function bandRadiance(d: LightDescription, from_m: number, to_m: number, steps = 48): number {
  const a = Math.log(from_m);
  const b = Math.log(to_m);
  let sum = 0;
  for (let i = 0; i <= steps; i++) {
    const l = Math.exp(a + ((b - a) * i) / steps);
    const w = i === 0 || i === steps ? 0.5 : 1;
    sum += w * radiance(d, l) * l;
  }
  return (sum * ((b - a) / steps)) / (to_m - from_m);
}

/** Through a narrowband filter on a line: the continuum under it plus the line. */
export function lineRadiance(d: LightDescription, line: string): number {
  const l = LINES.find((x) => x.id === line);
  if (!l) return 0;
  const cont = bandRadiance(d, l.at_m - NARROW_M / 2, l.at_m + NARROW_M / 2, 4);
  const strength = d.lines.find(([id]) => id === line)?.[1] ?? 0;
  return cont + strength / NARROW_M;
}

// CIE 1931 2° colour matching functions, the multi-lobe Gaussian fit of
// Wyman, Sloan and Shirley (2013), JCGT 2(2), http://jcgt.org/published/0002/02/01/
const g = (x: number, mu: number, s1: number, s2: number) => {
  const t = (x - mu) / (x < mu ? s1 : s2);
  return Math.exp(-0.5 * t * t);
};
export function cmf(nm: number): [number, number, number] {
  return [
    1.056 * g(nm, 599.8, 37.9, 31.0) + 0.362 * g(nm, 442.0, 16.0, 26.7) - 0.065 * g(nm, 501.1, 20.4, 26.2),
    0.821 * g(nm, 568.8, 46.9, 40.5) + 0.286 * g(nm, 530.9, 16.3, 31.1),
    1.217 * g(nm, 437.0, 11.8, 36.0) + 0.681 * g(nm, 459.0, 26.0, 13.8),
  ];
}

/** CIE XYZ of a description, 380 to 780 nm in 5 nm steps. */
export function xyz(d: LightDescription): [number, number, number] {
  let X = 0,
    Y = 0,
    Z = 0;
  for (let nm = 380; nm <= 780; nm += 5) {
    const r = radiance(d, nm * 1e-9);
    const [x, y, z] = cmf(nm);
    X += r * x;
    Y += r * y;
    Z += r * z;
  }
  return [X, Y, Z];
}

/** XYZ to linear sRGB (IEC 61966-2-1, D65 white). */
export function linearSrgb([X, Y, Z]: [number, number, number]): [number, number, number] {
  return [3.2406 * X - 1.5372 * Y - 0.4986 * Z, -0.9689 * X + 1.8758 * Y + 0.0415 * Z, 0.0557 * X - 0.204 * Y + 1.057 * Z];
}

const encode = (v: number) => (v <= 0.0031308 ? 12.92 * v : 1.055 * v ** (1 / 2.4) - 0.055);
const byte = (v: number) => Math.round(Math.max(0, Math.min(1, v)) * 255);
export const hex = (rgb: readonly number[]) => "#" + rgb.map((v) => byte(v).toString(16).padStart(2, "0")).join("");

/** Out-of-gamut channels are clipped at zero, then the brightest channel is set to one: colour only, not brightness. */
export function chromaticity(rgb: [number, number, number]): [number, number, number] {
  const c = rgb.map((v) => Math.max(0, v)) as [number, number, number];
  const m = Math.max(...c);
  return m > 0 ? (c.map((v) => v / m) as [number, number, number]) : [0, 0, 0];
}

/** The colour the eye would see from a thermal source at temperature T, as sRGB hex. */
export function thermalColour(t_k: number): string {
  const d: LightDescription = { address: "", continuum: [{ kind: "thermal", t_k, weight: 1 }], lines: [], stubbed: false, why: "" };
  return hex(chromaticity(linearSrgb(xyz(d))).map(encode));
}

// ── Recipes ──────────────────────────────────────────────────────────────

export type Honesty = "actual" | "actual-enhanced" | "mapped";
export const HONESTY_WORDS: Record<Honesty, string> = {
  actual: "Actual colour",
  "actual-enhanced": "Actual colour, enhanced",
  mapped: "Mapped colour",
};

export type Source = { band: string } | { line: string } | { from_m: number; to_m: number };
export interface Channel {
  source: Source;
  /** The display colour this channel paints with (#rrggbb). */
  colour: string;
  /** What this channel shows, for the legend. */
  label: string;
}

export interface BandRecipe {
  schema: typeof LIGHT_SCHEMA;
  id: string;
  name: string;
  honesty: Honesty;
  /**
   * eye: CIE colour matching, what a person would see.
   * camera: an RGB sensor's three broad filters, then saturation raised.
   * composite: each channel's light, scaled to that body's brightest channel, painted in its colour.
   * single: one band, brightness across the view on a log scale, painted along a ramp.
   */
  mode: "eye" | "camera" | "composite" | "single";
  channels: Channel[];
  /** single mode: decades of brightness the ramp spans, faintest to brightest. */
  decades?: number;
  /** camera mode: how far saturation is raised (1 = none). */
  saturation?: number;
  legend: string;
  why: string;
}

export const MODES = ["eye", "camera", "composite", "single"] as const;
const HEX = /^#[0-9a-f]{6}$/i;

export function checkLightRecipe(r: unknown): string[] {
  const out: string[] = [];
  const x = r as Partial<BandRecipe>;
  if (!x || typeof x !== "object") return ["a recipe is an object"];
  if (x.schema !== LIGHT_SCHEMA) out.push(`schema must be ${LIGHT_SCHEMA}`);
  if (!x.id || typeof x.id !== "string") out.push("id is required");
  if (!x.name) out.push("name is required");
  if (!x.honesty || !(x.honesty in HONESTY_WORDS)) out.push("honesty must be actual, actual-enhanced or mapped");
  if (!x.mode || !(MODES as readonly string[]).includes(x.mode)) out.push(`mode must be one of ${MODES.join(", ")}`);
  if (!x.legend) out.push("every recipe says what its colours mean (legend)");
  if (!x.why) out.push("every recipe says why");
  if (x.mode === "eye" && x.honesty !== "actual") out.push("eye is actual colour by definition");
  if (x.mode && x.mode !== "eye" && x.honesty === "actual") out.push("only eye mode may claim actual colour");
  if ((x.mode === "composite" || x.mode === "single") && x.honesty !== "mapped") out.push(`${x.mode} recipes are mapped colour`);
  const ch = Array.isArray(x.channels) ? x.channels : [];
  if (!Array.isArray(x.channels)) out.push("channels must be a list");
  if (x.mode === "single" && ch.length !== 1) out.push("a single-band recipe has one channel");
  if ((x.mode === "composite" || x.mode === "camera") && ch.length < 2) out.push(`${x.mode} needs at least two channels`);
  for (const c of ch) {
    if (!HEX.test(c?.colour ?? "")) out.push(`channel colour ${c?.colour} must be #rrggbb`);
    if (!c?.label) out.push("every channel has a label");
    const s = c?.source as Record<string, unknown> | undefined;
    if (s && "band" in s && !BANDS.some((b) => b.id === s.band)) out.push(`unknown band ${String(s.band)}`);
    else if (s && "line" in s && !LINES.some((l) => l.id === s.line)) out.push(`unknown line ${String(s.line)}`);
    else if (s && "from_m" in s && !(typeof s.from_m === "number" && typeof s.to_m === "number" && s.from_m > 0 && s.to_m > s.from_m)) out.push("a wavelength range needs 0 < from_m < to_m");
    else if (!s || !("band" in s || "line" in s || "from_m" in s)) out.push("a channel's source is a band, a line or a wavelength range");
  }
  if (x.mode === "single" && !(typeof x.decades === "number" && x.decades >= 1 && x.decades <= 30)) out.push("decades must be a number from 1 to 30");
  return out;
}

export function sourceRadiance(d: LightDescription, s: Source): number {
  if ("line" in s) return lineRadiance(d, s.line);
  if ("band" in s) {
    const b = BANDS.find((x) => x.id === s.band)!;
    return bandRadiance(d, b.from_m, b.to_m);
  }
  return bandRadiance(d, s.from_m, s.to_m);
}

const parse = (h: string): [number, number, number] => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16) / 255) as [number, number, number];

/** Colours for every description under one recipe; null where there is no light to show. */
/** The Draper point: the temperature at which hot solids begin to glow visibly (Draper 1847). */
export const DRAPER_K = 798;

const DAYLIGHT: LightDescription = { address: "", continuum: [{ kind: "thermal", t_k: 6504, weight: 1 }], lines: [], stubbed: false, why: "" };

export function paint(recipe: BandRecipe, ds: readonly LightDescription[]): Map<string, string | null> {
  const out = new Map<string, string | null>();
  if (recipe.mode === "single") {
    const src = recipe.channels[0]!.source;
    const vals = ds.map((d) => sourceRadiance(d, src));
    const top = Math.max(0, ...vals);
    // A floor in energy per decade of wavelength (λB), against the brightest visible light in view:
    // light more than 30 decades below that is nothing any instrument sees, so it is not stretched into view.
    const mid = (x: Source) => ("line" in x ? LINES.find((l) => l.id === x.line)!.at_m : "band" in x ? ((b) => Math.sqrt(b.from_m * b.to_m))(BANDS.find((b) => b.id === x.band)!) : Math.sqrt(x.from_m * x.to_m));
    const peak = Math.max(0, ...ds.map((d) => bandRadiance(d, 3.8e-7, 7.5e-7, 16) * 5.5e-7));
    const floor = peak * 1e-30;
    const span = recipe.decades ?? 8;
    const ramp = parse(recipe.channels[0]!.colour);
    ds.forEach((d, i) => {
      const v = vals[i]!;
      if (!(v > 0) || !(top > 0) || v * mid(src) < floor) return void out.set(d.address, null);
      const t = Math.max(0, 1 + Math.log10(v / top) / span);
      out.set(d.address, t > 0 ? hex(ramp.map((c) => c * (0.15 + 0.85 * t)).map(encode)) : null);
    });
    return out;
  }
  for (const d of ds) {
    // Below the Draper point (about 800 K) heat gives no light an eye or camera sees; a body with nothing else shows nothing.
    if (d.continuum.every((c) => c.kind === "thermal" && c.t_k < DRAPER_K) && recipe.channels.every((c) => !("band" in c.source) || c.source.band === "visible")) {
      out.set(d.address, null);
      continue;
    }
    let rgb: [number, number, number];
    if (recipe.mode === "eye") rgb = linearSrgb(xyz(d));
    else {
      // A camera is white-balanced to daylight: each filter is scaled so a 6504 K source (about D65) comes out white.
      const balance = recipe.mode === "camera" ? recipe.channels.map((c) => sourceRadiance(DAYLIGHT, c.source)) : null;
      const vals = recipe.channels.map((c, i) => sourceRadiance(d, c.source) / (balance ? balance[i]! : 1));
      const m = Math.max(...vals);
      if (!(m > 0)) {
        out.set(d.address, null);
        continue;
      }
      rgb = [0, 0, 0];
      recipe.channels.forEach((c, i) => {
        const col = parse(c.colour);
        for (let k = 0; k < 3; k++) rgb[k]! += (vals[i]! / m) * col[k]!;
      });
      if (recipe.mode === "camera") {
        const mean = (rgb[0] + rgb[1] + rgb[2]) / 3;
        const s = recipe.saturation ?? 1;
        rgb = rgb.map((v) => mean + (v - mean) * s) as [number, number, number];
      }
    }
    const c = chromaticity(rgb);
    out.set(d.address, c[0] + c[1] + c[2] > 0 ? hex(c.map(encode)) : null);
  }
  return out;
}

// ── Descriptions from what a scene already knows (stubbed) ─────────────────

/** What light descriptions are built from: a body's kind, its numbers and its parent. */
export interface LitBody {
  address: string;
  name: string;
  kind: string;
  parent: string | null;
  num(field: string): number | undefined;
}

const R_SUN_M = 695_700_000;
const AU_M = 149_597_870_700;

/**
 * Stubbed descriptions: a star is a blackbody at its surface temperature; a
 * planet reflects its star with a grey albedo (diluted by its distance) and
 * glows at its own surface temperature. Bodies without the numbers get none.
 */
export function describeLight(bodies: readonly LitBody[]): { lights: LightDescription[]; missing: string[] } {
  const lights: LightDescription[] = [];
  const missing: string[] = [];
  const byId = new Map(bodies.map((b) => [b.address, b]));
  const starOf = (b: LitBody): LitBody | undefined => {
    let p = b.parent ? byId.get(b.parent) : undefined;
    while (p && p.kind !== "star") p = p.parent ? byId.get(p.parent) : undefined;
    return p;
  };
  const distanceAu = (b: LitBody, star: LitBody): number | undefined => {
    let x: LitBody | undefined = b;
    while (x && x.parent !== star.address) x = x.parent ? byId.get(x.parent) : undefined;
    return x?.num("a_au");
  };
  for (const b of bodies) {
    if (b.kind === "star") {
      const t = b.num("teff_k");
      if (t === undefined) missing.push(b.name);
      else lights.push({ address: b.address, continuum: [{ kind: "thermal", t_k: t, weight: 1 }], lines: [], stubbed: true, why: `A blackbody at ${Math.round(t)} K, the star's surface temperature. Stubbed: no absorption lines, no corona, no flares yet.` });
      continue;
    }
    if (b.kind !== "planet" && b.kind !== "moon") continue;
    const star = starOf(b);
    const ts = star?.num("teff_k");
    const albedo = b.num("albedo");
    const surface = b.num("surface_k");
    const a = star ? distanceAu(b, star) : undefined;
    const rs = (star?.num("radius_sun") ?? 1) * R_SUN_M;
    const c: Continuum[] = [];
    const parts: string[] = [];
    if (ts !== undefined && albedo !== undefined && a !== undefined && a > 0) {
      // A lit hemisphere's mean radiance: albedo × (R*/a)² / 4 of the star's.
      const dilution = (rs / (a * AU_M)) ** 2 / 4;
      c.push({ kind: "reflected", t_k: ts, weight: albedo * dilution });
      parts.push(`starlight reflected with a grey albedo of ${albedo}`);
    }
    if (surface !== undefined) {
      c.push({ kind: "thermal", t_k: surface, weight: 1 - (albedo ?? 0) });
      parts.push(`its own heat at ${Math.round(surface)} K`);
    }
    if (!c.length) {
      missing.push(b.name);
      continue;
    }
    lights.push({ address: b.address, continuum: c, lines: [], stubbed: true, why: `${parts.join(", plus ")[0]!.toUpperCase()}${parts.join(", plus ").slice(1)}. Stubbed: real reflectance spectra and atmospheres come with the kernel's light facts.` });
  }
  return { lights, missing };
}

/** The setting this device keeps: a recipe id, or "symbols" for the chart's own colours. */
export const SYMBOLS = "symbols";
export function readLight(raw: string | null, ids: readonly string[]): string {
  try {
    const v = raw ? JSON.parse(raw) : null;
    return typeof v?.recipe === "string" && (v.recipe === SYMBOLS || ids.includes(v.recipe)) ? v.recipe : SYMBOLS;
  } catch {
    return SYMBOLS;
  }
}

// Adapters: the GM's system scene and a players' scene, as lit bodies.

export function litFromSystem(scene: SystemScene): LitBody[] {
  const star = scene.subjects.find((s) => s.kind === "star");
  return scene.subjects.map((s) => ({
    address: s.address,
    name: s.label,
    kind: s.kind,
    parent: s === star ? null : (s.parent ?? star?.address ?? null),
    num: (f: string) => {
      const v = f === "a_au" ? s.orbit?.a_au : s.facts.find((x) => x.field === f)?.value;
      return typeof v === "number" ? v : undefined;
    },
  }));
}

/** Rough surface temperatures for spectral classes O to M at subclass 0 (main sequence); subclasses step toward the next class. */
const CLASS_K: Record<string, number> = { O: 41000, B: 31000, A: 9700, F: 7200, G: 5900, K: 5250, M: 3850, L: 2100 };
const ORDER = "OBAFGKML";
/** A main-sequence temperature from a spectral type such as "K1V", or undefined. Stubbed, and labelled so where used. */
export function teffFromSpectral(t: string): number | undefined {
  const m = /^([OBAFGKML])(\d(?:\.\d)?)?/.exec(t.trim());
  if (!m) return undefined;
  const a = CLASS_K[m[1]!]!;
  const next = ORDER[ORDER.indexOf(m[1]!) + 1];
  const b = next ? CLASS_K[next]! : a * 0.8;
  const sub = m[2] ? Number(m[2]) : 5;
  return a + ((b - a) * sub) / 10;
}

const PLAYER_FIELDS: Record<string, string> = { a_au: "orbit.semi_major_axis_m", surface_k: "surface.temp_mean_k" };
export function litFromPlayers(scene: PlayerScene): LitBody[] {
  return scene.subjects.map((s) => {
    const kind = s.address === scene.scope ? "star" : playerSubjectOf(s).kind || "planet";
    const get = (field: string) => {
      const x = s.facts.find((y) => y.field === field);
      return typeof x?.value === "number" ? x.value : x?.band ? (x.band[0] + x.band[1]) / 2 : typeof x?.value === "string" ? x.value : undefined;
    };
    return {
      address: s.address,
      name: subjectName(s),
      kind,
      parent: s.address === scene.scope ? null : (parentOf(s.address, scene.scope) ?? scene.scope),
      num: (f: string) => {
        if (f === "teff_k") {
          const sp = get("spectral");
          return typeof sp === "string" ? teffFromSpectral(sp) : undefined;
        }
        const v = get(PLAYER_FIELDS[f] ?? f);
        if (typeof v !== "number") return undefined;
        return f === "a_au" ? v / AU_M : v;
      },
    };
  });
}
