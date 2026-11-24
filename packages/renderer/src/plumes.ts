// Plumes above the limb in True (third audit R10): the kernel's plume facts (sse-surface plumes.rs, on each world's
// `resolved_system` surface record) as grains in flight over the vents. Nothing is invented: the vents lie in the
// kernel's polar cap, each grain rises on its own ballistic arc to the height its launch speed gives, and the plume's
// brightness follows its strength and how ice grains scatter light (strongly forward, so a plume lit from behind
// shines and one lit from the front is faint, as Cassini saw).

/** A world's plume facts as the kernel writes them (`surfaces[].plumes`). */
export interface PlumeFacts {
  strength: number;
  pole: number;
  cap_deg: number;
  grain_speed_m_s: number;
  escape_speed_m_s: number;
  grain_height_m: number | null;
  gas_escapes: boolean;
  vapour_kg_s: number;
  why: string[];
}

/** The plume facts on a surface record, or null when it has none (or they are malformed). */
export function plumeFacts(surface: unknown): PlumeFacts | null {
  const p = (surface as { plumes?: Partial<PlumeFacts> } | null)?.plumes;
  if (!p || typeof p.strength !== "number" || !(p.strength > 0) || typeof p.cap_deg !== "number") return null;
  return {
    strength: p.strength,
    pole: p.pole === 1 ? 1 : -1,
    cap_deg: p.cap_deg,
    grain_speed_m_s: p.grain_speed_m_s ?? 0,
    escape_speed_m_s: p.escape_speed_m_s ?? 0,
    grain_height_m: typeof p.grain_height_m === "number" ? p.grain_height_m : null,
    gas_escapes: !!p.gas_escapes,
    vapour_kg_s: p.vapour_kg_s ?? 0,
    why: Array.isArray(p.why) ? p.why : [],
  };
}

/** Grains drawn for a plume of strength 1 (a drawing's count, not the plume's: fudge register). */
export const PLUME_GRAINS = 1800;
/** Vents drawn in the cap (Enceladus has about a hundred jets along four fractures; Porco et al. 2014). */
export const PLUME_VENTS = 24;
/** Half-angle of each jet's spread about the local vertical, radians (Porco et al. 2014: jets of about 10° to 15°). */
export const JET_SPREAD = 0.22;

/** Deterministic unit hash, so a plume looks the same every picture. */
const unit = (i: number, k: number) => {
  let h = Math.imul(i ^ 0x9e3779b9, 0x85ebca6b) ^ Math.imul(k + 0x27d4eb2f, 0xc2b2ae35);
  h ^= h >>> 15;
  h = Math.imul(h, 0x2c1b3c6d);
  h ^= h >>> 12;
  return (h >>> 0) / 4294967296;
};

/**
 * The plume's grains in the world's own frame (+z its north pole), in units of its radius: each vent in the cap,
 * each grain on its arc at a moment drawn evenly in its flight. A grain's height follows its launch speed (spread
 * about the kernel's grain speed) by energy, h = R v² / (v_esc² − v²); grains faster than escape are drawn leaving,
 * thinning out to two radii. `radius_m` is the world's radius.
 */
export function plumeGrains(p: PlumeFacts, radius_m: number): Float32Array {
  const n = Math.round(PLUME_GRAINS * Math.min(1, p.strength));
  const out = new Float32Array(n * 3);
  const cap = (p.cap_deg * Math.PI) / 180;
  const vesc = p.escape_speed_m_s;
  for (let i = 0; i < n; i++) {
    const vent = Math.floor(unit(i, 1) * PLUME_VENTS);
    // Vents spread over the cap by area: colatitude from 1 − cos θ uniform.
    const ct = 1 - unit(vent, 2) * (1 - Math.cos(cap));
    const st = Math.sqrt(Math.max(0, 1 - ct * ct));
    const ph = 2 * Math.PI * unit(vent, 3);
    const up: [number, number, number] = [st * Math.cos(ph), st * Math.sin(ph), p.pole * ct];
    // Launch speed: a spread of ±40% about the grain speed (Hedman et al. 2009: 80 to 160 m/s).
    const v = p.grain_speed_m_s * (0.6 + 0.8 * unit(i, 4));
    const hmax = v < vesc ? (radius_m * v * v) / (vesc * vesc - v * v) : 2 * radius_m;
    const u = unit(i, 5);
    // Evenly in time along a rising-and-falling arc (escaping grains: evenly in height, they do not come back).
    const h = (v < vesc ? 4 * hmax * u * (1 - u) : hmax * u) / radius_m;
    // Off the vertical within the jet's cone, growing with height.
    const a = JET_SPREAD * Math.sqrt(unit(i, 6));
    const b = 2 * Math.PI * unit(i, 7);
    const t1 = perp(up);
    const t2 = cross(up, t1);
    const off = Math.tan(a) * h;
    const o = i * 3;
    for (let k = 0; k < 3; k++) out[o + k] = up[k]! * (1 + h) + off * (Math.cos(b) * t1[k]! + Math.sin(b) * t2[k]!);
  }
  return out;
}

/**
 * How bright the grains look, 0..1: Henyey-Greenstein scattering at asymmetry g (micron ice grains scatter forward,
 * g about 0.6 to 0.8 at visible wavelengths; Hedman et al. 2009 fit forward-scattering phase curves), normalised to
 * 1 at full back-lighting, times the plume's strength. `cosPhase` is the cosine of the angle between the light's
 * travel and the line of sight (1: light straight from behind the plume into the eye).
 */
export function plumeBrightness(p: PlumeFacts, cosPhase: number, g = 0.7): number {
  const hg = (c: number) => (1 - g * g) / Math.pow(1 + g * g - 2 * g * c, 1.5);
  return Math.max(0, Math.min(1, p.strength)) * (hg(Math.max(-1, Math.min(1, cosPhase))) / hg(1));
}

/** One sentence for True's note. */
export function plumeNote(name: string, p: PlumeFacts): string {
  const h = p.grain_height_m === null ? "grains escaping" : `grains rising about ${Math.round(p.grain_height_m / 1000)} km`;
  return `${name}: plumes from its ${p.pole > 0 ? "north" : "south"} pole, ${h}, about ${Math.round(p.vapour_kg_s)} kg of vapour a second${p.gas_escapes ? ", the gas escaping along its orbit" : ""} (kernel plume facts; brighter lit from behind, as ice grains scatter forward).`;
}

function perp(v: readonly number[]): [number, number, number] {
  const a: [number, number, number] = Math.abs(v[0]!) < 0.9 ? [1, 0, 0] : [0, 1, 0];
  const c = cross(v, a);
  const l = Math.hypot(c[0], c[1], c[2]) || 1;
  return [c[0] / l, c[1] / l, c[2] / l];
}
function cross(a: readonly number[], b: readonly number[]): [number, number, number] {
  return [a[1]! * b[2]! - a[2]! * b[1]!, a[2]! * b[0]! - a[0]! * b[2]!, a[0]! * b[1]! - a[1]! * b[0]!];
}
