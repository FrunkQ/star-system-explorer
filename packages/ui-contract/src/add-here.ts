// Add here (V3 gap 2): V3's commonest edit, adding one body by hand, done the SSE4 way. There is no list of types to
// keep in step with the physics: the offer is what this orbit would make, sampled from real formation runs round a
// star like this one, plus anything you like, marked impossible because you chose it and the physics did not.
import type { Json } from "./json.ts";
import type { SystemScene } from "./scene.ts";
import type { ViewSession } from "./session.ts";
import type { PackDoc } from "./physics.ts";
import { AU_M, M_EARTH, M_SUN, R_EARTH_M, sceneFromFormation } from "./make.ts";
import { copyClip, clipWords, type Clip } from "./clip.ts";

export interface FormsHere {
  clip: Clip;
  words: string;
  /** Where that run formed it, and how far that is from the orbit asked for (a ratio, 1 is exact). */
  formed_au: number;
  mass_earth: number;
  radius_earth: number;
  surface_k: number | null;
  seed: string;
}

const num = (s: SystemScene["subjects"][number], field: string) => {
  const v = s.facts.find((f) => f.field === field)?.value;
  return typeof v === "number" ? v : null;
};

/**
 * Worlds that form near `a_au` round a star of `star_mass_sun`, from `tries` formation runs (deterministic: the same
 * place gives the same offer). Each is the nearest world of its run within a factor `within` of the orbit; runs that
 * make nothing there add nothing. Nothing is written: formation runs are read only.
 */
export async function formsHere(
  session: ViewSession,
  at: { system: string; star_mass_sun: number; a_au: number; tries?: number; within?: number; packs?: readonly { id: string; version: string }[] },
): Promise<{ ok: true; found: FormsHere[]; runs: number } | { ok: false; message: string }> {
  const tries = at.tries ?? 3;
  const within = at.within ?? 1.6;
  const found: FormsHere[] = [];
  for (let k = 0; k < tries; k++) {
    const seed = `here ${at.system} ${at.a_au.toPrecision(3)} ${k}`;
    const r = await session.runFormation({ seed, star_pins: { primary_mass: at.star_mass_sun } });
    if (!r.ok) return { ok: false, message: r.message };
    const scene = sceneFromFormation(r.result, seed);
    let best: { s: SystemScene["subjects"][number]; d: number } | null = null;
    for (const s of scene.subjects) {
      if (s.kind !== "planet" || !s.orbit) continue;
      const d = Math.abs(Math.log(s.orbit.a_au / at.a_au));
      if (d <= Math.log(within) && (!best || d < best.d)) best = { s, d };
    }
    if (!best) continue;
    const c = copyClip(scene, best.s.address, { packs: at.packs ?? [] });
    if (!c) continue;
    // Only the world itself (its moons, if any, stay with the run that made them), with what the run made it of: its
    // mass and size go in as stated values, so the physics here works out the rest from them.
    const mass = num(best.s, "mass_earth");
    const radius = num(best.s, "radius_earth");
    const ecc = num(best.s, "eccentricity");
    const own = c.clip.events.filter((e) => e.node === c.clip.root && !["body.mass", "body.radius", "orbit.eccentricity"].includes(e.field));
    const stated: [string, number | null][] = [
      ["body.mass", mass === null ? null : mass * M_EARTH],
      ["body.radius", radius === null ? null : radius * R_EARTH_M],
      ["orbit.eccentricity", ecc],
    ];
    const clip: Clip = {
      ...c.clip,
      nodes: c.clip.nodes.slice(0, 1),
      events: [...own, ...stated.filter((x): x is [string, number] => x[1] !== null).map(([field, value]) => ({ kind: "authored.set" as const, node: c.clip.root, field, value }))],
    };
    found.push({
      clip,
      words: clipWords(clip),
      formed_au: best.s.orbit!.a_au,
      mass_earth: mass ?? 0,
      radius_earth: radius ?? 0,
      surface_k: num(best.s, "surface_k"),
      seed,
    });
  }
  return { ok: true, found, runs: tries };
}

/**
 * Anything you like: a world of the mass and size you give, every value marked impossible (the physics did not make
 * it here; you did). The honesty mark shows on the GM's view only, and the physics still works out the rest.
 */
export function anythingClip(x: { name: string; kind: "planet" | "moon"; mass_earth: number; radius_earth: number }): Clip {
  const values: [string, Json][] = [
    ["body.mass", x.mass_earth * M_EARTH],
    ["body.radius", x.radius_earth * R_EARTH_M],
  ];
  return {
    sseClip: 2,
    generation: "v4",
    source: { site: "sse4", title: x.name, creditChain: [] },
    packs: [],
    root: "n0",
    nodes: [{ id: "n0", kind: x.kind, parent: null }],
    events: [{ kind: "authored.set", node: "n0", field: "name", value: x.name }, ...values.map(([field, value]) => ({ kind: "authored.set" as const, node: "n0", field, value, impossible: true }))],
  };
}

/** A clip moved to another orbit: its root's semi-major axis replaced (or added). */
export function clipAt(c: Clip, a_au: number): Clip {
  const events = c.events.filter((e) => !(e.node === c.root && e.field === "orbit.semi_major_axis"));
  events.push({ kind: "authored.set", node: c.root, field: "orbit.semi_major_axis", value: a_au * AU_M });
  return { ...c, events };
}

/**
 * Where a new world could go round `host`: the middle of the widest gap between the orbits already there (by ratio,
 * as orbits space out), or past the last one. In AU.
 */
export function freeOrbit(orbits_au: readonly number[], first_au = 0.4): number {
  const a = [...orbits_au].filter((x) => x > 0).sort((p, q) => p - q);
  if (!a.length) return first_au;
  let best = { ratio: 0, at: a[a.length - 1]! * 1.7 };
  for (let i = 1; i < a.length; i++) {
    const ratio = a[i]! / a[i - 1]!;
    if (ratio > best.ratio && ratio > 1.6) best = { ratio, at: Math.sqrt(a[i]! * a[i - 1]!) };
  }
  return Number(best.at.toPrecision(2));
}

/** One of the three moon recipes: formation's own leftovers rules (F5) for one chosen planet, until Ask K5 lands. */
export interface MoonRecipe {
  id: "disc" | "impact" | "captured";
  title: string;
  /** What the rule is, in words. */
  says: string;
  /** The paper the rule comes from. */
  cite: string;
  clip: Clip;
  /** Where it goes round the planet, AU. */
  a_au: number;
  mass_earth: number;
  radius_earth: number;
  /** The same in SI, for the units engine to say. */
  mass_kg: number;
  a_m: number;
}

/** Over this, a planet is a giant with a disc of its own (F5 gives regular moons to giants only). */
export const GIANT_MASS_EARTH = 10;

/**
 * The three moon recipes for one planet (editing design ED7, question 13), from the same rules formation's leftovers
 * stage (F5, `sse-formation/src/leftovers.rs`) runs for a generated system, taken at the middle of their ranges:
 * - a disc moon, for a giant: regular satellites hold about 1e-4 of their planet (Canup and Ward 2006), shared by two
 *   to five moons (one is a third of it), the first at 4 to 8 planet radii and each next 1.5 to 2 times further out,
 *   on round, flat orbits;
 * - an impact moon, for a rocky world: an oblique merging impact leaves 0.5 to 2 percent of the planet as a moon
 *   (Canup 2004) at 3 to 6 planet radii, where it is born (the tides then move it);
 * - a captured body, for any planet: 0.1 to 0.5 of the Hill radius, eccentric, half of them backwards (Jewitt and
 *   Haghighipour 2007). F5 gives small rocky planets none yet (Phobos and Deimos are its one listed gap).
 * Each goes in as authored values, not impossible: the physics made moons this way; you chose which one.
 */
export function moonRecipes(
  planet: { name: string; mass_earth: number; radius_earth: number; a_au: number | null; star_mass_sun: number | null },
  moons_au: readonly number[] = [],
): MoonRecipe[] {
  const out: MoonRecipe[] = [];
  const giant = planet.mass_earth >= GIANT_MASS_EARTH;
  const rp = (planet.radius_earth * R_EARTH_M) / AU_M;
  const free = (a: number, step: number) => {
    for (let k = 0; k < 20 && moons_au.some((m) => Math.abs(Math.log(m / a)) < Math.log(1.3)); k++) a *= step;
    return a;
  };
  const make = (id: MoonRecipe["id"], title: string, says: string, cite: string, mass_earth: number, rho: number, a_au: number, orbit: [string, number][]) => {
    const radius_earth = Math.cbrt((3 * mass_earth * M_EARTH) / (4 * Math.PI * rho)) / R_EARTH_M;
    const name = `${title} of ${planet.name}`;
    const clip: Clip = {
      sseClip: 2,
      generation: "v4",
      source: { site: "sse4", title: name, creditChain: [] },
      packs: [],
      root: "n0",
      nodes: [{ id: "n0", kind: "moon", parent: null }],
      events: [
        { kind: "authored.set", node: "n0", field: "name", value: name },
        { kind: "authored.set", node: "n0", field: "body.mass", value: mass_earth * M_EARTH },
        { kind: "authored.set", node: "n0", field: "body.radius", value: radius_earth * R_EARTH_M },
        ...orbit.map(([field, value]) => ({ kind: "authored.set" as const, node: "n0", field, value })),
      ],
    };
    out.push({ id, title, says, cite, clip: clipAt(clip, a_au), a_au, mass_earth, radius_earth, mass_kg: mass_earth * M_EARTH, a_m: a_au * AU_M });
  };
  if (giant)
    make(
      "disc",
      "A disc moon",
      "Grown in the planet's own disc of gas and dust, which holds about a ten-thousandth of the planet and makes two to five moons: this one takes a third of it, close in, on a round orbit in the planet's plane.",
      "Canup and Ward 2006, Nature 441, 834",
      (planet.mass_earth * 1e-4) / 3,
      2_000,
      free(6 * rp, 1.75),
      [["orbit.eccentricity", 0.005], ["orbit.inclination", 0.25 * (Math.PI / 180)]],
    );
  else
    make(
      "impact",
      "An impact moon",
      "Thrown out by a glancing giant impact, as the Moon was: about one and a quarter percent of the planet, mostly rock, born a few planet radii out. The tides then move it.",
      "Canup 2004, Icarus 168, 433",
      planet.mass_earth * 0.0125,
      3_000,
      free(4.5 * rp, 1.5),
      [["orbit.eccentricity", 0]],
    );
  if (planet.a_au && planet.star_mass_sun) {
    const hill = planet.a_au * Math.cbrt((planet.mass_earth * M_EARTH) / (3 * planet.star_mass_sun * M_SUN));
    const d_km = giant ? 60 : 20;
    const rho = 2_000;
    const mass = ((4 / 3) * Math.PI * (d_km * 500) ** 3 * rho) / M_EARTH;
    make(
      "captured",
      "A captured body",
      `A small body caught as it passed, ${d_km} km across, far out at a third of the planet's reach, on a stretched and backwards orbit (half of them go round backwards).`,
      "Jewitt and Haghighipour 2007, ARA&A 45, 261",
      mass,
      rho,
      free(0.3 * hill, 1.3),
      [["orbit.eccentricity", 0.35], ["orbit.inclination", 145 * (Math.PI / 180)]],
    );
  }
  return out;
}

/** A kind of world to add (ED6, editing design section 9): a class's "make one" recipe. */
export interface WorldKind {
  /** The atlas class it makes. */
  class: string;
  /** "Gas giant". */
  name: string;
  /** The class's representative world, whose inputs the recipe is ("Jupiter"). */
  like: string;
  says: string;
  mass_kg: number;
  radius_m: number;
  clip: Clip;
}

/**
 * The Kinds a world can be added as, from the class atlases on the stack (question 8, Alex yes): each base class of
 * the physical scheme (the top level only: rocky, iron-rich, water-rich, sub-Neptune, ice giant, gas giant, brown
 * dwarf) offers its first named world's mass and size. They go in as authored values (a kind is a starting point the
 * physics then works from, not a claim the physics made it there); the class is read off the results as always, so
 * the label can change once it is in place. Classes without a size in their example (small bodies) are left out.
 */
export function worldKinds(packs: readonly PackDoc[], name?: string): WorldKind[] {
  const byId = new Map<string, WorldKind>();
  for (const p of packs) {
    const o = (p.provides as { other?: { classes?: unknown } } | undefined)?.other;
    if (!o || !Array.isArray(o.classes)) continue;
    for (const c of o.classes as {
      id: string;
      scheme?: string;
      kind?: string;
      parent?: string;
      names?: { plain?: string; physics?: string };
      learn?: { summary?: string };
      examples?: { name: string; expect?: string; values?: Record<string, unknown> }[];
    }[]) {
      if (c.scheme !== "physical" || (c.kind ?? "base") !== "base" || c.parent || c.id.includes("/")) continue;
      const ex = (c.examples ?? []).find((e) => e.expect !== "out" && typeof e.values?.["body.mass"] === "number" && typeof e.values?.["body.radius"] === "number");
      if (!ex) continue;
      const kindName = c.names?.plain ?? c.names?.physics ?? c.id;
      const title = kindName.charAt(0).toUpperCase() + kindName.slice(1);
      const mass = ex.values!["body.mass"] as number;
      const radius = ex.values!["body.radius"] as number;
      const n = name ?? `New ${kindName}`;
      byId.set(c.id, {
        class: c.id,
        name: title,
        like: ex.name,
        says: c.learn?.summary ?? "",
        mass_kg: mass,
        radius_m: radius,
        clip: {
          sseClip: 2,
          generation: "v4",
          source: { site: "sse4", title: n, creditChain: [] },
          packs: [],
          root: "n0",
          nodes: [{ id: "n0", kind: "planet", parent: null }],
          events: [
            { kind: "authored.set", node: "n0", field: "name", value: n },
            { kind: "authored.set", node: "n0", field: "body.mass", value: mass },
            { kind: "authored.set", node: "n0", field: "body.radius", value: radius },
          ],
        },
      });
    }
  }
  return [...byId.values()].sort((x, y) => x.mass_kg - y.mass_kg);
}
