// Copy and paste (top-level design 8.2). A clip is one body and everything
// under it, as authored facts in the kernel's own fields (SI units), with the
// packs it was made under named. It is plain text on the clipboard, so the
// same clip moves between campaigns, browsers and the Explorers site, which
// stores and serves the bytes as they are (`sseClip: 2`, agreed with the
// Explorers thread; V3's `sseClip: 1` is unchanged and goes through the V3
// importer).
//
// Nothing here computes physics: values are converted between the scene's
// display units and the kernel's SI fields, and derived values are left out
// because the physics works them out again wherever the clip lands.
import type { Json } from "./json.ts";
import type { SceneFact, SceneSubject, SystemScene } from "./scene.ts";
import { primaryOf } from "./scene.ts";
import { AU_M, L_SUN_W, M_EARTH, M_SUN, R_EARTH_M, R_SUN_M } from "./make.ts";

export const CLIP_VERSION = 2;

/** One node of a clip: a body, its kind, and the node it orbits (null for the root). */
export interface ClipNode {
  id: string;
  kind: string;
  parent: string | null;
}

/** One authored fact, on a node, in the kernel's field and SI unit. */
export interface ClipEvent {
  kind: "authored.set";
  node: string;
  field: string;
  value: Json;
  /** Set against the physics (the honesty mark IMPOSSIBLE travels with it). */
  impossible?: boolean;
}

export interface ClipSource {
  site: string;
  title: string;
  url?: string;
  /** Who made it, oldest first; a paste and a later publish add to it. */
  creditChain: string[];
}

export interface Clip {
  sseClip: 2;
  generation: "v4";
  source: ClipSource;
  /** The packs the clip was made under, bottom to top. */
  packs: { id: string; version: string }[];
  root: string;
  /** Parents before children. */
  nodes: ClipNode[];
  events: ClipEvent[];
}

/**
 * Scene fields and the kernel fields they are kept as. `scale` turns the
 * scene's display value into SI; kinds limits a field to stars or to
 * everything else, where the same scene field differs.
 */
const FIELDS: { scene: string; fact: string; scale?: number; kinds?: "star" | "body" }[] = [
  { scene: "name", fact: "name" },
  { scene: "a_au", fact: "orbit.semi_major_axis", scale: AU_M },
  { scene: "eccentricity", fact: "orbit.eccentricity" },
  { scene: "mass_earth", fact: "body.mass", scale: M_EARTH, kinds: "body" },
  { scene: "radius_earth", fact: "body.radius", scale: R_EARTH_M, kinds: "body" },
  { scene: "mass_sun", fact: "body.mass", scale: M_SUN, kinds: "star" },
  { scene: "radius_sun", fact: "body.radius", scale: R_SUN_M, kinds: "star" },
  { scene: "day_h", fact: "body.rotation_period", scale: 3600 },
  { scene: "teff_k", fact: "star.effective_temperature", kinds: "star" },
  { scene: "luminosity_sun", fact: "star.luminosity", scale: L_SUN_W, kinds: "star" },
  { scene: "albedo", fact: "body.albedo", kinds: "body" },
];

const fitsKind = (kinds: "star" | "body" | undefined, kind: string) => !kinds || (kinds === "star") === (kind === "star");

/** Kinds that are drawn from other bodies, never copied: the temperate zone follows its star. */
const NOT_COPIED = new Set(["zone"]);

/** Everything under `address`, by orbit (parent links), parents first. The primary owns bodies with no parent. */
export function subtree(scene: Pick<SystemScene, "subjects">, address: string): SceneSubject[] {
  const primary = primaryOf(scene)?.address;
  const parentOf = (s: SceneSubject) => s.parent ?? (s.address === primary ? undefined : primary);
  const root = scene.subjects.find((s) => s.address === address);
  if (!root) return [];
  const out: SceneSubject[] = [root];
  for (let i = 0; i < out.length; i++) {
    const at = out[i]!.address;
    for (const s of scene.subjects) if (parentOf(s) === at && !NOT_COPIED.has(s.kind) && !out.includes(s)) out.push(s);
  }
  return out;
}

/** The facts one subject carries into a clip, and the ones left for the physics to work out again. */
function factsOf(s: SceneSubject): { kept: { field: string; value: Json }[]; left: SceneFact[] } {
  const kept: { field: string; value: Json }[] = [];
  const left: SceneFact[] = [];
  const has = new Set<string>();
  for (const f of s.facts) {
    const m = FIELDS.find((x) => x.scene === f.field && fitsKind(x.kinds, s.kind));
    if (!m || f.source === "derived" || f.value === null || f.value === undefined) {
      if (f.field !== "name") left.push(f);
      continue;
    }
    const v = m.scale !== undefined && typeof f.value === "number" ? f.value * m.scale : f.value;
    kept.push({ field: m.fact, value: v });
    has.add(m.fact);
  }
  if (!has.has("name")) kept.unshift({ field: "name", value: s.label });
  if (!has.has("orbit.semi_major_axis") && s.orbit) kept.push({ field: "orbit.semi_major_axis", value: s.orbit.a_au * AU_M });
  if (s.band_au) {
    kept.push({ field: "body.radius_inner", value: s.band_au[0] * AU_M }, { field: "body.radius_outer", value: s.band_au[1] * AU_M });
    if (s.shape) kept.push({ field: "belt.shape", value: s.shape });
  }
  return { kept, left };
}

/** What a copy took and what it left behind, in words for the status line. */
export interface Copied {
  clip: Clip;
  text: string;
  words: string;
  /** Values left out because the physics works them out again where the clip lands. */
  left: string[];
}

/** Copy one body and everything under it. Null when the address is not in the scene. */
export function copyClip(scene: SystemScene, address: string, opts: { packs?: readonly { id: string; version: string }[]; site?: string; credit?: string[] } = {}): Copied | null {
  const tree = subtree(scene, address);
  if (!tree.length) return null;
  const ids = new Map(tree.map((s, i) => [s.address, `n${i}`]));
  const primary = primaryOf(scene)?.address;
  const nodes: ClipNode[] = tree.map((s, i) => ({ id: `n${i}`, kind: s.kind, parent: i === 0 ? null : (ids.get(s.parent ?? primary ?? "") ?? "n0") }));
  const events: ClipEvent[] = [];
  const left = new Set<string>();
  tree.forEach((s, i) => {
    const impossible = s.stamps.includes("impossible");
    const { kept, left: l } = factsOf(s);
    for (const k of kept) events.push({ kind: "authored.set", node: `n${i}`, field: k.field, value: k.value, ...(impossible && k.field !== "name" ? { impossible: true } : {}) });
    for (const f of l) left.add(f.label);
  });
  const clip: Clip = {
    sseClip: 2,
    generation: "v4",
    source: { site: opts.site ?? "sse4", title: tree[0]!.label, creditChain: [...(opts.credit ?? [])] },
    packs: (opts.packs ?? []).map((p) => ({ id: p.id, version: p.version })),
    root: "n0",
    nodes,
    events,
  };
  return { clip, text: JSON.stringify(clip), words: clipWords(clip), left: [...left] };
}

/** "Tarn, with 1 moon", "Kestrel, with 5 planets, 4 moons and a belt". */
export function clipWords(c: Clip): string {
  const name = (id: string) => {
    const e = c.events.find((x) => x.node === id && x.field === "name");
    return typeof e?.value === "string" ? e.value : "an unnamed body";
  };
  const counts = new Map<string, number>();
  for (const n of c.nodes) if (n.id !== c.root) counts.set(n.kind, (counts.get(n.kind) ?? 0) + 1);
  const PLURAL: Record<string, string> = { moon: "moons", planet: "planets", belt: "belts", station: "stations", star: "stars", ship: "ships" };
  const parts = [...counts].map(([k, n]) => (n === 1 ? `${/^[aeiou]/.test(k) ? "an" : "a"} ${k}` : `${n} ${PLURAL[k] ?? `${k}s`}`));
  const list = parts.length > 1 ? `${parts.slice(0, -1).join(", ")} and ${parts.at(-1)}` : parts[0];
  return list ? `${name(c.root)}, with ${list}` : name(c.root);
}

export type ClipRead =
  | { ok: true; clip: Clip }
  | { ok: false; v3: true; why: string; text: string }
  | { ok: false; v3?: false; why: string };

/** Read clipboard text as a clip. A V3 clip is recognised and handed back for the V3 importer. */
export function readClip(text: string): ClipRead {
  let j: unknown;
  try {
    j = JSON.parse(text.trim());
  } catch {
    return { ok: false, why: "The clipboard does not hold a Star System Explorer clip." };
  }
  if (!j || typeof j !== "object" || Array.isArray(j)) return { ok: false, why: "The clipboard does not hold a Star System Explorer clip." };
  const o = j as Record<string, unknown>;
  if (o.sseClip === 1) return { ok: false, v3: true, why: "This is a clip from Star System Explorer 3. V3 clips go through the V3 importer, which comes with the next part of paste.", text };
  if (o.sseClip !== CLIP_VERSION) return { ok: false, why: o.sseClip === undefined ? "The clipboard does not hold a Star System Explorer clip." : `This clip is version ${String(o.sseClip)}, newer than this chart room reads.` };
  const nodes = o.nodes as ClipNode[] | undefined;
  const events = o.events as ClipEvent[] | undefined;
  if (!Array.isArray(nodes) || !nodes.length || !Array.isArray(events)) return { ok: false, why: "The clip has no bodies in it." };
  const ids = new Set<string>();
  for (const n of nodes) {
    if (!n || typeof n.id !== "string" || typeof n.kind !== "string") return { ok: false, why: "A body in the clip has no id or kind." };
    if (n.parent !== null && !ids.has(n.parent)) return { ok: false, why: "The clip lists a body before the one it orbits." };
    ids.add(n.id);
  }
  if (typeof o.root !== "string" || !ids.has(o.root) || nodes[0]!.id !== o.root) return { ok: false, why: "The clip's first body is not its root." };
  for (const e of events) {
    if (!e || e.kind !== "authored.set" || !ids.has(e.node) || typeof e.field !== "string" || !/^[a-z][a-z0-9_.-]*$/.test(e.field)) return { ok: false, why: "A fact in the clip is malformed." };
  }
  const src = (o.source ?? {}) as Partial<ClipSource>;
  return {
    ok: true,
    clip: {
      sseClip: 2,
      generation: "v4",
      source: { site: String(src.site ?? "unknown"), title: String(src.title ?? ""), ...(src.url ? { url: String(src.url) } : {}), creditChain: Array.isArray(src.creditChain) ? src.creditChain.map(String) : [] },
      packs: Array.isArray(o.packs) ? (o.packs as { id: unknown; version: unknown }[]).map((p) => ({ id: String(p.id), version: String(p.version) })) : [],
      root: o.root,
      nodes,
      events,
    },
  };
}

/** Packs the clip names that the stack in force lacks (by id; another version is named, not refused). */
export function missingPacks(c: Clip, stack: readonly { id: string; version: string }[]): { id: string; version: string; have?: string }[] {
  const out: { id: string; version: string; have?: string }[] = [];
  for (const p of c.packs) {
    const h = stack.find((s) => s.id === p.id);
    if (!h) out.push(p);
    else if (h.version !== p.version) out.push({ ...p, have: h.version });
  }
  return out;
}

/** One body a paste would make, at its new address. */
export interface PlacedBody {
  address: string;
  parent: string;
  kind: string;
  label: string;
}

/**
 * Where a clip would land: new addresses under `host` (the next free numbers,
 * never an address the system already has), the root's orbit moved to
 * `a_au` when given, and every fact as one edit. A body orbiting a star sits
 * at the system's top level, as the chart's planets do; a moon sits under
 * its planet.
 */
export function placeClip(
  c: Clip,
  at: { system: string; host: string; hostKind: string; a_au?: number; taken: Iterable<string> },
): { placed: PlacedBody[]; edits: { address: string; field: string; value: Json; impossible: boolean }[] } {
  const taken = new Set(at.taken);
  const addr = new Map<string, string>();
  const placed: PlacedBody[] = [];
  const kindOf = new Map(c.nodes.map((n) => [n.id, n.kind]));
  const next = (under: string, kind: string) => {
    const seg = kind === "belt" ? "belt" : kind === "station" ? "craft" : "body";
    let i = 0;
    while (taken.has(`${under}/${seg}:${i}`)) i++;
    const a = `${under}/${seg}:${i}`;
    taken.add(a);
    return a;
  };
  for (const n of c.nodes) {
    const parent = n.parent === null ? at.host : addr.get(n.parent)!;
    const parentKind = n.parent === null ? at.hostKind : kindOf.get(n.parent)!;
    const a = next(parentKind === "star" ? at.system : parent, n.kind);
    addr.set(n.id, a);
    const name = c.events.find((e) => e.node === n.id && e.field === "name")?.value;
    placed.push({ address: a, parent, kind: n.kind, label: typeof name === "string" ? name : n.kind });
  }
  const edits: { address: string; field: string; value: Json; impossible: boolean }[] = [];
  for (const n of c.nodes) {
    const a = addr.get(n.id)!;
    edits.push({ address: a, field: "body.kind", value: n.kind, impossible: false });
    edits.push({ address: a, field: "parent", value: n.parent === null ? at.host : addr.get(n.parent)!, impossible: false });
  }
  for (const e of c.events) {
    const value = e.node === c.root && e.field === "orbit.semi_major_axis" && at.a_au !== undefined ? at.a_au * AU_M : e.value;
    edits.push({ address: addr.get(e.node)!, field: e.field, value, impossible: e.impossible ?? false });
  }
  if (at.a_au !== undefined && !c.events.some((e) => e.node === c.root && e.field === "orbit.semi_major_axis"))
    edits.push({ address: addr.get(c.root)!, field: "orbit.semi_major_axis", value: at.a_au * AU_M, impossible: false });
  return { placed, edits };
}
