// The Air box's mix (ED8, editing design section 11A): the whole air as shares by volume that always sum to one. The
// kernel reads `atmosphere.composition.<gas>` (volume fractions) for the climate's gas mix (SOL5), so the box writes
// those fields. Setting one gas's share trades it against the rest, which keep their proportions, so nothing needs fixing
// up by hand; the change is one set of fields, one Undo. Gases under 1% are traces, said in ppm.
import type { Json } from "./json.ts";

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
