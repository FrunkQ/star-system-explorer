// Names that never sit on each other (Alex, 24 Nov, Sol chart note 3: "the main map needs to feel better"). Each body's
// name tries eight places round its mark, the usual one first (up and right), and takes the first that overlaps no
// name already placed, no other body's mark, nothing the chart has said must stay clear (belt names, scale figures)
// and stays inside the chart. When none is clear it takes the one that overlaps least. The selected body goes first,
// so its name never moves for another. Deterministic: the same chart places the same way every time.

export interface LabelBody {
  id: string;
  /** The mark's centre and its radius on screen. */
  x: number;
  y: number;
  r: number;
  /** The name's box size. */
  w: number;
  h: number;
  /** Placed before the rest (the selected body). */
  first?: boolean;
}

export interface Box {
  x: number;
  y: number;
  w: number;
  h: number;
}

export type Side = "ne" | "nw" | "se" | "sw" | "e" | "w" | "n" | "s";

export interface PlacedLabel {
  id: string;
  side: Side;
  /** Where the text goes: its anchor point, its text-anchor and its box. */
  x: number;
  y: number;
  anchor: "start" | "middle" | "end";
  box: Box;
  /** True when no clear place was found (it overlaps least). */
  crowded: boolean;
}

/** The usual place first; then the other corners, then the sides, then above and below. */
export const LABEL_SIDES: readonly Side[] = ["ne", "nw", "se", "sw", "e", "w", "n", "s"];

const GAP = 6;

/** The text's anchor and box for one side of a mark (SVG text: y is the baseline, the box rises `h` above it). */
export function labelAt(b: LabelBody, side: Side): { x: number; y: number; anchor: "start" | "middle" | "end"; box: Box } {
  const d = b.r + GAP;
  const descent = b.h * 0.25;
  const right = side === "ne" || side === "se" || side === "e";
  const left = side === "nw" || side === "sw" || side === "w";
  const anchor = right ? "start" : left ? "end" : "middle";
  const x = right ? b.x + d : left ? b.x - d : b.x;
  // Baselines: above the mark for n, ne and nw; below for s, se and sw; centred beside it for e and w.
  const y = side === "n" || side === "ne" || side === "nw" ? b.y - d * 0.55 : side === "s" || side === "se" || side === "sw" ? b.y + d * 0.55 + b.h - descent : b.y + b.h / 2 - descent;
  const bx = anchor === "start" ? x : anchor === "end" ? x - b.w : x - b.w / 2;
  return { x, y, anchor, box: { x: bx, y: y - b.h + descent, w: b.w, h: b.h } };
}

function overlap(a: Box, b: Box): number {
  const w = Math.min(a.x + a.w, b.x + b.w) - Math.max(a.x, b.x);
  const h = Math.min(a.y + a.h, b.y + b.h) - Math.max(a.y, b.y);
  return w > 0 && h > 0 ? w * h : 0;
}

function outside(a: Box, bounds: Box): number {
  const inside = overlap(a, bounds);
  return a.w * a.h - inside;
}

/** Place every name; `clear` are boxes no name may cover, `bounds` the chart's visible area. */
export function placeLabels(bodies: readonly LabelBody[], clear: readonly Box[], bounds: Box): PlacedLabel[] {
  const order = [...bodies.filter((b) => b.first), ...bodies.filter((b) => !b.first)];
  const marks: (Box & { id: string })[] = bodies.map((b) => ({ id: b.id, x: b.x - b.r, y: b.y - b.r, w: b.r * 2, h: b.r * 2 }));
  const placed: PlacedLabel[] = [];
  for (const b of order) {
    let best: { side: Side; cost: number; at: ReturnType<typeof labelAt> } | null = null;
    for (const side of LABEL_SIDES) {
      const at = labelAt(b, side);
      let cost = outside(at.box, bounds) * 4;
      for (const p of placed) cost += overlap(at.box, p.box) * 3;
      for (const m of marks) if (m.id !== b.id) cost += overlap(at.box, m) * 2;
      for (const c of clear) cost += overlap(at.box, c);
      if (cost === 0) {
        best = { side, cost, at };
        break;
      }
      if (!best || cost < best.cost) best = { side, cost, at };
    }
    const at = best!.at;
    placed.push({ id: b.id, side: best!.side, x: at.x, y: at.y, anchor: at.anchor, box: at.box, crowded: best!.cost > 0 });
  }
  return bodies.map((b) => placed.find((p) => p.id === b.id)!);
}

/** A name's width on screen, estimated from its length and size (the chart's italic display face runs about half an em a letter). */
export function textWidth(text: string, px: number): number {
  return Math.ceil(text.length * px * 0.5 + px * 0.3);
}
