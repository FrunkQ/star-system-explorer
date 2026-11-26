// The light inside each shadow, read from the kernel (`read_shadow_light`, VP-L3) for the pairs a view may show: a
// world and its moons each way round. The caster's air comes from `resolved_system` (atmosphere.ts, as the air's rim
// reads it), the distance is the receiver's orbit, the star is the system's. An airless caster leaves no light, so it
// is not asked about.
import { airFactsOf, airRequest } from "./atmosphere.ts";
import { umbraLight, type ShadowLightRead } from "./eclipse.ts";
import type { Json } from "../../ui-contract/src/index.ts";

export interface UmbraReader {
  resolvedSystem(system: string): Promise<{ ok: true; result: { surfaces: { address: string }[]; facts: Record<string, Record<string, Json>> } } | { ok: false; message: string }>;
  readShadowLight(items: Json[]): Promise<{ ok: true; items: Json[] } | { ok: false; message: string }>;
}

/** One shadow to ask about: `caster` between its star and `receiver`, `distance_m` apart. */
export interface UmbraPair {
  caster: string;
  receiver: string;
  distance_m: number;
}

/** The key a view looks an umbra up by. */
export const umbraKey = (caster: string, receiver: string) => `${caster}>${receiver}`;

/**
 * Each pair's umbra light (eclipse.ts `umbraLight`: per channel, a share of the star's), keyed by `umbraKey`. Pairs
 * whose caster has no air, or whose light does not reach the receiver, are left out (their umbra is black).
 */
export async function readUmbras(k: UmbraReader, system: string, pairs: readonly UmbraPair[], star: { teff_k: number; radius_m: number; distance_m: (caster: string) => number | null }): Promise<Map<string, [number, number, number]>> {
  const out = new Map<string, [number, number, number]>();
  if (!pairs.length) return out;
  const rs = await k.resolvedSystem(system);
  if (!rs.ok) return out;
  const items: Json[] = [];
  for (const p of pairs) {
    const surf = rs.result.surfaces.find((x) => x.address === p.caster) as Parameters<typeof airFactsOf>[1];
    const facts = airFactsOf(rs.result.facts[p.caster], surf);
    const req = facts ? airRequest(p.caster, facts) : null;
    const d = star.distance_m(p.caster);
    if (!facts || !req || !(d && d > 0) || !(p.distance_m > 0)) continue;
    items.push({ id: umbraKey(p.caster, p.receiver), air: req.air, radius_m: facts.radius_m, distance_m: p.distance_m, star: { teff_k: star.teff_k, radius_m: star.radius_m, distance_m: d } });
  }
  if (!items.length) return out;
  const r = await k.readShadowLight(items);
  if (!r.ok) return out;
  for (const it of r.items as unknown as (ShadowLightRead & { id: string })[]) {
    const u = umbraLight(it);
    if (u) out.set(it.id, u);
  }
  return out;
}
