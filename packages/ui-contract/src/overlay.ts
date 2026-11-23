// Overlays a pack lays over another view (V3 gap 18, Traveller thread 23 Nov 2026): a hex grid, subsector bounds,
// borders, pinned to a map's own coordinates and drawn under the HUD switch. Nothing here names a game: a stacked
// pack declares its overlays as data (`provides.other.overlays`), the campaign keeps where the GM put each one
// (its anchor), and a view that carries the overlay's frame draws whatever this file returns for its bounds.
//
// A view draws an overlay only while its HUD is on (a view with no HUD switch always shows its labels, so always
// draws them), and lists it in its Layers tool so the GM can turn one off.

/** Coordinate frames a view can carry. galaxy-plane-pc: x and y in parsecs on the galactic plane (star cells' xyz_pc). */
export type OverlayFrame = "galaxy-plane-pc" | "cluster-pc" | "system-au";
export const OVERLAY_FRAMES: readonly OverlayFrame[] = ["galaxy-plane-pc", "cluster-pc", "system-au"];

/** A hex grid as data: the same shape as the hex map view's grid (hex-map/src/grid.ts GridConfig). */
export interface OverlayGrid {
  block: { columns: number; rows: number };
  blocks: { across: number; down: number };
  label: string;
  block_names?: string[];
  shifted: "even" | "odd";
  /** Centre to centre, in the frame's unit. */
  hex_distance: { value: number; unit: string };
}

/** One overlay as a pack declares it. `grid` is a grid, or the id of a view config whose `grid` to use. */
export interface OverlaySpec {
  id: string;
  label: string;
  frame: OverlayFrame;
  grid?: OverlayGrid | string;
  /** Fixed lines and labels in the overlay's own coordinates (before the anchor), e.g. drawn borders. */
  layers?: OverlayLayer[];
  /** The pack it came from (filled in when read from the stack). */
  pack?: string;
}

/** Where the GM put an overlay: its origin (hex 0101's centre for a grid) in the frame, and a turn. Kept in the campaign. */
export interface OverlayAnchor {
  origin: [number, number];
  rotation_deg: number;
}

export type OverlayInk = "faint" | "ink" | "accent";
export type OverlayLayer =
  | { kind: "lines"; ink: OverlayInk; items: { points: [number, number][]; closed?: boolean }[] }
  | { kind: "hexes"; ink: OverlayInk; /** The anchor's turn, for drawing each hex. */ rotation_deg?: number; items: { centre: [number, number]; size: number; orient: "flat" | "point"; label?: string }[] }
  | { kind: "labels"; ink: OverlayInk; items: { at: [number, number]; text: string }[] };

/** An overlay ready to draw: every point already in the frame's coordinates. */
export interface ViewOverlay {
  id: string;
  label: string;
  pack: string;
  frame: OverlayFrame;
  layers: OverlayLayer[];
  /** Set instead of layers when there is nothing to draw yet, and why ("place the sector on the map first"). */
  why?: string;
}

type PackLike = { id?: string; provides?: { other?: Record<string, unknown> } } | null | undefined;
const other = (p: unknown): Record<string, unknown> => ((p as PackLike)?.provides?.other ?? {}) as Record<string, unknown>;

/** Every overlay the stack declares (`provides.other.overlays`), bottom to top; a higher pack's overlay of the same id wins. */
export function overlaySpecsOf(packs: readonly unknown[]): OverlaySpec[] {
  const out = new Map<string, OverlaySpec>();
  for (const p of packs) {
    const list = other(p).overlays;
    if (!Array.isArray(list)) continue;
    for (const o of list as OverlaySpec[]) {
      if (!o || typeof o.id !== "string" || !OVERLAY_FRAMES.includes(o.frame)) continue;
      out.set(o.id, { ...o, pack: String((p as PackLike)?.id ?? "") });
    }
  }
  return [...out.values()];
}

/** The grid an overlay draws: its own, or the named view config's (the top pack that has one wins). */
export function overlayGrid(spec: OverlaySpec, packs: readonly unknown[]): OverlayGrid | null {
  if (spec.grid && typeof spec.grid === "object") return spec.grid;
  if (typeof spec.grid !== "string") return null;
  let found: OverlayGrid | null = null;
  for (const p of packs) {
    const v = (other(p).views as Record<string, { grid?: OverlayGrid }> | undefined)?.[spec.grid];
    if (v?.grid) found = v.grid;
  }
  return found;
}

const pad = (n: number, w: number) => String(n).padStart(w, "0");
/** A hex's label by the grid's label form ("{c2}{r2}" gives 0101). */
export function hexLabel(g: OverlayGrid, col: number, row: number): string {
  return g.label.replace(/\{c(\d)?\}/g, (_, w) => (w ? pad(col, Number(w)) : String(col))).replace(/\{r(\d)?\}/g, (_, w) => (w ? pad(row, Number(w)) : String(row)));
}

/**
 * A hex's centre in the overlay's own frame, hex 0101 at the origin: flat-topped hexes whose centres are
 * `hex_distance` apart, columns to +x, rows counting down the sheet (to -y, north up), shifted columns half a hex
 * lower (Patel, "Hexagonal Grids", redblobgames.com).
 */
export function hexCentre(g: OverlayGrid, col: number, row: number): [number, number] {
  const d = g.hex_distance.value;
  const size = d / Math.sqrt(3);
  const shifted = g.shifted === "even" ? col % 2 === 0 : col % 2 === 1;
  return [(col - 1) * 1.5 * size, -((row - 1) * d + (shifted ? d / 2 : 0))];
}

/** A point in the overlay's own coordinates placed by its anchor: turned about the origin, then moved to it. */
export function placeBy(a: OverlayAnchor, p: [number, number]): [number, number] {
  const t = (a.rotation_deg * Math.PI) / 180;
  const c = Math.cos(t);
  const s = Math.sin(t);
  return [a.origin[0] + p[0] * c - p[1] * s, a.origin[1] + p[0] * s + p[1] * c];
}
/** The inverse of placeBy. */
function unplace(a: OverlayAnchor, p: [number, number]): [number, number] {
  const t = (a.rotation_deg * Math.PI) / 180;
  const c = Math.cos(t);
  const s = Math.sin(t);
  const x = p[0] - a.origin[0];
  const y = p[1] - a.origin[1];
  return [x * c + y * s, -x * s + y * c];
}

/** Below this many pixels across a hex the hexes are not drawn, only the sheet and its blocks. */
export const HEX_MIN_PX = 6;
/** From this many pixels across a hex its label is drawn. */
export const HEX_LABEL_PX = 28;
/** Most hexes drawn at once. */
const HEX_MAX = 4000;

/**
 * A grid overlay ready to draw over `bounds` ([x0, y0, x1, y1] in the frame) at `px_per_unit` pixels a frame
 * unit: the sheet's outline and its block bounds always, block names when they fit, the hexes when each is at least
 * HEX_MIN_PX across, and their labels from HEX_LABEL_PX. Without an anchor, nothing to draw and why.
 */
export function gridOverlay(spec: OverlaySpec, g: OverlayGrid, anchor: OverlayAnchor | undefined, bounds: [number, number, number, number], px_per_unit: number): ViewOverlay {
  const base = { id: spec.id, label: spec.label, pack: spec.pack ?? "", frame: spec.frame };
  if (!anchor) return { ...base, layers: [], why: `Place ${spec.label} on the map first` };
  const d = g.hex_distance.value;
  const size = d / Math.sqrt(3);
  const cols = g.block.columns * g.blocks.across;
  const rows = g.block.rows * g.blocks.down;
  // The sheet in its own frame: from the left edge of column 1 to the right edge of the last, top of row 1 to the
  // bottom of the last (shifted columns hang half a hex lower).
  const left = -size;
  const right = (cols - 1) * 1.5 * size + size;
  const top = d / 2;
  const edge = (r: number) => -((r - 1) * d + d / 2);
  const P = (p: [number, number]) => placeBy(anchor, p);
  const box = (x0: number, y0: number, x1: number, y1: number) => ({ points: [P([x0, y0]), P([x1, y0]), P([x1, y1]), P([x0, y1])], closed: true });
  const layers: OverlayLayer[] = [];
  const bottom = edge(rows + 1);
  layers.push({ kind: "lines", ink: "ink", items: [box(left, top, right, bottom)] });
  // Block bounds.
  const blockLines: { points: [number, number][] }[] = [];
  const blockW = g.block.columns * 1.5 * size;
  for (let i = 1; i < g.blocks.across; i++) {
    const x = left + i * blockW;
    blockLines.push({ points: [P([x, top]), P([x, bottom])] });
  }
  for (let j = 1; j < g.blocks.down; j++) {
    const y = top - j * g.block.rows * d;
    blockLines.push({ points: [P([left, y]), P([right, y])] });
  }
  if (blockLines.length) layers.push({ kind: "lines", ink: "faint", items: blockLines });
  const hexPx = 2 * size * px_per_unit;
  const blockPx = blockW * px_per_unit;
  if (g.block_names && blockPx >= 24 && hexPx < HEX_LABEL_PX) {
    const names: { at: [number, number]; text: string }[] = [];
    for (let by = 0; by < g.blocks.down; by++)
      for (let bx = 0; bx < g.blocks.across; bx++) {
        const n = g.block_names[by * g.blocks.across + bx];
        if (n) names.push({ at: P([left + (bx + 0.5) * blockW, top - (by + 0.5) * g.block.rows * d]), text: n });
      }
    layers.push({ kind: "labels", ink: "faint", items: names });
  }
  if (hexPx >= HEX_MIN_PX) {
    // Only the hexes inside the bounds, found in the overlay's own frame.
    const corners = [unplace(anchor, [bounds[0], bounds[1]]), unplace(anchor, [bounds[2], bounds[1]]), unplace(anchor, [bounds[2], bounds[3]]), unplace(anchor, [bounds[0], bounds[3]])];
    const xs = corners.map((c) => c[0]);
    const ys = corners.map((c) => c[1]);
    const c0 = Math.max(1, Math.floor((Math.min(...xs) - size) / (1.5 * size)) + 1);
    const c1 = Math.min(cols, Math.ceil((Math.max(...xs) + size) / (1.5 * size)) + 1);
    const r0 = Math.max(1, Math.floor(-Math.max(...ys) / d));
    const r1 = Math.min(rows, Math.ceil(-Math.min(...ys) / d) + 2);
    const items: { centre: [number, number]; size: number; orient: "flat" | "point"; label?: string }[] = [];
    const labelled = hexPx >= HEX_LABEL_PX;
    outer: for (let c = c0; c <= c1; c++)
      for (let r = r0; r <= r1; r++) {
        if (items.length >= HEX_MAX) break outer;
        items.push({ centre: P(hexCentre(g, c, r)), size, orient: "flat", ...(labelled ? { label: hexLabel(g, c, r) } : {}) });
      }
    if (items.length) layers.push({ kind: "hexes", ink: "faint", rotation_deg: anchor.rotation_deg, items });
  }
  return { ...base, layers };
}

/**
 * The overlays a view draws: every overlay of the stack in one of the view's frames, placed by the campaign's
 * anchors (keyed by overlay id), as far as it falls in `bounds`.
 */
export function overlaysFor(packs: readonly unknown[], frames: readonly OverlayFrame[], anchors: Readonly<Record<string, OverlayAnchor>> | undefined, bounds: [number, number, number, number], px_per_unit: number): ViewOverlay[] {
  const out: ViewOverlay[] = [];
  for (const s of overlaySpecsOf(packs)) {
    if (!frames.includes(s.frame)) continue;
    const a = anchors?.[s.id];
    const g = overlayGrid(s, packs);
    if (g) {
      const o = gridOverlay(s, g, a, bounds, px_per_unit);
      // A grid's fixed layers ride on it, placed by the same anchor.
      if (a && s.layers) o.layers.push(...s.layers.map((l) => placeLayer(l, a)));
      out.push(o);
    } else if (s.layers) {
      out.push(a ? { id: s.id, label: s.label, pack: s.pack ?? "", frame: s.frame, layers: s.layers.map((l) => placeLayer(l, a)) } : { id: s.id, label: s.label, pack: s.pack ?? "", frame: s.frame, layers: [], why: `Place ${s.label} on the map first` });
    }
  }
  return out;
}

function placeLayer(l: OverlayLayer, a: OverlayAnchor): OverlayLayer {
  const P = (p: [number, number]) => placeBy(a, p);
  if (l.kind === "lines") return { ...l, items: l.items.map((i) => ({ ...i, points: i.points.map(P) })) };
  if (l.kind === "hexes") return { ...l, rotation_deg: (l.rotation_deg ?? 0) + a.rotation_deg, items: l.items.map((i) => ({ ...i, centre: P(i.centre) })) };
  return { ...l, items: l.items.map((i) => ({ ...i, at: P(i.at) })) };
}

/** The six corners of a hex of circumradius `size`, turned by `rotation_deg` (flat-topped at 0). */
export function hexCorners(centre: [number, number], size: number, orient: "flat" | "point", rotation_deg = 0): [number, number][] {
  const out: [number, number][] = [];
  const t0 = ((orient === "flat" ? 0 : 30) + rotation_deg) * (Math.PI / 180);
  for (let k = 0; k < 6; k++) {
    const t = t0 + (k * Math.PI) / 3;
    out.push([centre[0] + size * Math.cos(t), centre[1] + size * Math.sin(t)]);
  }
  return out;
}
