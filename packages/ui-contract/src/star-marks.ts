// Stars that show what they do (V3 gap 19). A star's mark reads the kernel's
// facts about it, the way a planet's reads its class: its power (so a
// supergiant's mark is bigger than a red dwarf's) and its activity (a star
// whose corona is saturated or active flares often enough to draw). The facts
// come from the resolved system (power, temperature, radius, mass, age) and the
// kernel's read_activity; the symbology binds them to channels (size from
// `number:luminosity_lsun`, aura from `tag:activity:`). Presentation only.

import { constant } from "./sheet.ts";
import type { MarkSubject } from "./symbology.ts";
import type { Json } from "./json.ts";
import type { ViewSession } from "./session.ts";

/** What a star's mark reads beyond its kind and classes. */
export interface StarFacts {
  /** `activity:<regime>` (saturated, active, moderate, quiet, hot-wind, no-corona, compact), and `flares` when it flares at all. */
  tags: string[];
  /** `luminosity_lsun`. */
  numbers: Record<string, number>;
}

/** A star's power in Suns, from its radius and temperature (Stefan-Boltzmann), or null without both. */
export function luminosityLsun(radius_m: number, teff_k: number): number | null {
  if (!(radius_m > 0 && teff_k > 0)) return null;
  return (4 * Math.PI * radius_m * radius_m * constant("sigma_SB") * teff_k ** 4) / constant("L_sun");
}

/** The facts from one star's resolved facts and its read_activity row (either may be missing). */
export function starFactsFrom(facts: Readonly<Record<string, Json>> | undefined, activity: { regime?: unknown; flares?: unknown } | undefined): StarFacts {
  const out: StarFacts = { tags: [], numbers: {} };
  const given = Number(facts?.["star.luminosity"]);
  const l = given > 0 ? given / constant("L_sun") : luminosityLsun(Number(facts?.["body.radius"]), Number(facts?.["star.effective_temperature"]));
  if (l !== null) out.numbers.luminosity_lsun = l;
  if (typeof activity?.regime === "string") out.tags.push(`activity:${activity.regime}`);
  if (activity?.flares === true) out.tags.push("flares");
  return out;
}

/** A subject with a star's facts added (unchanged without them). */
export function withStarFacts(s: MarkSubject, f: StarFacts | undefined): MarkSubject {
  return f ? { ...s, tags: [...s.tags, ...f.tags], numbers: { ...s.numbers, ...f.numbers } } : s;
}

const YEAR_S = 365.25 * 86400;

/**
 * The facts for a system's stars, by address: one resolved_system and one
 * read_activity for all of them. A star the kernel cannot describe yet (no
 * temperature or radius) is left out, so its mark stays the plain star.
 */
export async function readStarFacts(session: Pick<ViewSession, "resolvedSystem" | "readActivity">, system: string, stars: readonly string[]): Promise<Map<string, StarFacts>> {
  const out = new Map<string, StarFacts>();
  if (!stars.length) return out;
  const rs = await session.resolvedSystem(system);
  if (!rs.ok) return out;
  const age = Number(rs.result.facts[system]?.["system.age"] ?? 0);
  const asks: { address: string; mass_kg: number; radius_m: number; luminosity_w: number; teff_k: number; age_s: number }[] = [];
  for (const a of stars) {
    const f = rs.result.facts[a];
    const teff = Number(f?.["star.effective_temperature"]);
    const radius = Number(f?.["body.radius"]);
    const mass = Number(f?.["body.mass"]);
    const l = starFactsFrom(f, undefined).numbers.luminosity_lsun;
    if (l === undefined || !(mass > 0 && radius > 0 && teff > 0)) continue;
    out.set(a, starFactsFrom(f, undefined));
    asks.push({ address: a, mass_kg: mass, radius_m: radius, luminosity_w: l * constant("L_sun"), teff_k: teff, age_s: age > 0 ? age : 4.6e9 * YEAR_S });
  }
  if (!asks.length) return out;
  const r = await session.readActivity({ stars: asks as unknown as Json[] });
  if (!r.ok) return out;
  r.stars.forEach((row, i) => {
    const a = asks[i]!.address;
    const act = (row as { activity?: { regime?: unknown; flares?: unknown } }).activity;
    out.set(a, starFactsFrom(rs.result.facts[a], act));
  });
  return out;
}
