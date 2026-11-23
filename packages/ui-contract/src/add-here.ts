// Add here (V3 gap 2): V3's commonest edit, adding one body by hand, done the SSE4 way. There is no list of types to
// keep in step with the physics: the offer is what this orbit would make, sampled from real formation runs round a
// star like this one, plus anything you like, marked impossible because you chose it and the physics did not.
import type { Json } from "./json.ts";
import type { SystemScene } from "./scene.ts";
import type { ViewSession } from "./session.ts";
import { AU_M, M_EARTH, R_EARTH_M, sceneFromFormation } from "./make.ts";
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
