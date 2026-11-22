// Time-lapse layers on the Body globe (for the system generation video).
// The kernel's `surface_frames` op gives dated keyframes of per-cell fields on
// the S2 plate grid (a cube sphere, n cells a face) and a dated event list.
// This file reads them, finds the frame pair around any time, and colours a
// field for the globe. Nothing is simulated here: plates, eruptions, ice, heat
// and rain are the kernel's. Between keyframes, smooth fields (temperature,
// rain, ice, runoff) blend; plates, coasts and heights show the nearer frame,
// because a blend of two plate maps is not a world that existed.

export type V3 = readonly [number, number, number];

/** One raw keyframe as the kernel sends it (arrays as JSON numbers, or base64 f32 under `<name>_b64`). */
export type RawFrame = Record<string, unknown> & { t_myr: number; sea_level_m?: number };

export interface FrameEvent {
  t_myr: number;
  kind: string;
  text: string;
  dir?: V3;
  radius_km?: number;
  diameter_km?: number;
  plates?: number[];
  why?: string[];
}

export interface RawFrames {
  grid: { n: number; cells: number };
  dirs?: number[];
  dirs_b64?: string;
  frames: RawFrame[];
  events?: FrameEvent[];
  pending?: string[];
  why?: string[];
}

/** The fields a frame carries, by name (units in the name, as the kernel gives them). */
export const FIELDS = ["plate", "continental", "elevation_m", "vent", "arc", "ridge", "ice_m", "temp_k", "rain_m_yr", "runoff_m_yr"] as const;
export type FieldName = (typeof FIELDS)[number];
/** Fields that may blend between keyframes; the rest show the nearer frame. */
const SMOOTH: ReadonlySet<FieldName> = new Set(["ice_m", "temp_k", "rain_m_yr", "runoff_m_yr"]);

export interface Frame {
  t_myr: number;
  sea_level_m: number;
  fields: Partial<Record<FieldName, Float32Array>>;
}

export interface TimeLapse {
  n: number;
  cells: number;
  dirs: Float32Array;
  frames: Frame[];
  events: FrameEvent[];
  pending: string[];
  why: string[];
  /** Direction to cell lookup. */
  index: { cell: number; grid: Map<number, number[]> };
}

function fromB64(s: string): Float32Array {
  const bin = atob(s);
  const u8 = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) u8[i] = bin.charCodeAt(i);
  return new Float32Array(u8.buffer, 0, Math.floor(u8.length / 4));
}

function arrayOf(o: Record<string, unknown>, name: string, len: number): Float32Array | null {
  const b = o[`${name}_b64`];
  const raw = typeof b === "string" ? fromB64(b) : Array.isArray(o[name]) ? Float32Array.from(o[name] as number[], (x) => (typeof x === "boolean" ? (x ? 1 : 0) : Number(x))) : null;
  if (!raw) return null;
  if (raw.length !== len) throw new Error(`surface_frames: ${name} has ${raw.length} values for ${len}`);
  return raw;
}

const gkey = (i: number, j: number, k: number) => (i + 512) * 1048576 + (j + 512) * 1024 + (k + 512);

/** Read the kernel's reply: typed arrays, frames in time order, a direction lookup built once. */
export function readFrames(r: RawFrames): TimeLapse {
  const { n, cells } = r.grid;
  if (cells !== 6 * n * n) throw new Error(`surface_frames: grid ${n} should have ${6 * n * n} cells, not ${cells}`);
  const dirs = arrayOf(r as unknown as Record<string, unknown>, "dirs", cells * 3);
  if (!dirs) throw new Error("surface_frames: no cell directions");
  const frames = r.frames
    .map((f) => {
      const fields: Frame["fields"] = {};
      for (const name of FIELDS) {
        const a = arrayOf(f, name, cells);
        if (a) fields[name] = a;
      }
      return { t_myr: Number(f.t_myr), sea_level_m: Number(f.sea_level_m ?? 0), fields };
    })
    .sort((a, b) => a.t_myr - b.t_myr);
  if (!frames.length) throw new Error("surface_frames: no frames");
  // Lookup grid: buckets about one and a half cells wide.
  const cell = (Math.sqrt((4 * Math.PI) / cells) * 1.5) as number;
  const grid = new Map<number, number[]>();
  for (let i = 0; i < cells; i++) {
    const k = gkey(Math.floor(dirs[i * 3]! / cell), Math.floor(dirs[i * 3 + 1]! / cell), Math.floor(dirs[i * 3 + 2]! / cell));
    const b = grid.get(k);
    if (b) b.push(i);
    else grid.set(k, [i]);
  }
  const events = [...(r.events ?? [])].sort((a, b) => a.t_myr - b.t_myr);
  return { n, cells, dirs, frames, events, pending: r.pending ?? [], why: r.why ?? [], index: { cell, grid } };
}

/** The cell a unit direction falls in (nearest centre). */
export function cellOf(tl: TimeLapse, x: number, y: number, z: number): number {
  const { cell, grid } = tl.index;
  const d = tl.dirs;
  const i0 = Math.floor(x / cell);
  const j0 = Math.floor(y / cell);
  const k0 = Math.floor(z / cell);
  let best = -1;
  let bd = -Infinity;
  for (let i = i0 - 1; i <= i0 + 1; i++)
    for (let j = j0 - 1; j <= j0 + 1; j++)
      for (let k = k0 - 1; k <= k0 + 1; k++) {
        const b = grid.get(gkey(i, j, k));
        if (!b) continue;
        for (const p of b) {
          const dot = x * d[p * 3]! + y * d[p * 3 + 1]! + z * d[p * 3 + 2]!;
          if (dot > bd) ((bd = dot), (best = p));
        }
      }
  return best;
}

/** The keyframes around `t`: the earlier, the later and the blend weight toward the later (0 at the earlier). */
export function bracket(tl: TimeLapse, t_myr: number): { a: Frame; b: Frame; w: number } {
  const fs = tl.frames;
  if (t_myr <= fs[0]!.t_myr) return { a: fs[0]!, b: fs[0]!, w: 0 };
  const last = fs[fs.length - 1]!;
  if (t_myr >= last.t_myr) return { a: last, b: last, w: 0 };
  let k = 1;
  while (fs[k]!.t_myr < t_myr) k++;
  const a = fs[k - 1]!;
  const b = fs[k]!;
  return { a, b, w: (t_myr - a.t_myr) / (b.t_myr - a.t_myr) };
}

/** A field's value at a cell and time: blended for smooth fields, the nearer frame's otherwise. Null if no frame has it. */
export function valueAt(tl: TimeLapse, field: FieldName, cell: number, t_myr: number): number | null {
  const { a, b, w } = bracket(tl, t_myr);
  const fa = a.fields[field];
  const fb = b.fields[field];
  if (!fa && !fb) return null;
  if (!SMOOTH.has(field)) return (w < 0.5 ? (fa ?? fb)! : (fb ?? fa)!)[cell]!;
  if (!fa || !fb) return (fa ?? fb)![cell]!;
  return fa[cell]! + (fb[cell]! - fa[cell]!) * w;
}

/** Sea level at a time (blended). */
export function seaLevelAt(tl: TimeLapse, t_myr: number): number {
  const { a, b, w } = bracket(tl, t_myr);
  return a.sea_level_m + (b.sea_level_m - a.sea_level_m) * w;
}

/** Events between two times (inclusive), in order: what the scrubber marks and the player announces. */
export function eventsBetween(tl: TimeLapse, from_myr: number, to_myr: number): FrameEvent[] {
  const lo = Math.min(from_myr, to_myr);
  const hi = Math.max(from_myr, to_myr);
  return tl.events.filter((e) => e.t_myr >= lo && e.t_myr <= hi);
}

// ---------------------------------------------------------------- layers

type RGB = [number, number, number];
type Stops = readonly (readonly [number, RGB])[];

function rampOf(stops: Stops, v: number): RGB {
  if (v <= stops[0]![0]) return [...stops[0]![1]] as RGB;
  for (let k = 1; k < stops.length; k++) {
    const [b, cb] = stops[k]!;
    const [a, ca] = stops[k - 1]!;
    if (v <= b) {
      const f = (v - a) / (b - a);
      return [ca[0] + (cb[0] - ca[0]) * f, ca[1] + (cb[1] - ca[1]) * f, ca[2] + (cb[2] - ca[2]) * f];
    }
  }
  return [...stops[stops.length - 1]![1]] as RGB;
}

const OCEAN: Stops = [
  [-6000, [18, 40, 82]],
  [-200, [44, 86, 140]],
  [0, [92, 140, 176]],
];
const LAND: Stops = [
  [0, [96, 128, 86]],
  [1000, [158, 152, 104]],
  [3000, [150, 118, 88]],
  [6000, [236, 234, 228]],
];
const TEMPERATURE: Stops = [
  [200, [44, 54, 140]],
  [250, [104, 160, 214]],
  [273.15, [238, 238, 232]],
  [300, [232, 150, 72]],
  [330, [168, 36, 40]],
];
const RAIN: Stops = [
  [0, [214, 196, 152]],
  [0.25, [188, 184, 120]],
  [1, [96, 150, 88]],
  [3, [40, 104, 150]],
];
const RUNOFF: Stops = [
  [0, [70, 66, 60]],
  [0.1, [70, 96, 120]],
  [1, [64, 150, 214]],
  [3, [190, 232, 250]],
];
/** Recency (1: at this frame, decaying with the gap between frames) above which a cell counts as erupting. */
export const VENT_ERUPTING = 0.1;
const VENT: Stops = [
  [0, [0, 0, 0]],
  [0.3, [190, 60, 20]],
  [1, [255, 214, 120]],
];

/** One layer, as data: what it shows, its legend, and what it leaves out. */
export interface LayerSpec {
  id: string;
  label: string;
  /** The fields it reads; if a field is pending, the layer says so. */
  reads: readonly FieldName[];
  legend: { unit: string; stops: readonly { at: number; label: string }[] };
  limits: string;
}

export const LAYERS: readonly LayerSpec[] = [
  { id: "land", label: "Land and sea", reads: ["elevation_m"], legend: { unit: "m from the sea", stops: [{ at: -6000, label: "deep sea" }, { at: 0, label: "coast" }, { at: 3000, label: "3 km" }, { at: 6000, label: "6 km" }] }, limits: "Heights of each keyframe, shown flat-lit when not today (the hill shading is today's relief); coasts from that frame's sea level." },
  { id: "plates", label: "Plates", reads: ["plate", "continental"], legend: { unit: "one colour per plate", stops: [{ at: 0, label: "ocean crust darker" }, { at: 1, label: "continent lighter" }] }, limits: "Plate colours are labels, not physics; boundaries are where neighbouring cells change plate." },
  { id: "volcanism", label: "Volcanism", reads: ["vent", "elevation_m"], legend: { unit: "how recently, 1 at this frame", stops: [{ at: 0, label: "quiet" }, { at: 0.3, label: "ridges, faint" }, { at: 0.6, label: "arcs" }, { at: 1, label: "eruptions" }] }, limits: "Each kind fades with time since it last happened (1 means at this frame); single eruptions shorter than the gap between frames are in the event list." },
  { id: "temperature", label: "Surface temperature", reads: ["temp_k"], legend: { unit: "K, yearly mean", stops: [{ at: 200, label: "200" }, { at: 273.15, label: "freezing" }, { at: 300, label: "300" }, { at: 330, label: "330" }] }, limits: "Yearly mean; seasons and day and night are not shown." },
  { id: "rain", label: "Rain and snow", reads: ["rain_m_yr"], legend: { unit: "m a year", stops: [{ at: 0, label: "desert" }, { at: 0.25, label: "0.25" }, { at: 1, label: "1" }, { at: 3, label: "3" }] }, limits: "Yearly totals from the one shared climate." },
  { id: "ice", label: "Ice and glaciers", reads: ["ice_m", "elevation_m"], legend: { unit: "m of ice", stops: [{ at: 0, label: "none" }, { at: 100, label: "100 m" }, { at: 1000, label: "1 km" }, { at: 3000, label: "3 km" }] }, limits: "Ice thickness from the ice balance; sea ice shows where the kernel gives it." },
  { id: "runoff", label: "Water running off", reads: ["runoff_m_yr", "elevation_m"], legend: { unit: "m a year leaving the ground", stops: [{ at: 0, label: "dry" }, { at: 0.1, label: "0.1" }, { at: 1, label: "1" }, { at: 3, label: "3" }] }, limits: "Water leaving each cell; river channels themselves come with a flow-routing field (not yet sent)." },
];

const hue = (k: number): RGB => {
  // Golden-ratio hues, muted: one per plate, stable for a plate id.
  const h = ((k * 0.61803398875) % 1) * 6;
  const c = 0.42;
  const x = c * (1 - Math.abs((h % 2) - 1));
  const [r, g, b] = h < 1 ? [c, x, 0] : h < 2 ? [x, c, 0] : h < 3 ? [0, c, x] : h < 4 ? [0, x, c] : h < 5 ? [x, 0, c] : [c, 0, x];
  return [(r + 0.38) * 255, (g + 0.38) * 255, (b + 0.38) * 255];
};

function landColour(tl: TimeLapse, cell: number, t: number, sea: number): RGB {
  const e = valueAt(tl, "elevation_m", cell, t);
  if (e === null) return [128, 128, 128];
  return e < sea ? rampOf(OCEAN, e - sea) : rampOf(LAND, e - sea);
}

const mix = (a: RGB, b: RGB, f: number): RGB => [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f, a[2] + (b[2] - a[2]) * f];

/**
 * A layer's colour for a cell at a time, 0..255. `edge` is true where a
 * neighbouring cell lies on another plate (the caller knows its neighbours).
 */
export function layerColour(tl: TimeLapse, layer: string, cell: number, t_myr: number, edge = false): RGB {
  const sea = seaLevelAt(tl, t_myr);
  switch (layer) {
    case "land":
      return landColour(tl, cell, t_myr, sea);
    case "plates": {
      const p = valueAt(tl, "plate", cell, t_myr) ?? 0;
      const cont = (valueAt(tl, "continental", cell, t_myr) ?? 0) > 0.5;
      const c = hue(Math.round(p) + 1);
      const shaded = cont ? mix(c, [255, 255, 255], 0.35) : mix(c, [0, 0, 0], 0.2);
      return edge ? mix(shaded, [20, 20, 24], 0.75) : shaded;
    }
    case "volcanism": {
      const base = mix(landColour(tl, cell, t_myr, sea), [110, 110, 110], 0.6);
      // Three kinds, each a recency 0 to 1: new ocean crust at ridges (faint), arcs over sinking plates (chains of
      // volcanoes), and eruptions (plumes, floods, shields and plains: bright).
      const clamp = (x: number | null) => Math.max(0, Math.min(1, x ?? 0));
      const ridge = clamp(valueAt(tl, "ridge", cell, t_myr));
      const arc = clamp(valueAt(tl, "arc", cell, t_myr));
      const v = clamp(valueAt(tl, "vent", cell, t_myr));
      let c = base;
      if (ridge > 0.02) c = mix(c, [176, 120, 96], 0.35 * ridge);
      if (arc > 0.02) c = mix(c, [210, 92, 40], 0.3 + 0.5 * arc);
      if (v > 0.02) c = mix(c, rampOf(VENT, v), Math.min(1, 0.5 + v / 2));
      return c;
    }
    case "temperature":
      return rampOf(TEMPERATURE, valueAt(tl, "temp_k", cell, t_myr) ?? 273.15);
    case "rain": {
      const e = valueAt(tl, "elevation_m", cell, t_myr);
      if (e !== null && e < sea) return mix(rampOf(OCEAN, e - sea), [150, 150, 160], 0.5);
      return rampOf(RAIN, valueAt(tl, "rain_m_yr", cell, t_myr) ?? 0);
    }
    case "ice": {
      const base = landColour(tl, cell, t_myr, sea);
      const ice = valueAt(tl, "ice_m", cell, t_myr) ?? 0;
      if (ice <= 0.5) return base;
      const f = Math.min(1, 0.45 + Math.log10(1 + ice) / 3.5 * 0.55);
      return mix(base, [244, 248, 252], f);
    }
    case "runoff": {
      const e = valueAt(tl, "elevation_m", cell, t_myr);
      if (e !== null && e < sea) return mix(rampOf(OCEAN, e - sea), [40, 40, 50], 0.4);
      return rampOf(RUNOFF, valueAt(tl, "runoff_m_yr", cell, t_myr) ?? 0);
    }
    default:
      return landColour(tl, cell, t_myr, sea);
  }
}

/** Layers whose fields are all present, and those whose fields the kernel lists as pending (shown, but said). */
/** Words in the kernel's pending notes that tie a note to a field (the kernel names fields or says them in words). */
const FIELD_WORDS: Record<FieldName, readonly string[]> = {
  plate: ["plate"],
  continental: ["continent"],
  elevation_m: ["elevation", "height", "worn"],
  vent: ["vent", "eruption", "volcan"],
  arc: ["arc", "subduct"],
  ridge: ["ridge", "spreading"],
  ice_m: ["ice", "glacier"],
  temp_k: ["temp", "climate"],
  rain_m_yr: ["rain", "climate"],
  runoff_m_yr: ["runoff", "river"],
};

/** Each layer: whether its fields are present, and the kernel's pending notes that bear on it (shown as its limits). */
export function layersFor(tl: TimeLapse): { spec: LayerSpec; available: boolean; pending: string[] }[] {
  return LAYERS.map((spec) => {
    const missing = spec.reads.filter((f) => !tl.frames.some((fr) => fr.fields[f]));
    const pending = tl.pending.filter((note) => spec.reads.some((f) => note === f || FIELD_WORDS[f].some((w) => note.toLowerCase().includes(w))));
    return { spec, available: missing.length === 0, pending };
  });
}

/** What is under a direction at a time: every field's value, the cell, and the nearest dated event within `near_deg`. */
export function pickAt(tl: TimeLapse, d: V3, t_myr: number, near_deg = 10): { cell: number; values: Partial<Record<FieldName, number>>; sea_level_m: number; event: FrameEvent | null; why: string[] } {
  const cell = cellOf(tl, d[0], d[1], d[2]);
  const values: Partial<Record<FieldName, number>> = {};
  for (const f of FIELDS) {
    const v = valueAt(tl, f, cell, t_myr);
    if (v !== null) values[f] = v;
  }
  const cosNear = Math.cos((near_deg * Math.PI) / 180);
  const { a, b } = bracket(tl, t_myr);
  let event: FrameEvent | null = null;
  let best = -Infinity;
  for (const e of eventsBetween(tl, a.t_myr, b.t_myr === a.t_myr ? a.t_myr : b.t_myr)) {
    if (!e.dir) continue;
    const c = e.dir[0] * d[0] + e.dir[1] * d[1] + e.dir[2] * d[2];
    if (c > cosNear && c > best) ((best = c), (event = e));
  }
  return { cell, values, sea_level_m: seaLevelAt(tl, t_myr), event, why: event?.why ?? [] };
}

/**
 * Neighbour test for plate boundaries: true where any of the four directions a
 * small step away falls on another plate. `step` is in radians (about a cell).
 */
export function onPlateEdge(tl: TimeLapse, d: V3, t_myr: number, step: number): boolean {
  const c = cellOf(tl, d[0], d[1], d[2]);
  const p = valueAt(tl, "plate", c, t_myr);
  if (p === null) return false;
  // Two tangents.
  const ax: V3 = Math.abs(d[2]) < 0.9 ? [0, 0, 1] : [1, 0, 0];
  const t1 = norm3(cross(d, ax));
  const t2 = cross(d, t1);
  for (const [u, v] of [[1, 0], [-1, 0], [0, 1], [0, -1]] as const) {
    const q = norm3([d[0] + step * (u * t1[0] + v * t2[0]), d[1] + step * (u * t1[1] + v * t2[1]), d[2] + step * (u * t1[2] + v * t2[2])]);
    if (valueAt(tl, "plate", cellOf(tl, q[0], q[1], q[2]), t_myr) !== p) return true;
  }
  return false;
}

const cross = (a: V3, b: V3): V3 => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
const norm3 = (a: V3): V3 => {
  const l = Math.hypot(a[0], a[1], a[2]) || 1;
  return [a[0] / l, a[1] / l, a[2] / l];
};

/** The time lapse in words, for the text tier and screen readers. */
export function narrateFrame(tl: TimeLapse, layer: string, t_myr: number, label: string): string {
  const spec = LAYERS.find((l) => l.id === layer);
  const ago = tl.frames[tl.frames.length - 1]!.t_myr - t_myr;
  const when = ago <= 0.5 ? "today" : ago >= 1000 ? `${(ago / 1000).toFixed(2)} billion years ago` : `${Math.round(ago)} million years ago`;
  // Shares over the globe from the cells (equal-area enough at this level of telling).
  let land = 0;
  let iced = 0;
  let erupting = 0;
  let tsum = 0;
  let tn = 0;
  const sea = seaLevelAt(tl, t_myr);
  for (let c = 0; c < tl.cells; c++) {
    const e = valueAt(tl, "elevation_m", c, t_myr);
    if (e !== null && e >= sea) land++;
    if ((valueAt(tl, "ice_m", c, t_myr) ?? 0) > 1) iced++;
    if ((valueAt(tl, "vent", c, t_myr) ?? 0) > VENT_ERUPTING) erupting++;
    const t = valueAt(tl, "temp_k", c, t_myr);
    if (t !== null) ((tsum += t), tn++);
  }
  const pct = (x: number) => `${Math.round((100 * x) / tl.cells)}%`;
  const facts = [`land ${pct(land)} of the surface`, `ice on ${pct(iced)}`, `eruptions on ${pct(erupting)}`];
  if (tn) facts.push(`mean surface ${Math.round(tsum / tn)} K`);
  const ev = eventsBetween(tl, t_myr - 50, t_myr + 50)
    .slice(0, 3)
    .map((e) => e.text.replace(/\.$/, ""));
  return `${label}, ${when}, ${spec?.label.toLowerCase() ?? layer}: ${facts.join(", ")}.${ev.length ? ` Around then: ${ev.join("; ")}.` : ""}`;
}

/**
 * The globe's prepared samples recoloured by a layer at a time, for every tier
 * (canvas and GPU both draw from these numbers). Away from the last keyframe
 * the shading normals are the sphere's own: today's hill shading on a past
 * world's coasts would be a false picture.
 */
export function recolour<P extends { faces: ({ s: number; rgb: Uint8ClampedArray; normal: Float32Array; cover: Uint8Array } | null)[] }>(p: P, tl: TimeLapse, layer: string, t_myr: number, dirOf: (face: number, u: number, v: number) => V3): P {
  const today = t_myr >= tl.frames[tl.frames.length - 1]!.t_myr - 0.5;
  const step = Math.sqrt((4 * Math.PI) / tl.cells) * 0.7;
  const faces = p.faces.map((f, face) => {
    if (!f) return null;
    const s = f.s;
    const rgb = new Uint8ClampedArray(f.rgb.length);
    const normal = today ? f.normal : new Float32Array(f.normal.length);
    const span = 2 / s;
    for (let j = 0; j < s; j++)
      for (let i = 0; i < s; i++) {
        const k = j * s + i;
        const d = dirOf(face, -1 + (i + 0.5) * span, -1 + (j + 0.5) * span);
        const cell = cellOf(tl, d[0], d[1], d[2]);
        const edge = layer === "plates" && onPlateEdge(tl, d, t_myr, step);
        const c = layerColour(tl, layer, cell, t_myr, edge);
        rgb[k * 3] = c[0];
        rgb[k * 3 + 1] = c[1];
        rgb[k * 3 + 2] = c[2];
        if (!today) normal.set(d, k * 3);
      }
    return { ...f, rgb, normal };
  });
  return { ...p, faces };
}
