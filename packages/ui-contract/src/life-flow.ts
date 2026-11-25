// The LifeFlow board (U7 part 2): the pure parts. The life pipeline the kernel runs for a world (habitats, the
// energy budget, origin, opportunity, innovations, catastrophes, the ecosystem, traits, fluxes, signatures and the
// story hooks they leave) drawn as a flow of stages, each showing what the kernel's `life_view` gives for it and the
// dials it leans on. Nothing here computes biology: every number and reason is the kernel's, sorted into the stage
// that made it. Life comes from energy gradients, so the flow starts from where energy is, not from a class.
import { carbonWords, confidenceWords, words, type LifeEvent, type LifeView } from "./life.ts";
import { sayNumber } from "./figures.ts";

export type StageId = "habitats" | "budget" | "origin" | "opportunity" | "innovations" | "catastrophes" | "ecosystem" | "traits" | "fluxes" | "signatures" | "hooks";

export interface FlowStage {
  id: StageId;
  label: string;
  /** What the stage does, in one plain sentence. */
  does: string;
  /** Its result in a few words, for the node. */
  headline: string;
  /** Its results in full, for the stage's card. */
  lines: string[];
  /** The kernel's dated events this stage made, with their reasons. */
  events: LifeEvent[];
  /** The dials (unknowns) it leans on, with their values. */
  dials: { id: string; value: number }[];
  /** The stages it takes from. */
  from: StageId[];
  /** False when the stage has nothing for this world (no life began, no history kept). */
  live: boolean;
}

/** The stages in the order life is worked out, each with the stages it takes from. */
export const STAGES: readonly { id: StageId; label: string; does: string; from: StageId[] }[] = [
  { id: "habitats", label: "Habitats", does: "Where on the world energy and a liquid meet: each place life could live.", from: [] },
  { id: "budget", label: "Energy budget", does: "How much living matter each habitat's energy gradients can pay for.", from: ["habitats"] },
  { id: "origin", label: "Origin", does: "Whether and when life starts, and where, given the budget and the time.", from: ["habitats", "budget"] },
  { id: "opportunity", label: "Opportunity", does: "How much living has been done since: the more cells over more time, the more chances to invent.", from: ["origin", "budget"] },
  { id: "catastrophes", label: "Catastrophes", does: "Impacts, freezes and losses that knock life back, and the recoveries after.", from: ["habitats"] },
  { id: "innovations", label: "Innovations", does: "The big inventions (new ways to harvest energy, complex cells, bodies), each when opportunity allows.", from: ["opportunity", "catastrophes"] },
  { id: "ecosystem", label: "Ecosystem", does: "Who makes food and who eats it: the guilds the budget and the inventions support.", from: ["innovations", "budget", "catastrophes"] },
  { id: "traits", label: "Traits", does: "What the life is like: colour from the starlight, solvent, chemistry, sizes, senses.", from: ["ecosystem"] },
  { id: "fluxes", label: "Fluxes", does: "What the living world puts into its air and rocks: oxygen, methane, buried carbon.", from: ["ecosystem"] },
  { id: "signatures", label: "Signatures", does: "What an outside scan would see, how sure it could be, and what it is blind to.", from: ["fluxes", "traits"] },
  { id: "hooks", label: "Story hooks", does: "Where the science leaves a story: false alarms, hidden life, a dead world's record.", from: ["signatures"] },
];

const EVENT_STAGE: Record<string, StageId> = {
  origin: "origin",
  capability: "innovations",
  capability_lost: "catastrophes",
  catastrophe: "catastrophes",
  recovery: "catastrophes",
  sterilisation: "catastrophes",
  extinction: "catastrophes",
};

/** The stage that made an event: origins to Origin, inventions to Innovations, losses and recoveries to Catastrophes. */
export const stageOfEvent = (e: Pick<LifeEvent, "kind">): StageId => EVENT_STAGE[e.kind] ?? "innovations";

const habitatOf = (subject: string): string | null => {
  const m = /habitat:@?([^/]+)$/.exec(subject);
  return m ? m[1]!.replace(/[-_]/g, " ") : null;
};
const myr = (m: number) => (m >= 1000 ? `${sayNumber(Math.round(m / 10) / 100, 6)} Gyr` : `${Math.round(m)} Myr`);

/** The board's stages for one world's `life_view`. */
export function lifeFlow(v: LifeView): FlowStage[] {
  const now = v.timeline.at(-1) ?? null;
  const ev = (s: StageId) => v.events.filter((e) => stageOfEvent(e) === s);
  // A dial belongs to the stages whose events lean on it; one nobody cites sits with Innovations, where most are read.
  const dialStages = new Map<string, Set<StageId>>();
  for (const e of v.events) for (const d of e.dials) (dialStages.get(d) ?? dialStages.set(d, new Set()).get(d)!).add(stageOfEvent(e));
  const dialsOf = (s: StageId) =>
    Object.entries(v.dials)
      .filter(([id]) => (dialStages.get(id) ?? new Set<StageId>(["innovations"])).has(s))
      .map(([id, value]) => ({ id, value }));
  const alive = v.signatures.alive;
  const ever = v.signatures.ever_alive;

  const habitats = [...new Set([...v.events.map((e) => habitatOf(e.subject)).filter((x): x is string => !!x), ...v.guilds.map((g) => g.habitat_kind.replace(/[-_]/g, " "))])];
  const production = v.guilds.reduce((n, g) => n + g.production_gc_yr, 0);
  const peak = v.timeline.reduce((n, p) => Math.max(n, p.production_gc_yr), 0);
  const origin = ev("origin")[0] ?? null;
  const gains = ev("innovations");
  const losses = ev("catastrophes");
  const knocks = losses.filter((e) => e.kind !== "recovery");
  const levels = new Set(v.guilds.map((g) => g.level)).size;
  const t = v.traits ?? {};
  const sig = v.signatures;

  const out: Record<StageId, Omit<FlowStage, "id" | "label" | "does" | "from">> = {
    habitats: {
      headline: habitats.length ? `${habitats.length} ${habitats.length === 1 ? "habitat" : "habitats"}` : "none known",
      lines: habitats.length ? habitats.map((h) => `Life lives in: ${h}`) : ["The kernel names no habitat for this world."],
      events: [],
      dials: dialsOf("habitats"),
      live: habitats.length > 0,
    },
    budget: {
      headline: production > 0 ? carbonWords(production) : peak > 0 ? `peaked ${carbonWords(peak)}` : "nothing to spend",
      lines: [
        ...(production > 0 ? [`Production now: ${carbonWords(production)} (all guilds together).`] : []),
        ...(peak > 0 ? [`Highest it reached: ${carbonWords(peak)}.`] : []),
        ...(sig.biomass_gc > 0 ? [`Standing biomass: ${carbonWords(sig.biomass_gc).replace(/\/yr$/, "")}.`] : []),
      ],
      events: [],
      dials: dialsOf("budget"),
      live: production > 0 || peak > 0,
    },
    origin: {
      headline: origin ? `began at ${myr(origin.at_myr)}` : alive ? "alive (start not dated)" : "never began",
      lines: origin ? [origin.why] : alive ? ["Life is here; this world's view keeps no dated history of its start."] : ["Life never started here."],
      events: ev("origin"),
      dials: dialsOf("origin"),
      live: !!origin || alive || ever,
    },
    opportunity: {
      headline: now ? `${sayNumber(Math.round(now.opportunity), 0, { decimals: 0 })} Earth-Myr` : "no history kept",
      lines: now ? [`Opportunity so far: ${sayNumber(Math.round(now.opportunity), 0, { decimals: 0 })} Earth-Myr of living since the origin.`, `Diversity now: ${sayNumber(now.diversity, 0, { decimals: 1 })} on the kernel's scale.`] : ["This world's view keeps no timeline."],
      events: [],
      dials: dialsOf("opportunity"),
      live: !!now && now.opportunity > 0,
    },
    catastrophes: {
      headline: knocks.length ? `${knocks.length} ${knocks.length === 1 ? "blow" : "blows"}${losses.length > knocks.length ? `, ${losses.length - knocks.length} recovered` : ""}` : "none",
      lines: losses.map((e) => `${myr(e.at_myr)}: ${e.why}`),
      events: losses,
      dials: dialsOf("catastrophes"),
      live: losses.length > 0,
    },
    innovations: {
      headline: gains.length ? `${gains.length} ${gains.length === 1 ? "invention" : "inventions"}` : "none",
      lines: gains.map((e) => `${myr(e.at_myr)}: ${words(e.subject)}`),
      events: gains,
      dials: dialsOf("innovations"),
      live: gains.length > 0,
    },
    ecosystem: {
      headline: v.guilds.length ? `${v.guilds.length} guilds, ${levels} ${levels === 1 ? "level" : "levels"}` : "empty",
      lines: [...v.guilds].sort((a, b) => b.share - a.share).map((g) => `${g.plain_name} (${g.role}, level ${g.level}, ${g.habitat_kind.replace(/[-_]/g, " ")})`),
      events: [],
      dials: dialsOf("ecosystem"),
      live: v.guilds.length > 0,
    },
    traits: {
      headline: typeof t.producer_colour === "string" ? `${t.producer_colour} producers` : typeof t.solvent === "string" ? `in ${t.solvent}` : "none read",
      lines: Object.entries(t)
        .filter(([, x]) => x !== null && x !== undefined && (typeof x !== "object" || Array.isArray(x)))
        .map(([k, x]) => `${words(k)}: ${Array.isArray(x) ? x.join(", ") : typeof x === "number" ? sayNumber(x, 3) : String(x)}`),
      events: [],
      dials: dialsOf("traits"),
      live: v.traits !== null && Object.keys(t).length > 0,
    },
    fluxes: {
      headline: `O₂ ${sayNumber(sig.o2 * 100, 2)}%, CH₄ ${sayNumber(sig.ch4_ppm, 2)} ppm`,
      lines: [`Oxygen in the air: ${sayNumber(sig.o2 * 100, 3)}%.`, `Methane: ${sayNumber(sig.ch4_ppm, 3)} ppm.`, ...(sig.traces.length ? [`Traces in the rocks: ${sig.traces.map((x) => `${words(x.kind)} (${Math.round(x.surviving * 100)}% surviving)`).join(", ")}.`] : [])],
      events: [],
      dials: dialsOf("fluxes"),
      live: alive || ever,
    },
    signatures: {
      headline: sig.blind ? "blind from outside" : sig.candidate ? `candidate, ${confidenceWords(sig.confidence)}` : "nothing to see",
      lines: [...sig.signals.map((s) => `${words(s.id)}: ${s.why}`), ...(sig.blind ? [`Blind to: ${sig.blind}`] : []), sig.candidate ? `A scan would offer life as a candidate with ${confidenceWords(sig.confidence)} confidence.` : "A scan would not offer life."],
      events: [],
      dials: dialsOf("signatures"),
      live: sig.signals.length > 0 || sig.candidate || !!sig.blind,
    },
    hooks: {
      headline: v.hooks.length ? `${v.hooks.length} ${v.hooks.length === 1 ? "hook" : "hooks"}` : "none",
      lines: v.hooks.map((h) => h.text),
      events: [],
      dials: dialsOf("hooks"),
      live: v.hooks.length > 0,
    },
  };
  return STAGES.map((s) => ({ id: s.id, label: s.label, does: s.does, from: [...s.from], ...out[s.id] }));
}

export interface PlacedStage {
  stage: FlowStage;
  x: number;
  y: number;
  col: number;
}

/**
 * Stages placed left to right by how many steps they sit from the first (their longest path in), stacked within a
 * column; wires run from the right edge of each input to the left edge of the stage. Sizes are the caller's.
 */
export function placeStages(stages: readonly FlowStage[], o: { w: number; h: number; gapX: number; gapY: number }): { placed: PlacedStage[]; wires: { from: StageId; to: StageId; d: string; live: boolean }[]; width: number; height: number } {
  const col = new Map<StageId, number>();
  const by = new Map(stages.map((s) => [s.id, s]));
  const depth = (id: StageId): number => {
    if (col.has(id)) return col.get(id)!;
    const s = by.get(id);
    const c = s && s.from.length ? 1 + Math.max(...s.from.filter((f) => by.has(f)).map(depth)) : 0;
    col.set(id, c);
    return c;
  };
  stages.forEach((s) => depth(s.id));
  const cols = Math.max(0, ...col.values()) + 1;
  const rowsIn = new Map<number, number>();
  const placed: PlacedStage[] = stages.map((s) => {
    const c = col.get(s.id)!;
    const r = rowsIn.get(c) ?? 0;
    rowsIn.set(c, r + 1);
    return { stage: s, col: c, x: c * (o.w + o.gapX), y: r * (o.h + o.gapY) };
  });
  const rows = Math.max(1, ...rowsIn.values());
  // Centre each column's stack in the board's height.
  const height = rows * o.h + (rows - 1) * o.gapY;
  for (const [c, n] of rowsIn) {
    const shift = (height - (n * o.h + (n - 1) * o.gapY)) / 2;
    for (const p of placed) if (p.col === c) p.y += shift;
  }
  const at = new Map(placed.map((p) => [p.stage.id, p]));
  const wires = placed.flatMap((p) =>
    p.stage.from
      .filter((f) => at.has(f))
      .map((f) => {
        const a = at.get(f)!;
        const x0 = a.x + o.w;
        const y0 = a.y + o.h / 2;
        const x1 = p.x;
        const y1 = p.y + o.h / 2;
        const mx = (x0 + x1) / 2;
        return { from: f, to: p.stage.id, d: `M${x0},${y0} C${mx},${y0} ${mx},${y1} ${x1},${y1}`, live: a.stage.live && p.stage.live };
      }),
  );
  return { placed, wires, width: cols * o.w + (cols - 1) * o.gapX, height };
}

/**
 * A what-if under other dial values: the kernel's life_view re-run with `dials`, read only. `honoured` is false
 * when the kernel answered without taking the asked values (a kernel from before dials ignores the field and gives
 * the world as the pack sets it), so a caller keeps its knobs read only instead of showing a what-if that is not one.
 */
export type LifeWhatIf = { ok: true; honoured: boolean; view: LifeView } | { ok: false; message: string };

export async function askWhatIf(
  session: { lifeView(by: { body?: string; fixture?: string; dials?: Record<string, number> }): Promise<{ ok: true; view: LifeView } | { ok: false; message: string }> },
  by: { body?: string; fixture?: string },
  dials: Record<string, number>,
): Promise<LifeWhatIf> {
  const r = await session.lifeView({ ...by, dials });
  if (!r.ok) return { ok: false, message: r.message };
  const honoured = Object.entries(dials).every(([k, v]) => k in r.view.dials && Math.abs(r.view.dials[k]! - v) < 1e-9);
  return { ok: true, honoured, view: r.view };
}

/** One stage's change between the pack's world and a what-if. */
export interface StageMove {
  id: StageId;
  /** Its node words before and after (equal when only its events moved). */
  was: string;
  now: string;
  /** Whether it had anything to give, before and after. */
  wasLive: boolean;
  nowLive: boolean;
  /** Dated events the what-if adds and takes away, by their reasons. */
  added: string[];
  gone: string[];
}

/** What moved, stage by stage, between the pack's world and a what-if (stages that did not move are left out). */
export function whatMoved(base: LifeView, whatIf: LifeView): StageMove[] {
  const a = new Map(lifeFlow(base).map((s) => [s.id, s]));
  const out: StageMove[] = [];
  for (const s of lifeFlow(whatIf)) {
    const b = a.get(s.id);
    if (!b) continue;
    const key = (e: LifeEvent) => `${e.kind}|${e.subject ?? ""}`;
    const before = new Map(b.events.map((e) => [key(e), e]));
    const after = new Map(s.events.map((e) => [key(e), e]));
    const added = [...after].filter(([k]) => !before.has(k)).map(([, e]) => e.why);
    const gone = [...before].filter(([k]) => !after.has(k)).map(([, e]) => e.why);
    const linesMoved = b.lines.join("\n") !== s.lines.join("\n");
    if (b.headline === s.headline && b.live === s.live && !added.length && !gone.length && !linesMoved) continue;
    out.push({ id: s.id, was: b.headline, now: s.headline, wasLive: b.live, nowLive: s.live, added, gone });
  }
  return out;
}
