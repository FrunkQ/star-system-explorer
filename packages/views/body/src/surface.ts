// A world's level-0 surface tiles and air, as the Body globe draws them, kept per system and body for the page's life
// (they take about a second to build). Shared by the Body view and the body card's pictures, so a world is built once.
import type { SurfaceTile, ViewContext } from "../../../ui-contract/src/index.ts";
import { airFactsOf, airFromOptics, airRequest, type AirFacts } from "../../../renderer/src/index.ts";

export type Surface = { tiles: SurfaceTile[]; regime?: string; refused: number; air: AirFacts | null };

const cache = new Map<string, Surface>();
const pending = new Map<string, Promise<Surface | string>>();

/** The surface already built for this body, if any (no work). */
export function surfaceKept(system: string, address: string): Surface | undefined {
  return cache.get(`${system}|${address}`);
}

/** The surface of one body: from the cache, else built once (concurrent asks share the build). A string is the kernel's refusal. */
export function loadSurface(ctx: ViewContext, system: string, address: string): Promise<Surface | string> {
  const key = `${system}|${address}`;
  const hit = cache.get(key);
  if (hit) return Promise.resolve(hit);
  const on = pending.get(key);
  if (on) return on;
  const p = (async (): Promise<Surface | string> => {
    const r = await ctx.session.surfaceTiles(system, address);
    if (!r.ok) return r.message;
    // The air the atmosphere pass draws: the kernel's surface pressure, temperature, gravity and gas mix.
    const rs = await ctx.session.resolvedSystem(system);
    const surf = rs.ok ? (rs.result.surfaces.find((x) => x.address === address) as Parameters<typeof airFactsOf>[1]) : undefined;
    const facts = rs.ok ? airFactsOf(rs.result.facts[address], surf) : null;
    // The optics themselves come from the kernel (light track, read_air_optics).
    const req = facts ? airRequest(address, facts) : null;
    if (facts && req) {
      const ao = await ctx.session.readAirOptics([req.air]);
      facts.optics = ao.ok && ao.airs[0] ? airFromOptics(ao.airs[0] as Parameters<typeof airFromOptics>[0], req) : null;
    }
    const v: Surface = { tiles: r.tiles, regime: r.regime, refused: r.refused.length, air: facts };
    cache.set(key, v);
    return v;
  })().finally(() => pending.delete(key));
  pending.set(key, p);
  return p;
}
