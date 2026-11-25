// Overlays a pack lays over another view (V3 gap 18, Traveller thread 23 Nov 2026): a hex grid, subsector bounds,
// borders, pinned to a map's own coordinates and drawn under the HUD switch. Nothing here names a game: a stacked
// pack declares its overlays as data (`provides.other.overlays`), the campaign keeps where the GM put each one
// (its anchor), and a view that carries the overlay's frame draws whatever this file returns for its bounds.
//
// A view draws an overlay only while its HUD is on (a view with no HUD switch always shows its labels, so always
// draws them), and lists it in its Layers tool so the GM can turn one off.

/**
 * Coordinate frames a view can carry. galaxy-plane-pc: x and y in parsecs on the galactic plane (star cells' xyz_pc).
 * body-latlon (OV2): x longitude east and y latitude north, degrees, on a body's own surface. web-mpc (OV2): x and y
 * in megaparsecs on the cosmic web's chart.
 */
export type OverlayFrame = "galaxy-plane-pc" | "cluster-pc" | "system-au" | "body-latlon" | "web-mpc";
export const OVERLAY_FRAMES: readonly OverlayFrame[] = ["galaxy-plane-pc", "cluster-pc", "system-au", "body-latlon", "web-mpc"];

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

/**
 * One overlay as a pack declares it. `grid` is a grid, or the id of a view config whose `grid` to use. A layer of
 * areas (`areas`, OV2) is about places, not a frame: every view that shows places draws it in its own frame, so it
 * needs no `frame`. A grid or fixed drawing still names one.
 */
export interface OverlaySpec {
  id: string;
  label: string;
  frame?: OverlayFrame;
  /** A map layer: each place gets at most one answer from a short list, and the map colours places by it (OV2). */
  areas?: AreaSpec;
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

/** A map palette slot (territory design 6, ruling 9): one of the theme's eight map colours, each paired with a pattern. */
export type MapSlot = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8;
export const MAP_SLOTS: readonly MapSlot[] = [1, 2, 3, 4, 5, 6, 7, 8];
/** The patterns paired with the slots, for themes and eyes that need more than colour: slot n's is MAP_PATTERNS[n-1]. */
export const MAP_PATTERNS = ["plain", "stripes", "dots", "cross", "waves", "grid", "back-stripes", "chevrons"] as const;
export type MapPattern = (typeof MAP_PATTERNS)[number];
/** The theme token that holds a slot's colour (each theme defines all eight, tuned to its ground: OV4). */
export const mapSlotToken = (slot: MapSlot): string => `--sse-map-${slot}`;

/** The plain inks, or a map palette slot (OV2: only inside map layers, the one exception to "one accent"). */
export type OverlayInk = "faint" | "ink" | "accent" | { slot: MapSlot; pattern?: MapPattern };
export type OverlayLayer =
  | { kind: "lines"; ink: OverlayInk; items: { points: [number, number][]; closed?: boolean }[] }
  | { kind: "hexes"; ink: OverlayInk; /** The anchor's turn, for drawing each hex. */ rotation_deg?: number; items: { centre: [number, number]; size: number; orient: "flat" | "point"; label?: string }[] }
  | { kind: "labels"; ink: OverlayInk; items: { at: [number, number]; text: string }[] }
  | {
      kind: "areas";
      /** Where it sits in the Layers list's stack (design 4.6): the top area layer fills, the next draws edges and
       * hatching only, a third marks places. */
      as: AreaRole;
      items: AreaItem[];
    };

/** How an area layer draws in a stack (design 4.6). */
export type AreaRole = "fill" | "edges" | "marks";
/** The role of the area layer at `index` among the area layers on, top first. */
export const areaRole = (index: number): AreaRole => (index <= 0 ? "fill" : index === 1 ? "edges" : "marks");

/** Holds is control, claims an assertion by someone, influences a soft presence with a strength (design 4.3). */
export type AreaStatus = "holds" | "claims" | "influences";
export const AREA_STATUSES: readonly AreaStatus[] = ["holds", "claims", "influences"];

/**
 * One territory ready to draw, in the view's frame: its answer's key and ink, its status, and its outline as rings
 * (the first the outside, any after it holes or further pieces, by the even-odd rule). `label` and `label_turn`
 * letter its name along its long axis (degrees from +x). `divided` is set on a place no answer holds 60 percent of:
 * the shares, largest first, for stripes and a split mark (design 4.4).
 */
export interface AreaItem {
  key: string;
  ink: OverlayInk;
  status: AreaStatus;
  /** 0..1, for influence: drawn as tint depth. */
  strength?: number;
  rings: [number, number][][];
  label?: string;
  label_turn?: number;
  divided?: { key: string; share: number }[];
  /** The place or holding it came from, for pointing and pressing (opens its card). */
  place?: string;
  /** Not known to the table: drawn in pencil on the GM's map (SV3), never sent to a player's. */
  pencil?: boolean;
}

/** A distance with its unit, as packs write them ("4 pc"). */
export interface Quantity {
  value: number;
  unit: string;
}

/** What a map layer reads at each place (design 4.2). */
export type AreaSource =
  /** A relation to an actor: holds, claims or influences (the kernel's OV3). */
  | { relation: string }
  /** An exclusive tag group: each tag an answer. */
  | { tag_group: string }
  /** A `choice` kept value: each choice an answer. */
  | { choice: string }
  /** A fact with few answers, or a number cut into bands at `bands` (ascending). Read-only from physics. */
  | { fact: string; bands?: number[] }
  /** What an audience knows ("charted by the party"). */
  | { knowledge: "charted" }
  /** A reach the layer computes: from a place, out to a distance given or read from a fact, growing at a rate
   * (light speed for the news horizon: `rate: { value: 1, unit: "c" }`). */
  | { reach: { from: string; radius: Quantity | { fact: string }; rate?: Quantity } };

/** How the gaps between places are filled, per scale (design 4.5). The kernel holds the rules; views draw the answers. */
export interface AreaFill {
  /** Cosmic web: galaxies and groups colour space out to a reach. */
  web?: { reach: Quantity };
  /** Local stars and sectors: each held system colours space out to a reach, met halfway; "grid" follows a placed pack grid's hexes. */
  between_stars?: { reach: Quantity } | "grid";
  /** In a system: Hill spheres round bodies, or bands between orbits. */
  in_system?: "hill" | "bands";
  /** On a surface: the atlas's regions, or outlines drawn by hand. */
  on_surface?: "regions" | "outlines";
}

/** A map layer as a pack declares it (design 4.1). */
export interface AreaSpec {
  by: AreaSource;
  /** The set of answers, and so the key: actors under a parent, a tag group's tags, a choice's options, the bands. */
  answers?: { from: "actors" | "tags" | "choices" | "bands"; under?: string };
  /** How far down the answer tree to split: "auto" by zoom, or a depth from 0 (the top). */
  level?: "auto" | number;
  fill?: AreaFill;
  /** What "most of it" means when rolling up a place: population where kept, else area, else count. */
  weigh_by?: "population" | "area" | "count";
  /** A drawing recipe (design 6): sse.look.atlas by default, sse.look.cells for the flat hex look. */
  look?: string;
  /** Default disclosure, as for a tag. */
  tier?: string;
}

/** The default reach between stars (ruling 6). */
export const AREA_REACH_DEFAULT: Quantity = { value: 4, unit: "pc" };
/** Below this share a rolled-up place is drawn divided (ruling 5). */
export const AREA_DIVIDED_BELOW = 0.6;
export const AREA_LOOKS = ["sse.look.atlas", "sse.look.cells"] as const;

const SOURCES = ["relation", "tag_group", "choice", "fact", "knowledge", "reach"] as const;
/**
 * What is wrong with a map layer as written, in words for its author (a GM, a pack maker or Orac); empty when it
 * can be read. Unknown looks are allowed (a pack may bring its own recipe).
 */
export function areaProblems(a: unknown): string[] {
  const out: string[] = [];
  const o = (a ?? {}) as Record<string, unknown>;
  const by = o.by as Record<string, unknown> | undefined;
  const keys = by && typeof by === "object" ? Object.keys(by).filter((k) => (SOURCES as readonly string[]).includes(k)) : [];
  if (keys.length !== 1) out.push(`"by" names one thing to read at each place: one of ${SOURCES.join(", ")}`);
  else {
    const k = keys[0]!;
    const v = by![k];
    if (k === "knowledge" && v !== "charted") out.push(`"knowledge" can only be "charted" for now`);
    else if (k === "fact" && by!.bands !== undefined) {
      const b = by!.bands;
      if (!Array.isArray(b) || b.some((x, i) => typeof x !== "number" || !Number.isFinite(x) || (i > 0 && x <= (b[i - 1] as number)))) out.push(`"bands" is a list of numbers, each larger than the last`);
    } else if (k === "reach") {
      const r = v as Record<string, unknown> | undefined;
      if (!r || typeof r.from !== "string" || !r.radius) out.push(`a reach needs "from" (a place) and "radius" (a distance or a fact)`);
    } else if (k !== "fact" && k !== "reach" && (typeof v !== "string" || !v)) out.push(`"${k}" names what to read`);
  }
  if (o.level !== undefined && o.level !== "auto" && !(Number.isInteger(o.level) && (o.level as number) >= 0)) out.push(`"level" is "auto" or a depth from 0`);
  if (o.weigh_by !== undefined && !["population", "area", "count"].includes(String(o.weigh_by))) out.push(`"weigh_by" is population, area or count`);
  const f = o.fill as Record<string, unknown> | undefined;
  const reachOk = (q: unknown) => {
    const r = (q as { reach?: Quantity } | undefined)?.reach;
    return !!r && typeof r.value === "number" && r.value > 0 && typeof r.unit === "string";
  };
  if (f) {
    if (f.web !== undefined && !reachOk(f.web)) out.push(`"fill.web" is { reach: { value, unit } }`);
    if (f.between_stars !== undefined && f.between_stars !== "grid" && !reachOk(f.between_stars)) out.push(`"fill.between_stars" is "grid" or { reach: { value, unit } }`);
    if (f.in_system !== undefined && !["hill", "bands"].includes(String(f.in_system))) out.push(`"fill.in_system" is hill or bands`);
    if (f.on_surface !== undefined && !["regions", "outlines"].includes(String(f.on_surface))) out.push(`"fill.on_surface" is regions or outlines`);
  }
  return out;
}

/** Every map layer the stack declares (overlays with `areas` that read cleanly), bottom to top, a higher pack's winning. */
export function areaSpecsOf(packs: readonly unknown[]): (OverlaySpec & { areas: AreaSpec })[] {
  return overlaySpecsOf(packs).filter((s): s is OverlaySpec & { areas: AreaSpec } => !!s.areas && areaProblems(s.areas).length === 0);
}

/**
 * Give each answer in view a palette slot (design 6): neighbours never share one, an answer's own preference is
 * kept when no neighbour has it, and with more than eight answers in view slots are reused by answers that do not
 * touch. Deterministic: answers are taken in order of key (code units), so every device colours the map alike.
 * Where every slot is already on a neighbour (eight or more neighbours), the one fewest neighbours use is shared.
 */
export function assignSlots(answers: readonly { key: string; prefer?: MapSlot }[], touching: readonly (readonly [string, string])[]): Map<string, MapSlot> {
  const near = new Map<string, Set<string>>();
  for (const [a, b] of touching) {
    if (a === b) continue;
    (near.get(a) ?? near.set(a, new Set()).get(a)!).add(b);
    (near.get(b) ?? near.set(b, new Set()).get(b)!).add(a);
  }
  const order = [...answers].sort((x, y) => (x.key < y.key ? -1 : x.key > y.key ? 1 : 0));
  const out = new Map<string, MapSlot>();
  for (const a of order) {
    if (out.has(a.key)) continue;
    const used = new Map<MapSlot, number>();
    for (const n of near.get(a.key) ?? []) {
      const s = out.get(n);
      if (s) used.set(s, (used.get(s) ?? 0) + 1);
    }
    const pick = a.prefer && !used.has(a.prefer) ? a.prefer : (MAP_SLOTS.find((s) => !used.has(s)) ?? MAP_SLOTS.reduce((m, s) => ((used.get(s) ?? 0) < (used.get(m) ?? 0) ? s : m), 1 as MapSlot));
    out.set(a.key, pick);
  }
  return out;
}

/**
 * Where a holding is, when the place's own address is not enough (design 4.3), anchored to that address so it moves
 * with it: everything within a reach of it, a band between two distances, or an outline drawn in its own frame.
 */
export type HoldingExtent = { reach: Quantity } | { band: [Quantity, Quantity] } | { outline: [number, number][]; frame: OverlayFrame };

/**
 * One answer at one place, as the kernel's `holdings_in` and `who_holds` give it (OV3; the shape agreed with the
 * kernel thread). `via` says how the answer was reached: held there, inherited from a place it is inside, rolled up
 * from what is inside it, or by a fill rule. `why` is the words behind the "Why?".
 */
export interface Holding {
  place: string;
  layer: string;
  key: string;
  status: AreaStatus;
  strength?: number;
  extent?: HoldingExtent;
  /** Who asserts it (a claim); absent means it is simply so. */
  by?: string;
  /** From and until when, on the one universe clock (s); a holding with `until_s` has ended, read as history. */
  since_s?: number;
  until_s?: number;
  /** The relation record or rule id it comes from, so the Why line and the GM's edit can go to it. */
  source?: string;
  via: "held" | "inherited" | "rolled-up" | "rule";
  divided?: { key: string; share: number }[];
  why: string;
}

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
      if (!o || typeof o.id !== "string") continue;
      // A grid or drawing names a frame it is drawn in; a map layer of areas needs none.
      if (o.frame === undefined ? !o.areas : !OVERLAY_FRAMES.includes(o.frame)) continue;
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
// A grid always names its frame (overlaySpecsOf keeps none without one but a map layer); galaxy-plane-pc stands in for a hand-built spec.
export function gridOverlay(spec: OverlaySpec, g: OverlayGrid, anchor: OverlayAnchor | undefined, bounds: [number, number, number, number], px_per_unit: number): ViewOverlay {
  const base = { id: spec.id, label: spec.label, pack: spec.pack ?? "", frame: spec.frame ?? "galaxy-plane-pc" };
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
    // Map layers of areas are drawn from holdings (areaSpecsOf), not from fixed data in a frame.
    if (!s.frame || !frames.includes(s.frame)) continue;
    const frame = s.frame;
    const a = anchors?.[s.id];
    const g = overlayGrid(s, packs);
    if (g) {
      const o = gridOverlay(s, g, a, bounds, px_per_unit);
      // A grid's fixed layers ride on it, placed by the same anchor.
      if (a && s.layers) o.layers.push(...s.layers.map((l) => placeLayer(l, a)));
      out.push(o);
    } else if (s.layers) {
      out.push(a ? { id: s.id, label: s.label, pack: s.pack ?? "", frame, layers: s.layers.map((l) => placeLayer(l, a)) } : { id: s.id, label: s.label, pack: s.pack ?? "", frame, layers: [], why: `Place ${s.label} on the map first` });
    }
  }
  return out;
}

function placeLayer(l: OverlayLayer, a: OverlayAnchor): OverlayLayer {
  const P = (p: [number, number]) => placeBy(a, p);
  if (l.kind === "lines") return { ...l, items: l.items.map((i) => ({ ...i, points: i.points.map(P) })) };
  if (l.kind === "hexes") return { ...l, rotation_deg: (l.rotation_deg ?? 0) + a.rotation_deg, items: l.items.map((i) => ({ ...i, centre: P(i.centre) })) };
  if (l.kind === "areas") return { ...l, items: l.items.map((i) => ({ ...i, rings: i.rings.map((r) => r.map(P)), ...(i.label_turn !== undefined ? { label_turn: i.label_turn + a.rotation_deg } : {}) })) };
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
