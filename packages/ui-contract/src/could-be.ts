// "Could be…" (ED10 part 1, editing design section 11C): one line on a world's card that looks ahead ("Could be: have a
// field of its own · be locked to its star"), each a test over the kernel's results. Pressing one works out a recipe:
// for each handle the record names, the nearest value that makes the test true (Reach it on a scratch what-if, so
// nothing is written), smallest move first; each step is its own change when pressed. The records are data (`could_be`
// in the handle sheet), so a pack can add its own.
import sheet from "./handles.json" with { type: "json" };
import type { Json } from "./json.ts";
import { fieldValue, handleValue, HANDLES, type Handle } from "./handles.ts";
import { reachIt, resultFrom } from "./reach.ts";
import type { ViewSession } from "./session.ts";

export interface CouldBe {
  id: string;
  label: string;
  test: { result: string; is: boolean };
  /** The world handles that may make it true, in the order to try. */
  by: string[];
}

export const COULD_BE: readonly CouldBe[] = (sheet as unknown as { could_be: CouldBe[] }).could_be;

/** The records worth offering for a world now: those whose result the kernel gives and whose test it does not pass. */
export function couldBeNow(results: Record<string, Json>, records: readonly CouldBe[] = COULD_BE): CouldBe[] {
  return records.filter((c) => typeof results[c.test.result] === "boolean" && results[c.test.result] !== c.test.is);
}

export interface RecipeStep {
  handle: Handle;
  /** The value that makes it true, in SI, and how far that is from now (a ratio, 1 is no move). */
  si: number;
  ratio: number;
}

/**
 * The recipe for one record: each named handle's nearest value that passes the test, searched from an eighth to eight
 * times its value now; the smallest move first. Handles that cannot get there are left out.
 */
export async function recipeFor(session: ViewSession, c: CouldBe, w: { system: string; address: string; facts: Record<string, Json>; at_yr: number }): Promise<RecipeStep[]> {
  const out: RecipeStep[] = [];
  for (const id of c.by) {
    const h = HANDLES.find((x) => x.id === id && x.on === "world");
    const now = h ? handleValue(h, w.facts) : null;
    if (!h || now === null || now <= 0) continue;
    const r = await reachIt(session, { system: w.system, address: w.address, field: h.part.field, at_yr: w.at_yr, lo: fieldValue(h, now / 8), hi: fieldValue(h, now * 8), log: true, target: 0.5, read: resultFrom(w.address, c.test.result), within: 0, tries: 12, want: (g) => (g === 1) === c.test.is });
    if (!r.ok || (r.got === 1) !== c.test.is) continue;
    const si = handleValue(h, { [h.part.field]: r.value })!;
    out.push({ handle: h, si, ratio: Math.max(si / now, now / si) });
  }
  return out.sort((a, b) => a.ratio - b.ratio);
}
