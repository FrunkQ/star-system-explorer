// The Makeup box's bulk bar (ED9 part 1, editing design section 11B): what a world is built from, the five materials
// formation uses (iron and metal, rock, carbon, ices, gas) as shares by mass that always sum to one. The kernel reads
// `makeup.<part>` (K3): its radius follows from the mass and the makeup when none is set (core share = metal against
// metal, rock and carbon; water share = ices of the whole). Setting one part trades it against the rest, which keep
// their proportions (the Air box's retrade), so the change is one set of fields, one Undo. No physics here.
import { retrade, type MixGas } from "./air.ts";
import type { Json } from "./json.ts";

export const MAKEUP_FIELD = "makeup.";
/** The five parts, in the bar's order (heaviest stuff at the core end). */
export const MAKEUP_PARTS = ["metal", "rock", "carbon", "ice", "gas"] as const;
export type MakeupPart = (typeof MAKEUP_PARTS)[number];

/** Each part's words: Curious and Science. */
export const PART_WORDS: Record<MakeupPart, { curious: string; science: string }> = {
  metal: { curious: "iron and metal", science: "metal" },
  rock: { curious: "rock", science: "silicate rock" },
  carbon: { curious: "carbon", science: "carbon" },
  ice: { curious: "ices and water", science: "ices" },
  gas: { curious: "gas", science: "hydrogen and helium" },
};

/** A world's makeup from its facts, in the bar's order, every part listed (0 where it has none); empty when none is set. */
export function makeupOf(facts: Record<string, Json>): MixGas[] {
  const v = (p: string) => {
    const x = facts[MAKEUP_FIELD + p];
    return typeof x === "number" && x >= 0 ? x : null;
  };
  if (MAKEUP_PARTS.every((p) => v(p) === null)) return [];
  const raw = MAKEUP_PARTS.map((p) => ({ gas: p as string, share: v(p) ?? 0 }));
  const total = raw.reduce((t, g) => t + g.share, 0);
  return total > 0 ? raw.map((g) => ({ gas: g.gas, share: g.share / total })) : raw;
}

/** Back into the bar's order (retrade sorts largest first). */
const ordered = (m: readonly MixGas[]) => MAKEUP_PARTS.map((p) => ({ gas: p as string, share: m.find((g) => g.gas === p)?.share ?? 0 }));

/**
 * Set one part's share and trade it against the rest, which keep their proportions; the sum stays one. A world with no
 * makeup set starts from Earth's bulk (metal 0.32, rock 0.68, the kernel's own default when none is set).
 */
export function retradeMakeup(m: readonly MixGas[], part: MakeupPart, share: number): MixGas[] {
  const from = m.length ? ordered(m) : ordered([{ gas: "metal", share: 0.32 }, { gas: "rock", share: 0.68 }]);
  return ordered(retrade(from, part, share));
}

/**
 * The fields a makeup change writes, for one change: those that moved, and every part not yet set (at nothing), so the
 * world carries all five as the class atlas's examples do (a class region reads each part; one missing and no class
 * fits). A part never set before has null before it.
 */
export function makeupEdits(facts: Record<string, Json>, after: readonly MixGas[]): { field: string; value: number; before: Json }[] {
  const out: { field: string; value: number; before: Json }[] = [];
  for (const g of ordered(after)) {
    const was = facts[MAKEUP_FIELD + g.gas];
    if (typeof was === "number" && was === g.share) continue;
    out.push({ field: MAKEUP_FIELD + g.gas, value: g.share, before: was ?? null });
  }
  return out;
}

/** A makeup to start from: a class's named example (Like Mercury, Like Ganymede), its shares from the class atlas. */
export interface MakeupKind {
  id: string;
  /** "Like Mercury". */
  label: string;
  /** The class it is an example of, in plain words ("iron-rich world"). */
  of: string;
  makeup: MixGas[];
}

/**
 * Kinds for makeup from the class atlas on the stack (packs/base-atlas): the top level of the physical scheme, each
 * class's first named example that belongs to it and carries makeup shares. The shares are the atlas's, never typed
 * here; classes with no makeup example are left out. In the atlas's order.
 */
export function makeupKinds(packs: readonly unknown[]): MakeupKind[] {
  const out: MakeupKind[] = [];
  const seen = new Set<string>();
  for (const p of packs) {
    const cs = (p as { provides?: { other?: { classes?: unknown } } })?.provides?.other?.classes;
    if (!Array.isArray(cs)) continue;
    for (const c of cs as { id: string; scheme?: string; kind?: string; parent?: string; names?: { plain?: string }; examples?: { name: string; expect?: string; values?: Record<string, unknown> }[] }[]) {
      if (c.scheme !== "physical" || (c.kind ?? "base") !== "base" || c.parent || c.id.includes("/")) continue;
      const ex = (c.examples ?? []).find((e) => e.expect !== "out" && MAKEUP_PARTS.some((k) => typeof e.values?.[MAKEUP_FIELD + k] === "number"));
      if (!ex) continue;
      const id = `${c.id}:${ex.name}`;
      if (seen.has(ex.name)) continue;
      seen.add(ex.name);
      out.push({ id, label: `Like ${ex.name}`, of: c.names?.plain ?? c.id, makeup: makeupOf(ex.values as Record<string, Json>) });
    }
  }
  return out;
}
