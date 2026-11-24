// Turning typed text into a value. Pure, so it is tested without a DOM.
import { readFact, type Json, type SceneFact } from "../../../ui-contract/src/index.ts";

export type Parsed = { ok: true; value: Json; display: string } | { ok: false; message: string };

/**
 * A number in a unit the sheet holds takes any unit of its kind; any other number accepts a trailing unit ("0.9 dex")
 * and a decimal comma ("0,9").
 */
export function parseFact(fact: SceneFact, text: string, shownUnit?: string): Parsed {
  const t = text.trim();
  // A decimal comma ("0,9") is not a thousands comma ("412,000").
  const dotted = t.replace(/^([-+]?\d+),(\d{1,2}|\d{4,})(?![\d,])/, "$1.$2");
  // A number in a unit the sheet holds is read by the units engine (ED1a): any unit of its kind, words and prefixes
  // ("412 thousand km" for an orbit held in AU); a bare number is in the unit on screen.
  if (typeof fact.value === "number") {
    const r = readFact(fact, dotted, shownUnit);
    if (r?.ok) return { ok: true, value: r.value, display: dotted.replace(/^about\s+/i, "") };
    if (r && !r.ok) return { ok: false, message: `${r.why}. Type ${fact.label.toLowerCase()} in any unit, for example ${fact.display}.` };
  }
  if (typeof fact.value === "number") {
    const m = /^([-+]?\d+(?:[.,]\d+)?(?:e[-+]?\d+)?)/i.exec(t);
    const n = m ? Number(m[1]!.replace(",", ".")) : NaN;
    if (!Number.isFinite(n)) return { ok: false, message: `Type a number for ${fact.label.toLowerCase()}, for example ${fact.display}.` };
    return { ok: true, value: n, display: fact.unit ? `${m![1]!.replace(",", ".")} ${fact.unit}` : m![1]!.replace(",", ".") };
  }
  if (!t) return { ok: false, message: `${fact.label} cannot be empty.` };
  return { ok: true, value: t, display: t };
}
