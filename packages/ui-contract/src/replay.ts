/**
 * The formation replay (for the system-generation video): a grown system's
 * nine stages played over time, read entirely from the formation run's own
 * record. Nothing here simulates: the disc and its ice lines come from the
 * history slices, growth from each embryo's ledger of claims, the violent
 * phase from the upheaval events, the leftovers from F5's clouds and moons.
 * Between recorded moments a body's orbit is interpolated in log time, and
 * the view says so.
 */
import type { FormationRan, LeftoverCloud } from "./make.ts";
import { cloudShape } from "./make.ts";

type Solid = number[];
interface LedgerEntry { kind: string; t_myr?: number; solid?: Solid; gas?: number; ledger?: { entries: LedgerEntry[] } }
interface GrowthBody { id: string; a_au: number; e: number; born_au: number; born_band: string; born_myr: number; gas: number; solid: Solid; opened_gap: boolean; ledger?: { entries: LedgerEntry[] } }
interface GrowthEvent { type: string; t_myr: number; id?: string; a_au?: number; band?: string; inner?: string; p?: number; q?: number }
interface UpheavalWorld { id: string; a_au: number; e: number }
interface UpheavalEvent { type: string; t_myr: number; ids?: string[]; thrown?: string; kept?: string; to_au?: number; e?: number; impactor?: string; target?: string; outcome?: string; id?: string; by?: string }
interface UpheavalBelt { id: string; a_au: number; origin: string }
interface UpheavalMoon { parent: string; t_myr: number }
interface DiscSlice { t_myr: number; disc?: { lines: number[]; mass: number; rc: number } | null; stars: { luminosity: number; mass: number; phase: string }[] }
/** The parts of the run the replay reads, beyond what `FormationRan` already types. */
interface RunMore {
  growth: { bodies?: GrowthBody[]; events: GrowthEvent[]; end_myr?: number; inner_edge_au?: number };
  upheaval?: { events: UpheavalEvent[]; worlds: UpheavalWorld[]; belts: UpheavalBelt[]; moons: UpheavalMoon[]; break_myr?: number | null; settled_myr?: number | null };
  protoplanets?: { address: string; a_au: number; mass: number }[];
  leftovers?: { clouds: LeftoverCloud[]; moons?: { parent: string; diameter_km: number; origin: string }[] };
  history: Omit<FormationRan["run"]["history"], "slices"> & { disc?: { bands: { key: string }[]; t_disperse: number; params?: { mass0: number } }; slices: DiscSlice[] };
}

export interface ReplayStage {
  n: number;
  name: string;
  from_myr: number;
  /** What this system did in the stage, in a sentence or two, from its own run. */
  text: string;
}
export interface ReplayEvent {
  t_myr: number;
  kind: string;
  text: string;
  /** Bodies it involves, by replay id. */
  ids: string[];
}
interface Key { t: number; a: number; e: number; jump?: boolean }
export interface ReplayBody {
  id: string;
  label: string;
  born_myr: number;
  keys: Key[];
  /** Cumulative mass in M⊕ at recorded times. */
  mass: { t: number; m: number }[];
  gone?: { t_myr: number; how: "ejected" | "merged" | "shattered"; into?: string };
  /** The address it ends with, when it survives. */
  address?: string;
  giant: boolean;
  angle_deg: number;
}
export interface ReplayCloud {
  kind: string;
  a_min_au: number;
  a_max_au: number;
  shape: "disc" | "sphere";
  from_myr: number;
}
export interface ReplayModel {
  t0: number;
  t1: number;
  star: { mass: number; companions: number };
  disc: { t_disperse: number; bands: string[]; mass0: number; slices: { t: number; lines: number[]; mass: number; rc: number }[] };
  lum: { t: number; l: number }[];
  bodies: ReplayBody[];
  events: ReplayEvent[];
  stages: ReplayStage[];
  clouds: ReplayCloud[];
  moons: { parent: string; from_myr: number; diameter_km: number }[];
  /** Times at which playback lingers: every event, every stage start, and steps in between. */
  beats: number[];
  inner_au: number;
  outer_au: number;
}

export interface ReplayFrame {
  t: number;
  stage: ReplayStage;
  luminosity: number;
  disc: { lines: number[]; opacity: number; outer_au: number; inner_au: number } | null;
  bodies: { id: string; label: string; a_au: number; e: number; m: number; angle_deg: number; giant: boolean }[];
  clouds: ReplayCloud[];
  moons: { parent: string; diameter_km: number }[];
  /** The latest event at or before t, and whether it is still "fresh" (within the last beat). */
  last: ReplayEvent | null;
  fresh: ReplayEvent[];
}

const sum = (s: Solid | undefined) => (s ?? []).reduce((a, b) => a + b, 0);
const lg = (t: number) => Math.log10(Math.max(t, 1e-4));
const dedupe = (xs: number[]) => [...new Set(xs.map((x) => Number(x.toPrecision(9))))].sort((a, b) => a - b);

function claims(entries: LedgerEntry[] | undefined, out: { t: number; u: number }[] = []): { t: number; u: number }[] {
  for (const e of entries ?? []) {
    if (e.kind === "claim" && e.t_myr !== undefined) out.push({ t: e.t_myr, u: sum(e.solid) + (e.gas ?? 0) });
    if (e.ledger) claims(e.ledger.entries, out);
  }
  return out;
}

const BAND_WORDS: Record<string, string> = { vapour: "too hot for solids", rock: "rock", carbon: "rock and carbon", "water-ice": "water ice", "co2-ice": "CO₂ ice", "co-ice": "CO ice" };
export function bandWords(key: string): string {
  return BAND_WORDS[key] ?? key.replace(/-/g, " ");
}

const ago = (t: number) => (t < 1 ? `${Math.round(t * 1000).toLocaleString("en-GB")} thousand years` : t < 1000 ? `${Number(t.toPrecision(2))} million years` : `${Number((t / 1000).toPrecision(2))} billion years`);
const au = (a: number) => `${Number(a.toPrecision(2))} AU`;

/** The replay of one formation run. Pure: the same run gives the same replay. */
export function replayModel(run: FormationRan): ReplayModel {
  const r = run.run as unknown as Omit<FormationRan["run"], "history"> & RunMore;
  const h = r.history;
  const up = r.upheaval;
  const growthEnd = r.growth.end_myr ?? h.disc?.t_disperse ?? 3;
  const tDisperse = h.disc?.t_disperse ?? growthEnd;
  const settled = up?.settled_myr ?? growthEnd;
  const survivors = up?.worlds ?? [];
  const proto = r.protoplanets ?? [];
  const surviving = new Map(survivors.map((w, i) => [w.id, { w, i, address: proto[i]?.address, mass: proto[i]?.mass }]));
  const growth = r.growth.bodies ?? [];

  // Mass units: the ledger counts in the run's own units; the survivors' final masses (M⊕) fix the scale.
  const units = new Map(growth.map((b) => [b.id, sum(b.solid) + b.gas]));
  const events = up?.events ?? [];
  const merged = new Map<string, string>(); // impactor -> target
  for (const e of events) if (e.type === "impact" && (e.outcome === "merge" || e.outcome === "merge-moon") && e.impactor && e.target) merged.set(e.impactor, e.target);
  const finalUnits = (id: string, seen = new Set<string>()): number => {
    if (seen.has(id)) return 0;
    seen.add(id);
    let u = units.get(id) ?? 0;
    for (const [imp, tgt] of merged) if (tgt === id) u += finalUnits(imp, seen);
    return u;
  };
  let uSum = 0;
  let mSum = 0;
  for (const [id, s] of surviving) if (s.mass) ((uSum += finalUnits(id)), (mSum += s.mass));
  const k = uSum > 0 ? mSum / uSum : 1e-6;

  const name = (id: string) => {
    const s = surviving.get(id);
    return s ? `World ${s.i + 1}` : `Embryo ${id.replace(/^embryo-0?/, "")}`;
  };
  const golden = 137.50776;
  const bodies: ReplayBody[] = growth.map((b, i) => {
    const s = surviving.get(b.id);
    const keys: Key[] = [
      { t: b.born_myr, a: b.born_au, e: 0 },
      { t: growthEnd, a: b.a_au, e: b.e },
    ];
    let a = b.a_au;
    for (const e of events) {
      if (e.type === "scatter" && e.thrown === b.id && e.to_au) {
        keys.push({ t: e.t_myr, a, e: 0, jump: false }, { t: e.t_myr * (1 + 1e-9), a: e.to_au, e: e.e ?? 0.3, jump: true });
        a = e.to_au;
      }
    }
    if (s) keys.push({ t: Math.max(settled, keys[keys.length - 1]!.t), a: s.w.a_au, e: s.w.e });
    keys.sort((x, y) => x.t - y.t);
    // Mass over time: its own claims, then whatever merged into it, at the impact.
    const own = claims(b.ledger?.entries).sort((x, y) => x.t - y.t);
    // The ledger gives the shape of the growth; the body's own total at the end of growth gives its size.
    const total = own.reduce((a, c) => a + c.u, 0);
    const scale = total > 0 ? (units.get(b.id) ?? total) / total : 1;
    const mass: { t: number; m: number }[] = [];
    let u = 0;
    for (const c of own) ((u += c.u * scale), mass.push({ t: c.t, m: u * k }));
    if (!mass.length) mass.push({ t: b.born_myr, m: (units.get(b.id) ?? 0) * k });
    for (const e of events)
      if (e.type === "impact" && e.target === b.id && (e.outcome === "merge" || e.outcome === "merge-moon") && merged.get(e.impactor ?? "") === b.id) {
        u += finalUnits(e.impactor!);
        mass.push({ t: e.t_myr, m: u * k });
      }
    const ej = events.find((e) => e.type === "ejected" && e.id === b.id);
    const into = merged.get(b.id);
    const mergedAt = into ? events.find((e) => e.type === "impact" && e.impactor === b.id && merged.get(b.id) === e.target && (e.outcome === "merge" || e.outcome === "merge-moon")) : undefined;
    // Lost some other way (shattered into a debris belt): it goes at the last event it takes part in.
    const lastSeen = !s && !ej && !mergedAt ? [...events].reverse().find((e) => [e.impactor, e.target, e.thrown, e.kept, e.id, e.by].includes(b.id)) : undefined;
    return {
      id: b.id,
      label: name(b.id),
      born_myr: b.born_myr,
      keys,
      mass,
      ...(ej ? { gone: { t_myr: ej.t_myr, how: "ejected" as const } } : mergedAt ? { gone: { t_myr: mergedAt.t_myr, how: "merged" as const, into } } : !s ? { gone: { t_myr: lastSeen?.t_myr ?? settled, how: "shattered" as const } } : {}),
      ...(s?.address ? { address: s.address } : {}),
      giant: b.gas > sum(b.solid) || b.opened_gap,
      angle_deg: (i * golden) % 360,
    };
  });
  const byId = new Map(bodies.map((b) => [b.id, b]));
  const lbl = (id?: string) => (id ? (byId.get(id)?.label ?? id) : "a body");

  // Events in words.
  const ev: ReplayEvent[] = [];
  for (const e of h.events) if (e.text) ev.push({ t_myr: e.t_myr, kind: e.kind ?? "history", text: e.text.charAt(0).toUpperCase() + e.text.slice(1), ids: [] });
  for (const e of r.growth.events) {
    if (e.type === "embryo-forms" && e.id) ev.push({ t_myr: e.t_myr, kind: e.type, text: `An embryo gathers at ${au(e.a_au ?? 0)}, in the ${bandWords(e.band ?? "")} band: it will become ${lbl(e.id)}.`, ids: [e.id] });
    if (e.type === "captured-in-resonance" && e.id && e.inner) ev.push({ t_myr: e.t_myr, kind: e.type, text: `${lbl(e.inner)} and ${lbl(e.id)} lock into a ${e.p}:${e.q} resonance as they drift through the gas.`, ids: [e.inner, e.id] });
  }
  for (const e of events) {
    const t = e.t_myr;
    if (e.type === "chain-breaks") ev.push({ t_myr: t, kind: e.type, text: `With the gas gone, the resonant chain of ${(e.ids ?? []).map(lbl).join(", ")} breaks.`, ids: e.ids ?? [] });
    else if (e.type === "scatter") ev.push({ t_myr: t, kind: e.type, text: `${lbl(e.kept)} flings ${lbl(e.thrown)} out to ${au(e.to_au ?? 0)} on a stretched orbit (e ${(e.e ?? 0).toFixed(2)}).`, ids: [e.thrown!, e.kept!].filter(Boolean) });
    else if (e.type === "ejected") ev.push({ t_myr: t, kind: e.type, text: `${lbl(e.by)} throws ${lbl(e.id)} out of the system for good: it becomes a rogue planet.`, ids: [e.id!, e.by!].filter(Boolean) });
    else if (e.type === "impact") {
      const words: Record<string, string> = {
        merge: `${lbl(e.impactor)} hits ${lbl(e.target)} and they merge.`,
        "merge-moon": `Giant impact: ${lbl(e.impactor)} hits ${lbl(e.target)}; they merge and the debris forms a large moon.`,
        "hit-and-run": `${lbl(e.impactor)} grazes ${lbl(e.target)} and flies on.`,
        erosion: `${lbl(e.impactor)} strikes ${lbl(e.target)} and is shattered; the pieces spread into a debris belt.`,
      };
      ev.push({ t_myr: t, kind: `impact.${e.outcome}`, text: words[e.outcome ?? ""] ?? `${lbl(e.impactor)} strikes ${lbl(e.target)}.`, ids: [e.impactor!, e.target!].filter(Boolean) });
    } else if (e.type === "settled") ev.push({ t_myr: t, kind: e.type, text: "The orbits settle into the lasting arrangement.", ids: [] });
  }
  for (const e of r.events) if (e.text) ev.push({ t_myr: e.t_myr, kind: e.kind ?? "life", text: `${e.subject ? `${lbl(addressToId(e.subject, bodies))}: ` : ""}${e.text}`, ids: [] });
  ev.sort((a, b) => a.t_myr - b.t_myr);

  // The disc over time.
  const slices = h.slices
    .filter((s) => s.disc)
    .map((s) => ({ t: s.t_myr, lines: s.disc!.lines, mass: s.disc!.mass, rc: s.disc!.rc }));
  const mass0 = slices[0]?.mass ?? 0.01;
  const lum = h.slices.map((s) => ({ t: s.t_myr, l: s.stars[0]?.luminosity ?? 1 }));

  // Leftovers: debris belts appear at the impact that made them, the rest when the system settles.
  const beltTime = new Map<string, number>();
  for (const b of up?.belts ?? []) {
    const src = b.origin.replace(/^impact:/, "");
    const hit = events.find((e) => e.type === "impact" && (e.impactor === src || e.target === src));
    beltTime.set(b.id, hit?.t_myr ?? settled);
  }
  const seen = new Set<string>();
  const clouds: ReplayCloud[] = [];
  for (const c of r.leftovers?.clouds ?? []) {
    if (c.around || !(c.a_max_au > c.a_min_au)) continue;
    const key = `${c.kind}|${c.a_min_au.toPrecision(4)}|${c.a_max_au.toPrecision(4)}`;
    if (seen.has(key)) continue;
    seen.add(key);
    const src = (c as LeftoverCloud & { source?: string }).source;
    clouds.push({ kind: c.kind, a_min_au: c.a_min_au, a_max_au: c.a_max_au, shape: cloudShape(c), from_myr: (src ? beltTime.get(src) : undefined) ?? settled });
  }
  const lm = r.leftovers?.moons ?? [];
  const moons = (up?.moons ?? []).map((m, i) => ({ parent: m.parent, from_myr: m.t_myr, diameter_km: lm[i]?.diameter_km ?? 1000 }));

  // Stages, each told from this run.
  const inh = h.inheritance;
  const star0 = h.stars[0];
  const companions = Math.max(0, h.stars.length - 1);
  const firstEmbryo = Math.min(...r.growth.events.filter((e) => e.type === "embryo-forms").map((e) => e.t_myr), growthEnd);
  const giants = bodies.filter((b) => b.giant);
  const violentFrom = events.find((e) => e.type !== "settled")?.t_myr;
  // lines: rock, carbon, water ice, CO₂ ice, CO ice (the band edges, inside out).
  const water = slices[0]?.lines[2];
  const waterEnd = slices[slices.length - 1]?.lines[2];
  const lastMagma = Math.max(0, ...r.worlds.map((w) => (w as unknown as { magma_ocean_end_myr?: number }).magma_ocean_end_myr ?? 0));
  const tms = (star0 as { life?: { t_tms?: number } } | undefined)?.life?.t_tms;
  const t0 = 0.01;
  const t1 = Math.max(300, settled * 4, lastMagma * 1.5);
  const stages: ReplayStage[] = [
    { n: 1, name: "Inheritance", from_myr: t0 * 0.5, text: `A cloud core collapses in a birth cluster of ${(h.inheritance as { cluster_members?: number }).cluster_members ?? "many"} stars. Its metals ([Fe/H] ${inh.feh.toFixed(2)}) and the cluster's ultraviolet glare (${Math.round(inh.fuv_g0)} G₀) were set by stars that lived and died there before.` },
    { n: 2, name: "Star", from_myr: t0, text: `A ${star0 ? star0.mass.toFixed(2) : "?"} solar-mass star forms${companions ? ` with ${companions === 1 ? "a companion star" : `${companions} companion stars`}` : ""}, still contracting and not yet burning hydrogen.` },
    { n: 3, name: "Disc and material bands", from_myr: 0.03, text: `A disc of ${mass0.toFixed(3)} solar masses of gas and dust. It cools, so its condensation lines sweep inward: water ice freezes out beyond ${au(water ?? 0)} at first, ${au(waterEnd ?? 0)} by the end. Between the lines lie bands of different stuff.` },
    { n: 4, name: "Growth", from_myr: firstEmbryo, text: `Dust piles into pebbles and planetesimals at the band edges; ${growth.length} embryos gather and feed on pebbles drifting in, each recording what it ate.` },
    { n: 5, name: giants.length ? "Giants and migration" : "Migration (no giants)", from_myr: Math.max(firstEmbryo * 1.3, Math.min(tDisperse * 0.6, growthEnd * 0.6)), text: giants.length ? `${giants.map((g) => g.label).join(" and ")} pull${giants.length === 1 ? "s" : ""} in gas before the disc clears, and the worlds drift through the gas into resonant chains.` : `No core grows heavy enough to pull in gas before the disc clears after ${ago(tDisperse)}, so no giants; the embryos drift through the gas into resonant chains.` },
    { n: 6, name: "The violent phase", from_myr: violentFrom ?? tDisperse * 1.5, text: violentFrom ? `With the gas gone the system rearranges: ${countWords(events)}. It settles after ${ago(settled)}.` : "With the gas gone, the system is spaced widely enough to stay calm: no break." },
    { n: 7, name: "Leftovers", from_myr: settled * 1.05, text: `What did not become a world stays as ${clouds.length ? [...new Set(clouds.map((c) => (c.shape === "sphere" ? "a sphere of comets round the whole system" : `a ${c.kind.replace(/-/g, " ")}`)))].join(", ") : "dust"}${moons.length ? `, and ${moons.length === 1 ? "a moon" : `${moons.length} moons`} from giant impacts` : ""}.` },
    { n: 8, name: "Each world's early life", from_myr: settled * 2, text: `Magma oceans freeze${lastMagma ? ` (the last by ${ago(lastMagma)})` : ""}, cores separate, first air is kept or lost, and oceans may condense. Each world's surface and climate start from here.` },
    { n: 9, name: "Long life", from_myr: t1 * 0.6, text: `Then billions of years of slow change: the star brightens${tms ? ` until it runs out of hydrogen near ${ago(tms)}` : ""}, tides lock and moons drift, and geology runs down. The system is ${inh.age_gyr.toFixed(2)} billion years old now.` },
  ].sort((a, b) => a.from_myr - b.from_myr);

  // Beats: every event and stage start, plus even steps in log time so quiet stretches still move.
  const steps: number[] = [];
  for (let x = lg(t0 * 0.5); x <= lg(t1); x += 0.08) steps.push(Math.pow(10, x));
  const beats = dedupe([...steps, ...ev.map((e) => e.t_myr), ...stages.map((s) => s.from_myr), t1]).filter((t) => t >= t0 * 0.5 && t <= t1);

  const radii = [...bodies.flatMap((b) => b.keys.map((x) => x.a * (1 + x.e))), ...clouds.filter((c) => c.shape === "disc").map((c) => c.a_max_au), ...(slices[0]?.lines ?? [])];
  const inner = Math.min(r.growth.inner_edge_au ?? 0.05, ...bodies.flatMap((b) => b.keys.map((x) => x.a * (1 - x.e))));
  return {
    t0,
    t1,
    star: { mass: star0?.mass ?? 1, companions },
    disc: { t_disperse: tDisperse, bands: h.disc?.bands.map((b) => b.key) ?? [], mass0, slices },
    lum,
    bodies,
    events: ev,
    stages,
    clouds,
    moons,
    beats,
    inner_au: Math.max(0.005, inner / 1.4),
    outer_au: Math.max(...radii, 1) * 1.3,
  };
}

function addressToId(address: string, bodies: ReplayBody[]): string {
  return bodies.find((b) => b.address === address)?.id ?? address;
}

function countWords(events: UpheavalEvent[]): string {
  const n = (f: (e: UpheavalEvent) => boolean) => events.filter(f).length;
  const parts: string[] = [];
  if (n((e) => e.type === "chain-breaks")) parts.push("a resonant chain breaks");
  const sc = n((e) => e.type === "scatter");
  if (sc) parts.push(`${sc} ${sc === 1 ? "world is" : "worlds are"} flung onto stretched orbits`);
  const im = n((e) => e.type === "impact");
  if (im) parts.push(`${im} giant ${im === 1 ? "impact" : "impacts"}`);
  const ej = n((e) => e.type === "ejected");
  if (ej) parts.push(`${ej} ${ej === 1 ? "world is" : "worlds are"} thrown out as rogue planets`);
  return parts.join(", ") || "orbits shift";
}

function interp<T extends { t: number }>(xs: T[], t: number, get: (x: T) => number): number {
  if (!xs.length) return 0;
  if (t <= xs[0]!.t) return get(xs[0]!);
  for (let i = 1; i < xs.length; i++) {
    const b = xs[i]!;
    if (t <= b.t) {
      const a = xs[i - 1]!;
      const f = b.t > a.t ? (lg(t) - lg(a.t)) / (lg(b.t) - lg(a.t)) : 1;
      return get(a) + (get(b) - get(a)) * f;
    }
  }
  return get(xs[xs.length - 1]!);
}

function stepAt<T extends { t: number }>(xs: T[], t: number, get: (x: T) => number): number {
  let v = xs.length ? get(xs[0]!) : 0;
  for (const x of xs) if (x.t <= t) v = get(x);
  return v;
}

/** The system at time t (Myr since the disc formed). */
export function replayAt(m: ReplayModel, t: number): ReplayFrame {
  const stage = [...m.stages].reverse().find((s) => s.from_myr <= t * (1 + 1e-9)) ?? m.stages[0]!;
  let disc: ReplayFrame["disc"] = null;
  if (t < m.disc.t_disperse && m.disc.slices.length) {
    const lines = m.disc.slices[0]!.lines.map((_, i) => interp(m.disc.slices, t, (s) => s.lines[i]!));
    const mass = interp(m.disc.slices, t, (s) => s.mass);
    const fade = Math.min(1, Math.max(0, (lg(m.disc.t_disperse) - lg(t)) / 0.3));
    disc = { lines, opacity: Math.sqrt(Math.max(0, mass / m.disc.mass0)) * fade, inner_au: m.inner_au * 1.4, outer_au: Math.max(lines[lines.length - 1]! * 1.5, interp(m.disc.slices, t, (s) => s.rc) * 3) };
  }
  const bodies = m.bodies
    .filter((b) => b.born_myr <= t && !(b.gone && b.gone.t_myr < t))
    .map((b) => {
      const ks = b.keys;
      let a = ks[0]!.a;
      let e = ks[0]!.e;
      for (let i = 1; i < ks.length; i++) {
        const k1 = ks[i]!;
        const k0 = ks[i - 1]!;
        if (t >= k1.t) ((a = k1.a), (e = k1.e));
        else if (t > k0.t) {
          const f = k1.jump ? 0 : (lg(t) - lg(k0.t)) / Math.max(1e-12, lg(k1.t) - lg(k0.t));
          a = Math.exp(Math.log(k0.a) + (Math.log(k1.a) - Math.log(k0.a)) * f);
          e = k0.e + (k1.e - k0.e) * f;
          break;
        } else break;
      }
      // A little motion round the star, slower further out (display only: real orbits would blur at these time steps).
      const spin = (lg(t) - lg(m.t0)) * 55 * Math.pow(Math.max(a, 0.05), -0.5);
      return { id: b.id, label: b.label, a_au: a, e, m: stepAt(b.mass, t, (x) => x.m), angle_deg: (b.angle_deg + spin) % 360, giant: b.giant };
    });
  const alive = new Set(bodies.map((b) => b.id));
  const beat = m.beats.findIndex((x) => x > t);
  const prev = beat > 1 ? m.beats[beat - 2]! : 0;
  const fresh = m.events.filter((e) => e.t_myr <= t && e.t_myr > prev);
  const last = [...m.events].reverse().find((e) => e.t_myr <= t) ?? null;
  return {
    t,
    stage,
    luminosity: interp(m.lum, t, (x) => x.l),
    disc,
    bodies,
    clouds: m.clouds.filter((c) => c.from_myr <= t),
    moons: m.moons.filter((x) => x.from_myr <= t && alive.has(x.parent)).map((x) => ({ parent: x.parent, diameter_km: x.diameter_km })),
    last,
    fresh,
  };
}

/** Time in words for the replay's clock. */
export function replayClock(t: number): string {
  return t < 1 ? `${Math.round(t * 1000).toLocaleString("en-GB")} thousand years` : t < 1000 ? `${Number(t.toPrecision(3))} million years` : `${Number((t / 1000).toPrecision(3))} billion years`;
}
