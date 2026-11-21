// Map sifters (handoff 7A part 9) over the worlds on a hex map: mains
// (chains of worlds a jump apart), choke points (worlds whose loss splits
// a main), and pack-defined filters such as good refuelling stops. Pure and
// deterministic: worlds in, named groups with their reasons out.

import { distance, parse, type GridConfig, type Hex } from "./grid.ts";
import { holds, type MapWorld, type When } from "./glyphs.ts";

/** A pack-defined filter: worlds where any of `any` holds (or all of `all`). */
export interface MapSifter {
  id: string;
  label: string;
  any?: When[];
  all?: When[];
  /** Why a world is listed, in plain words. */
  explain: string;
}

export interface Main {
  /** Hex labels, in label order. */
  hexes: string[];
  jump: number;
}

interface Node {
  w: MapWorld;
  h: Hex;
}

function nodes(cfg: GridConfig, worlds: readonly MapWorld[]): Node[] {
  const out: Node[] = [];
  for (const w of worlds) {
    const h = parse(cfg, w.hex);
    if (h) out.push({ w, h });
  }
  return out.sort((a, b) => (a.w.hex < b.w.hex ? -1 : a.w.hex > b.w.hex ? 1 : 0));
}

function links(cfg: GridConfig, ns: Node[], jump: number): number[][] {
  return ns.map((a, i) => ns.flatMap((b, j) => (i !== j && distance(cfg, a.h, b.h) <= jump ? [j] : [])));
}

/** Chains of at least `min` worlds, each within `jump` hexes of another; largest first. */
export function mains(cfg: GridConfig, worlds: readonly MapWorld[], jump = 1, min = 3): Main[] {
  const ns = nodes(cfg, worlds);
  const adj = links(cfg, ns, jump);
  const seen = new Array<boolean>(ns.length).fill(false);
  const out: Main[] = [];
  for (let i = 0; i < ns.length; i++) {
    if (seen[i]) continue;
    const group: number[] = [];
    const stack = [i];
    seen[i] = true;
    while (stack.length) {
      const k = stack.pop()!;
      group.push(k);
      for (const j of adj[k]!) if (!seen[j]) (seen[j] = true), stack.push(j);
    }
    if (group.length >= min) out.push({ hexes: group.map((k) => ns[k]!.w.hex).sort(), jump });
  }
  return out.sort((a, b) => b.hexes.length - a.hexes.length || (a.hexes[0]! < b.hexes[0]! ? -1 : 1));
}

/** Worlds whose loss splits a chain (articulation points of the jump graph), in label order. */
export function chokePoints(cfg: GridConfig, worlds: readonly MapWorld[], jump = 1): string[] {
  const ns = nodes(cfg, worlds);
  const adj = links(cfg, ns, jump);
  const disc = new Array<number>(ns.length).fill(-1);
  const low = new Array<number>(ns.length).fill(0);
  const cut = new Set<number>();
  let t = 0;
  // Iterative depth-first search, so long chains cannot overflow the stack.
  for (let r = 0; r < ns.length; r++) {
    if (disc[r] !== -1) continue;
    const st: { v: number; parent: number; next: number; children: number }[] = [{ v: r, parent: -1, next: 0, children: 0 }];
    disc[r] = low[r] = t++;
    while (st.length) {
      const top = st[st.length - 1]!;
      const nb = adj[top.v]!;
      if (top.next < nb.length) {
        const u = nb[top.next++]!;
        if (disc[u] === -1) {
          disc[u] = low[u] = t++;
          top.children++;
          st.push({ v: u, parent: top.v, next: 0, children: 0 });
        } else if (u !== top.parent) low[top.v] = Math.min(low[top.v]!, disc[u]!);
      } else {
        st.pop();
        const p = st[st.length - 1];
        if (p) {
          low[p.v] = Math.min(low[p.v]!, low[top.v]!);
          if (p.parent !== -1 && low[top.v]! >= disc[p.v]!) cut.add(p.v);
        } else if (top.children > 1) cut.add(top.v);
      }
    }
  }
  return [...cut].map((k) => ns[k]!.w.hex).sort();
}

/** Worlds a pack sifter lists, in label order. */
export function sift(s: MapSifter, worlds: readonly MapWorld[]): string[] {
  return worlds
    .filter((w) => (s.all ? s.all.every((c) => holds(w, c)) : true) && (s.any ? s.any.some((c) => holds(w, c)) : true))
    .map((w) => w.hex)
    .sort();
}
