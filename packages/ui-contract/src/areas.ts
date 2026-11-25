// Territory areas, the drawing side (territory overlays OV4): the kernel's holdings (OV3) turned into rings a view
// draws in its own frame, and what the Atlas look needs to letter and key them. The kernel sends no geometry: a
// holding names a place, and the view knows where places are. Between stars the rule is the kernel's (OV3 "reach"):
// each held place colours space out to its reach, the nearest held place wins where reaches meet (so borders run
// midway), ties go by address, and space beyond every reach stays bare. A holding drawn as an outline travels as
// stored. Presentation only: nothing here decides who holds anything.
import { AREA_REACH_DEFAULT, type AreaItem, type AreaStatus, type Holding, type OverlayFrame, type OverlayInk, type Quantity } from "./overlay.ts";

/** A length in parsecs (the galaxy-plane and cluster frames' unit); null when the unit is not a length. */
export function parsecs(q: Quantity): number | null {
  const per: Record<string, number> = { pc: 1, kpc: 1000, Mpc: 1e6, ly: 1 / 3.26156, au: 1 / 206264.806, AU: 1 / 206264.806 };
  const f = per[q.unit];
  return f === undefined ? null : q.value * f;
}

export interface AreasAsk {
  /** The view's frame and the part of it in view: [xmin, ymin, xmax, ymax] in frame units. */
  frame: OverlayFrame;
  bounds: [number, number, number, number];
  /** Where a place is in the frame (null: not drawn here). */
  positionOf: (place: string) => [number, number] | null;
  /** The layer's reach between stars, in frame units (from `fill.between_stars.reach`); the ruling's 4 pc otherwise. */
  reach?: number;
  /** Each answer's ink (its map slot), by key. */
  inkOf: (key: string) => OverlayInk;
  /** Each answer's name, for lettering. */
  nameOf?: (key: string) => string | undefined;
  /** True for a holding the table does not know (drawn in pencil on the GM's map, SV3). */
  pencil?: (h: Holding) => boolean;
  /** Sample cells across the longer side of the bounds (the border's fineness): 160 by default. */
  cells?: number;
}

interface Seed {
  place: string;
  key: string;
  at: [number, number];
  reach: number;
  strength?: number;
  pencil: boolean;
}

/** Who a point is coloured by under the reach rule: the nearest seed within its reach, ties by address. */
export function nearestWithin(seeds: readonly { place: string; key: string; at: readonly [number, number]; reach: number }[], p: readonly [number, number]): string | null {
  let best: { key: string; d: number; place: string } | null = null;
  for (const s of seeds) {
    const d = Math.hypot(p[0] - s.at[0], p[1] - s.at[1]);
    if (d > s.reach) continue;
    if (!best || d < best.d || (d === best.d && s.place < best.place)) best = { key: s.key, d, place: s.place };
  }
  return best?.key ?? null;
}

/** Whether a point is inside rings by the even-odd rule. */
export function insideRings(rings: readonly (readonly (readonly [number, number])[])[], p: readonly [number, number]): boolean {
  let inside = false;
  for (const r of rings)
    for (let i = 0, j = r.length - 1; i < r.length; j = i++) {
      const [xi, yi] = r[i]!;
      const [xj, yj] = r[j]!;
      if (yi > p[1] !== yj > p[1] && p[0] < ((xj - xi) * (p[1] - yi)) / (yj - yi) + xi) inside = !inside;
    }
  return inside;
}

/**
 * The outline of where a field is below zero, as closed rings (marching squares on the cell corners). The border is
 * put where the field crosses zero along each cell edge (linear between the corners), so a reach's circle comes out
 * round and a border between two stars straight; a yes/no test gives the midpoints instead. Rings are even-odd:
 * holes and separate pieces come out as further rings.
 */
export function traceRings(nx: number, ny: number, field: ((i: number, j: number) => number) | ((i: number, j: number) => boolean), at: (i: number, j: number) => [number, number]): [number, number][][] {
  const f = (i: number, j: number): number => {
    if (i < 0 || j < 0 || i > nx || j > ny) return 1;
    const v = field(i, j);
    return typeof v === "boolean" ? (v ? -1 : 1) : v;
  };
  const cache = new Map<string, number>();
  const val = (i: number, j: number) => {
    const k = `${i},${j}`;
    let v = cache.get(k);
    if (v === undefined) cache.set(k, (v = f(i, j)));
    return v;
  };
  const v = (i: number, j: number) => val(i, j) < 0;
  // Edge points are named by the doubled lattice so equal points match exactly when chaining.
  type P = [number, number];
  const segs: [P, P][] = [];
  for (let j = -1; j <= ny; j++)
    for (let i = -1; i <= nx; i++) {
      const a = v(i, j);
      const b = v(i + 1, j);
      const c = v(i + 1, j + 1);
      const d = v(i, j + 1);
      const code = (a ? 1 : 0) | (b ? 2 : 0) | (c ? 4 : 0) | (d ? 8 : 0);
      if (code === 0 || code === 15) continue;
      const top: P = [2 * i + 1, 2 * j];
      const right: P = [2 * i + 2, 2 * j + 1];
      const bottom: P = [2 * i + 1, 2 * j + 2];
      const left: P = [2 * i, 2 * j + 1];
      const T: Record<number, [P, P][]> = {
        1: [[left, top]],
        2: [[top, right]],
        3: [[left, right]],
        4: [[right, bottom]],
        5: [[left, top], [right, bottom]],
        6: [[top, bottom]],
        7: [[left, bottom]],
        8: [[bottom, left]],
        9: [[bottom, top]],
        10: [[top, right], [bottom, left]],
        11: [[bottom, right]],
        12: [[right, left]],
        13: [[right, top]],
        14: [[top, left]],
      };
      for (const s of T[code]!) segs.push(s);
    }
  // Where along its edge the field crosses zero.
  const place = ([X, Y]: P): [number, number] => {
    const [i0, j0, i1, j1] = X % 2 === 1 ? [(X - 1) / 2, Y / 2, (X + 1) / 2, Y / 2] : [X / 2, (Y - 1) / 2, X / 2, (Y + 1) / 2];
    const a = val(i0, j0);
    const b = val(i1, j1);
    const t = Number.isFinite(a) && Number.isFinite(b) && a !== b ? Math.min(1, Math.max(0, a / (a - b))) : 0.5;
    return at(i0 + (i1 - i0) * t, j0 + (j1 - j0) * t);
  };
  const key = (p: P) => `${p[0]},${p[1]}`;
  const from = new Map<string, number[]>();
  segs.forEach((s, n) => from.set(key(s[0]), [...(from.get(key(s[0])) ?? []), n]));
  const used = new Set<number>();
  const rings: [number, number][][] = [];
  for (let n = 0; n < segs.length; n++) {
    if (used.has(n)) continue;
    const ring: P[] = [];
    let cur = n;
    while (!used.has(cur)) {
      used.add(cur);
      ring.push(segs[cur]![0]);
      const next = (from.get(key(segs[cur]![1])) ?? []).find((m) => !used.has(m));
      if (next === undefined) break;
      cur = next;
    }
    const pts = ring.map(place);
    // Drop points on a straight run (a border between two stars is one line, not a point per cell).
    const kept = pts.filter((p, k) => {
      const a = pts[(k + pts.length - 1) % pts.length]!;
      const b = pts[(k + 1) % pts.length]!;
      const cross = (p[0] - a[0]) * (b[1] - p[1]) - (p[1] - a[1]) * (b[0] - p[0]);
      const len = Math.hypot(b[0] - a[0], b[1] - a[1]);
      return Math.abs(cross) > 1e-6 * len * len;
    });
    if (kept.length >= 3) rings.push(kept);
  }
  return rings;
}

/** The holds field for one answer: below zero where it wins by the reach rule (nearest within reach, borders midway). */
function holdsField(seeds: readonly Seed[], key: string, p: readonly [number, number]): number {
  // Out of reach: how far past the nearest reach of its own; in reach: how much nearer it is than any rival in reach.
  let past = Infinity;
  let mine = Infinity;
  let other = Infinity;
  for (const s of seeds) {
    const d = Math.hypot(p[0] - s.at[0], p[1] - s.at[1]);
    if (s.key === key) {
      past = Math.min(past, d - s.reach);
      if (d <= s.reach) mine = Math.min(mine, d);
    } else if (d <= s.reach) other = Math.min(other, d);
  }
  if (!Number.isFinite(past)) return 1;
  if (!Number.isFinite(mine)) return past;
  return Number.isFinite(other) ? Math.max(past, mine - other) : past;
}

/** The field for claims or influence: below zero within any of the answer's reaches. */
function reachField(seeds: readonly Seed[], p: readonly [number, number]): number {
  let m = Infinity;
  for (const s of seeds) m = Math.min(m, Math.hypot(p[0] - s.at[0], p[1] - s.at[1]) - s.reach);
  return Number.isFinite(m) ? m : 1;
}

/**
 * Holdings (the kernel's `holdings_in`) to areas in the view's frame. Holds compete by the reach rule; each
 * claimant's claims and each influence are drawn on their own (they may overlap a holder: disputed ground). An outline
 * in this frame is drawn as stored. Places the view cannot place, and answers that end up empty, are left out.
 */
export function areasFromHoldings(holdings: readonly Holding[], ask: AreasAsk): AreaItem[] {
  const reach = ask.reach ?? parsecs(AREA_REACH_DEFAULT)!;
  // Traced a reach beyond the view on every side, so a territory running off the view has no border drawn along the
  // view's edge (the view clips it).
  const far = Math.max(reach, ...holdings.map((h) => (h.extent && "reach" in h.extent ? (parsecs(h.extent.reach) ?? 0) : 0)));
  const [x0, y0, x1, y1] = [ask.bounds[0] - far, ask.bounds[1] - far, ask.bounds[2] + far, ask.bounds[3] + far];
  const n = Math.max(8, ask.cells ?? 160);
  const step = Math.max(x1 - x0, y1 - y0) / n;
  const nx = Math.max(1, Math.ceil((x1 - x0) / step));
  const ny = Math.max(1, Math.ceil((y1 - y0) / step));
  const at = (i: number, j: number): [number, number] => [x0 + i * step, y0 + j * step];
  const seedsOf = (status: AreaStatus) =>
    holdings
      .filter((h) => h.status === status && h.until_s === undefined && (!h.extent || "reach" in h.extent))
      .flatMap((h): Seed[] => {
        const p = ask.positionOf(h.place);
        if (!p) return [];
        const r = h.extent && "reach" in h.extent ? (parsecs(h.extent.reach) ?? reach) : reach;
        return [{ place: h.place, key: h.key, at: p, reach: r, ...(h.strength !== undefined ? { strength: h.strength } : {}), pencil: !!ask.pencil?.(h) }];
      });
  const out: AreaItem[] = [];
  const item = (key: string, status: AreaStatus, rings: [number, number][][], seeds: Seed[]) => {
    if (!rings.length) return;
    const name = ask.nameOf?.(key);
    const strength = seeds.length && seeds.every((s) => s.strength !== undefined) ? seeds.reduce((m, s) => m + s.strength!, 0) / seeds.length : undefined;
    out.push({ key, ink: ask.inkOf(key), status, rings, ...(name ? { label: name } : {}), ...(strength !== undefined ? { strength } : {}), ...(seeds.length && seeds.every((s) => s.pencil) ? { pencil: true } : {}), ...(seeds.length === 1 ? { place: seeds[0]!.place } : {}) });
  };
  // Holds: one field, nearest wins.
  const holds = seedsOf("holds");
  for (const key of [...new Set(holds.map((s) => s.key))].sort()) item(key, "holds", traceRings(nx, ny, (i, j) => holdsField(holds, key, at(i, j)), at), holds.filter((s) => s.key === key));
  // Claims and influences: each answer on its own, out to its reaches.
  for (const status of ["claims", "influences"] as const) {
    const seeds = seedsOf(status);
    for (const key of [...new Set(seeds.map((s) => s.key))].sort()) {
      const mine = seeds.filter((s) => s.key === key);
      item(key, status, traceRings(nx, ny, (i, j) => reachField(mine, at(i, j)), at), mine);
    }
  }
  // Outlines drawn in this frame, as stored.
  for (const h of holdings)
    if (h.until_s === undefined && h.extent && "outline" in h.extent && h.extent.frame === ask.frame && h.extent.outline.length >= 3) {
      const name = ask.nameOf?.(h.key);
      out.push({ key: h.key, ink: ask.inkOf(h.key), status: h.status, rings: [h.extent.outline.map((p) => [p[0], p[1]] as [number, number])], place: h.place, ...(name ? { label: name } : {}), ...(ask.pencil?.(h) ? { pencil: true } : {}), ...(h.divided ? { divided: h.divided } : {}) });
    }
  return out;
}

/** The signed area of one ring (positive anticlockwise in a y-up frame). */
export function ringArea(r: readonly (readonly [number, number])[]): number {
  let a = 0;
  for (let i = 0, j = r.length - 1; i < r.length; j = i++) a += (r[j]![0] + r[i]![0]) * (r[j]![1] - r[i]![1]);
  return a / 2;
}

/** The area an item covers (even-odd: holes inside pieces subtract). */
export function itemArea(rings: readonly (readonly (readonly [number, number])[])[]): number {
  // Each ring's area counts with the sign of how many other rings it sits inside (even: a piece, odd: a hole).
  return rings.reduce((sum, r, k) => {
    const inside = rings.filter((o, m) => m !== k && r.length > 0 && insideRings([o], r[0]!)).length;
    return sum + (inside % 2 === 0 ? 1 : -1) * Math.abs(ringArea(r));
  }, 0);
}

/**
 * Where an area's name is lettered (design 6): along the long axis of its largest piece, at the piece's centre, with
 * the room along that axis; the view drops the name when the room is too short for it. Angle in degrees from +x, kept
 * between -90 and 90 so the letters never read upside down.
 */
export function letterArea(rings: readonly (readonly (readonly [number, number])[])[]): { at: [number, number]; turn: number; length: number; width: number } | null {
  const big = [...rings].sort((a, b) => Math.abs(ringArea(b)) - Math.abs(ringArea(a)))[0];
  if (!big || big.length < 3) return null;
  const n = big.length;
  const cx = big.reduce((s, p) => s + p[0], 0) / n;
  const cy = big.reduce((s, p) => s + p[1], 0) / n;
  let sxx = 0;
  let syy = 0;
  let sxy = 0;
  for (const [x, y] of big) {
    sxx += (x - cx) ** 2;
    syy += (y - cy) ** 2;
    sxy += (x - cx) * (y - cy);
  }
  const theta = 0.5 * Math.atan2(2 * sxy, sxx - syy);
  let turn = (theta * 180) / Math.PI;
  if (turn > 90) turn -= 180;
  if (turn < -90) turn += 180;
  const ux = Math.cos(theta);
  const uy = Math.sin(theta);
  const along = big.map(([x, y]) => (x - cx) * ux + (y - cy) * uy);
  const across = big.map(([x, y]) => -(x - cx) * uy + (y - cy) * ux);
  return { at: [cx, cy], turn, length: Math.max(...along) - Math.min(...along), width: Math.max(...across) - Math.min(...across) };
}

/** Each answer's share of what is drawn (for the key), largest first. */
export function areaShares(items: readonly AreaItem[]): { key: string; share: number }[] {
  const by = new Map<string, number>();
  for (const it of items) if (it.status === "holds") by.set(it.key, (by.get(it.key) ?? 0) + Math.abs(itemArea(it.rings)));
  const total = [...by.values()].reduce((a, b) => a + b, 0);
  return [...by]
    .map(([key, a]) => ({ key, share: total > 0 ? a / total : 0 }))
    .sort((a, b) => b.share - a.share || (a.key < b.key ? -1 : 1));
}
