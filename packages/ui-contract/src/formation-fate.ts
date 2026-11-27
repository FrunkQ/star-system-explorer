/**
 * The star's end on the Formation screen (formation design section 2, stage 9's close; package F10c): the run's
 * surviving worlds handed to the ageing model (kernel `look_ahead`, sse-time) and aged to the star's end and a little
 * past it, on a branch that is thrown away at once. The star swells, sheds mass so the orbits widen, swallows the
 * worlds it reaches, and leaves its remnant; after that little happens, so the chart ends there.
 *
 * Nothing here simulates: `fatePlan` builds the ageing model's system from the run's own record, and `chartFate` and
 * `withFate` read the look-ahead's answer onto the chart. Pure.
 */
import type { FormationRan } from "./make.ts";
import type { ChartEvent, ChartTrack, FormationChart, TrackSeg } from "./formation-chart.ts";
import { chartTime } from "./formation-chart.ts";
import type { LookedAhead, PlanetSpec, SystemSpec } from "./fate.ts";
import { sayNumber } from "./figures.ts";

/**
 * How far ahead the screen looks: 100 billion years, about seven times the universe's age. A star that lives longer
 * (every red dwarf) is still shining when every star now alive but the faintest has gone out, so its end is said in
 * words, not run: the ageing model's giant tables are fitted on stars that do end, and a red dwarf never swells.
 */
export const FATE_LIMIT_MYR = 100_000;
/** Look a little past the end, so the remnant is seen cooling and the survivors' widened orbits settle. */
const PAST_END = 1.03;
/** One solar radius in AU. */
const RSUN_AU = 0.00465047;

export type FatePlan =
  | { kind: "ahead"; spec: SystemSpec; target_myr: number; end_myr: number; remnant: string; assumed: string[] }
  | { kind: "outlives"; end_myr: number; words: string }
  | { kind: "none"; words: string };

interface RunStar {
  address: string;
  mass: number;
  life?: { t_end?: number; t_tms?: number; remnant?: string; z?: number };
}
const REMNANT: Record<string, string> = { "white-dwarf": "white dwarf", "neutron-star": "neutron star", "black-hole": "black hole" };

/** What to ask the ageing model for, from the run's own record and its chart. */
export function fatePlan(run: FormationRan, c: FormationChart): FatePlan {
  const h = run.run.history as unknown as { stars: RunStar[]; inheritance: { z?: number } };
  const star = h.stars[0];
  const end = star?.life?.t_end;
  if (!star || !end || !(end > 0)) return { kind: "none", words: "The record carries no life line for the star, so its end cannot be shown." };
  const remnant = REMNANT[star.life?.remnant ?? ""] ?? (star.life?.remnant ?? "remnant").replace(/-/g, " ");
  if (end > FATE_LIMIT_MYR)
    return {
      kind: "outlives",
      end_myr: end,
      words: `The star burns for about ${sayNumber(end / 1e6, 2)} trillion years, far longer than the universe has yet existed. Nothing ends here within the chart: its worlds keep these orbits while it slowly brightens.`,
    };
  const ws = new Map((run.run as unknown as { upheaval?: { worlds?: { id: string; e?: number }[] } }).upheaval?.worlds?.map((w) => [w.id, w]) ?? []);
  const used = new Set<string>();
  const planets: PlanetSpec[] = [];
  for (const t of c.tracks) {
    if (t.end.how !== "lasts" || !t.address) continue;
    let n = t.label;
    while (used.has(n)) n = `${n}′`;
    used.add(n);
    planets.push({ name: n, address: t.address, mass_me: t.m_end, a_au: t.end.a, e: ws.get(t.id)?.e ?? 0, i_deg: 0, node_deg: 0, varpi_deg: 0, mean_longitude_deg: 0 });
  }
  const assumed = ["orbits coplanar, periapses aligned"];
  if (h.stars.length > 1) assumed.push("the companion stars left out: only the primary ages");
  // The ageing model names the star in its reasons by the system's name: "the star" reads right in every one.
  const spec: SystemSpec = { name: "the star", system: systemOf(star.address), star_address: star.address, star_mass: star.mass, z: star.life?.z ?? h.inheritance.z ?? 0.02, age_myr: c.now, birth_year: 0, planets, sources: "the system's formation record" };
  return { kind: "ahead", spec, target_myr: Math.max(c.now * 1.01, end * PAST_END), end_myr: end, remnant, assumed };
}
const systemOf = (a: string) => a.replace(/\/star:[^/]*$/, "");
const upper = (x: string) => x.charAt(0).toUpperCase() + x.slice(1);
const andList = (xs: string[]) => (xs.length < 2 ? xs.join("") : `${xs.slice(0, -1).join(", ")} and ${xs[xs.length - 1]}`);

/** One world through the star's end. */
export interface FateWorld {
  id: string;
  label: string;
  m: number;
  pts: { t: number; a: number }[];
  /** When the star's surface reached it. */
  engulfed?: number;
}
export interface FateMark {
  t: number;
  kind: string;
  words: string;
  why: string;
  ids: string[];
}
export interface ChartFate {
  /** Star ages in Myr: hydrogen runs out, the giant begins, it sheds its envelope, and the look ends. */
  ms_end: number;
  giant: number | null;
  end: number;
  until: number;
  remnant: string;
  /** Past this age the look-ahead is one plausible future. */
  plausible_after: number;
  star: { t: number; r_au: number; phase: string; l: number; m: number }[];
  worlds: FateWorld[];
  marks: FateMark[];
  /** The ending, in words. */
  text: string;
  assumed: string[];
}

const MARK: Record<string, (n: string) => string> = {
  leaves_main_sequence: () => "hydrogen runs out in the core",
  red_giant: () => "swells into a red giant",
  engulfed: (n) => `${n} swallowed`,
  planetary_nebula: () => "sheds its outer layers",
  supernova: () => "explodes",
  // Any other change of phase (a Wolf-Rayet star, a neutron star, a black hole): the time model says what it becomes.
  phase: () => "",
};

/** The look-ahead's answer read onto the chart's tracks. */
export function chartFate(c: FormationChart, plan: Extract<FatePlan, { kind: "ahead" }>, la: LookedAhead): ChartFate {
  const byName = new Map(plan.spec.planets.map((p) => [p.name, p.address]));
  const byAddr = new Map(c.tracks.filter((t) => t.address).map((t) => [t.address!, t]));
  const ev = (k: string) => la.events.find((e) => e.kind === k)?.age_myr ?? null;
  const ms_end = ev("leaves_main_sequence") ?? plan.end_myr;
  const giant = ev("red_giant");
  const end = ev("planetary_nebula") ?? ev("supernova") ?? plan.end_myr;
  const snaps = la.snapshots.filter((s) => s.age_myr >= c.now);
  const engulfedAt = new Map<string, number>();
  for (const e of la.events) if (e.kind === "engulfed") for (const a of e.subjects) if (byAddr.has(a) && !engulfedAt.has(a)) engulfedAt.set(a, e.age_myr);
  const worlds: FateWorld[] = [];
  for (const p of plan.spec.planets) {
    const t = byAddr.get(p.address);
    if (!t) continue;
    const gone = engulfedAt.get(p.address);
    const pts: FateWorld["pts"] = [];
    for (const s of snaps) {
      if (gone !== undefined && s.age_myr > gone) break;
      const q = s.planets.find((x) => byName.get(x.name) === p.address);
      if (q) pts.push({ t: s.age_myr, a: q.a_au });
    }
    if (gone !== undefined) pts.push({ t: gone, a: pts[pts.length - 1]?.a ?? p.a_au });
    worlds.push({ id: t.id, label: t.label, m: t.m_end, pts, ...(gone !== undefined ? { engulfed: gone } : {}) });
  }
  const label = (a: string) => byAddr.get(a)?.label ?? a.split("/").pop()!;
  const marks: FateMark[] = [];
  for (const e of la.events) {
    const f = MARK[e.kind];
    if (!f) continue;
    const ids = e.subjects.filter((a) => byAddr.has(a)).map((a) => byAddr.get(a)!.id);
    const who = e.subjects.filter((a) => byAddr.has(a)).map(label).join(", ");
    marks.push({ t: e.age_myr, kind: e.kind, words: f(who) || e.why.replace(/^the star is now an? /, "becomes a "), why: upper(e.why), ids });
  }
  const star = snaps.map((s) => ({ t: s.age_myr, r_au: s.star.radius_rsun * RSUN_AU, phase: s.star.phase, l: s.star.luminosity, m: s.star.mass }));
  const lost = worlds.filter((w) => w.engulfed !== undefined);
  const kept = worlds.filter((w) => w.engulfed === undefined);
  const m0 = star[0]?.m ?? plan.spec.star_mass;
  const m1 = star[star.length - 1]?.m ?? m0;
  const peak = Math.max(0, ...star.map((s) => s.r_au));
  const text = [
    `Hydrogen runs out in the star's core at ${chartTime(ms_end)}.`,
    // A star may skip a span (a red dwarf has no giant phase): say only what it does.
    giant !== null || peak > 0.01 ? `It swells${giant !== null ? " into a giant" : ""}, reaching out to ${sayNumber(peak, 2)} AU at its largest,` : "It never swells into a giant,",
    lost.length ? `swallowing ${lost.length === worlds.length ? "every world" : andList(lost.map((w) => w.label))}.` : "and reaches none of its worlds.",
    `It sheds ${sayNumber(Math.round((1 - m1 / m0) * 100), 2)}% of its mass and leaves a ${plan.remnant} by ${chartTime(end)}.`,
    kept.length ? `With less to hold them, the orbits that are left widen by about ${sayNumber(m0 / m1, 2)} times. Little happens after that: the remnant cools and fades.` : "Nothing is left in orbit. The remnant cools and fades.",
  ].join(" ");
  return { ms_end, giant, end, until: la.target_myr, remnant: plan.remnant, plausible_after: la.horizon_myr, star, worlds, marks, text, assumed: plan.assumed };
}

/** The chart run on to the star's end: tracks follow the widened orbits, swallowed worlds stop at the star's surface. */
export function withFate(c: FormationChart, f: ChartFate): FormationChart & { fate: ChartFate } {
  const lg = Math.log10;
  const t_bottom = Math.pow(10, Math.ceil(lg(f.until) - 1e-9));
  const ticks_t: FormationChart["ticks_t"] = [];
  for (let e = Math.round(lg(c.t_top)); e <= Math.round(lg(t_bottom)); e++) ticks_t.push({ t: Math.pow(10, e), label: chartTime(Math.pow(10, e)) });
  const fw = new Map(f.worlds.map((w) => [w.id, w]));
  const tracks: ChartTrack[] = c.tracks.map((t) => {
    const w = fw.get(t.id);
    if (t.end.how !== "lasts" || !w || !w.pts.length) return t.end.how === "lasts" ? { ...t, end: { ...t.end, t: t_bottom } } : t;
    const segs: TrackSeg[] = [];
    for (const g of t.segs) {
      if (g.t0 >= c.now) break;
      segs.push(g.t1 <= c.now ? g : { ...g, t1: c.now, a1: g.a0 + (g.a1 - g.a0) * ((lg(c.now) - lg(g.t0)) / (lg(g.t1) - lg(g.t0))) });
    }
    let prev = { t: c.now, a: segs[segs.length - 1]?.a1 ?? w.pts[0]!.a };
    for (const p of w.pts) {
      if (p.t <= prev.t) continue;
      segs.push({ t0: prev.t, t1: p.t, a0: prev.a, a1: p.a, m: t.m_end });
      prev = p;
    }
    if (w.engulfed !== undefined) return { ...t, segs, end: { how: "engulfed" as const, t: w.engulfed, a: prev.a } };
    if (prev.t < t_bottom) segs.push({ t0: prev.t, t1: t_bottom, a0: prev.a, a1: prev.a, m: t.m_end });
    return { ...t, segs, end: { how: "lasts" as const, t: t_bottom, a: prev.a } };
  });
  const at = (id: string, t: number) => {
    const w = fw.get(id);
    return w?.pts.filter((p) => p.t <= t).pop()?.a ?? w?.pts[0]?.a;
  };
  const events: ChartEvent[] = [...c.events];
  for (const m of f.marks)
    if (m.kind === "engulfed") for (const id of m.ids) events.push({ t: m.t, a: at(id, m.t) ?? c.a_in, kind: "engulfed", short: "", text: `${m.words}: ${m.why}`, ids: [id] });
  const stages = c.stages.map((s) => (s.to >= c.t_bottom * (1 - 1e-9) ? { ...s, to: t_bottom } : s));
  return { ...c, t_bottom, ticks_t, tracks, events, stages, fate: f };
}

/** Where a world is at the star's end: its orbit when the look ends, or when it was swallowed. */
export function fateLine(f: ChartFate, id: string): { t: number; text: string } | null {
  const w = f.worlds.find((x) => x.id === id);
  if (!w || !w.pts.length) return null;
  if (w.engulfed !== undefined) return { t: w.engulfed, text: `swallowed by the swelling star at ${sayNumber(w.pts[w.pts.length - 1]!.a, 2)} AU` };
  const a0 = w.pts[0]!.a;
  const a1 = w.pts[w.pts.length - 1]!.a;
  return { t: f.end, text: `the star is a ${f.remnant}; its orbit has widened from ${sayNumber(a0, 2)} to ${sayNumber(a1, 2)} AU` };
}
