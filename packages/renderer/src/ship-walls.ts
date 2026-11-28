// Tank walls for the ship builder (CM14c part 3): each tank's wall mass from the kernel's thin-wall flow
// `sse.base.ship-tank-wall` (packs/base-physics/flows/ship-tank-wall.json, block sse.ship.tank_wall_mass,
// m = phi p V rho / sigma). The tank part carries its design pressure and design factors (cited figures in the parts
// pack); the wall's density and strength are the master data sheet's row for the part's wall material. Nothing here
// is typed in: a tank whose figures or material the sheet lacks gets no wall, and the build says it needs part masses.
import { readRow, type DeriveRef, type MatterReader } from "../../ui-contract/src/index.ts";
import type { PlacedPart } from "./ship-build.ts";

export const TANK_WALL_FLOW = "sse.base.ship-tank-wall";

/** Shape factor phi for a long cylinder (the block's upper end: a sphere is 1.5). The palette's tanks are cylinders
 * about twice as long as they are wide with domed ends, which the block does not size, so they are weighed as long
 * cylinders: heavier than their real shape, never lighter. Fudge register F-CM14c-wall-shape. */
export const TANK_SHAPE_FACTOR = 2;

/** One tank's wall, and where its mass came from in words. */
export interface TankWall {
  kg: number;
  said: string;
}

export interface WallReader extends MatterReader {
  derive(ref: DeriveRef): Promise<{ ok: true; values: Record<string, { value: number }> } | { ok: false; message: string }>;
}

interface Alloy {
  names?: { plain?: string };
  density?: { v: number };
  tensile_strength?: { v: number };
  yield_strength?: { v: number };
}

/** What a tank's wall depends on, as a key: the same key gives the same wall, so a cache by it never goes stale. */
export function wallKey(q: PlacedPart): string | null {
  const f = q.part.figures;
  if (q.part.kind !== "tank" || !q.part.wall_material || !f.volume_m3 || !f.design_pressure_pa || !f.ultimate_factor || !f.yield_factor) return null;
  return [q.part.wall_material, f.volume_m3.v, f.design_pressure_pa.v, f.ultimate_factor.v, f.yield_factor.v].join("|");
}

/** The allowed wall stress: the lower of strength over the ultimate factor and yield over the yield factor. */
export function allowedStress(a: Alloy, ultimate: number, yieldF: number): number | null {
  const u = a.tensile_strength?.v;
  const y = a.yield_strength?.v;
  const s = [u !== undefined ? u / ultimate : Infinity, y !== undefined ? y / yieldF : Infinity];
  const m = Math.min(...s);
  return Number.isFinite(m) && m > 0 ? m : null;
}

/**
 * Walls for the tanks not yet in `have` (by wallKey). Returns the new ones only; a tank the sheet or the flow cannot
 * size is left out, so its build still says it needs part masses.
 */
export async function readTankWalls(k: WallReader, placed: readonly PlacedPart[], have: Readonly<Record<string, TankWall>>): Promise<Record<string, TankWall>> {
  const out: Record<string, TankWall> = {};
  const alloys: Record<string, Alloy | null> = {};
  for (const q of placed) {
    const key = wallKey(q);
    if (!key || key in have || key in out) continue;
    const f = q.part.figures;
    const mat = q.part.wall_material!;
    if (!(mat in alloys)) alloys[mat] = (await readRow<Alloy>(k, "mixtures", mat))?.row ?? null;
    const a = alloys[mat];
    const sigma = a ? allowedStress(a, f.ultimate_factor!.v, f.yield_factor!.v) : null;
    if (!a?.density?.v || !sigma) continue;
    const d = await k.derive({
      flow: TANK_WALL_FLOW,
      inputs: {
        tank_pressure: { value: f.design_pressure_pa!.v, unit: "pa" },
        stowed_volume: { value: f.volume_m3!.v, unit: "m3" },
        wall_density: { value: a.density.v, unit: "kg_m3" },
        wall_stress: { value: sigma, unit: "pa" },
        vessel_shape: { value: TANK_SHAPE_FACTOR, unit: "one" },
      },
    });
    const kg = d.ok ? d.values["ship.wall_mass"]?.value : undefined;
    if (!(typeof kg === "number" && Number.isFinite(kg) && kg >= 0)) continue;
    const name = a.names?.plain ?? mat;
    out[key] = { kg, said: `wall from the thin-wall pressure block (m = φ p V ρ / σ): ${Math.round(f.design_pressure_pa!.v / 1e3)} kPa in ${name} at ${Math.round(sigma / 1e6)} MPa allowed, weighed as a long cylinder` };
  }
  return out;
}
