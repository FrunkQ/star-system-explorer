// Data views (H6.7, build plan U5 PlayerData board). What players can learn
// is pack data: a disclosure map (which tier each field and tag sits at) and
// the campaign's data views (which tiers a view shows, which tags, how
// unknowns read). The kernel reads it from `provides.other.observe`, highest
// pack wins whole. The GM edits a copy and applies it as the campaign's own
// player-data pack, stacked like any other; the base pack is never touched.
import type { Json } from "./json.ts";
import type { PackDoc } from "./physics.ts";
import type { ViewSession } from "./session.ts";
import type { Workspace } from "./view.ts";
import type { IntentResult } from "./session.ts";
import { stackPacks } from "./packs-edit.ts";

export type TagMode = "all" | "none" | "listed";
export type UnknownStyle = "placeholder" | "omit";

export interface Override {
  target: { kind: "field" | "tag"; id: string };
  show: boolean;
  precision?: { as: "exact" } | { as: "rounded"; digits?: number } | { as: "band"; step: number; half_width: number } | { as: "hidden" };
}

export interface DataView {
  id: string;
  label: string;
  description?: string;
  show_tiers: string[];
  tags: { mode: TagMode; list?: string[] };
  overrides?: Override[];
  unknown_style: UnknownStyle;
  show_honesty?: boolean;
}

export interface ObserveSetup {
  disclosure: {
    ladder?: { rungs: [string, string][] };
    fields: Record<string, string>;
    tags: Record<string, string>;
    default_tier?: string | null;
    internal?: string[];
    [other: string]: unknown;
  };
  views: DataView[];
  campaign: { default_view: string; before: string; after: string };
}

/** The pack that carries the campaign's own edits to its data views. */
export const PLAYER_DATA_PACK = "campaign.player-data";

/** The default ladder (sse-observe `Ladder::default`): tier, and what an unknown reads at it. */
export const DEFAULT_LADDER: [string, string][] = [
  ["obvious", "unknown"],
  ["scanned", "not yet scanned"],
  ["surveyed", "survey required"],
  ["secret", ""],
];

export function ladderOf(s: ObserveSetup): [string, string][] {
  return s.disclosure.ladder?.rungs ?? DEFAULT_LADDER;
}

/** The setup the kernel uses for a stack (bottom to top): the highest pack that provides one. */
export function observeOf(packs: readonly PackDoc[]): { setup: ObserveSetup; pack: string } | null {
  for (let i = packs.length - 1; i >= 0; i--) {
    const o = (packs[i]!.provides?.other as { observe?: ObserveSetup } | undefined)?.observe;
    if (o) return { setup: o, pack: packs[i]!.id };
  }
  return null;
}

/** Why a setup cannot be applied, or null. The kernel validates it again at load_stack. */
export function checkSetup(s: ObserveSetup): string | null {
  const tiers = ladderOf(s).map((r) => r[0]);
  const ids = new Set<string>();
  for (const v of s.views) {
    if (!/^[a-z0-9][a-z0-9-]*$/.test(v.id)) return `view id "${v.id}" must be lower-case words joined by hyphens`;
    if (ids.has(v.id)) return `two views are called ${v.id}`;
    ids.add(v.id);
    if (!v.label.trim()) return `view ${v.id} needs a label`;
    for (const t of v.show_tiers) if (!tiers.includes(t)) return `view ${v.id} shows an unknown tier ${t}`;
    if (v.show_tiers.includes(tiers[tiers.length - 1]!)) return `view ${v.id} would show GM-only facts to players`;
  }
  for (const [k, id] of Object.entries(s.campaign)) if (!ids.has(id)) return `the campaign's ${k.replace("_", " ")} view ${id} does not exist`;
  for (const [f, t] of [...Object.entries(s.disclosure.fields), ...Object.entries(s.disclosure.tags)]) {
    if (!tiers.includes(t)) return `${f} sits at an unknown tier ${t}`;
  }
  return null;
}

/** The campaign's player-data pack carrying `setup`. */
export function playerDataPack(setup: ObserveSetup, version = "0.1.0"): PackDoc {
  return {
    format: "sse.pack/1",
    id: PLAYER_DATA_PACK,
    version,
    kind: ["game-system"],
    kernel_range: "^0.1",
    licence: "Apache-2.0",
    provides: { other: { observe: structuredClone(setup) as unknown as Json } },
  };
}

/** What changed between two setups, in plain words, for the Apply button. */
export function setupChanges(a: ObserveSetup, b: ObserveSetup): string[] {
  const out: string[] = [];
  const va = new Map(a.views.map((v) => [v.id, v]));
  for (const v of b.views) {
    const w = va.get(v.id);
    if (!w) out.push(`new view ${v.label}`);
    else if (JSON.stringify(w) !== JSON.stringify(v)) out.push(`view ${v.label} changed`);
    va.delete(v.id);
  }
  for (const w of va.values()) out.push(`view ${w.label} removed`);
  for (const kind of ["fields", "tags"] as const) {
    const da = a.disclosure[kind];
    const db = b.disclosure[kind];
    for (const k of new Set([...Object.keys(da), ...Object.keys(db)])) {
      if (da[k] !== db[k]) out.push(db[k] === undefined ? `${k} no longer listed` : da[k] === undefined ? `${k} at ${db[k]}` : `${k} ${da[k]} to ${db[k]}`);
    }
  }
  for (const k of ["default_view", "before", "after"] as const) if (a.campaign[k] !== b.campaign[k]) out.push(`${k.replace("_", " ")} view ${a.campaign[k]} to ${b.campaign[k]}`);
  return out;
}

/**
 * Apply a setup to the campaign: one stack_packs intent that puts the
 * campaign's player-data pack above the packs already added. Chronicled by
 * the kernel as a stack.adopt event; players see it at once.
 */
export async function applySetup(session: ViewSession, ws: Workspace, setup: ObserveSetup): Promise<IntentResult> {
  const bad = checkSetup(setup);
  if (bad) return { status: "refused", reason: bad };
  const st = session.stack();
  const current = st.added.find((p) => p.id === PLAYER_DATA_PACK);
  const next = bumpVersion(current?.version ?? "0.0.0");
  const added = [...st.added.filter((p) => p.id !== PLAYER_DATA_PACK), playerDataPack(setup, next)];
  return stackPacks(session, ws, added);
}

function bumpVersion(v: string): string {
  const [a = 0, b = 0, c = 0] = v.split(".").map(Number);
  return `${a}.${b}.${c + 1}`;
}

/** Facts the GM has recorded for players, as entered when a campaign opens. */
export interface AuthoredFacts {
  schema: "sse.authored-facts/0";
  universe: { id: string; seed: string };
  scope: string;
  at_yr: number;
  facts: { address: string; field: string; value: Json }[];
}

/** Enter recorded facts as authored edits, in order. Stops at the first refusal. */
export async function enterFacts(session: ViewSession, doc: AuthoredFacts): Promise<IntentResult | null> {
  for (const f of doc.facts) {
    const r = await session.send({ kind: "set_value", address: f.address, field: f.field, value: f.value, at_yr: doc.at_yr });
    if (r.status !== "applied") return r;
  }
  return null;
}
