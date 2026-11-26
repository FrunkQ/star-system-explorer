/**
 * The Formation screen's chart (formation design section 7, package F10, the reading half): distance from the star
 * across, time down the page, both on log scales. Each body is a track whose width follows its mass and which bends as
 * it migrates; tracks join at merging impacts and run off the edge at ejections. The gas disc is a tint that ends when
 * the gas leaves, its condensation lines lean inward as it cools, belts are hatched strips, events are diamonds and
 * the upheaval is a marked span.
 *
 * Everything is read from the formation run's own record, through the replay's reader (`replay.ts`), so the chart, the
 * replay and the system chart never disagree. Nothing here simulates, and moving the time slice recomputes nothing.
 * Pure: the same run gives the same chart.
 */
import type { FormationRan } from "./make.ts";
import { replayAt, replayModel, type ReplayBody, type ReplayModel } from "./replay.ts";
import { sayNumber, sayText } from "./figures.ts";
import { sheetUnitFactor } from "./sheet.ts";

/** A stretch of one track: from (t0, a0) to (t1, a1), drawn at width w (in "mass widths", see `massWidth`). */
export interface TrackSeg {
  t0: number;
  t1: number;
  a0: number;
  a1: number;
  m: number;
}
export interface ChartTrack {
  id: string;
  label: string;
  address?: string;
  giant: boolean;
  born: { t: number; a: number };
  segs: TrackSeg[];
  /** How the track ends: it runs on to the foot of the chart, joins another track, leaves at the right, or stops. */
  end: { how: "lasts"; t: number; a: number } | { how: "merged"; t: number; a: number; into: string; into_a: number } | { how: "ejected"; t: number; a: number } | { how: "shattered"; t: number; a: number };
  /** Its mass at the foot of the chart (or when it went), in Earth masses. */
  m_end: number;
}
export interface ChartEvent {
  t: number;
  a: number;
  kind: string;
  /** A few words for the chart, beside the diamond. Empty: a diamond with no words (the full text is its title). */
  short: string;
  text: string;
  ids: string[];
}
export interface ChartLine {
  /** Index into the run's lines: rock, carbon, water ice, CO₂ ice, CO ice. */
  i: number;
  words: string;
  pts: { t: number; a: number }[];
}
export interface ChartBelt {
  kind: string;
  words: string;
  a0: number;
  a1: number;
  from: number;
  sphere: boolean;
}
export interface ChartStage {
  n: number;
  name: string;
  from: number;
  to: number;
  text: string;
}
export interface InheritedRow {
  key: string;
  label: string;
  value: string;
  /** Said in full, for the row's title. */
  title?: string;
}
export interface FormationChart {
  /** The chart's span in time (Myr since the star lit) and distance (AU), on log scales. */
  t_top: number;
  t_bottom: number;
  a_in: number;
  a_out: number;
  ticks_t: { t: number; label: string }[];
  ticks_a: { a: number; label: string }[];
  /** The system's age now, in Myr (where the slice starts). */
  now: number;
  gas_until: number;
  lines: ChartLine[];
  tracks: ChartTrack[];
  events: ChartEvent[];
  belts: ChartBelt[];
  upheaval: { from: number; to: number } | null;
  stages: ChartStage[];
  inherited: InheritedRow[];
  worked_at: InheritedRow[];
  model: ReplayModel;
}

const MYR_S = sheetUnitFactor("myr");
const story = { depth: "curious" as const, about: false };
const lg = (x: number) => Math.log10(Math.max(x, 1e-12));
const decade = (x: number, up: boolean) => Math.pow(10, up ? Math.ceil(lg(x) - 1e-9) : Math.floor(lg(x) + 1e-9));

/** A time on the chart in words ("0.8 Myr", "41 Myr", "2.2 Gyr"). */
export function chartTime(t_myr: number): string {
  if (t_myr >= 1000) return `${sayNumber(t_myr / 1000, 2)} Gyr`;
  if (t_myr < 0.001) return `${sayNumber(t_myr * 1e6, 2)} yr`;
  return `${sayNumber(t_myr, 2)} Myr`;
}
/** A distance on the chart in words ("0.72 AU"). */
export function chartAu(a: number): string {
  return `${sayNumber(a, 2)} AU`;
}
/** A time said by the units engine for prose ("about 4 million years"). */
const ago = (t: number) => sayText(t * MYR_S, "time", undefined, story);

/**
 * A track's width for a mass in Earth masses: the cube root, so width follows size. A thousandth of an Earth is a
 * hairline, an Earth a pen line, a Jupiter the broadest stroke. In pixels at the chart's normal size.
 */
export function massWidth(m_earth: number): number {
  return Math.max(0.9, Math.min(8, 2.2 * Math.cbrt(Math.max(m_earth, 0))));
}

const SHORT: Record<string, (who: (id?: string) => string, ids: string[]) => string> = {
  "impact.merge-moon": (w, ids) => `impact · makes ${w(ids[1])}'s moon`,
  "impact.merge": (w, ids) => `${w(ids[0])} merges into ${w(ids[1])}`,
  "impact.hit-and-run": () => "grazing hit",
  "impact.erosion": () => "shattered into a belt",
  scatter: () => "flung out",
  ejected: () => "thrown out · now a rogue",
  "chain-breaks": () => "chain breaks",
};

/** The Formation chart of one run. `opts.names` maps a world's address to the name the campaign gives it. */
export function formationChart(run: FormationRan, opts: { names?: Record<string, string>; seed?: string } = {}): FormationChart {
  const names = opts.names ?? {};
  const m = replayModel(run);
  const r = run.run as unknown as {
    upheaval?: { break_myr?: number | null; settled_myr?: number | null };
    history: { inheritance: Record<string, number | string | undefined>; disc?: { t_disperse: number; params?: { mass0?: number; rc?: number } } };
    block: string;
  };
  const inh = r.history.inheritance;
  const now = Math.max(1, Number(inh.age_gyr ?? 4.6) * 1000);
  // From a little before the first embryo (at most 0.1 Myr, where the mockup starts), so growth has room.
  const firstBorn = Math.min(...m.bodies.map((b) => b.born_myr), Infinity);
  const t_top = decade(Math.max(1e-3, Math.min(0.1, firstBorn * 0.8)), false);
  const t_bottom = decade(Math.max(now * 1.6, 1000), true);
  const a_in = decade(m.inner_au, false);
  const a_out = decade(m.outer_au, true);

  const ticks_t: FormationChart["ticks_t"] = [];
  for (let e = Math.round(lg(t_top)); e <= Math.round(lg(t_bottom)); e++) ticks_t.push({ t: Math.pow(10, e), label: chartTime(Math.pow(10, e)) });
  const ticks_a: FormationChart["ticks_a"] = [];
  for (let e = Math.round(lg(a_in)); e <= Math.round(lg(a_out)); e++) ticks_a.push({ a: Math.pow(10, e), label: chartAu(Math.pow(10, e)) });

  // Names: the campaign's, else the run's ("World 2").
  const label = (b: ReplayBody) => (b.address && names[b.address]) || b.label;
  const byId = new Map(m.bodies.map((b) => [b.id, b]));
  const who = (id?: string) => {
    const b = id ? byId.get(id) : undefined;
    return b ? label(b) : "a body";
  };

  // Where each body is at time t, read through the replay (the same interpolation it plays).
  const cache = new Map<number, Map<string, { a: number; m: number }>>();
  const at = (t: number) => {
    let f = cache.get(t);
    if (!f) {
      f = new Map(replayAt(m, t).bodies.map((b) => [b.id, { a: b.a_au, m: b.m }]));
      cache.set(t, f);
    }
    return f;
  };
  const tracks: ChartTrack[] = m.bodies.map((b) => {
    const end = b.gone ? b.gone.t_myr : t_bottom;
    // Sample at every recorded moment, every mass step and every tenth of a decade between, so bends read as bends.
    const ts = new Set<number>([b.born_myr, ...b.keys.map((k) => k.t), ...b.mass.map((x) => x.t)].filter((t) => t >= b.born_myr && t <= end));
    for (let x = lg(b.born_myr); x < lg(end); x += 0.1) ts.add(Math.pow(10, x));
    ts.add(end);
    const times = [...ts].sort((x, y) => x - y);
    const pts = times.map((t) => {
      // At the moment it goes, read just before, while it is still there.
      const p = at(t >= end && b.gone ? t * (1 - 1e-9) : t).get(b.id);
      return { t, a: p?.a ?? b.keys[0]!.a, m: p?.m ?? 0 };
    });
    const segs: TrackSeg[] = [];
    for (let i = 1; i < pts.length; i++) segs.push({ t0: pts[i - 1]!.t, t1: pts[i]!.t, a0: pts[i - 1]!.a, a1: pts[i]!.a, m: pts[i - 1]!.m });
    const last = pts[pts.length - 1]!;
    let endOf: ChartTrack["end"];
    if (!b.gone) endOf = { how: "lasts", t: t_bottom, a: last.a };
    else if (b.gone.how === "merged" && b.gone.into) endOf = { how: "merged", t: b.gone.t_myr, a: last.a, into: b.gone.into, into_a: at(b.gone.t_myr * (1 - 1e-9)).get(b.gone.into)?.a ?? last.a };
    else endOf = { how: b.gone.how === "ejected" ? "ejected" : "shattered", t: b.gone.t_myr, a: last.a };
    return { id: b.id, label: label(b), ...(b.address ? { address: b.address } : {}), giant: b.giant, born: { t: b.born_myr, a: pts[0]!.a }, segs, end: endOf, m_end: last.m };
  });

  // Events with a place on the chart: those that involve a body. The rest (the star's own) are told in the stages.
  const events: ChartEvent[] = [];
  for (const e of m.events) {
    if (!e.ids.length || e.kind === "embryo-forms") continue;
    const p = at(e.t_myr * (1 - 1e-9)).get(e.ids[e.kind === "ejected" ? 0 : e.ids.length - 1]!) ?? at(e.t_myr * (1 - 1e-9)).get(e.ids[0]!);
    if (!p) continue;
    const short = SHORT[e.kind]?.(who, e.ids) ?? "";
    events.push({ t: e.t_myr, a: p.a, kind: e.kind, short, text: e.text, ids: e.ids });
  }

  // The condensation lines while the gas lasts.
  const LINE_WORDS = ["rock line", "carbon line", "water ice line", "CO₂ ice line", "CO ice line"];
  const nLines = m.disc.slices[0]?.lines.length ?? 0;
  const lines: ChartLine[] = [];
  for (let i = 0; i < nLines; i++) {
    const pts = m.disc.slices.filter((s) => s.t <= m.disc.t_disperse).map((s) => ({ t: s.t, a: s.lines[i]! }));
    if (pts.length) lines.push({ i, words: LINE_WORDS[i] ?? `line ${i + 1}`, pts });
  }

  const belts: ChartBelt[] = m.clouds.map((c) => ({ kind: c.kind, words: c.kind.replace(/-/g, " "), a0: c.a_min_au, a1: c.a_max_au, from: c.from_myr, sphere: c.shape === "sphere" }));

  const up = r.upheaval;
  const upheaval = up?.break_myr && up.settled_myr && up.settled_myr > up.break_myr ? { from: up.break_myr, to: up.settled_myr } : null;

  const byN = [...m.stages].sort((x, y) => x.from_myr - y.from_myr);
  const stages: ChartStage[] = byN
    .map((s, i) => ({ n: s.n, name: s.name, from: Math.max(t_top, s.from_myr), to: byN[i + 1]?.from_myr ?? t_bottom, text: s.text }))
    .sort((x, y) => x.n - y.n);

  return {
    t_top,
    t_bottom,
    a_in,
    a_out,
    ticks_t,
    ticks_a,
    now,
    gas_until: m.disc.t_disperse,
    lines,
    tracks,
    events,
    belts,
    upheaval,
    stages,
    inherited: inheritedRows(run, m, opts.seed),
    worked_at: workedAt(run),
    model: m,
  };
}

/** What the system inherited from its patch, in words, for the left panel. */
export function inheritedRows(run: FormationRan, m: ReplayModel = replayModel(run), seed?: string): InheritedRow[] {
  const h = run.run.history as unknown as { inheritance: Record<string, number | string | undefined>; stars: { mass: number }[]; disc?: { t_disperse: number; params?: { mass0?: number; rc?: number } } };
  const inh = h.inheritance;
  const n = (k: string) => (typeof inh[k] === "number" ? (inh[k] as number) : undefined);
  const rows: InheritedRow[] = [];
  const star = h.stars[0];
  const companions = Math.max(0, h.stars.length - 1);
  if (star) rows.push({ key: "star", label: "Star", value: `${sayNumber(star.mass, 2)} Sun · ${companions === 0 ? "single" : companions === 1 ? "binary" : `${companions + 1} stars`}`, title: `A star of ${sayNumber(star.mass, 3)} solar masses${companions ? ` with ${companions} companion${companions > 1 ? "s" : ""}` : ", on its own"}` });
  const feh = n("feh");
  if (feh !== undefined) rows.push({ key: "metals", label: "Metals", value: `${sayNumber(Math.pow(10, feh), 2)} × Sun`, title: `[Fe/H] ${sayNumber(feh, 2, { decimals: 2 })}: the share of elements heavier than helium, against the Sun's` });
  const co = n("c_o");
  if (co !== undefined) rows.push({ key: "c_o", label: "Carbon to oxygen", value: sayNumber(co, 2), title: "Carbon atoms per oxygen atom in the cloud core (the Sun's is about 0.55): it sets how much rock is carbon and how much water there is" });
  const mass0 = h.disc?.params?.mass0 ?? m.disc.mass0;
  const rc = h.disc?.params?.rc;
  if (mass0) rows.push({ key: "disc", label: "Disc", value: `${sayNumber(mass0, 2)} Sun${rc ? ` · ${sayNumber(rc, 2)} AU` : ""}`, title: `A disc of ${sayNumber(mass0, 3)} solar masses of gas and dust${rc ? `, most of it within about ${sayNumber(rc, 2)} AU` : ""}` });
  rows.push({ key: "gas", label: "Gas lasts", value: chartTime(m.disc.t_disperse), title: `The gas is gone after ${ago(m.disc.t_disperse)}: the deadline for giants` });
  const fuv = n("fuv_g0");
  const members = n("cluster_members");
  if (fuv !== undefined) rows.push({ key: "cluster", label: "Birth cluster", value: fuv < 30 ? "faint glare" : fuv < 300 ? "mild glare" : fuv < 3000 ? "strong glare" : "fierce glare", title: `${members ? `About ${sayNumber(members, 2)} stars born together; ` : ""}ultraviolet from its brightest stars of ${sayNumber(fuv, 2)} G₀, which wears the disc away from outside` });
  const al = n("al26_rel");
  if (al !== undefined) rows.push({ key: "al26", label: "Aluminium-26", value: al < 0.3 ? "low" : al < 1.5 ? "about the Sun's" : "high", title: `${sayNumber(al, 2)} times the early Solar System's share: short-lived heat that melts young bodies and dries them out` });
  if (seed) rows.push({ key: "seed", label: "Seed", value: seed });
  return rows;
}

/** How each part was worked out (the level in use per stage, formation design section 4). */
export function workedAt(run: FormationRan): InheritedRow[] {
  const block = String((run.run as { block?: string }).block ?? "modern");
  const growth = /classic/i.test(block) ? "Classic Accrete" : "Standard";
  return [
    { key: "growth", label: "Growth, giants", value: growth, title: /classic/i.test(block) ? "SSE 3's Accrete, ported, for growth and giants" : "Band-by-band growth by pebbles, gas capture and migration, stepped in time" },
    { key: "violent", label: "Violent phase", value: "drawn", title: "The break date and each event of the cascade are drawn from odds calibrated on published orbit studies, keyed by the seed" },
    { key: "windows", label: "Orbit windows", value: "not here", title: "No stretch of this history was worked out by direct orbit integration: that is the Detailed level, still to come" },
  ];
}

/** The stage a time falls in. */
export function stageAt(c: Pick<FormationChart, "stages">, t: number): ChartStage {
  const s = [...c.stages].sort((x, y) => x.from - y.from);
  return [...s].reverse().find((x) => x.from <= t * (1 + 1e-9)) ?? s[0]!;
}

/** Where a time and a distance fall on a chart of the given size: log scales both ways. */
export function chartXY(c: Pick<FormationChart, "t_top" | "t_bottom" | "a_in" | "a_out">, w: number, h: number) {
  const x = (a: number) => ((lg(a) - lg(c.a_in)) / (lg(c.a_out) - lg(c.a_in))) * w;
  const y = (t: number) => ((lg(t) - lg(c.t_top)) / (lg(c.t_bottom) - lg(c.t_top))) * h;
  const t = (py: number) => Math.pow(10, lg(c.t_top) + (Math.max(0, Math.min(h, py)) / h) * (lg(c.t_bottom) - lg(c.t_top)));
  return { x, y, t };
}


