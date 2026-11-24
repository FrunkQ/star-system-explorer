// The detail ladder (VP6, SSE4-visual-polish-design.md sections 3 and 6): one mechanism for "more as you close in".
//
// A layer's detail is built from rungs, each a range of sizes on the body. The top rungs are facts (the kernel's
// tiles, the weather field, a ring's profile) and stay with the thread that draws them. Below the facts, rungs are
// "drawn from the numbers": noise whose largest size, spectrum, stretch, amplitude and speed are each bound to a
// kernel value by the layer's declaration. This file is the rung table, the bindings, the seed and the clock, and
// the HUD's detail line; the noises are in ladder-noise.ts and the GPU node in ladder-node.ts.
//
// Rules (design section 3): a drawn rung is never clickable, named or read by a rule; it is seeded by the body's
// address and clocked by the universe clock, so every seat sees the same detail; a declaration whose kernel value
// is missing draws nothing, and says why; each one is in docs/fudges.md.
import type { Tier } from "../../ui-contract/src/index.ts";
import starters from "./ladder-layers.json" with { type: "json" };
import { cell, curl, NUM, WRAP, ridged, simplex, type V3 } from "./ladder-noise.ts";

export type NoiseKind = "simplex" | "flow" | "curl" | "cell" | "ridged";

/** A kernel value by name (a key of the facts the area thread passes), times a factor, or over another value. */
export interface Bound {
  fact: string;
  times?: number;
  /** Divide by this fact (a ratio of two kernel values, such as jet shear over jet speed). */
  over?: string;
}

/**
 * A layer's declaration: data, so Orac can tune it and a pack can add one. Each area thread supplies its own
 * (ladder-layers.json holds the starters from the design's table) and never writes its own noise.
 */
export interface LadderLayer {
  id: string;
  /** Plain words for the layer ("Curls at band edges"). */
  name: string;
  kind: NoiseKind;
  /** The largest drawn size, metres (the Rhines scale for a giant's curls; the fact floor for ground roughness). */
  largest: Bound;
  /** The smallest size worth drawing, metres (a grain, a cloud droplet's cell); left out, rungs go on to the pixel. */
  smallest?: Bound;
  /** The size where the kernel's facts stop, metres: drawn rungs start no larger than this. Left out, at `largest`. */
  floor?: Bound;
  /**
   * The spectrum's slope as a Hurst exponent H: each rung half the size of the one above has 2^−H of its amplitude
   * (H = 1/3 for velocities in Kolmogorov turbulence, about 0.7 to 0.9 for rough ground). A shape number, not a fact.
   */
  hurst: number;
  /** How much longer features are across `axis` than along it (1 is round); a kernel value or a plain number. */
  stretch?: Bound | number;
  /** What the stretch and the flow follow, in words: the caller supplies the vector (ladder-node.ts `axis`, `along`). */
  along: "jets" | "wind" | "downslope" | "stress" | "ring" | "none";
  /**
   * The top rung's amplitude in the layer's own units (a share of the rung's size for curl and flow displacement; a
   * brightness or a height for the others).
   */
  amplitude: Bound;
  /** The speed of the largest rung, metres per second; "none" for still detail (ground, craters). */
  speed: Bound | "none";
  /** How the size steps from rung to rung (2 when left out: octaves). */
  lacunarity?: number;
  /** For the HUD: whose figures the detail is drawn from ("the wind's figures"). */
  from: string;
  /** Its row in docs/fudges.md (section "Light, rendering and sound", the detail ladder). */
  fudge: string;
}

const KINDS: readonly NoiseKind[] = ["simplex", "flow", "curl", "cell", "ridged"];
const isBound = (b: unknown): b is Bound => !!b && typeof b === "object" && typeof (b as Bound).fact === "string";

/** Check a declaration (from a pack, Orac or the starters): the list of problems, empty when it is sound. */
export function checkLayer(l: Partial<LadderLayer>): string[] {
  const bad: string[] = [];
  if (!l.id) bad.push("no id");
  if (!l.name) bad.push("no name");
  if (!l.kind || !KINDS.includes(l.kind)) bad.push(`kind must be one of ${KINDS.join(", ")}`);
  if (!isBound(l.largest)) bad.push("largest must be a kernel value");
  if (!isBound(l.amplitude)) bad.push("amplitude must be a kernel value");
  if (l.speed !== "none" && !isBound(l.speed)) bad.push("speed must be a kernel value or \"none\"");
  if (typeof l.hurst !== "number" || !(l.hurst >= 0 && l.hurst <= 2)) bad.push("hurst must be a number from 0 to 2");
  if (l.stretch !== undefined && typeof l.stretch !== "number" && !isBound(l.stretch)) bad.push("stretch must be a number or a kernel value");
  if (l.lacunarity !== undefined && !(l.lacunarity >= 1.25 && l.lacunarity <= 4)) bad.push("lacunarity must be from 1.25 to 4");
  if (!l.from) bad.push("no from (whose figures, for the HUD)");
  if (!l.fudge) bad.push("no fudge row");
  return bad;
}

/** The starter declarations (ladder-layers.json), checked. */
export const LADDER_LAYERS: readonly LadderLayer[] = (starters.layers as unknown as LadderLayer[]).map((l) => {
  const bad = checkLayer(l);
  if (bad.length) throw new Error(`ladder-layers.json ${l.id}: ${bad.join("; ")}`);
  return l;
});
export const ladderLayer = (id: string) => LADDER_LAYERS.find((l) => l.id === id);

/** The kernel values a layer's bindings read, by name, as the area thread has them for this body now. */
/**
 * The kernel values a layer's bindings read, by name. A value may be one number for the world, or a profile (one
 * number per latitude band, as the weather op's `jets` give a giant). The rung table is one for the whole body, so a
 * profile is read at its largest magnitude (a ratio at its largest ratio, element by element), which sets the
 * largest size, speed and amplitude drawn anywhere; the caller then scales the node's output per pixel by
 * `profileShare` so each band shows its own share.
 */
export type LadderFacts = Readonly<Record<string, number | readonly (number | null)[] | null | undefined>>;

/** The largest magnitude in a profile (finite entries only), or null when none. */
function peak(xs: readonly (number | null)[]): number | null {
  let best: number | null = null;
  for (const x of xs) if (num(x) && (best === null || Math.abs(x) > best)) best = Math.abs(x);
  return best;
}

/**
 * A profile's share at each entry against the value the ladder was resolved with: |v_i| / peak (a ratio binding
 * element by element), 0 where missing. For a per-pixel amplitude: interpolate it by latitude and multiply the
 * ladder node's output by it. A single number gives [1].
 */
export function profileShare(b: Bound, facts: LadderFacts): number[] {
  const v = facts[b.fact];
  const d = b.over === undefined ? undefined : facts[b.over];
  if (!Array.isArray(v) && !Array.isArray(d)) return [1];
  const n = Math.max(Array.isArray(v) ? v.length : 1, Array.isArray(d) ? d.length : 1);
  const at = (x: unknown, i: number) => (Array.isArray(x) ? x[i] : x);
  const each = Array.from({ length: n }, (_, i) => {
    const a = at(v, i);
    if (!num(a)) return null;
    if (b.over === undefined) return Math.abs(a);
    const q = at(d, i);
    return num(q) && q !== 0 ? Math.abs(a / q) : null;
  });
  const p = peak(each);
  return each.map((x) => (x === null || !p ? 0 : x / p));
}

export interface ResolvedLayer {
  drawn: true;
  layer: LadderLayer;
  largest_m: number;
  smallest_m: number;
  floor_m: number | null;
  stretch: number;
  amplitude: number;
  speed_m_s: number;
  lacunarity: number;
}
export interface NotDrawn {
  drawn: false;
  layer: LadderLayer;
  /** In plain words, for the HUD: what is missing. */
  why: string;
}

const num = (v: unknown): v is number => typeof v === "number" && Number.isFinite(v);
const value = (b: Bound, facts: LadderFacts): number | null => {
  const v = facts[b.fact];
  const d = b.over === undefined ? undefined : facts[b.over];
  if (Array.isArray(v) || Array.isArray(d)) {
    // A profile: the largest magnitude of the (element-by-element) value.
    const n = Math.max(Array.isArray(v) ? v.length : 1, Array.isArray(d) ? d.length : 1);
    const at = (x: unknown, i: number) => (Array.isArray(x) ? x[i] : x);
    const each: (number | null)[] = [];
    for (let i = 0; i < n; i++) {
      const a = at(v, i);
      const q = at(d, i);
      each.push(!num(a) ? null : b.over === undefined ? a : num(q) && q !== 0 ? a / q : null);
    }
    const p = peak(each);
    return p === null ? null : p * (b.times ?? 1);
  }
  if (!num(v)) return null;
  if (b.over === undefined) return v * (b.times ?? 1);
  return num(d) && d !== 0 ? (v / d) * (b.times ?? 1) : null;
};

/**
 * Bind a declaration to this body's facts. Any binding whose fact is missing, not a number, or out of its sense
 * (a size not above zero, a negative amplitude or speed) means the layer is not drawn: rule 3, a flourish with no
 * number behind it is cut. The reason names the fact.
 */
export function resolveLayer(layer: LadderLayer, facts: LadderFacts): ResolvedLayer | NotDrawn {
  const missing: string[] = [];
  const need = (b: Bound | undefined, ok: (v: number) => boolean): number | null => {
    if (!b) return null;
    const v = value(b, facts);
    if (v === null || !ok(v)) {
      const f = facts[b.fact];
      missing.push(b.over && (num(f) || Array.isArray(f)) ? b.over : b.fact);
      return null;
    }
    return v;
  };
  const largest = need(layer.largest, (v) => v > 0);
  const amplitude = need(layer.amplitude, (v) => v >= 0);
  const speed = layer.speed === "none" ? 0 : need(layer.speed, (v) => v >= 0);
  const smallest = need(layer.smallest, (v) => v > 0);
  const floor = need(layer.floor, (v) => v > 0);
  const stretch = typeof layer.stretch === "number" ? layer.stretch : layer.stretch ? need(layer.stretch, (v) => v > 0) : 1;
  if (missing.length || largest === null || amplitude === null || speed === null || stretch === null) {
    const list = [...new Set(missing)].join(", ");
    return { drawn: false, layer, why: `${layer.name}: not drawn, because this world has no ${list} for it to be drawn from.` };
  }
  return {
    drawn: true,
    layer,
    largest_m: largest,
    smallest_m: smallest ?? 0,
    floor_m: floor,
    stretch: Math.min(MAX_STRETCH, Math.max(1, stretch)),
    amplitude,
    speed_m_s: speed,
    lacunarity: Math.max(1.25, layer.lacunarity ?? 2),
  };
}

/**
 * The most a stretch may squeeze the noise. A stretch multiplies the caller's coordinates along the axis, so a ring's
 * radius over its sample spacing (thousands) would push the shader's numbers past float precision; beyond 64 the
 * features are threads either way.
 */
export const MAX_STRETCH = 64;

/** How many drawn rungs each tier allows (design 6.5 and 8.5): B one, H four, U six, at rest all the shader holds. */
export const TIER_RUNGS: Readonly<Record<Tier | "gpu-ultra", number>> = { text: 0, canvas: 1, "gpu-basic": 1, "gpu-high": 4, "gpu-ultra": 6, still: 8 };
/** The most rungs one shader evaluates (ladder-node.ts uniform array length). */
export const MAX_RUNGS = 8;
/** A rung is drawn once its features are this many pixels across (design 6.2)... */
export const RUNG_MIN_PX = 2;
/** ...and fades in over this factor of size, so nothing pops. */
export const RUNG_FADE = 2;
/** Hard stop on the rungs looked at below the largest (a 2^40 range of sizes is far past any screen). */
const RUNG_SEARCH = 40;

/** How finely the view sees the body: metres on the body per screen pixel, and how much of the body is in view. */
export interface ViewScale {
  metres_per_px: number;
  /** The view's span on the body, metres: a rung larger than this is near-uniform across the screen and fades out. */
  span_m: number;
}

/**
 * The scale of a whole body seen as a disc `disc_px` across in a view `view_px` wide. A disc larger than the view
 * shows only part of the body; the span is never more than half way round.
 */
export function discScale(radius_m: number, disc_px: number, view_px: number): ViewScale {
  const mpp = (2 * radius_m) / Math.max(1e-6, disc_px);
  return { metres_per_px: mpp, span_m: Math.min(Math.PI * radius_m, Math.max(1, view_px) * mpp) };
}

/** One rung as the shader takes it. */
export interface Rung {
  /** 0 is the largest drawn rung. */
  index: number;
  size_m: number;
  /** Faded share, 0..1: by pixels at the fine end, by span at the coarse end, and by the tier's allowance. */
  weight: number;
  /** This rung's amplitude (the layer's, times 2^−H per halving). */
  amplitude: number;
  /** Gradient turn, radians 0..2π: one full turn per eddy turnover (flow noise). */
  alpha: number;
  /** Flow phase 0..1 for two-phase advection. */
  phase: number;
  /** Where the noise domain starts for this rung, lattice units modulo 867: the seed plus the anchor (see rungOrigin). */
  origin: [number, number, number];
}

const clamp01 = (x: number) => (x < 0 ? 0 : x > 1 ? 1 : x);
const log = (x: number, b: number) => Math.log(x) / Math.log(b);

/** The same address always gives the same seed (FNV-1a, then a lowbias32 finish). */
export function ladderSeed(address: string): number {
  let h = 2166136261;
  for (let i = 0; i < address.length; i++) h = Math.imul(h ^ address.charCodeAt(i), 16777619);
  h ^= h >>> 16;
  h = Math.imul(h, 0x7feb352d);
  h ^= h >>> 15;
  h = Math.imul(h, 0x846ca68b);
  h ^= h >>> 16;
  return h >>> 0;
}

function unitHash(seed: number, a: number, b: number): number {
  let h = (seed ^ Math.imul(a + 1, 0x9e3779b1) ^ Math.imul(b + 1, 0x85ebca77)) >>> 0;
  h ^= h >>> 16;
  h = Math.imul(h, 0x7feb352d);
  h ^= h >>> 15;
  h = Math.imul(h, 0x846ca68b);
  h ^= h >>> 16;
  return (h >>> 0) / 4294967296;
}

const wrap = (x: number) => ((x % WRAP) + WRAP) % WRAP;

/**
 * Where a rung's noise domain starts: a seeded offset per layer and rung, plus the anchor (the point the caller's
 * coordinates are measured from, metres in the body's frame, in f64) in this rung's lattice units, both modulo the
 * noises' shared 867-cell period (ladder-noise.ts WRAP). The shader then adds only the caller's small local coordinates (rendering rule 3), and the
 * detail stays fixed to the ground wherever the anchor moves.
 */
export function rungOrigin(seed: number, layerId: string, index: number, size_m: number, anchor_m: readonly [number, number, number] = [0, 0, 0]): [number, number, number] {
  const s = ladderSeed(`${seed}:${layerId}`);
  return [0, 1, 2].map((c) => wrap(unitHash(s, index, c) * WRAP + anchor_m[c]! / size_m)) as [number, number, number];
}

/** Eddies of a smaller size turn over faster: velocity ∝ size^(1/3) (Kolmogorov 1941), so turnover ∝ size^(2/3). */
export const EDDY_EXPONENT = 1 / 3;
/** A flow sample is carried this many of its rung's cells before the two-phase blend resets it (Vlachos 2010). */
export const FLOW_REACH = 0.5;

/**
 * The rungs to draw for a resolved layer at this view scale and tier, at universe time `t_s` (seconds; the clock
 * every seat shares). Coarsest first. A rung's weight is the product of its fine fade (0 at RUNG_MIN_PX pixels, 1 at
 * RUNG_FADE times that) and its coarse fade (1 up to the view's span, 0 at twice it), then capped so the weights
 * summed from the coarsest never pass the tier's allowance: the rung beyond the allowance gets what is left, so the
 * window slides smoothly as the view closes in and the cost per pixel stays about the same.
 */
export function rungs(r: ResolvedLayer, view: ViewScale, tier: Tier | "gpu-ultra", t_s: number, seed: number, anchor_m: readonly [number, number, number] = [0, 0, 0]): Rung[] {
  const cap = Math.min(MAX_RUNGS, TIER_RUNGS[tier] ?? 1);
  if (cap <= 0 || !(view.metres_per_px > 0)) return [];
  const top = r.floor_m !== null ? Math.min(r.largest_m, r.floor_m) : r.largest_m;
  const out: Rung[] = [];
  let used = 0;
  for (let i = 0; i < RUNG_SEARCH && out.length < MAX_RUNGS; i++) {
    const size = top / Math.pow(r.lacunarity, i);
    if (size < r.smallest_m) break;
    const px = size / view.metres_per_px;
    if (px < RUNG_MIN_PX) break;
    const fine = clamp01(log(px / RUNG_MIN_PX, RUNG_FADE));
    const coarse = clamp01(log((2 * view.span_m) / size, 2));
    const v = fine * coarse;
    if (v <= 0) continue;
    const w = Math.min(v, Math.max(0, cap - used));
    if (w <= 0) break;
    used += w;
    const amp = r.amplitude * Math.pow(r.lacunarity, -r.layer.hurst * i);
    let alpha = 0;
    let phase = 0;
    if (r.speed_m_s > 0) {
      // The rung's own speed by Kolmogorov scaling, its turnover time, and how many turnovers since the epoch.
      const v_i = r.speed_m_s * Math.pow(size / top, EDDY_EXPONENT);
      const turns = (t_s * v_i) / size;
      alpha = 2 * Math.PI * (turns - Math.floor(turns));
      const cyc = turns / (2 * FLOW_REACH);
      phase = cyc - Math.floor(cyc);
    }
    out.push({ index: i, size_m: size, weight: w, amplitude: amp, alpha, phase, origin: rungOrigin(seed, r.layer.id, i, size, anchor_m) });
  }
  return out;
}

/** A size in the body's own terms: km above 10 km, m above 1 m, else cm or mm. */
export function sizeWords(m: number): string {
  if (m >= 10_000) return `${Math.round(m / 1000).toLocaleString("en-GB")} km`;
  if (m >= 1000) return `${(m / 1000).toFixed(1)} km`;
  if (m >= 1) return `${Math.round(m)} m`;
  if (m >= 0.01) return `${Math.round(m * 100)} cm`;
  return `${Math.max(1, Math.round(m * 1000))} mm`;
}

/**
 * The HUD's one detail line for a view (design 3 and 6.6): where facts end and whose figures the finer detail is
 * drawn from, or why a layer draws nothing. Layers with nothing on screen right now are left out. "" when there is
 * nothing to say.
 */
export function detailLine(layers: readonly { layer: ResolvedLayer | NotDrawn; shown?: readonly Rung[] }[]): string {
  const parts: string[] = [];
  for (const { layer, shown } of layers) {
    if (!layer.drawn) {
      parts.push(layer.why);
      continue;
    }
    if (shown && !shown.length) continue;
    const top = layer.floor_m !== null ? Math.min(layer.largest_m, layer.floor_m) : layer.largest_m;
    parts.push(`${layer.layer.name}: finer than ${sizeWords(top)} is drawn from ${layer.layer.from}, not computed.`);
  }
  return parts.join(" ");
}

/**
 * The ladder's sum on the processor (the canvas tier, CPU bakes, tests): the same as ladder-node.ts's node at `at`,
 * metres from the anchor. A number for simplex, flow, cell (F1) and ridged; a displacement in metres for curl.
 */
export function ladderAt(kind: NoiseKind, rs: readonly Rung[], at: V3<number>, o: { normal?: V3<number>; along?: V3<number>; axis?: V3<number>; stretch?: number } = {}): number | V3<number> {
  const ax = o.axis ?? [0, 0, 1];
  const len = Math.hypot(ax[0], ax[1], ax[2]) || 1;
  const u: V3<number> = [ax[0] / len, ax[1] / len, ax[2] / len];
  const squash = (o.stretch ?? 1) - 1;
  const along = at[0] * u[0] + at[1] * u[1] + at[2] * u[2];
  const p: V3<number> = [at[0] + u[0] * along * squash, at[1] + u[1] * along * squash, at[2] + u[2] * along * squash];
  let s = 0;
  const v: V3<number> = [0, 0, 0];
  for (const r of rs.slice(0, MAX_RUNGS)) {
    const inv = 1 / r.size_m;
    const q: V3<number> = [p[0] * inv + r.origin[0], p[1] * inv + r.origin[1], p[2] * inv + r.origin[2]];
    const w = r.weight * r.amplitude;
    if (kind === "simplex") s += w * simplex(NUM, q, r.alpha).value;
    else if (kind === "ridged") s += w * ridged(NUM, q, r.alpha);
    else if (kind === "cell") s += w * cell(NUM, q).f1;
    else if (kind === "curl") {
      if (!o.normal) throw new Error("ladderAt: curl needs the surface normal");
      const c = curl(NUM, q, r.alpha, o.normal);
      for (let k = 0; k < 3; k++) v[k] = v[k]! + c[k]! * w * r.size_m;
    } else {
      if (!o.along) throw new Error("ladderAt: flow needs the direction it is carried");
      const d = o.along;
      const ph1 = (r.phase + 0.5) % 1;
      const at0 = (ph: number): V3<number> => [q[0] - d[0] * (ph - 0.5) * FLOW_REACH * 2, q[1] - d[1] * (ph - 0.5) * FLOW_REACH * 2, q[2] - d[2] * (ph - 0.5) * FLOW_REACH * 2];
      const w0 = 1 - Math.abs(r.phase * 2 - 1);
      s += w * (w0 * simplex(NUM, at0(r.phase), r.alpha).value + (1 - w0) * simplex(NUM, at0(ph1), r.alpha).value);
    }
  }
  return kind === "curl" ? v : s;
}
