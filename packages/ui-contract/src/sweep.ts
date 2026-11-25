// The sweep behind the strip (ED2, editing design section 3, point 5): the kernel's K1 sweep runs a handle's field
// over the strip's range on a scratch branch and says, step by step, what follows; this file turns that into the
// strip's marks ("stops lasting", "a field of its own from here"). The tipping points are the kernel's (the stability
// verdict and the class, narrowed by halving); a yes-or-no result the handle can reach is marked where it flips
// between two steps. No physics here: every value is the kernel's.
import type { Json } from "./json.ts";
import type { Handle, PartMark, ResultRow } from "./handles.ts";
import { fieldValue, RESULTS } from "./handles.ts";

export interface SweepAsk {
  system: string;
  address: string;
  field: string;
  from: number;
  to: number;
  count: number;
  log?: boolean;
  results?: string[];
  refine?: number;
  at_s?: number;
}

export interface SweepStep {
  value: number;
  results: Record<string, Json>;
  class: string | null;
  verdict: string;
}

export interface SweepTip {
  what: "verdict" | "class" | string;
  from: Json;
  to: Json;
  between: [number, number];
}

export interface Swept {
  address: string;
  field: string;
  steps: SweepStep[];
  tipping_points: SweepTip[];
}

/** The verdict words on a strip (the kernel's stability verdict: lasts, watch, unstable). */
const VERDICT: Record<string, string> = { lasts: "lasts", watch: "worth watching", unstable: "does not last" };

/** The sweep's ask for a handle's strip: its field over the strip's range in the kernel's units, the yes-or-no results it reaches. */
export function sweepAsk(h: Handle, at: { system: string; address: string }, lo_si: number, hi_si: number, count = 12): SweepAsk {
  const flips = (h.reach ?? []).filter((k) => {
    const r = RESULTS.find((x) => x.key === k);
    return !!r?.yes && !!r.no;
  });
  return {
    system: at.system,
    address: at.address,
    field: h.part.field,
    from: fieldValue(h, lo_si),
    to: fieldValue(h, hi_si),
    count,
    log: h.part.scale !== "linear",
    results: flips,
  };
}

const mid = (a: number, b: number, log: boolean) => (log && a > 0 && b > 0 ? Math.sqrt(a * b) : (a + b) / 2);

/**
 * The marks a sweep puts on its strip, in SI: each verdict tipping point ("does not last from here"), each class
 * change ("becomes a super-Earth"), and each yes-or-no result flipping between two steps ("a field of its own from
 * here"). Read left to right, the words say what the far side is.
 */
export function sweepMarks(h: Handle, s: Swept, rows: readonly ResultRow[] = RESULTS): PartMark[] {
  const log = h.part.scale !== "linear";
  const si = (v: number) => v / fieldValue(h, 1);
  const out: PartMark[] = [];
  const up = (a: number, b: number) => b >= a;
  for (const t of s.tipping_points) {
    const [a, b] = t.between;
    const at = si(mid(a, b, log));
    if (t.what === "verdict") out.push({ at, says: `${VERDICT[String(t.to)] ?? String(t.to)} ${up(a, b) ? "from here" : "below here"}` });
    else if (t.what === "class" && t.to !== null) out.push({ at, says: `${String(t.to)} from here` });
  }
  const keys = [...new Set(s.steps.flatMap((x) => Object.keys(x.results)))];
  for (const k of keys) {
    const r = rows.find((x) => x.key === k);
    if (!r?.yes || !r.no) continue;
    for (let i = 1; i < s.steps.length; i++) {
      const [p, q] = [s.steps[i - 1]!, s.steps[i]!];
      const [a, b] = [p.results[k], q.results[k]];
      if (typeof a !== "boolean" || typeof b !== "boolean" || a === b) continue;
      out.push({ at: si(mid(p.value, q.value, log)), says: `${b ? r.yes : r.no} from here` });
    }
  }
  // Marks are keyed by their words; a second flip to the same words keeps the first.
  const seen = new Set<string>();
  return out.filter((m) => Number.isFinite(m.at) && !seen.has(m.says) && (seen.add(m.says), true)).sort((a, b) => a.at - b.at);
}
