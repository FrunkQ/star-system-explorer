/**
 * Words (handoff 7A part 8): a pack's vocabulary layer, so reports and views
 * say "world" or "dton" where the game does. Packs give
 * `provides.other.vocabulary.terms`, a map from a general term to the game's
 * word; a later pack in the stack wins. Anything not listed keeps its own word.
 */

export type Terms = Readonly<Record<string, string>>;

/** The terms of a stack, in stack order (later packs win). */
export function termsOf(packs: readonly unknown[]): Terms {
  const out: Record<string, string> = {};
  for (const p of packs) {
    const t = (p as { provides?: { other?: { vocabulary?: { terms?: unknown } } } } | null)?.provides?.other?.vocabulary?.terms;
    if (!t || typeof t !== "object" || Array.isArray(t)) continue;
    for (const [k, v] of Object.entries(t)) if (typeof v === "string") out[k] = v;
  }
  return out;
}

/** The game's word for a term, keeping a capital first letter if the term had one. */
export function word(terms: Terms, term: string): string {
  const w = terms[term.toLowerCase()];
  if (w === undefined) return term;
  return term[0] !== undefined && term[0] !== term[0].toLowerCase() ? w[0]!.toUpperCase() + w.slice(1) : w;
}
