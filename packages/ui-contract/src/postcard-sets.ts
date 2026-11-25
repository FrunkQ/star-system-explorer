// Postcard sets as pack data (PC6, SSE4-postcards-design.md sections 7 and 12A point 6): `sse.postcards/1` in a
// pack's `provides.other.postcard_sets`, a list of named sets of postcards. The base presentation pack carries the
// base set (served beside the shell, not on the stack, so no state hash changes); a universe pack carries its own,
// and the last set on the stack with pictures is that universe's default. A set's postcards travel as clips with their
// origin (or as addresses), so each draws live where it is opened, and each carries its stamp so a held screen can
// show it without drawing. A later pack's set with the same id replaces the earlier one. Presentation only.
import type { PackDoc } from "./physics.ts";
import { checkPostcard, type Postcard } from "./postcard.ts";

export const POSTCARD_SET_SCHEMA = "sse.postcards/1";

export interface PostcardSet {
  id: string;
  title: string;
  /** The pack it came from. */
  pack: string;
  cards: Postcard[];
}

/** A set kept as a file of its own beside a pack (the base set is): read as that pack carrying it. */
export function setPack(pack: string, set: unknown): PackDoc {
  return { id: pack, version: "0", provides: { other: { postcard_sets: [set] } } } as unknown as PackDoc;
}

/** The postcard sets on a stack of packs, in order, and what was wrong with any (one line each). */
export function postcardSetsOn(packs: readonly PackDoc[]): { sets: PostcardSet[]; problems: string[] } {
  const sets: PostcardSet[] = [];
  const problems: string[] = [];
  for (const p of packs) {
    const raw = (p.provides?.other as { postcard_sets?: unknown } | undefined)?.postcard_sets;
    if (raw === undefined) continue;
    if (!Array.isArray(raw)) {
      problems.push(`${p.id}: postcard_sets must be a list`);
      continue;
    }
    for (const s of raw) {
      const o = (s ?? {}) as { schema?: unknown; id?: unknown; title?: unknown; cards?: unknown };
      if (o.schema !== POSTCARD_SET_SCHEMA || typeof o.id !== "string" || !o.id || !Array.isArray(o.cards)) {
        problems.push(`${p.id}: a postcard set needs schema ${POSTCARD_SET_SCHEMA}, an id and a list of cards`);
        continue;
      }
      const cards: Postcard[] = [];
      for (const c of o.cards) {
        const r = checkPostcard(c);
        if (r.ok) cards.push(r.postcard);
        else problems.push(`${p.id}/${o.id}: ${r.why}`);
      }
      const set: PostcardSet = { id: o.id, title: typeof o.title === "string" && o.title.trim() ? o.title.trim() : o.id, pack: p.id, cards };
      const at = sets.findIndex((x) => x.id === set.id);
      if (at >= 0) sets.splice(at, 1);
      sets.push(set);
    }
  }
  return { sets, problems };
}
