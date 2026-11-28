// The parts palette (compound ships, CM13; design `handover/design/SSE4-compound-ships-design.md` sections 8A.4, 8A.5,
// 8B.2 and ruling 14A.5): the one part format every part uses, whether the base palette ships it, a store pack brings
// it or a user makes it in the part editor later (CM17). A drive from any pack clips in through the same format, with
// its shape rules as data, never as a special.
//
// A part is figures (what the physics reads, each with its source and mark), connection points (where it can clip,
// what size, what passes), a shape (drawn the same in 3D and as the plans' outline), a scaling law (how it stretches
// and widens), and the year it first flew. Nothing here is physics: stretching only rescales a part's own figures by
// the law the part names, and every figure the kernel sizes is marked so. The palette itself is pack data
// (`packs/base-parts`, built by scripts/build-base-parts.py).

/** What a part is for. A part that can exist on its own (a stage, a module with its own ports) is `alone`. */
export type PartKind = "stage" | "engine" | "tank" | "habitat" | "node" | "truss" | "power" | "solar" | "radiator" | "bay" | "pod" | "sensor" | "dish" | "shield" | "legs";

/** Where a figure comes from: read in a cited source, set for play, predicted from a trend (CM15), worked out here from
 * cited ones, or a stand-in until read. */
export type FigureMark = "cited" | "authored" | "predicted" | "derived" | "estimated";

export interface PartFigure {
  v: number;
  unit: string;
  src: string;
  mark: FigureMark;
  /** "unverified" until the citation run-through reads the source. */
  check?: "unverified";
  why?: string;
}

/** What may pass across a joint (the mount's `lines`, CM1). */
export type Line = "propellant" | "power" | "air" | "people";

/** Which kind of joint a connection point makes (the mount's places, design 4.2). `bay` is a bay's mouth. */
export type PointPlace = "front" | "behind" | "hull" | "bay" | "port";

export interface ConnectionPoint {
  id: string;
  place: PointPlace;
  /** Its size on the port size standard (pack `port_sizes`), metres across. */
  size_m: number;
  lines: Line[];
  /** Where on the part, in metres, in the layout frame (x from the stern, y to the side, z up). */
  at: [number, number, number];
  /** Which way it faces (a unit vector). Any direction: nothing is held to right angles (ruling 14A.3). */
  facing: [number, number, number];
}

/** One shape, drawn the same in 3D and in the plans. A lathe is a profile turned about x; a dish faces `facing`. */
export interface PartShape {
  hint: "lathe" | "panel" | "truss" | "dish" | "box";
  length_m: number;
  radius_m: number;
  /** Lathe: [share along, share of radius] from stern to nose. Missing is a plain cylinder. */
  profile?: [number, number][];
  /** Panel or box: its span across. */
  span_m?: number;
  /** How the builder draws it where its kind alone does not say (a capsule is a habitat, a docking port a node).
   * A picture hint only: no figure reads it. */
  look?: "capsule" | "port";
}

/** How a part resizes in place (ruling 14A.4: "you got the volume, that's all that matters"). `volume`: figures in
 * m3 and the contents follow length times radius squared, structure follows the area; `area`: output follows the
 * area; `fixed`: offered only in its listed sizes. */
export interface PartScale {
  law: "volume" | "area" | "fixed";
  stretch: boolean;
  widen: boolean;
  /** Figures that follow each measure. Anything not named stays as it is. */
  with_volume?: string[];
  with_area?: string[];
  /** The law's source; a structure scaled by area is marked estimated until a sizing block replaces it. */
  src: string;
}

/** A drive's need on the rest of the ship (ruling 14A.5), read by the builder, which warns and offers, never refuses. */
export interface ShapeRule {
  kind: "keep_away" | "shadow_cone" | "pusher_plate" | "radiator_area" | "clear_view";
  /** keep_away: metres the crew must be kept from it; shadow_cone: half angle in degrees; radiator_area: m2. */
  v: number;
  why: string;
  src: string;
}

export interface Part {
  id: string;
  /** Physics name only (14A.5): never a brand, a setting's coinage or a named screen ship. */
  name: string;
  kind: PartKind;
  /** Can exist on its own: becomes a construct with a mount when clipped, rather than a part of what it joins. */
  alone: boolean;
  figures: Record<string, PartFigure>;
  points: ConnectionPoint[];
  shape: PartShape;
  scale: PartScale;
  /** The year it first flew or was built at working scale (8A.5); missing for a generic primitive. */
  year?: number;
  year_src?: string;
  maker?: string;
  /** Substances it holds or burns, by the sheet's ids. */
  fuels?: string[];
  rules?: ShapeRule[];
  /** A part whose figures are a future tech advance's (base-tech `advances`), read at the setting's level, never copied. */
  advance?: string;
  /** The advance's home level on the ladder. */
  level?: number;
  /** The era a part with no first-flight year belongs to (a tech drive: when it first flew, or the first future era its
   * ladder level reaches), and why. A part with a year falls in the era its year does. */
  era?: string;
  era_src?: string;
  /** Figures still wanted from a source. */
  to_read?: string[];
  credit?: string;
}

export interface PortSize {
  size_m: number;
  name: string;
  src: string;
  check?: "unverified";
}

/** An era of spaceflight (Alex, 27 Nov 2026: "not by age but by eras... a few future eras", "they can vary in
 * length"): named by what changed, any length. Pack data, so a setting renames, moves or adds them. */
export interface Era {
  id: string;
  name: string;
  /** First and last year, inclusive; missing is open-ended. */
  from?: number;
  to?: number;
  /** The base-tech ladder level the era reaches. */
  level: number;
  /** Not flown yet: its parts are predicted or a tech advance's. */
  future?: boolean;
  about: string;
}

/** base-parts `palette`. */
export interface PartsPalette {
  /** Eras of spaceflight, in order. */
  eras?: Era[];
  port_sizes: PortSize[];
  parts: Part[];
  credits: string[];
}

const PAIRS: Record<PointPlace, PointPlace[]> = {
  front: ["behind"],
  behind: ["front"],
  hull: ["hull", "port"],
  port: ["port", "hull"],
  bay: ["behind", "front"],
};

/** Whether two connection points can clip together: matching kinds of joint (a front meets a behind, ports and hull
 * stations meet each other, a bay's mouth takes either end), the same size on the standard, so parts from different
 * packs fit (8B.3). Returns the lines both pass. */
export function clips(a: ConnectionPoint, b: ConnectionPoint, tol = 1e-6): { ok: boolean; lines: Line[]; why?: string } {
  if (!PAIRS[a.place].includes(b.place)) return { ok: false, lines: [], why: `a ${a.place} point does not meet a ${b.place} point` };
  if (Math.abs(a.size_m - b.size_m) > tol * Math.max(a.size_m, b.size_m, 1)) return { ok: false, lines: [], why: `sizes differ (${a.size_m} m and ${b.size_m} m): an adapter joins them` };
  return { ok: true, lines: a.lines.filter((l) => b.lines.includes(l)) };
}

/** The shape's volume: a lathe from its profile (frustum by frustum), a box or panel as length times span times its
 * radius doubled, a dish or truss none. */
export function shapeVolume(s: PartShape): number {
  if (s.hint === "lathe") {
    const prof = s.profile ?? [[0, 1], [1, 1]];
    let v = 0;
    for (let i = 1; i < prof.length; i++) {
      const [x0, a] = prof[i - 1]!;
      const [x1, b] = prof[i]!;
      const r0 = a * s.radius_m;
      const r1 = b * s.radius_m;
      v += (Math.PI / 3) * (x1 - x0) * s.length_m * (r0 * r0 + r0 * r1 + r1 * r1);
    }
    return v;
  }
  if (s.hint === "box" || s.hint === "panel") return s.length_m * (s.span_m ?? 2 * s.radius_m) * 2 * s.radius_m;
  return 0;
}

/** The shape's outer area (a lathe's side and ends; a panel's two faces). */
export function shapeArea(s: PartShape): number {
  if (s.hint === "lathe") {
    const prof = s.profile ?? [[0, 1], [1, 1]];
    let a = 0;
    for (let i = 1; i < prof.length; i++) {
      const r0 = prof[i - 1]![1] * s.radius_m;
      const r1 = prof[i]![1] * s.radius_m;
      const h = (prof[i]![0] - prof[i - 1]![0]) * s.length_m;
      a += Math.PI * (r0 + r1) * Math.hypot(h, r1 - r0);
    }
    const ends = prof[0]![1] ** 2 + prof[prof.length - 1]![1] ** 2;
    return a + Math.PI * s.radius_m * s.radius_m * ends;
  }
  if (s.hint === "panel" || s.hint === "dish") return 2 * s.length_m * (s.span_m ?? 2 * s.radius_m);
  return 2 * (s.length_m * (s.span_m ?? 2 * s.radius_m) + s.length_m * 2 * s.radius_m + (s.span_m ?? 2 * s.radius_m) * 2 * s.radius_m);
}

/** Stretch and widen a part (the mouse wheel and pinch of 14A.4). Its own figures follow by the law it names: those it
 * lists with the volume scale with the volume, those with the area with the area, and each rescaled figure is marked
 * `derived` from the original, with the law in `why`. A `fixed` part, or a measure it does not allow, is left as it
 * is and the reason given. Connection points move with the ends; their sizes stay on the standard. */
export function stretchPart(p: Part, to: { length_m?: number; radius_m?: number }): { part: Part; note?: string } {
  const length = p.scale.stretch ? to.length_m ?? p.shape.length_m : p.shape.length_m;
  const radius = p.scale.widen ? to.radius_m ?? p.shape.radius_m : p.shape.radius_m;
  const refused = (to.length_m !== undefined && !p.scale.stretch) || (to.radius_m !== undefined && !p.scale.widen);
  const note = refused ? `${p.name} comes in the sizes its pack offers; ${!p.scale.stretch && to.length_m !== undefined ? "it does not stretch" : "it does not widen"}.` : undefined;
  if (length === p.shape.length_m && radius === p.shape.radius_m) return note ? { part: p, note } : { part: p };
  const shape: PartShape = { ...p.shape, length_m: length, radius_m: radius, ...(p.shape.span_m !== undefined ? { span_m: (p.shape.span_m * radius) / p.shape.radius_m } : {}) };
  const kv = shapeVolume(shape) / Math.max(shapeVolume(p.shape), 1e-12);
  const ka = shapeArea(shape) / Math.max(shapeArea(p.shape), 1e-12);
  const figures: Record<string, PartFigure> = { ...p.figures };
  for (const [list, k, law] of [[p.scale.with_volume ?? [], kv, "volume"], [p.scale.with_area ?? [], ka, "area"]] as const)
    for (const id of list) {
      const f = p.figures[id];
      if (!f) continue;
      figures[id] = { v: f.v * k, unit: f.unit, src: p.scale.src, mark: "derived", why: `${id} follows the ${law} (${law === "volume" ? "length times radius squared" : "outer area"}) from ${p.name}'s own figure` };
    }
  const sx = length / p.shape.length_m;
  const sr = radius / p.shape.radius_m;
  const points = p.points.map((c) => ({ ...c, at: [c.at[0] * sx, c.at[1] * sr, c.at[2] * sr] as [number, number, number] }));
  return { part: { ...p, shape, figures, points }, ...(note ? { note } : {}) };
}

/** What a part's figures and shape disagree on (8B.2), in words: warnings, never refusals. A part with no figures is a
 * decoration and says so; a tank that cannot hold its stated volume; a port or hull point off the size standard (axial joints take their part's own diameter, and an adapter joins two that differ, as real stacks do). */
export function partWarnings(p: Part, sizes: readonly PortSize[]): string[] {
  const out: string[] = [];
  if (!Object.keys(p.figures).length && !p.advance) out.push(`${p.name} has no figures, so it is a decoration: the physics reads nothing from it.`);
  const vol = p.figures.volume_m3?.v;
  const room = shapeVolume(p.shape);
  if (vol !== undefined && room > 0 && vol > room * 1.001) out.push(`${p.name} is said to hold ${vol} m3 but its shape holds ${Math.round(room * 10) / 10} m3.`);
  for (const c of p.points) if ((c.place === "port" || c.place === "hull") && !sizes.some((s) => Math.abs(s.size_m - c.size_m) <= 1e-6 * s.size_m)) out.push(`${p.name}'s ${c.id} point (${c.size_m} m) is not on the port size standard, so only an adapter joins it.`);
  return out;
}

/** The era a part belongs to: its own `era` if it names one, else the era its first-flight year falls in. A generic
 * shape with neither belongs to none. */
export function eraOf(p: Part, eras: readonly Era[]): Era | undefined {
  if (p.era) return eras.find((e) => e.id === p.era);
  if (p.year === undefined) return undefined;
  return eras.find((e) => (e.from === undefined || p.year! >= e.from) && (e.to === undefined || p.year! <= e.to));
}

/** The palette offered in a year, grouped by era in order, generic shapes first. The picker reads exact years: a part
 * with a year is offered once it has flown; a part with only an era once that era has begun. */
export function paletteByEra(palette: PartsPalette, year: number): { era?: Era; label: string; parts: Part[] }[] {
  const eras = palette.eras ?? [];
  const from = (p: Part) => p.year ?? eraOf(p, eras)?.from ?? -Infinity;
  const offered = palette.parts.filter((p) => from(p) <= year);
  const out: { era?: Era; label: string; parts: Part[] }[] = [];
  const generic = offered.filter((p) => !eraOf(p, eras));
  if (generic.length) out.push({ label: "Shapes", parts: generic });
  for (const e of eras) {
    const ps = offered.filter((p) => eraOf(p, eras)?.id === e.id).sort((a, b) => from(a) - from(b) || a.name.localeCompare(b.name));
    if (ps.length) out.push({ era: e, label: e.name, parts: ps });
  }
  return out;
}

/** The palette by what parts are for (the mock-up's groups): crew and rooms, tanks, drives, power and heat, structure,
 * sensors. In each, shapes with no year first, then by first flight. Every part is listed whatever the year; the picker
 * fades the ones not flown yet. */
export const PART_USES: readonly { label: string; kinds: readonly PartKind[] }[] = [
  { label: "Crew and rooms", kinds: ["habitat", "node", "pod", "bay"] },
  { label: "Tanks and stages", kinds: ["tank", "stage"] },
  { label: "Drives", kinds: ["engine"] },
  { label: "Power and heat", kinds: ["power", "solar", "radiator"] },
  { label: "Structure", kinds: ["truss", "legs", "shield"] },
  { label: "Sensors and dishes", kinds: ["sensor", "dish"] },
];
export function paletteByUse(palette: PartsPalette): { label: string; parts: Part[] }[] {
  const eras = palette.eras ?? [];
  const from = (p: Part) => p.year ?? eraOf(p, eras)?.from ?? -Infinity;
  const out: { label: string; parts: Part[] }[] = [];
  for (const u of PART_USES) {
    const ps = palette.parts.filter((p) => u.kinds.includes(p.kind)).sort((a, b) => from(a) - from(b) || a.name.localeCompare(b.name));
    if (ps.length) out.push({ label: u.label, parts: ps });
  }
  const rest = palette.parts.filter((p) => !PART_USES.some((u) => u.kinds.includes(p.kind)));
  if (rest.length) out.push({ label: "Other", parts: rest });
  return out;
}

/** The palette as it stood in a year (the era control is CM15; this is its reading): parts that had flown by then,
 * and generic parts, which have no year. */
export function flownBy(parts: readonly Part[], year: number): Part[] {
  return parts.filter((p) => p.year === undefined || p.year <= year);
}
