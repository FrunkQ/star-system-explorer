// The Flow view (RK13, Atomic Rockets plan section 16 and question 18): one
// picture for any ledger that can hand over rows of (from, to, amount, kind).
// Nodes stand in columns, read left to right (top to bottom on a phone); a
// band's width is its amount. A ship's power is the first user; a world's
// energy budget, an element's journey and a habitat's air can follow with no
// new drawing code. This only lays out what it is given: it never sums physics.

export interface FlowRow {
  from: string;
  to: string;
  amount: number;
  /** What flows, for the band's colour and its words: "electric", "heat", "jet", "fuel", ... */
  kind: string;
  /** A band the books cannot account for (drawn dashed, from "Nowhere"). */
  dashed?: boolean;
  /** For heat: the temperature it leaves at, K. */
  k?: number;
}

export interface FlowNode {
  id: string;
  column: number;
  /** The larger of what comes in and what goes out. */
  amount: number;
  x: number;
  y: number;
  w: number;
  h: number;
  dashed: boolean;
}

export interface FlowBand {
  row: FlowRow;
  from: FlowNode;
  to: FlowNode;
  /** Drawn width in px. */
  width: number;
  /** Under one part in a hundred of the largest column: drawn as a hairline. */
  hairline: boolean;
  /** SVG path along the band's middle. */
  d: string;
}

export interface FlowLayout {
  width: number;
  height: number;
  vertical: boolean;
  nodes: FlowNode[];
  bands: FlowBand[];
  /** Where each column sits along the reading direction, px. */
  columns: number[];
}

export interface FlowOptions {
  width: number;
  height: number;
  /** Top to bottom (phones) instead of left to right. */
  vertical?: boolean;
  /** A node's column; nodes it does not place go by longest path from the sources. */
  column?: (id: string) => number | undefined;
  /** Order inside a column (smaller first); by default, first seen first. */
  rank?: (id: string) => number;
  /** Room kept before the first column and after the last, for labels, px. */
  margin?: [number, number];
  nodeSize?: number;
  gap?: number;
}

/** Below this share of the largest column a band is a hairline (plan 16: "honest about scale"). */
export const HAIRLINE_SHARE = 0.01;
const HAIRLINE_PX = 1;

/** Each node's column: given, or one past the furthest node that feeds it. */
export function flowRowColumns(rows: readonly FlowRow[], given?: (id: string) => number | undefined): Map<string, number> {
  const ids: string[] = [];
  for (const r of rows) for (const id of [r.from, r.to]) if (!ids.includes(id)) ids.push(id);
  const col = new Map<string, number>();
  for (const id of ids) {
    const c = given?.(id);
    if (c !== undefined) col.set(id, c);
  }
  // Longest path from the sources for the rest; a cycle stops after as many passes as there are nodes.
  for (let pass = 0; pass < ids.length; pass++) {
    let moved = false;
    for (const id of ids) {
      if (given?.(id) !== undefined) continue;
      const feeds = rows.filter((r) => r.to === id && r.from !== id).map((r) => col.get(r.from) ?? 0);
      const c = feeds.length ? Math.max(...feeds) + 1 : 0;
      if (col.get(id) !== c) ((moved = true), col.set(id, c));
    }
    if (!moved) break;
  }
  return col;
}

/** Lay the rows out: nodes in columns, bands stacked on each node in column order. */
export function layoutFlow(rows: readonly FlowRow[], o: FlowOptions): FlowLayout {
  const vertical = o.vertical ?? false;
  const live = rows.filter((r) => r.amount > 0 && Number.isFinite(r.amount));
  const col = flowRowColumns(live, o.column);
  const seen: string[] = [];
  for (const r of live) for (const id of [r.from, r.to]) if (!seen.includes(id)) seen.push(id);
  const rank = (id: string) => o.rank?.(id) ?? seen.indexOf(id);
  const nCols = Math.max(0, ...[...col.values()].map((c) => c + 1));
  const inOf = (id: string) => live.filter((r) => r.to === id).reduce((a, r) => a + r.amount, 0);
  const outOf = (id: string) => live.filter((r) => r.from === id).reduce((a, r) => a + r.amount, 0);
  const nodeSize = o.nodeSize ?? 10;
  const gap = o.gap ?? 22;
  const [m0, m1] = o.margin ?? (vertical ? [24, 24] : [150, 170]);
  const along = vertical ? o.height : o.width;
  const across = vertical ? o.width : o.height;
  const columns = Array.from({ length: nCols }, (_, c) => (nCols <= 1 ? m0 : m0 + ((along - m0 - m1 - nodeSize) * c) / (nCols - 1)));

  const byCol: string[][] = Array.from({ length: nCols }, () => []);
  for (const id of seen) byCol[col.get(id) ?? 0]!.push(id);
  for (const ids of byCol) ids.sort((a, b) => rank(a) - rank(b) || seen.indexOf(a) - seen.indexOf(b));
  const amount = (id: string) => Math.max(inOf(id), outOf(id));
  const colTotal = byCol.map((ids) => ids.reduce((a, id) => a + amount(id), 0));
  const biggest = Math.max(0, ...colTotal);
  const most = Math.max(1, ...byCol.map((ids) => ids.length));
  const top = vertical ? 8 : 24;
  const room = Math.max(10, across - top - gap * (most - 1) - 8);
  const scale = biggest > 0 ? room / biggest : 0;
  const thick = (v: number) => (v < biggest * HAIRLINE_SHARE ? HAIRLINE_PX : Math.max(v * scale, HAIRLINE_PX));

  const nodes = new Map<string, FlowNode>();
  byCol.forEach((ids, c) => {
    const sizes = ids.map((id) => thick(amount(id)));
    const used = sizes.reduce((a, s) => a + s, 0) + gap * (ids.length - 1);
    let at = top + Math.max(0, (room + gap * (most - 1) - used) / 2);
    ids.forEach((id, i) => {
      const s = sizes[i]!;
      const dashed = live.some((r) => r.from === id && r.dashed) && !live.some((r) => r.to === id);
      nodes.set(id, vertical ? { id, column: c, amount: amount(id), x: at, y: columns[c]!, w: s, h: nodeSize, dashed } : { id, column: c, amount: amount(id), x: columns[c]!, y: at, w: nodeSize, h: s, dashed });
      at += s + gap;
    });
  });

  // Stack each band on its two nodes: outgoing in the order of where they go, incoming in the order of where they come from.
  const place = live.map((row) => ({ row, a: nodes.get(row.from)!, b: nodes.get(row.to)!, t: thick(row.amount), s0: 0, s1: 0 }));
  const key = (n: FlowNode) => n.column * 1e6 + rank(n.id);
  const outAt = new Map<string, number>();
  const inAt = new Map<string, number>();
  for (const p of [...place].sort((p, q) => key(p.b) - key(q.b))) {
    const u = outAt.get(p.a.id) ?? 0;
    p.s0 = u + p.t / 2;
    outAt.set(p.a.id, u + p.t);
  }
  for (const p of [...place].sort((p, q) => key(p.a) - key(q.a))) {
    const u = inAt.get(p.b.id) ?? 0;
    p.s1 = u + p.t / 2;
    inAt.set(p.b.id, u + p.t);
  }
  const bands = place.map(({ row, a, b, t, s0, s1 }) => {
    let d: string;
    if (vertical) {
      const y0 = a.y + a.h;
      const y1 = b.y;
      const x0 = a.x + s0;
      const x1 = b.x + s1;
      const my = (y0 + y1) / 2;
      d = `M${r1(x0)},${r1(y0)} C${r1(x0)},${r1(my)} ${r1(x1)},${r1(my)} ${r1(x1)},${r1(y1)}`;
    } else {
      const x0 = a.x + a.w;
      const x1 = b.x;
      const y0 = a.y + s0;
      const y1 = b.y + s1;
      const mx = (x0 + x1) / 2;
      d = `M${r1(x0)},${r1(y0)} C${r1(mx)},${r1(y0)} ${r1(mx)},${r1(y1)} ${r1(x1)},${r1(y1)}`;
    }
    return { row, from: a, to: b, width: t, hairline: row.amount < biggest * HAIRLINE_SHARE, d };
  });
  return { width: o.width, height: o.height, vertical, nodes: [...nodes.values()], bands, columns };
}

const r1 = (v: number) => Math.round(v * 10) / 10;
