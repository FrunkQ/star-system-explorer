// Screens on this computer (PB9, playbook and switchboard design 3.2): a display attached to the GM's computer is a
// screen, or, split, several. "On this computer" lists the displays and opens a full-screen window on one; that window
// is a players' screen like any other, marked local so the board says "this computer". Split (Whole, Two, Three, Four)
// is kept out of the way: a split window frames one players' screen per part, each with its own id, its own box on
// the board, its own role, line, dial and hold. Nothing here touches the wire: a part is an ordinary join link.

/** How a display is divided: Two is side by side; Three is one large and two small; Four is two by two. */
export const SPLITS = {
  whole: { words: "Whole", parts: 1, columns: "1fr", rows: "1fr", areas: ['"p1"'] },
  two: { words: "Two", parts: 2, columns: "1fr 1fr", rows: "1fr", areas: ['"p1 p2"'] },
  three: { words: "Three", parts: 3, columns: "2fr 1fr", rows: "1fr 1fr", areas: ['"p1 p2"', '"p1 p3"'] },
  four: { words: "Four", parts: 4, columns: "1fr 1fr", rows: "1fr 1fr", areas: ['"p1 p2"', '"p3 p4"'] },
} as const;
export type Split = keyof typeof SPLITS;

export const isSplit = (s: unknown): s is Split => typeof s === "string" && Object.hasOwn(SPLITS, s);

/**
 * The link a window on a display opens: the role's join link, marked local, named for the display, and split if asked.
 * A whole display is one screen; a split one is a frame of parts (`partLinks`).
 */
export function displayLink(join: string, name: string, split: Split = "whole"): string {
  const u = new URL(join);
  u.searchParams.set("local", "1");
  u.searchParams.set("name", name.slice(0, 40));
  if (split === "whole") u.searchParams.delete("split");
  else u.searchParams.set("split", split);
  u.searchParams.delete("part");
  return u.href;
}

/** A split window's parts: each the same link, unsplit, with its part number and a name of its own ("Wall TV 2"). */
export function partLinks(link: string): string[] {
  const u = new URL(link);
  const split = u.searchParams.get("split");
  if (!isSplit(split) || split === "whole") return [link];
  const name = u.searchParams.get("name") ?? "Screen";
  u.searchParams.delete("split");
  return Array.from({ length: SPLITS[split].parts }, (_, i) => {
    const p = new URL(u.href);
    p.searchParams.set("part", String(i + 1));
    p.searchParams.set("name", `${name} ${i + 1}`.slice(0, 40));
    return p.href;
  });
}

/** A display as the board draws it: its label, its size, and where it sits among the others (for the small map). */
export interface Display {
  id: string;
  label: string;
  left: number;
  top: number;
  width: number;
  height: number;
  primary: boolean;
}

/** The displays, from the browser's screen details (or this screen alone), with plain labels and stable ids. */
export function displaysFrom(screens: readonly { label?: string; left?: number; top?: number; width: number; height: number; isPrimary?: boolean }[]): Display[] {
  const used = new Set<string>();
  return screens.map((s, i) => {
    const base = (s.label ?? "").trim() || (s.isPrimary ? "This screen" : `Display ${i + 1}`);
    let label = base;
    for (let n = 2; used.has(label); n++) label = `${base} ${n}`;
    used.add(label);
    return { id: `d${i + 1}`, label, left: s.left ?? 0, top: s.top ?? 0, width: s.width, height: s.height, primary: !!s.isPrimary };
  });
}

/** The small map: each display as a box within a 100-wide frame, in its real position. */
export function displayMap(ds: readonly Display[]): { id: string; x: number; y: number; w: number; h: number }[] {
  if (!ds.length) return [];
  const x0 = Math.min(...ds.map((d) => d.left));
  const y0 = Math.min(...ds.map((d) => d.top));
  const x1 = Math.max(...ds.map((d) => d.left + d.width));
  const y1 = Math.max(...ds.map((d) => d.top + d.height));
  const k = 100 / Math.max(1, x1 - x0, ((y1 - y0) * 100) / 60);
  return ds.map((d) => ({ id: d.id, x: (d.left - x0) * k, y: (d.top - y0) * k, w: d.width * k, h: d.height * k }));
}

/** The key a screen keeps its id under on this device: one per room, and one per part of a split window. */
export function screenKey(room: string | null, part: string | null): string {
  const p = part && /^[1-4]$/.test(part) ? `.${part}` : "";
  return `sse4.screen.${room ?? "alone"}${p}`;
}
