// A world's level-0 surface tiles and air, as the Body globe draws them, kept per system and body for the page's life
// (they take about a second to build). Shared by the Body view and the body card's pictures, so a world is built once.
import type { SurfaceTile, ViewContext } from "../../../ui-contract/src/index.ts";
import { airFactsOf, airFromOptics, airRequest, readGroundOptics, readTrueFace, withGround, type AirFacts, type AirFlow, type FaceReader, type TrueTexture } from "../../../renderer/src/index.ts";

export type Surface = { tiles: SurfaceTile[]; regime?: string; refused: number; air: AirFacts | null; /** The world's outermost layer as the kernel names it ("solid", "cloud-deck"...), when it says. */ kind?: string };

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
    const kind = (r as { kind?: unknown }).kind;
    // Bare ground in its measured colour where the kernel weathers it (renderer ground-optics.ts, VP3 part 5).
    const tiles = r.tiles.some((t) => t.maturity) ? withGround(r.tiles, await readGroundOptics(ctx.session)) : r.tiles;
    const v: Surface = { tiles, regime: r.regime, refused: r.refused.length, air: facts, ...(typeof kind === "string" ? { kind } : {}) };
    cache.set(key, v);
    return v;
  })().finally(() => pending.delete(key));
  pending.set(key, p);
  return p;
}

// A giant's True face (VP3 part 3): the same face the system chart wears (renderer true-air.ts `readTrueFace`), kept
// beside the surface so the Body globe and the card show its cloud decks, not the grey stand-in its tiles give. Only a
// world the face reader marks as decks (`decks: true`, its outermost layer a cloud deck) gets one; any other world
// keeps its tiles' own colours and relief.
export type Decks = { tex: TrueTexture; flow?: AirFlow };
const decks = new Map<string, Decks | null>();
const decksPending = new Map<string, Promise<Decks | null>>();

/** The decks already read for this body (undefined: not asked yet; null: not a giant, or none could be read). */
export function decksKept(system: string, address: string): Decks | null | undefined {
  return decks.get(`${system}|${address}`);
}

/** Whether a surface is a giant's cloud deck (the kernel's outermost layer), so its decks are worth reading. */
export const isDecks = (s: Surface | null | undefined): boolean => s?.kind === "cloud-deck";

/** A giant's decks, read once (concurrent asks share the read); null for a world with ground. */
export function loadDecks(ctx: ViewContext, system: string, address: string): Promise<Decks | null> {
  const key = `${system}|${address}`;
  if (decks.has(key)) return Promise.resolve(decks.get(key)!);
  const on = decksPending.get(key);
  if (on) return on;
  const p = (async () => {
    const look = await readTrueFace(ctx.session as unknown as FaceReader, system, address);
    const v = typeof look !== "string" && (look as { decks?: boolean }).decks === true ? { tex: look.tex, ...(look.flow ? { flow: look.flow } : {}) } : null;
    decks.set(key, v);
    return v;
  })()
    .catch(() => null)
    .finally(() => decksPending.delete(key));
  decksPending.set(key, p);
  return p;
}
