/**
 * One world's story on the Formation screen (formation design section 7, right panel; package F10b): where its
 * material came from, by band, from its provenance ledger and the bodies that merged into it; its events in order;
 * and how sure each part is. Also the screen's "How this was worked out" panel. Read from the run's own record; pure.
 */
import type { FormationRan } from "./make.ts";
import type { FormationChart } from "./formation-chart.ts";
import { chartAu, chartTime } from "./formation-chart.ts";
import { bandWords } from "./replay.ts";
import { sayNumber } from "./figures.ts";
import { fateLine } from "./formation-fate.ts";
import type { ChartTrack } from "./formation-chart.ts";

/** A world that is still there now (it may be swallowed when the star ends). */
const alive = (t: ChartTrack) => t.end.how === "lasts" || t.end.how === "engulfed";
/** Where a track is now: the end of the stretch that reaches now (the chart may run on past it). */
function aNow(c: FormationChart, t: ChartTrack): number {
  const g = t.segs.find((x) => x.t1 >= c.now * (1 - 1e-9));
  return g ? g.a1 : t.end.a;
}

type Solid = number[];
interface Entry { kind: string; band?: string; t_myr?: number; solid?: Solid; gas?: number; ledger?: { entries: Entry[] } }
interface GrowthBody { id: string; born_au: number; born_band: string; born_myr: number; a_au: number; gas: number; solid: Solid; ledger?: { entries: Entry[] } }
interface UpEvent { type: string; t_myr: number; impactor?: string; target?: string; outcome?: string }

/** One slice of the provenance bar. */
export interface MaterialShare {
  key: string;
  words: string;
  share: number;
  kind: "band" | "impactor" | "gas";
}
export interface StoryLine {
  t: number;
  /** "0.8 Myr", or a span ("1 to 4 Myr"). */
  when: string;
  text: string;
  /** A violent moment (impact, scattering, ejection): drawn in the accent. */
  violent?: boolean;
}
export interface SureRow {
  label: string;
  value: string;
  title: string;
}
export interface WorldStory {
  id: string;
  label: string;
  address?: string;
  material: MaterialShare[];
  /** The bar in words: "58% rock band · 27% from World 4 · …". */
  material_words: string;
  /** Where its water came from, in words; null when it holds almost none. */
  water_words: string | null;
  lines: StoryLine[];
  sure: SureRow[];
}

/** The water ice line and the bands beyond it. */
const BEYOND_SNOW = new Set(["water-ice", "co2-ice", "co-ice"]);
const WATER = 3;
const sum = (s: Solid | undefined) => (s ?? []).reduce((a, b) => a + b, 0);
const pct = (x: number) => `${sayNumber(Math.round(x * 100), 2)}%`;

function claims(entries: Entry[] | undefined, out: Entry[] = []): Entry[] {
  for (const e of entries ?? []) {
    if (e.kind === "claim") out.push(e);
    if (e.ledger) claims(e.ledger.entries, out);
  }
  return out;
}

/** The story of one body of the chart (by its replay id). Null when the chart has no such body. */
export function worldStory(run: FormationRan, c: FormationChart, id: string): WorldStory | null {
  const track = c.tracks.find((t) => t.id === id);
  if (!track) return null;
  const r = run.run as unknown as { growth: { bodies?: GrowthBody[]; end_myr?: number }; upheaval?: { events: UpEvent[] }; events: { subject?: string; t_myr: number; text?: string }[] };
  const bodies = new Map((r.growth.bodies ?? []).map((b) => [b.id, b]));
  const ups = r.upheaval?.events ?? [];
  const label = (x: string) => c.tracks.find((t) => t.id === x)?.label ?? x;
  // Everything that merged into it, directly or through another merger.
  const into = (target: string, seen = new Set<string>()): string[] => {
    const out: string[] = [];
    for (const e of ups)
      if (e.type === "impact" && e.target === target && (e.outcome === "merge" || e.outcome === "merge-moon") && e.impactor && !seen.has(e.impactor)) {
        seen.add(e.impactor);
        out.push(e.impactor, ...into(e.impactor, seen));
      }
    return out;
  };
  const self = bodies.get(id);
  const own = claims(self?.ledger?.entries);

  // Material: its own claims by band (gas apart), then each merged body whole.
  const byBand = new Map<string, number>();
  let gas = 0;
  for (const e of own) {
    byBand.set(e.band ?? "?", (byBand.get(e.band ?? "?") ?? 0) + sum(e.solid));
    gas += e.gas ?? 0;
  }
  const merged = into(id);
  const fromMerged = merged.map((m) => ({ id: m, u: claims(bodies.get(m)?.ledger?.entries).reduce((a, e) => a + sum(e.solid) + (e.gas ?? 0), 0) }));
  const total = [...byBand.values()].reduce((a, b) => a + b, 0) + gas + fromMerged.reduce((a, b) => a + b.u, 0);
  const material: MaterialShare[] = [];
  if (total > 0) {
    for (const [band, u] of [...byBand].sort((a, b) => b[1] - a[1])) if (u > 0) material.push({ key: band, words: `${bandWords(band)} band`, share: u / total, kind: "band" });
    for (const m of fromMerged.sort((a, b) => b.u - a.u)) if (m.u > 0) material.push({ key: m.id, words: `from ${label(m.id)}${bodies.get(m.id) ? ` (born at ${chartAu(bodies.get(m.id)!.born_au)})` : ""}`, share: m.u / total, kind: "impactor" });
    if (gas > 0) material.push({ key: "gas", words: "gas pulled from the disc", share: gas / total, kind: "gas" });
  }
  const material_words = material.filter((x) => x.share >= 0.01).map((x) => `${pct(x.share)} ${x.words}`).join(" · ") || "No ledger for this body";

  // Water: which of it came from beyond the water ice line.
  const all = [...own, ...merged.flatMap((m) => claims(bodies.get(m)?.ledger?.entries))];
  const water = all.reduce((a, e) => a + (e.solid?.[WATER] ?? 0), 0);
  const beyond = all.filter((e) => BEYOND_SNOW.has(e.band ?? "")).reduce((a, e) => a + (e.solid?.[WATER] ?? 0), 0);
  const water_words = total > 0 && water / total > 0.001 ? `${pct(water / total)} of it is water, ${beyond / water > 0.97 ? "nearly all" : pct(beyond / water)} from beyond the water ice line` : null;

  // The story, in order.
  const lines: StoryLine[] = [];
  if (self) lines.push({ t: self.born_myr, when: chartTime(self.born_myr), text: `embryo at ${chartAu(self.born_au)}, in the ${bandWords(self.born_band)} band` });
  const end = r.growth.end_myr ?? c.gas_until;
  if (self && Math.abs(Math.log(self.a_au / self.born_au)) > 0.14) lines.push({ t: end, when: `${chartTime(self.born_myr)} to ${chartTime(end)}`, text: `drifts ${self.a_au < self.born_au ? "in" : "out"} to ${chartAu(self.a_au)}` });
  for (const e of c.events) if (e.ids.includes(id) && e.kind !== "captured-in-resonance") lines.push({ t: e.t, when: chartTime(e.t), text: e.text.replace(/\.$/, ""), violent: /^(impact|scatter|ejected)/.test(e.kind) });
  for (const e of c.events) if (e.ids.includes(id) && e.kind === "captured-in-resonance") lines.push({ t: e.t, when: chartTime(e.t), text: e.text.replace(/\.$/, "") });
  if (track.address) for (const e of r.events) if (e.subject === track.address && e.text) lines.push({ t: e.t_myr, when: chartTime(e.t_myr), text: e.text.replace(/\.$/, "") });
  if (alive(track)) lines.push({ t: c.now, when: chartTime(c.now), text: `now at ${chartAu(aNow(c, track))}, ${sayNumber(track.m_end, 2)} Earth masses` });
  // The star's end, once the look-ahead has answered.
  const fl = c.fate ? fateLine(c.fate, id) : null;
  if (fl) lines.push({ t: fl.t, when: chartTime(fl.t), text: fl.text, ...(track.end.how === "engulfed" ? { violent: true } : {}) });
  lines.sort((a, b) => a.t - b.t);
  // Lines at the same moment keep one "when".
  const seen = new Set<string>();
  for (const l of lines) (seen.has(l.when) ? (l.when = "") : seen.add(l.when));

  // How sure each part is.
  const sure: SureRow[] = [{ label: "Orbit, mass", value: "calculated", title: "Worked out by the growth and migration rules from the disc it grew in, stepped in time" }];
  const hit = ups.some((e) => e.type === "impact" && (e.target === id || e.impactor === id));
  const flung = ups.some((e) => (e.type === "scatter" && (e as { thrown?: string }).thrown === id) || (e.type === "ejected" && (e as { id?: string }).id === id));
  if (hit) sure.push({ label: "Impact dates and angles", value: "drawn", title: "Each impact's date, speed and angle are drawn from odds calibrated on published orbit studies, keyed by the seed: a different draw would give a different but equally likely history" });
  if (flung) sure.push({ label: "Scattering", value: "drawn", title: "Who flings whom, and how far, is drawn from calibrated odds" });
  if (material.length) sure.push({ label: "Where its material came from", value: "from the ledger", title: "Summed from the provenance ledger: every claim it made on a band of the disc, and every body that merged into it" });
  return { id, label: track.label, ...(track.address ? { address: track.address } : {}), material, material_words, water_words, lines, sure };
}

/** The body the story card opens on: the selected world, else the heaviest that lasts. */
export function storyFor(c: FormationChart, selected: string | null): string | null {
  const sel = selected ? c.tracks.find((t) => t.address === selected) : undefined;
  if (sel) return sel.id;
  return [...c.tracks].filter(alive).sort((a, b) => b.m_end - a.m_end)[0]?.id ?? null;
}

/** "How this was worked out" (formation design section 4): what each part of the history rests on. */
export function howWorkedOut(run: FormationRan): SureRow[] {
  const classic = /classic/i.test(String((run.run as { block?: string }).block ?? ""));
  return [
    { label: "Stars and their life lines", value: "tabulated", title: "Brightness, size and fate over time read from published stellar tracks by mass and metals" },
    { label: "Bands, growth, giants, migration", value: classic ? "Classic Accrete" : "calculated, stepped in time", title: classic ? "SSE 3's Accrete, ported, for growth and giants" : "The disc's bands and their ice lines, pebble growth, gas capture and drift through the disc, worked out step by step" },
    { label: "Impacts, ejections, scatterings", value: "drawn from calibrated odds", title: "Chaotic outcomes are keyed draws from distributions calibrated on published orbit integrations, never a free roll" },
    { label: "Each world's early life", value: "calculated", title: "Magma ocean, core, first air kept or lost, oceans: from each world's own make-up and light" },
    { label: "Passing stars, comet showers", value: "not yet", title: "These come from the system's place in the galaxy and the stars that pass it, which is still being tied in" },
  ];
}
