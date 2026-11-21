// Hex grid geometry for the hex map view (X3c). Pure functions, no DOM.
//
// The grid is configured by data (a pack's view config), so one view serves
// any game: block (subsector) size, how many blocks make a sheet (sector),
// how a hex is labelled, which columns sit half a hex lower, and what one
// hex stands for. Hexes are flat-topped and addressed by column and row,
// both counted from 1 as the labels count them.

export interface GridConfig {
  /** Columns and rows in one block (a subsector: 8 by 10). */
  block: { columns: number; rows: number };
  /** Blocks across and down one sheet (a sector: 4 by 4). */
  blocks: { across: number; down: number };
  /** Label form: "cc rr" digits, e.g. "{c2}{r2}" gives 0101; "{c}.{r}" gives 1.1. */
  label: string;
  /** Block names by position, across then down ("A".."P"). */
  block_names?: string[];
  /** Which 1-based columns are shifted half a hex down: "even" (Traveller) or "odd". */
  shifted: "even" | "odd";
  /** What one hex is across, for distances: value and unit (1 parsec). */
  hex_distance: { value: number; unit: string };
}

export interface Hex {
  col: number;
  row: number;
}

export const TRAVELLER_GRID: GridConfig = {
  block: { columns: 8, rows: 10 },
  blocks: { across: 4, down: 4 },
  label: "{c2}{r2}",
  block_names: "ABCDEFGHIJKLMNOP".split(""),
  shifted: "even",
  hex_distance: { value: 1, unit: "pc" },
};

function pad(n: number, w: number): string {
  const s = String(n);
  return s.length >= w ? s : "0".repeat(w - s.length) + s;
}

/** The label of a hex, by the config's label form. */
export function label(cfg: GridConfig, h: Hex): string {
  return cfg.label
    .replace(/\{c(\d)?\}/g, (_, w) => (w ? pad(h.col, Number(w)) : String(h.col)))
    .replace(/\{r(\d)?\}/g, (_, w) => (w ? pad(h.row, Number(w)) : String(h.row)));
}

/** Read a label back to a hex (null if it is not one this grid writes). */
export function parse(cfg: GridConfig, text: string): Hex | null {
  const parts: string[] = [];
  let re = "^";
  for (const m of cfg.label.matchAll(/\{([cr])(\d)?\}|([^{]+)/g)) {
    if (m[3] !== undefined) re += m[3].replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    else {
      parts.push(m[1] ?? "c");
      re += m[2] ? `(\\d{${m[2]}})` : "(\\d+)";
    }
  }
  const got = new RegExp(re + "$").exec(text.trim());
  if (!got) return null;
  const h: Hex = { col: 0, row: 0 };
  parts.forEach((p, i) => {
    if (p === "c") h.col = Number(got[i + 1]);
    else h.row = Number(got[i + 1]);
  });
  return h.col >= 1 && h.row >= 1 ? h : null;
}

/** Sheet size in hexes. */
export function extent(cfg: GridConfig): { columns: number; rows: number } {
  return { columns: cfg.block.columns * cfg.blocks.across, rows: cfg.block.rows * cfg.blocks.down };
}

/** The block a hex is in: index across-then-down, and its name. */
export function blockOf(cfg: GridConfig, h: Hex): { index: number; name: string } {
  const bx = Math.floor((h.col - 1) / cfg.block.columns);
  const by = Math.floor((h.row - 1) / cfg.block.rows);
  const index = by * cfg.blocks.across + bx;
  return { index, name: cfg.block_names?.[index] ?? String(index + 1) };
}

/** The hexes of one block, column by column. */
export function blockHexes(cfg: GridConfig, index: number): Hex[] {
  const bx = index % cfg.blocks.across;
  const by = Math.floor(index / cfg.blocks.across);
  const out: Hex[] = [];
  for (let c = 1; c <= cfg.block.columns; c++)
    for (let r = 1; r <= cfg.block.rows; r++)
      out.push({ col: bx * cfg.block.columns + c, row: by * cfg.block.rows + r });
  return out;
}

function isShifted(cfg: GridConfig, col: number): boolean {
  return cfg.shifted === "even" ? col % 2 === 0 : col % 2 === 1;
}

/** Cube coordinates (x + y + z = 0) of a hex. With columns counted from 0,
 * "even" shifting (1-based even columns lower) is the odd-q layout and "odd"
 * shifting the even-q layout (Patel, "Hexagonal Grids", redblobgames.com). */
export function cube(cfg: GridConfig, h: Hex): [number, number, number] {
  const q = h.col - 1;
  const r = h.row - 1;
  const odd = q & 1;
  const z = cfg.shifted === "even" ? r - (q - odd) / 2 : r - (q + odd) / 2;
  return [q, -q - z, z];
}

/** Hex distance (steps between centres). */
export function distance(cfg: GridConfig, a: Hex, b: Hex): number {
  const [ax, ay, az] = cube(cfg, a);
  const [bx, by, bz] = cube(cfg, b);
  return Math.max(Math.abs(ax - bx), Math.abs(ay - by), Math.abs(az - bz));
}

/** Every hex on the sheet within `n` steps of `h`, nearest first, then by label. */
export function within(cfg: GridConfig, h: Hex, n: number): Hex[] {
  const { columns, rows } = extent(cfg);
  const out: { h: Hex; d: number }[] = [];
  for (let c = Math.max(1, h.col - n); c <= Math.min(columns, h.col + n); c++)
    for (let r = Math.max(1, h.row - n - 1); r <= Math.min(rows, h.row + n + 1); r++) {
      const d = distance(cfg, h, { col: c, row: r });
      if (d <= n) out.push({ h: { col: c, row: r }, d });
    }
  return out
    .sort((p, q) => p.d - q.d || p.h.col - q.h.col || p.h.row - q.h.row)
    .map((x) => x.h);
}

/** Centre of a hex in drawing units for hexes of circumradius `size` (flat-topped). */
export function centre(cfg: GridConfig, h: Hex, size: number): { x: number; y: number } {
  const w = 1.5 * size;
  const hgt = Math.sqrt(3) * size;
  return { x: size + (h.col - 1) * w, y: hgt / 2 + (h.row - 1) * hgt + (isShifted(cfg, h.col) ? hgt / 2 : 0) };
}

/** The six corners of a flat-topped hex, as an SVG points string. */
export function corners(c: { x: number; y: number }, size: number): string {
  const pts: string[] = [];
  for (let i = 0; i < 6; i++) {
    const a = (Math.PI / 3) * i;
    pts.push(`${(c.x + size * Math.cos(a)).toFixed(2)},${(c.y + size * Math.sin(a)).toFixed(2)}`);
  }
  return pts.join(" ");
}
