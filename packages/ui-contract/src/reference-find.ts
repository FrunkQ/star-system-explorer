// Find in Look up (rough edge 4 of handover/test-drive.md): one box finds an element, a constant or a unit by name,
// symbol or id, so looking something up is a press and a word rather than three presses through the pages.
import type { ConstantRow, ElementRow, UnitRow } from "./matter.ts";

export type RefHit =
  | { table: "elements"; id: string; label: string; hint: string }
  | { table: "constants"; id: string; label: string; hint: string }
  | { table: "units"; id: string; label: string; hint: string };

const fold = (s: string) => s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase().trim();

/** How well `q` names something called `names` (best first): 0 exact, 1 starts a name, 2 starts a word, 3 inside; null no match. */
function score(q: string, names: readonly string[], exactCase?: string): number | null {
  let best: number | null = null;
  for (const n of names) {
    if (!n) continue;
    const f = fold(n);
    const s = f === q ? 0 : f.startsWith(q) ? 1 : f.split(/[\s,()-]+/).some((w) => w.startsWith(q)) ? 2 : q.length >= 3 && f.includes(q) ? 3 : null;
    if (s !== null && (best === null || s < best)) best = s;
  }
  // A symbol typed with its own case ("Fe", "He") is the surest hit there is.
  if (exactCase !== undefined && names[0] === exactCase) best = -1;
  return best;
}

/** Up to `max` things in the sheet that `query` names, best first: elements, then constants, then units at equal score. */
export function findInSheet(query: string, sheet: { elements?: readonly ElementRow[]; constants?: readonly ConstantRow[]; units?: readonly UnitRow[] }, max = 6): RefHit[] {
  const q = fold(query);
  if (!q) return [];
  const raw = query.trim();
  const hits: { s: number; order: number; hit: RefHit }[] = [];
  for (const e of sheet.elements ?? []) {
    const s = score(q, [e.symbol, e.name, e.id, String(e.z)], raw);
    if (s !== null) hits.push({ s, order: 0, hit: { table: "elements", id: e.id, label: `${e.name} (${e.symbol})`, hint: `element ${e.z}` } });
  }
  for (const c of sheet.constants ?? []) {
    const s = score(q, [c.name, c.id]);
    if (s !== null) hits.push({ s, order: 1, hit: { table: "constants", id: c.id, label: c.name, hint: "constant" } });
  }
  for (const u of sheet.units ?? []) {
    const s = score(q, [u.name, u.symbol, u.id]);
    if (s !== null) hits.push({ s, order: 2, hit: { table: "units", id: u.id, label: `${u.name} (${u.symbol})`, hint: `unit of ${u.measures}` } });
  }
  return hits
    .sort((a, b) => a.s - b.s || a.order - b.order || a.hit.label.localeCompare(b.hit.label))
    .slice(0, max)
    .map((x) => x.hit);
}
