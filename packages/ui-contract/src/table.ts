// ER11a: `sse.table/1`, the one format the kernel bakes tables in for the renderer (edits-reach design section 11;
// Rust `sse_physics::table`). The renderer keeps only geometry and lookups: a law whose input varies across a picture
// is baked by the kernel (`bake` op) from the physics in force, and the picture samples it. This file is the
// renderer's reader of that format: the same node layout and the same multilinear read as the kernel's
// `Table::sample`, which every table's sampling bound is measured with. A GPU pass reads a table as a float texture
// whose texels are the nodes (first axis across, second down; channel c in texel channel c), and indexes it with
// `texelOf` so its hardware or hand-written bilinear read lands on the same nodes.

/** How an axis's nodes are spaced. */
export type Spacing = "even" | "log";

export interface TableAxis {
  name: string;
  unit: string;
  spacing: Spacing;
  /** The first and last nodes. */
  from: number;
  to: number;
  /** Nodes along the axis. */
  n: number;
  about: string;
}

export interface TableChannel {
  name: string;
  unit: string;
  /** A block id or a compiled function. */
  from: string;
  about: string;
}

export interface TableSampling {
  probes: number;
  worst: number[];
  bound: number[];
  floor: number[];
  within: boolean;
}

/** A baked table, as the `bake` op hands it over. */
export interface Table {
  format: "sse.table/1";
  id: string;
  subject: unknown;
  size: "small" | "medium" | "large";
  tier?: string;
  axes: TableAxis[];
  channels: TableChannel[];
  /** Channel fastest, then the first axis, then the second. 32-bit floats. */
  values: ArrayLike<number>;
  /** Keys the renderer's texture. */
  hash: string;
  /** The physics hash it was baked under. */
  physics: string;
  reads: { kind: string; id: string; version?: string }[];
  /** The edits it rests on (filled from ER8). */
  rests_on: string[];
  sampling: TableSampling;
  why: string[];
  stand_ins: string[];
}

/** The fractional node index of `x` on an axis, held to its ends (Rust `Axis::index_of`). */
export function indexOf(a: TableAxis, x: number): number {
  let s: number, lo: number, hi: number;
  if (a.spacing === "log") {
    lo = Math.log(a.from);
    hi = Math.log(a.to);
    s = x > 0 ? Math.log(x) : lo;
  } else {
    lo = a.from;
    hi = a.to;
    s = x;
  }
  const f = ((s - lo) / (hi - lo)) * (a.n - 1);
  if (Number.isNaN(f)) return 0;
  return Math.min(Math.max(f, 0), a.n - 1);
}

/** The value at node `i` of an axis (Rust `Axis::at`). */
export function nodeAt(a: TableAxis, i: number): number {
  if (i <= 0) return a.from;
  if (i >= a.n - 1) return a.to;
  if (a.spacing === "log") return Math.exp(Math.log(a.from) + ((Math.log(a.to) - Math.log(a.from)) * i) / (a.n - 1));
  return a.from + ((a.to - a.from) * i) / (a.n - 1);
}

/** A channel's index by name; throws when the table has no such channel, so a renamed channel fails loudly. */
export function channelOf(t: Table, name: string): number {
  const c = t.channels.findIndex((ch) => ch.name === name);
  if (c < 0) throw new Error(`table ${t.id} has no channel ${name} (it has ${t.channels.map((ch) => ch.name).join(", ")})`);
  return c;
}

/** Multilinear read of every channel at a point (one coordinate per axis), as the kernel's `Table::sample`. */
export function sampleTable(t: Table, x: readonly number[]): number[] {
  const nd = t.axes.length;
  const nch = t.channels.length;
  const lo: number[] = [];
  const fr: number[] = [];
  for (let d = 0; d < nd; d++) {
    const a = t.axes[d]!;
    const f = indexOf(a, x[d] ?? a.from);
    const i = Math.min(Math.floor(f), a.n - 2);
    lo.push(i);
    fr.push(f - i);
  }
  const out = new Array<number>(nch).fill(0);
  for (let corner = 0; corner < 1 << nd; corner++) {
    let w = 1;
    let k = 0;
    for (let d = nd - 1; d >= 0; d--) {
      const up = (corner >> d) & 1;
      k = k * t.axes[d]!.n + lo[d]! + up;
      w *= up ? fr[d]! : 1 - fr[d]!;
    }
    if (w === 0) continue;
    for (let c = 0; c < nch; c++) out[c]! += w * t.values[k * nch + c]!;
  }
  return out;
}

/**
 * Where a point falls in a table's texture, as texture coordinates 0..1 on each axis with texel centres on the
 * nodes: `(index + 0.5) / n`. A bilinear texture read there is the same multilinear read as `sampleTable`.
 */
export function texelOf(t: Table, x: readonly number[]): number[] {
  return t.axes.map((a, d) => (indexOf(a, x[d] ?? a.from) + 0.5) / a.n);
}

/** A table's values as one RGBA float array for a texture (channels padded to four), first axis across. */
export function rgbaOf(t: Table): Float32Array {
  const nch = t.channels.length;
  if (nch > 4) throw new Error(`table ${t.id} has ${nch} channels; a texture holds four`);
  const cells = t.axes.reduce((p, a) => p * a.n, 1);
  const out = new Float32Array(cells * 4);
  for (let k = 0; k < cells; k++) for (let c = 0; c < nch; c++) out[k * 4 + c] = t.values[k * nch + c]!;
  return out;
}
