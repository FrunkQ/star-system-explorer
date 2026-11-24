// The sea as the kernel describes it (W-sea2 to W-sea4, contract in handover/status/surface-timelapse.md): a few
// numbers per region, the same keys at a landing site (weather `report`, `data.sea`) and per climate band on the
// globe (`survey.climate.sea.bands[]`). Every key is optional: one the kernel has not sent yet is listed as missing,
// and the drawing keeps its older look for it and says so. Nothing here is physics of its own: the deep colour,
// the absorption and the liquid's refractive index are the kernel's; the drawing only fades the sea floor through
// the water by Beer's law and splits light at the surface by Fresnel's equations, both of which are optics of the
// picture, not of the world.

export type RGB = [number, number, number];

/** One wave train as the kernel draws it from its spectrum (capillary-gravity dispersion, the liquid's own). */
export interface WaveTrain {
  wavelength_m: number;
  amplitude_m: number;
  period_s: number;
  /** Direction it travels, degrees from east, anticlockwise (as the wind's `dir`). */
  dir_deg: number;
}

export interface SeaOptics {
  /** The three wavelengths the colour numbers are given at, nm (red, green, blue). */
  bands_nm?: RGB;
  /** The liquid's absorption, per metre, at bands_nm. */
  absorb_per_m?: RGB;
  /** Its backscattering, per metre, at bands_nm. */
  backscatter_per_m?: RGB;
  /** Refractive index of the liquid (visible light). */
  refractive_index?: number;
  /** Reflectance of deep water, linear, from R∞ ≈ 0.33 b_b/(a + b_b) (Gordon and Morel; the kernel's). */
  deep_rgb?: RGB;
  trains?: WaveTrain[];
  /** The shortest wave that survives the liquid's viscosity, m (the finest ripple drawn). */
  ripple_cutoff_m?: number;
  superfluid?: boolean;
  liquid_note?: string;
  /** Share of the surface white with breaking crests, 0 to 1. */
  whitecap_fraction?: number;
  /** Reflectance of the froth. */
  foam_albedo?: number;
  /** Streaks of froth lie along the wind: degrees from east. */
  streak_dir_deg?: number;
  /** Significant wave height and peak period (kept from W-sea). */
  wave_m?: number;
  period_s?: number;
}

/** Which keys each drawing step needs, by the kernel package that sends them. */
export const SEA_STEPS = {
  colour: ["deep_rgb", "absorb_per_m", "refractive_index"],
  waves: ["trains"],
  froth: ["whitecap_fraction", "foam_albedo"],
} as const satisfies Record<string, readonly (keyof SeaOptics)[]>;

const num = (x: unknown): number | undefined => (typeof x === "number" && Number.isFinite(x) ? x : undefined);
const rgb = (x: unknown): RGB | undefined => {
  if (!Array.isArray(x) || x.length !== 3) return undefined;
  const v = x.map(num);
  return v.every((n) => n !== undefined) ? (v as RGB) : undefined;
};

/** Read the kernel's sea numbers (any object carrying the agreed keys), keeping only well-formed ones. */
export function readSeaOptics(raw: unknown): { optics: SeaOptics; missing: { colour: string[]; waves: string[]; froth: string[] } } {
  const o = (raw && typeof raw === "object" ? raw : {}) as Record<string, unknown>;
  const optics: SeaOptics = {};
  const set = <K extends keyof SeaOptics>(k: K, v: SeaOptics[K] | undefined) => {
    if (v !== undefined) optics[k] = v;
  };
  set("bands_nm", rgb(o.bands_nm));
  set("absorb_per_m", rgb(o.absorb_per_m));
  set("backscatter_per_m", rgb(o.backscatter_per_m));
  set("refractive_index", num(o.refractive_index));
  set("deep_rgb", rgb(o.deep_rgb));
  if (Array.isArray(o.trains)) {
    const trains = o.trains
      .map((t) => (t && typeof t === "object" ? (t as Record<string, unknown>) : {}))
      .map((t) => ({ wavelength_m: num(t.wavelength_m), amplitude_m: num(t.amplitude_m), period_s: num(t.period_s), dir_deg: num(t.dir_deg) }))
      .filter((t): t is WaveTrain => t.wavelength_m !== undefined && t.wavelength_m > 0 && t.amplitude_m !== undefined && t.period_s !== undefined && t.period_s > 0 && t.dir_deg !== undefined)
      .slice(0, MAX_TRAINS);
    if (trains.length) optics.trains = trains;
  }
  set("ripple_cutoff_m", num(o.ripple_cutoff_m));
  if (typeof o.superfluid === "boolean") optics.superfluid = o.superfluid;
  if (typeof o.liquid_note === "string" && o.liquid_note) optics.liquid_note = o.liquid_note;
  set("whitecap_fraction", num(o.whitecap_fraction));
  set("foam_albedo", num(o.foam_albedo));
  set("streak_dir_deg", num(o.streak_dir_deg));
  set("wave_m", num(o.wave_m));
  set("period_s", num(o.period_s));
  const miss = (keys: readonly (keyof SeaOptics)[]) => keys.filter((k) => optics[k] === undefined) as string[];
  return { optics, missing: { colour: miss(SEA_STEPS.colour), waves: miss(SEA_STEPS.waves), froth: miss(SEA_STEPS.froth) } };
}

/** The most wave trains a shader sums (the kernel sends at most this many). */
export const MAX_TRAINS = 6;

/**
 * Fresnel reflectance of unpolarised light at a surface from a medium of index 1 into one of index n, at the cosine
 * of the angle of incidence (the mean of the s and p reflectances). Total internal reflection cannot happen from
 * above, so for n ≥ 1 this is always below 1.
 */
export function fresnel(n: number, cosI: number): number {
  const c = Math.min(1, Math.max(0, cosI));
  const s2 = 1 - c * c;
  const st2 = s2 / (n * n);
  if (st2 >= 1) return 1;
  const ct = Math.sqrt(1 - st2);
  const rs = (c - n * ct) / (c + n * ct);
  const rp = (n * c - ct) / (n * c + ct);
  return 0.5 * (rs * rs + rp * rp);
}

/**
 * Light coming back up out of water `depth_m` deep over a floor of reflectance `floor`: the deep water's own
 * reflectance where the water is thick, the floor where it is thin, faded by the round trip down and up through
 * the absorbing liquid, exp(−2 a z) per channel (Beer–Lambert; the diffuse path length is taken as the depth, a
 * fudge-register row). Without the kernel's numbers, the floor alone (the older look).
 */
export function seaBody(o: SeaOptics, depth_m: number, floor: RGB): RGB {
  if (!o.deep_rgb || !o.absorb_per_m) return [...floor];
  const z = Math.max(0, depth_m);
  return [0, 1, 2].map((i) => {
    const t = Math.exp(-2 * o.absorb_per_m![i]! * z);
    return o.deep_rgb![i]! * (1 - t) + floor[i]! * t;
  }) as RGB;
}

/** The sea in words for the HUD and the card: what the kernel said about the liquid, and what is still to come. */
export function seaWords(o: SeaOptics, missing: ReturnType<typeof readSeaOptics>["missing"]): string {
  const parts: string[] = [];
  if (o.liquid_note) parts.push(o.liquid_note.replace(/\.$/, ""));
  if (o.superfluid) parts.push("superfluid");
  if (o.deep_rgb && o.absorb_per_m) {
    const a = o.absorb_per_m;
    const reddest = a[0] >= a[2] ? "red" : "blue";
    parts.push(`light fades through it fastest in ${reddest} (${fmt(Math.max(a[0], a[2]))} per metre)`);
  }
  const waiting = [missing.colour.length ? "its own colour" : "", missing.waves.length ? "its own waves" : "", missing.froth.length ? "froth" : ""].filter(Boolean);
  if (waiting.length) parts.push(`waiting on the kernel for ${waiting.join(", ")}`);
  return parts.length ? `${parts.join("; ")}.` : "";
}

/** Two significant figures, as a plain number (figures through ui-contract once its plain-number helper lands). */
const fmt = (x: number) => {
  if (!(x > 0)) return "0";
  const e = Math.floor(Math.log10(x)) - 1;
  return String(e < 0 ? Math.round(x * 10 ** -e) / 10 ** -e : Math.round(x / 10 ** e) * 10 ** e);
};
