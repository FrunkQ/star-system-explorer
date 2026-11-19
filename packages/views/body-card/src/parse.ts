// Turning typed text into a value. Pure, so it is tested without a DOM.
import type { Json, SceneFact } from "../../../ui-contract/src/index.ts";

export type Parsed = { ok: true; value: Json; display: string } | { ok: false; message: string };

/** Numbers accept a trailing unit ("0.9 M⊕") and a decimal comma ("0,9"). */
export function parseFact(fact: SceneFact, text: string): Parsed {
  const t = text.trim();
  if (typeof fact.value === "number") {
    const m = /^([-+]?\d+(?:[.,]\d+)?(?:e[-+]?\d+)?)/i.exec(t);
    const n = m ? Number(m[1]!.replace(",", ".")) : NaN;
    if (!Number.isFinite(n)) return { ok: false, message: `Type a number for ${fact.label.toLowerCase()}, for example ${fact.display}.` };
    return { ok: true, value: n, display: fact.unit ? `${m![1]!.replace(",", ".")} ${fact.unit}` : m![1]!.replace(",", ".") };
  }
  if (!t) return { ok: false, message: `${fact.label} cannot be empty.` };
  return { ok: true, value: t, display: t };
}
