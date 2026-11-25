// "Find a home" (V9, U6 part 2b): search the universe's halo cells for a galaxy whose history fits what the GM
// wants, nearest cells first. Pure: reads the kernel's `halo_cell` answers (each halo's mass, mergers and nearest
// big neighbour), never invents a galaxy. The wishes are plain data the GM can change; "Like the Milky Way" is a
// cited starting point, not a rule.
import { sci, type CellHalo, type HaloCell } from "./galaxy.ts";

export interface HomeWish {
  /** Dark-matter halo mass, solar masses. */
  mass_min: number;
  mass_max: number;
  /** No major merger for at least this long before now (Gyr); 0 asks nothing. */
  quiet_gyr: number;
  /** A big neighbour bound to it ("bound"), any or none at all ("alone"), or no wish ("any"). */
  neighbour: "any" | "bound" | "alone";
  /** The neighbour no further than this (Mpc), when one is wished for. */
  neighbour_mpc: number;
}

/**
 * Like the Milky Way: a halo of about 0.5 to 2 x 10^12 solar masses (the spread of measurements reviewed by
 * Wang, Han, Cooper et al. 2020, Sci. China Phys. Mech. Astron. 63, 109801), no major merger for 8 Gyr (the last,
 * Gaia-Enceladus, about 10 Gyr ago: Helmi et al. 2018, Nature 563, 85), and a bound big neighbour within 1 Mpc
 * (Andromeda, 0.78 Mpc away and falling in).
 */
export const LIKE_THE_MILKY_WAY: HomeWish = { mass_min: 5e11, mass_max: 2e12, quiet_gyr: 8, neighbour: "bound", neighbour_mpc: 1 };

export interface HomeFit {
  halo: CellHalo;
  cell: [number, number, number];
  /** How far its mass sits from the middle of the wished range, in decades (0 is right in the middle). */
  off: number;
  /** Why it fits, in words, one line per wish. */
  why: string[];
}

const log10 = (x: number) => Math.log10(Math.max(x, 1e-300));

/** Whether one halo fits, and in what words; null when it does not. */
export function homeFits(h: CellHalo, wish: HomeWish, now_gyr: number): string[] | null {
  if (!(h.mass_msun >= wish.mass_min && h.mass_msun <= wish.mass_max)) return null;
  const why = [`a halo of ${sci(h.mass_msun)} suns`];
  if (wish.quiet_gyr > 0) {
    const quiet = h.last_major_gyr === null ? Infinity : now_gyr - h.last_major_gyr;
    if (quiet < wish.quiet_gyr) return null;
    why.push(h.last_major_gyr === null ? "no major merger on record" : `quiet for ${Math.round(quiet * 10) / 10} Gyr since its last major merger`);
  }
  const n = h.neighbour;
  if (wish.neighbour === "bound") {
    if (!n || !n.bound || n.distance_mpc > wish.neighbour_mpc) return null;
    why.push(`a bound neighbour ${Math.round(n.distance_mpc * 100) / 100} Mpc away`);
  } else if (wish.neighbour === "alone") {
    if (n && n.distance_mpc <= wish.neighbour_mpc) return null;
    why.push(`no big neighbour within ${wish.neighbour_mpc} Mpc`);
  }
  return why;
}

/** Every fitting halo in the cells read so far, the closest to the middle of the mass range first. */
export function findHomes(cells: readonly HaloCell[], wish: HomeWish): HomeFit[] {
  const mid = (log10(wish.mass_min) + log10(wish.mass_max)) / 2;
  const out: HomeFit[] = [];
  for (const c of cells)
    for (const h of c.haloes) {
      const why = homeFits(h, wish, c.now_gyr);
      if (why) out.push({ halo: h, cell: c.index, off: Math.abs(log10(h.mass_msun) - mid), why });
    }
  return out.sort((a, b) => a.off - b.off || (a.halo.address < b.halo.address ? -1 : 1));
}

/** The cells around one, nearest first: the cell itself, then each shell of neighbours out to `radius`. */
export function cellsAround(c: readonly [number, number, number], radius: number): [number, number, number][] {
  const out: { at: [number, number, number]; d: number; r2: number }[] = [];
  for (let dx = -radius; dx <= radius; dx++)
    for (let dy = -radius; dy <= radius; dy++)
      for (let dz = -radius; dz <= radius; dz++) out.push({ at: [c[0] + dx, c[1] + dy, c[2] + dz], d: Math.max(Math.abs(dx), Math.abs(dy), Math.abs(dz)), r2: dx * dx + dy * dy + dz * dz });
  return out.sort((a, b) => a.d - b.d || a.r2 - b.r2).map((x) => x.at);
}

