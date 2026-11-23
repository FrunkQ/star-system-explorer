// Sound (N1, SSE4-sound-design.md): one more renderer, driven by the same
// facts as every picture after the same data-view filter. This module is
// pure data and arithmetic: the SoundScene (what is audible from here, and a
// deterministic score of timed notes), the recipe schema, and the user's
// settings. The browser engine (packages/sound) turns a score into samples;
// nothing reads its output back, and no sound setting ever reaches a session,
// so the state hash cannot change.
//
// Determinism: the score uses only + - * / and sqrt (all correctly rounded in
// IEEE 754, so the same on every engine) and integer hashing for keyed draws.
// No Math.random, no clock, no log or pow.
import { primaryOf, type SceneSubject, type SystemScene } from "./scene.ts";
import type { PlayerScene, PlayerSubject } from "./player-scene.ts";
import { constant } from "./sheet.ts";

export const SOUND_SCENE_SCHEMA = "sse.sound-scene/0" as const;
export const SOUND_RECIPE_SCHEMA = "sse.sound-recipe/0" as const;

/** The honesty label every sound carries (sound design 1.3). */
export type SoundLabel = "real" | "translated" | "designed";
export const SOUND_LABELS: Record<SoundLabel, string> = { real: "Real sound", translated: "Translated signal", designed: "Designed" };
/** The mixer's four layers. */
export type SoundLayer = "world" | "instruments" | "interface" | "music";
export const SOUND_LAYERS: readonly SoundLayer[] = ["world", "instruments", "interface", "music"];
/** Device sound tiers, declared and shown like render tiers. */
export type SoundTier = "silent" | "basic" | "full";

/** One thing that can be heard from this vantage. */
export interface SoundSource {
  id: string;
  /** The body or thing it belongs to. */
  address: string;
  name: string;
  recipe: string;
  layer: SoundLayer;
  label: SoundLabel;
  /** The physics or the mapping, in words ("Why?"). */
  why: string;
  /** Driving quantities, SI: orbital period, mass, ... Only what the audience may know. */
  drive: Record<string, number>;
  /** Base frequency of this source's tone, Hz, before the settled dial. */
  freq_true: number;
}

/** One timed note in the score. Times are seconds from the start of a window. */
export interface ScoreNote {
  t: number;
  source: string;
  freq: number;
  /** Peak level 0 to 1, before the mixer. */
  level: number;
  /** How long the note sounds, seconds. */
  dur: number;
}

export interface SoundScene {
  schema: typeof SOUND_SCENE_SCHEMA;
  /** The view this was built for (a scope or frame address). */
  vantage: string;
  sources: SoundSource[];
  /** Window length, seconds. Window w covers [w * window_s, (w + 1) * window_s). */
  window_s: number;
  /** One line describing what is sounding, for captions. */
  caption: string;
}

// ---------------------------------------------------------------- keyed draws

/** 32-bit FNV-1a of a string. Integer arithmetic only, so identical everywhere. */
export function hash32(s: string): number {
  let h = 0x811c9dc5;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return h >>> 0;
}

/** A keyed draw in [0, 1): the same key gives the same number on every device. */
export function draw(key: string): number {
  return hash32(key) / 4294967296;
}

// ---------------------------------------------------------------- periods to pitch

const SEC_PER_YEAR = constant("yr_julian");
const SEC_PER_DAY = constant("day");

/**
 * Raise a frequency by whole octaves into [lo, 2 lo). Doubling and halving
 * are exact in binary floating point, so the result is the same everywhere.
 * Returns the frequency and how many octaves it was raised.
 */
export function intoOctave(f: number, lo: number): { freq: number; octaves: number } {
  if (!(f > 0) || !Number.isFinite(f)) return { freq: lo, octaves: 0 };
  let x = f;
  let n = 0;
  while (x < lo) {
    x *= 2;
    n++;
  }
  while (x >= lo * 2) {
    x /= 2;
    n--;
  }
  return { freq: x, octaves: n };
}

/** Just-intonation ratios of a calm scale, for the settled end of the dial. */
const SCALE = [1, 9 / 8, 5 / 4, 4 / 3, 3 / 2, 5 / 3, 15 / 8, 2];

/** The nearest pitch of the calm scale built on `root`, in the same octave band. */
export function settle(f: number, root: number): number {
  let r = f / root;
  let k = 1;
  while (r < 1) {
    r *= 2;
    k /= 2;
  }
  while (r >= 2) {
    r /= 2;
    k *= 2;
  }
  let best = SCALE[0]!;
  for (const s of SCALE) if (Math.abs(s - r) < Math.abs(best - r)) best = s;
  return root * best * k;
}

/** Between the true ratios (dial 0) and the settled scale (dial 1). */
export function dialled(f: number, root: number, dial: number): number {
  const d = Math.min(1, Math.max(0, dial));
  return d === 0 ? f : f + d * (settle(f, root) - f);
}

// ---------------------------------------------------------------- building a scene

/** A body that can sound: what the audience knows of it. */
export interface Voice {
  address: string;
  name: string;
  period_s: number;
  /** Mass in Earth masses, when known: bigger is lower. */
  mass_earth?: number;
}

/** The octave band a body's tone sits in: giants low, worlds in the middle, small bodies high. */
export function bandFor(mass_earth: number | undefined): number {
  if (mass_earth === undefined) return 110;
  if (mass_earth >= 10) return 55;
  if (mass_earth >= 0.05) return 110;
  return 220;
}

const ORBIT_WHY =
  "Each tone is the body's orbital frequency (one over its year) raised by whole octaves into hearing, as the SYSTEM Sounds project does with TRAPPIST-1: period ratios become musical intervals by themselves. Bigger bodies sit lower. A designed sound, not something a microphone would hear.";

/** The background bed for a set of orbiting bodies (recipe "orbit-tones"). Silence is a valid result. */
export function orbitScene(vantage: string, voices: readonly Voice[], recipe = "orbit-tones"): SoundScene {
  const sources: SoundSource[] = voices
    .filter((v) => v.period_s > 0 && Number.isFinite(v.period_s))
    .map((v) => {
      const { freq, octaves } = intoOctave(1 / v.period_s, bandFor(v.mass_earth));
      return {
        id: v.address,
        address: v.address,
        name: v.name,
        recipe,
        layer: "music" as const,
        label: "designed" as const,
        why: `${ORBIT_WHY} ${v.name}: a year of ${fmt(v.period_s / SEC_PER_DAY)} days, raised ${octaves} octaves to ${fmt(freq)} Hz.`,
        drive: { period_s: v.period_s, ...(v.mass_earth !== undefined ? { mass_earth: v.mass_earth } : {}) },
        freq_true: freq,
      };
    });
  const names = sources.map((s) => s.name);
  const caption = sources.length ? `Soft tones, one for each orbit: ${names.join(", ")}` : "Silence: nothing here has a known orbit to sound";
  return { schema: SOUND_SCENE_SCHEMA, vantage, sources, window_s: 60, caption };
}

function fmt(x: number): string {
  return x >= 100 ? String(Math.round(x)) : x >= 10 ? (Math.round(x * 10) / 10).toString() : (Math.round(x * 100) / 100).toString();
}

/** The GM's voices: planets of one frame, with Kepler's third law from the centre's mass (P = sqrt(a^3 / M)). */
export function voicesFromSystem(scene: SystemScene, centre?: string): Voice[] {
  const star = primaryOf(scene);
  const num = (s: SceneSubject, f: string) => {
    const x = s.facts.find((y) => y.field === f)?.value;
    return typeof x === "number" ? x : undefined;
  };
  const m_sun = (star && num(star, "mass_sun")) ?? 1;
  const at = centre ?? star?.address;
  return scene.subjects
    .filter((s) => s.orbit && (s.parent ?? star?.address) === at)
    .map((s) => {
      const year_d = num(s, "year_d");
      const a = s.orbit!.a_au;
      // A moon's year needs its planet's mass; the scene's own year fact wins when it is there.
      const parent = s.parent && s.parent !== star?.address ? scene.subjects.find((p) => p.address === s.parent) : undefined;
      const m = parent ? ((num(parent, "mass_earth") ?? 1) / (constant("GM_sun") / constant("GM_earth"))) : m_sun;
      const period_s = year_d !== undefined ? year_d * SEC_PER_DAY : Math.sqrt((a * a * a) / m) * SEC_PER_YEAR;
      return { address: s.address, name: s.label, period_s, mass_earth: num(s, "mass_earth") };
    });
}

/** The players' voices: only bodies whose period they have been given (a hidden fact is never heard). */
export function voicesFromPlayers(scene: PlayerScene): Voice[] {
  const get = (s: PlayerSubject, f: string) => {
    const x = s.facts.find((y) => y.field === f)?.value;
    return typeof x === "number" ? x : undefined;
  };
  const name = (s: PlayerSubject) => {
    const x = s.facts.find((y) => y.field === "name")?.value;
    return typeof x === "string" ? x : "an unnamed body";
  };
  return scene.subjects.flatMap((s) => {
    const p = get(s, "orbit.period_s");
    if (p === undefined) return [];
    const kg = get(s, "mass_kg");
    return [{ address: s.address, name: name(s), period_s: p, ...(kg !== undefined ? { mass_earth: kg / constant("M_earth") } : {}) }];
  });
}

// ---------------------------------------------------------------- the score

/**
 * The notes of window `w`. Each source swells once per cycle, its cycle the
 * square root of its period relative to the fastest, so inner worlds pulse
 * more often; onsets are keyed draws, so every peer hears the same score.
 * The dial pulls pitches toward a calm scale built on the lowest tone.
 */
export function score(scene: SoundScene, w: number, dial = 0, seed = ""): ScoreNote[] {
  if (!scene.sources.length) return [];
  const pmin = Math.min(...scene.sources.map((s) => s.drive.period_s ?? 1));
  const root = Math.min(...scene.sources.map((s) => s.freq_true));
  const out: ScoreNote[] = [];
  const t0 = w * scene.window_s;
  for (const s of scene.sources) {
    const cycle = Math.min(40, 9 * Math.sqrt((s.drive.period_s ?? pmin) / pmin));
    const freq = dialled(s.freq_true, root, dial);
    // The first cycle that ends inside this window, counted from time zero.
    let k = Math.floor(t0 / cycle);
    for (; k * cycle < t0 + scene.window_s; k++) {
      const jitter = (draw(`${seed}|${scene.vantage}|${s.id}|${k}`) - 0.5) * 0.4 * cycle;
      const t = k * cycle + jitter;
      if (t < t0 || t >= t0 + scene.window_s) continue;
      const level = 0.5 + 0.3 * draw(`${seed}|${s.id}|level|${k}`);
      out.push({ t: round6(t - t0), source: s.id, freq: round6(freq), level: round6(level), dur: round6(Math.min(cycle * 0.9, 12)) });
    }
  }
  return out.sort((a, b) => a.t - b.t || (a.source < b.source ? -1 : a.source > b.source ? 1 : 0));
}

const round6 = (x: number) => Math.round(x * 1e6) / 1e6;

/** A short fingerprint of a score, to compare devices. */
export function scoreHash(notes: readonly ScoreNote[]): string {
  return hash32(JSON.stringify(notes)).toString(16).padStart(8, "0");
}

// ---------------------------------------------------------------- recipes

/** The fixed node set a recipe graph may use (sound design 2). */
export const NODE_TYPES = {
  osc: { wave: ["sine", "triangle", "square", "sawtooth"], ratio: [0.125, 8], detune_cents: [-1200, 1200] },
  noise: { colour: ["white", "pink", "brown"] },
  filter: { kind: ["lowpass", "highpass", "bandpass"], freq_hz: [20, 20000], q: [0.0001, 30] },
  env: { attack_s: [0.001, 30], release_s: [0.001, 60] },
  gain: { gain: [0, 2] },
  delay: { time_s: [0.01, 4], feedback: [0, 0.95] },
} as const;
export type NodeType = keyof typeof NODE_TYPES;

export interface RecipeNode {
  id: string;
  type: NodeType;
  [setting: string]: unknown;
}

export interface SoundRecipe {
  schema: typeof SOUND_RECIPE_SCHEMA;
  id: string;
  label: string;
  layer: SoundLayer;
  label_kind: SoundLabel;
  why: string;
  nodes: RecipeNode[];
  /** Edges [from, to]; "out" is the recipe's output. Delay feedback is inside the delay node, so the graph has no cycles. */
  wires: [string, string][];
  /** Nodes that only the full sound tier runs. */
  full_only?: string[];
}

/** Every problem with a recipe; empty when it is good. A recipe outside its schema is rejected, never half played. */
export function checkRecipe(r: unknown): string[] {
  const e: string[] = [];
  const o = r as Partial<SoundRecipe> | null;
  if (!o || typeof o !== "object") return ["a recipe is an object"];
  if (o.schema !== SOUND_RECIPE_SCHEMA) e.push(`schema must be ${SOUND_RECIPE_SCHEMA}`);
  if (typeof o.id !== "string" || !/^[a-z0-9][a-z0-9.-]*$/.test(o.id)) e.push("id must be lower case letters, digits, dots and dashes");
  if (!SOUND_LAYERS.includes(o.layer as SoundLayer)) e.push(`layer must be one of ${SOUND_LAYERS.join(", ")}`);
  if (!(o.label_kind && o.label_kind in SOUND_LABELS)) e.push("label_kind must be real, translated or designed");
  if (typeof o.why !== "string" || !o.why.trim()) e.push("every sound says why");
  const ids = new Set<string>();
  if (!Array.isArray(o.nodes) || !o.nodes.length) e.push("a recipe needs nodes");
  else
    for (const n of o.nodes) {
      if (typeof n?.id !== "string" || n.id === "out" || ids.has(n.id)) {
        e.push(`node id ${String(n?.id)} is missing, repeated or reserved`);
        continue;
      }
      ids.add(n.id);
      const spec = NODE_TYPES[n.type as NodeType] as Record<string, readonly unknown[]> | undefined;
      if (!spec) {
        e.push(`node ${n.id}: unknown type ${String(n.type)}`);
        continue;
      }
      for (const [k, v] of Object.entries(n)) {
        if (k === "id" || k === "type") continue;
        const allowed = spec[k];
        if (!allowed) e.push(`node ${n.id}: ${n.type} has no setting ${k}`);
        else if (typeof allowed[0] === "string") {
          if (!allowed.includes(v)) e.push(`node ${n.id}: ${k} must be one of ${allowed.join(", ")}`);
        } else if (typeof v !== "number" || v < (allowed[0] as number) || v > (allowed[1] as number)) e.push(`node ${n.id}: ${k} must be a number from ${allowed[0]} to ${allowed[1]}`);
      }
    }
  if (!Array.isArray(o.wires)) e.push("wires must be a list");
  else {
    for (const w of o.wires) {
      if (!Array.isArray(w) || w.length !== 2 || !ids.has(w[0]) || !(ids.has(w[1]) || w[1] === "out")) e.push(`wire ${JSON.stringify(w)} joins nodes that do not exist`);
    }
    if (!o.wires.some((w) => Array.isArray(w) && w[1] === "out")) e.push("nothing is wired to out");
    // No cycles: repeatedly remove nodes with no incoming wire.
    const edges = o.wires.filter((w) => Array.isArray(w) && ids.has(w[0]) && ids.has(w[1]));
    let left = [...ids];
    for (let changed = true; changed; ) {
      changed = false;
      for (const id of left) {
        if (!edges.some(([a, b]) => b === id && left.includes(a))) {
          left = left.filter((x) => x !== id);
          changed = true;
        }
      }
    }
    if (left.length) e.push(`the wires loop through ${left.join(", ")}; use a delay's feedback instead`);
  }
  for (const id of o.full_only ?? []) if (!ids.has(id)) e.push(`full_only names ${id}, which is not a node`);
  return e;
}

// ---------------------------------------------------------------- settings

/** The user's sound settings. They live on this device only: never in a session, a pack or the chronicle. */
export interface SoundSettings {
  on: boolean;
  tier: SoundTier;
  mixer: Record<SoundLayer, number>;
  /** 0: true ratios, exactly the physics; 1: settled into a calm scale. */
  dial: number;
  captions: boolean;
}

export function defaultSound(deviceTier?: string): SoundSettings {
  return { on: false, tier: deviceTier === "basic" ? "basic" : "full", mixer: { world: 0.8, instruments: 0.7, interface: 0.5, music: 0.35 }, dial: 0, captions: true };
}

/** Settings read back from storage, anything unknown replaced by the default. Off unless explicitly on. */
export function readSound(raw: string | null, deviceTier?: string): SoundSettings {
  const d = defaultSound(deviceTier);
  if (!raw) return d;
  try {
    const o = JSON.parse(raw) as Partial<SoundSettings>;
    const mixer = { ...d.mixer };
    for (const l of SOUND_LAYERS) {
      const v = o.mixer?.[l];
      if (typeof v === "number" && v >= 0 && v <= 1) mixer[l] = v;
    }
    return {
      on: o.on === true,
      tier: o.tier === "silent" || o.tier === "basic" || o.tier === "full" ? o.tier : d.tier,
      mixer,
      dial: typeof o.dial === "number" ? Math.min(1, Math.max(0, o.dial)) : 0,
      captions: o.captions !== false,
    };
  } catch {
    return d;
  }
}

/** How many voices a tier sounds at once. */
export function voiceLimit(tier: SoundTier): number {
  return tier === "silent" ? 0 : tier === "basic" ? 6 : 24;
}
