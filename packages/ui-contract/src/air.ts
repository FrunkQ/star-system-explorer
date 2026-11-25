// The Air box's mix (ED8, editing design section 11A): the whole air as shares by volume that always sum to one. The
// kernel reads `atmosphere.composition.<gas>` (volume fractions) for the climate's gas mix (SOL5), so the box writes
// those fields. Setting one gas's share trades it against the rest, which keep their proportions, so nothing needs fixing
// up by hand; the change is one set of fields, one Undo. Gases under 1% are traces, said in ppm.
import type { Json } from "./json.ts";
import { toSI, type SolSheet } from "./sol.ts";

export const MIX_FIELD = "atmosphere.composition.";
/** Under this share a gas is a trace (a chip in ppm, not a band of the bar). */
export const TRACE = 0.01;

export interface MixGas {
  gas: string;
  share: number;
}

/** A body's air mix from its facts, largest first (ties by name); empty when it has none. */
export function airMix(facts: Record<string, Json>): MixGas[] {
  const out: MixGas[] = [];
  for (const [k, v] of Object.entries(facts)) {
    if (!k.startsWith(MIX_FIELD) || typeof v !== "number" || !(v >= 0)) continue;
    out.push({ gas: k.slice(MIX_FIELD.length), share: v });
  }
  return out.sort((a, b) => b.share - a.share || (a.gas < b.gas ? -1 : a.gas > b.gas ? 1 : 0));
}

/**
 * Set one gas's share and trade it against the rest, which keep their proportions (a new gas joins the same way).
 * Returns every gas's new share; the sum is one. A share of one leaves only that gas (the others at zero, kept so
 * Undo has them to put back).
 */
export function retrade(mix: readonly MixGas[], gas: string, share: number): MixGas[] {
  const s = Math.min(1, Math.max(0, share));
  const rest = mix.filter((g) => g.gas !== gas);
  const left = rest.reduce((t, g) => t + g.share, 0);
  const out = rest.map((g) => ({ gas: g.gas, share: left > 0 ? (g.share / left) * (1 - s) : rest.length ? (1 - s) / rest.length : 0 }));
  out.push({ gas, share: rest.length ? s : 1 });
  return out.sort((a, b) => b.share - a.share || (a.gas < b.gas ? -1 : a.gas > b.gas ? 1 : 0));
}

/** The fields a retrade writes, for one change: only those that moved. */
export function mixEdits(before: readonly MixGas[], after: readonly MixGas[]): { field: string; before: number | null; value: number }[] {
  const was = new Map(before.map((g) => [g.gas, g.share]));
  return after.filter((g) => was.get(g.gas) !== g.share).map((g) => ({ field: MIX_FIELD + g.gas, before: was.get(g.gas) ?? null, value: g.share }));
}

/** A gas's formula as written (CO₂, N₂, SO₂): the id's letters upper-cased, its digits as subscripts. */
export function formulaOf(gas: string): string {
  const sub = "₀₁₂₃₄₅₆₇₈₉";
  const els = ["he", "ne", "ar", "kr", "xe", "cl", "na"];
  let out = "";
  for (let i = 0; i < gas.length; i++) {
    const c = gas[i]!;
    if (c >= "0" && c <= "9") out += sub[Number(c)];
    else if (els.includes(gas.slice(i, i + 2))) ((out += c.toUpperCase() + gas[i + 1]), i++);
    else out += c.toUpperCase();
  }
  return out;
}

/** Each substance's plain name from the data sheet's substances on the stack (`provides.reference.substances`). */
export function substanceNames(packs: readonly unknown[]): Map<string, string> {
  const out = new Map<string, string>();
  for (const p of packs) {
    const subs = (p as { provides?: { reference?: { substances?: { id?: string; names?: { plain?: string } }[] } } })?.provides?.reference?.substances;
    for (const s of subs ?? []) if (s.id && s.names?.plain) out.set(s.id, s.names.plain);
  }
  return out;
}

/** An air to start from (design 11A, Kinds for air): a catalogued world's own pressure and mix. */
export interface AirKind {
  id: string;
  /** "Like Mars". */
  label: string;
  pressure_pa: number;
  mix: MixGas[];
  /** Where the figures come from (the Sol sheet's source id). */
  src: string;
}

/**
 * Kinds for air from the catalogued Solar System (packs/base-sol): every body with a measured pressure and mix gives
 * one, so the figures are real and cited, never typed here. Thickest first.
 */
export function airKinds(sol: SolSheet | null): AirKind[] {
  if (!sol) return [];
  const out: AirKind[] = [];
  for (const b of sol.bodies) {
    const p = b.values["atmosphere.surface_pressure"];
    if (!p || typeof p.v !== "number") continue;
    const mix: MixGas[] = [];
    for (const [k, v] of Object.entries(b.values)) if (k.startsWith(MIX_FIELD) && typeof v.v === "number") mix.push({ gas: k.slice(MIX_FIELD.length), share: v.v });
    if (!mix.length) continue;
    out.push({ id: b.id, label: `Like ${b.name}`, pressure_pa: toSI(p, `${b.id} atmosphere.surface_pressure`), mix: mix.sort((a, c) => c.share - a.share), src: p.src });
  }
  return out.sort((a, b) => b.pressure_pa - a.pressure_pa);
}

/**
 * The edits that give a body a kind's air: its pressure and every gas of the kind, and every gas it has now that the
 * kind lacks set to nothing, so the mix is the kind's alone. One change.
 */
export function airKindEdits(facts: Record<string, Json>, k: AirKind): { field: string; value: number; before: Json }[] {
  const out: { field: string; value: number; before: Json }[] = [{ field: "atmosphere.surface_pressure", value: k.pressure_pa, before: facts["atmosphere.surface_pressure"] ?? null }];
  const now = airMix(facts);
  const has = new Set(k.mix.map((g) => g.gas));
  for (const g of k.mix) out.push({ field: MIX_FIELD + g.gas, value: g.share, before: facts[MIX_FIELD + g.gas] ?? null });
  for (const g of now) if (!has.has(g.gas) && g.share > 0) out.push({ field: MIX_FIELD + g.gas, value: 0, before: g.share });
  return out.filter((e) => e.before !== e.value);
}

/**
 * The substances "Add a gas" can offer, from the data sheet on the stack: not an ion, with phase data that puts it in
 * the air somewhere (a triple point, critical point or vapour pressure), and none boiling above 300 K at one bar
 * (rocket fuels and peroxide are liquids). Plus `also` (gases a catalogued air holds, which the sheet may name only).
 */
export function gasNames(packs: readonly unknown[], also: Iterable<string> = []): Map<string, string> {
  const all = substanceNames(packs);
  const out = new Map<string, string>();
  for (const p of packs) {
    type Ph = Record<string, { v?: number } | undefined>;
    const subs = (p as { provides?: { reference?: { substances?: { id?: string; phases?: Ph }[] } } })?.provides?.reference?.substances;
    for (const s of subs ?? []) {
      if (!s.id || /[+-]/.test(s.id) || !all.has(s.id)) continue;
      const ph = s.phases ?? {};
      const boils = ph.boiling_point_k?.v;
      const airy = ["triple_point_k", "critical_point_k", "vapour_pressure", "saturated_liquid"].some((k) => ph[k]) || (typeof boils === "number" && boils <= 300);
      if (airy && !(typeof boils === "number" && boils > 300)) out.set(s.id, all.get(s.id)!);
    }
  }
  for (const g of also) if (!out.has(g)) out.set(g, all.get(g) ?? formulaOf(g));
  return out;
}

/** The data sheet's substances not yet in the air, by plain name: what "Add a gas" offers. */
export function gasesToAdd(names: ReadonlyMap<string, string>, mix: readonly MixGas[]): { gas: string; name: string }[] {
  const has = new Set(mix.filter((g) => g.share > 0).map((g) => g.gas));
  return [...names].filter(([id]) => !has.has(id)).map(([gas, name]) => ({ gas, name })).sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0));
}
