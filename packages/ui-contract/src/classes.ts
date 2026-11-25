// The class gallery (V3 gaps item 35, NM6 "Names and classes"): the pure parts. A class atlas is pack data
// (`provides.other` with `schemes`, `axes` and `classes`, C1 and C2); this reads it into things to show: each
// scheme's question, its classes as a tree (base classes under their parents, modifiers apart), each class's
// bands in words with their units, and the named worlds the atlas checks it against. A class is a label read off a
// world's results; nothing here decides anything about a world, and nothing reaches the kernel.
import type { PackDoc } from "./physics.ts";
import { roundTo } from "./figures.ts";

export interface AtlasAxis {
  field: string;
  scale: "linear" | "log" | "fraction" | "category" | "flag" | string;
  label: string;
  unit?: string;
  span?: [number, number];
  provisional?: boolean;
}

/** One condition of a class: a band on a number axis (core, and the fringe it fades out over), a set, or a flag. */
export interface AtlasBand {
  axis: string;
  core?: [number, number];
  fringe?: [number, number];
  any_of?: string[];
  is?: boolean;
}

export interface AtlasClass {
  id: string;
  scheme: string;
  kind: "base" | "modifier" | string;
  parent?: string;
  names: { physics: string; plain: string; story?: string };
  summary: string;
  region: AtlasBand[];
  /** Conditions inherited from the scheme's gate (giants are left out of surface classes, say). */
  gate: AtlasBand[];
  /** What else must be true that no number shows (an authored settlement, a life model). */
  needs: string[];
  /** "fictional" for a class only a story can make (terraformed). */
  evidence?: string;
  /** Named worlds the atlas is checked against: those that land in it, and near misses that do not. */
  examples: { name: string; in: boolean }[];
  pack: string;
}

export interface AtlasScheme {
  id: string;
  question: string;
  pack: string;
  classes: AtlasClass[];
}

export interface ClassAtlas {
  schemes: AtlasScheme[];
  axes: Map<string, AtlasAxis>;
}

interface AtlasData {
  schemes?: { id: string; question?: string }[];
  axes?: [string, AtlasAxis][] | Record<string, AtlasAxis>;
  classes?: {
    id: string;
    scheme: string;
    kind?: string;
    parent?: string;
    names?: { physics?: string; plain?: string; story?: string };
    learn?: { summary?: string };
    region?: AtlasBand[];
    gate?: AtlasBand[];
    needs?: string[];
    evidence?: string;
    examples?: { name: string; expect?: string }[];
  }[];
}

const atlasOf = (p: PackDoc): AtlasData | null => {
  const o = (p.provides as { other?: AtlasData } | undefined)?.other;
  return o && Array.isArray(o.classes) && o.classes.length ? o : null;
};

/** Every class atlas on the stack, in stack order; a later pack's scheme or class of the same id replaces the earlier. */
export function classesOnStack(packs: readonly PackDoc[]): ClassAtlas {
  const schemes = new Map<string, AtlasScheme>();
  const axes = new Map<string, AtlasAxis>();
  for (const p of packs) {
    const a = atlasOf(p);
    if (!a) continue;
    for (const [id, ax] of Array.isArray(a.axes) ? a.axes : Object.entries(a.axes ?? {})) axes.set(id, ax);
    for (const s of a.schemes ?? []) schemes.set(s.id, { id: s.id, question: s.question ?? "", pack: p.id, classes: schemes.get(s.id)?.classes ?? [] });
    for (const c of a.classes ?? []) {
      const s = schemes.get(c.scheme) ?? { id: c.scheme, question: "", pack: p.id, classes: [] };
      schemes.set(c.scheme, s);
      s.classes = s.classes.filter((x) => x.id !== c.id);
      s.classes.push({
        id: c.id,
        scheme: c.scheme,
        kind: c.kind ?? "base",
        ...(c.parent ? { parent: c.parent } : {}),
        names: { physics: c.names?.physics ?? c.id, plain: c.names?.plain ?? c.names?.physics ?? c.id, ...(c.names?.story ? { story: c.names.story } : {}) },
        summary: c.learn?.summary ?? "",
        region: c.region ?? [],
        gate: c.gate ?? [],
        needs: c.needs ?? [],
        ...(c.evidence ? { evidence: c.evidence } : {}),
        examples: (c.examples ?? []).map((e) => ({ name: e.name, in: e.expect !== "out" })),
        pack: p.id,
      });
    }
  }
  return { schemes: [...schemes.values()].filter((s) => s.classes.length), axes };
}

/** A class's name with a capital, as a gallery tile shows it. */
export function className(c: Pick<AtlasClass, "names">): string {
  const n = c.names.plain;
  return n ? n[0]!.toUpperCase() + n.slice(1) : n;
}

export interface ClassNode {
  cls: AtlasClass;
  depth: number;
}

/**
 * A scheme's base classes as a tree, each under its parent in the atlas's own order (a class whose parent is not
 * in the scheme stands at the top), then its modifiers apart: modifiers add to a base class, never replace it.
 */
export function classTree(s: AtlasScheme): { base: ClassNode[]; modifiers: AtlasClass[] } {
  const base = s.classes.filter((c) => c.kind !== "modifier");
  const ids = new Set(base.map((c) => c.id));
  const kids = new Map<string, AtlasClass[]>();
  for (const c of base) {
    const p = c.parent && ids.has(c.parent) ? c.parent : "";
    kids.set(p, [...(kids.get(p) ?? []), c]);
  }
  const out: ClassNode[] = [];
  const seen = new Set<string>();
  const walk = (p: string, depth: number) => {
    for (const c of kids.get(p) ?? []) {
      if (seen.has(c.id)) continue;
      seen.add(c.id);
      out.push({ cls: c, depth });
      walk(c.id, depth + 1);
    }
  };
  walk("", 0);
  return { base: out, modifiers: s.classes.filter((c) => c.kind === "modifier") };
}

/** A number as a band shows it: four figures at most, powers of ten past a million or below a thousandth. */
export function classNumber(v: number): string {
  if (v === 0) return "0";
  const a = Math.abs(v);
  if (a >= 1e6 || a < 1e-3) {
    const e = Math.floor(Math.log10(a));
    const m = roundTo(v / 10 ** e, 2);
    return `${m === 1 ? "" : `${m}×`}10${superscript(e)}`;
  }
  return String(roundTo(v, 4));
}

const SUP: Record<string, string> = { "-": "⁻", "0": "⁰", "1": "¹", "2": "²", "3": "³", "4": "⁴", "5": "⁵", "6": "⁶", "7": "⁷", "8": "⁸", "9": "⁹" };
const superscript = (n: number) => [...String(n)].map((c) => SUP[c] ?? c).join("");

const pct = (v: number) => `${roundTo(v * 100, 3)}%`;

/** One band in words: "equilibrium temperature 0 to 150 K (fades out by 170 K)", "main gas: carbon dioxide", "has rings". */
export function classBandWords(b: AtlasBand, axes: ReadonlyMap<string, AtlasAxis>, withFringe = true): string {
  const ax = axes.get(b.axis);
  const label = ax?.label ?? b.axis;
  if (b.is !== undefined) return b.is ? label : `not ${label}`;
  if (b.any_of) return `${label}: ${b.any_of.map((x) => x.replace(/[-_]/g, " ")).join(" or ")}`;
  if (!b.core) return label;
  const frac = ax?.scale === "fraction";
  const n = (v: number) => (frac ? pct(v) : classNumber(v));
  const unit = !frac && ax?.unit ? ` ${ax.unit}` : "";
  const [lo, hi] = b.core;
  const top = ax?.span?.[1];
  const bottom = ax?.span?.[0] ?? 0;
  const core = lo <= bottom && hi !== undefined && top !== undefined && hi >= top ? `any ${label}` : lo <= bottom ? `${label} up to ${n(hi)}${unit}` : top !== undefined && hi >= top ? `${label} from ${n(lo)}${unit}` : `${label} ${n(lo)} to ${n(hi)}${unit}`;
  if (!withFringe || !b.fringe) return core;
  const [flo, fhi] = b.fringe;
  const parts = [flo < lo && lo > bottom ? `from ${n(Math.max(flo, bottom))}` : null, fhi > hi && (top === undefined || hi < top) ? `to ${n(fhi)}` : null].filter((x) => x);
  return parts.length ? `${core} (fades out ${parts.join(" and ")}${unit})` : core;
}

/** What a "needs" entry asks for, in words. */
export function needsWords(n: string): string {
  const [k, v] = n.split(":");
  if (k === "authored") return `a ${v ?? "story"} the GM or a pack puts there`;
  if (k === "life") return "a life model on the stack";
  return n;
}

/** Find a class by id across the atlas. */
export function findClass(a: ClassAtlas, id: string): AtlasClass | null {
  for (const s of a.schemes) for (const c of s.classes) if (c.id === id) return c;
  return null;
}

/** The classes whose name, id, summary or example worlds mention every word typed, in atlas order. */
export function findClasses(a: ClassAtlas, query: string): AtlasClass[] {
  // Every word must be found; a plural finds its singular ("rings" finds "ringed").
  const words = query.toLowerCase().split(/\s+/).filter((w) => w).map((w) => (w.length > 3 && w.endsWith("s") ? w.slice(0, -1) : w));
  if (!words.length) return [];
  return a.schemes.flatMap((s) => s.classes).filter((c) => {
    const text = [c.id, c.names.plain, c.names.physics, c.names.story ?? "", c.summary, ...c.examples.filter((e) => e.in).map((e) => e.name)].join(" ").toLowerCase();
    return words.every((w) => text.includes(w));
  });
}
